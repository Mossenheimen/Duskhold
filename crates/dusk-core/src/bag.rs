//! Bags (`CONCEPT.md`, the inventory: "lists with weights, not a grid; bags with capacities").
//!
//! - **Pockets and a bag.** With no bag you carry what fits in your pockets (`Defs::pockets`).
//!   A bag on your back (`Actor::bag`) holds its `capacity` more, and what's in it weighs less
//!   on you by its `reduce`: a good pack carries a load well. Each stack of the pack is in one or
//!   the other (`Item::bagged`).
//! - **What you take is bounded:** from a floor, a container or a counter, it goes into the bag
//!   while there's room, then the pockets, as many of a stack as fit; with no room, none. What
//!   comes to you otherwise (made, picked, cut up, coin for a sale, put away from your hand) is
//!   never refused: into the bag while there's room, then the pockets, over full if need be.
//! - **The bag is slower to reach into:** using a thing from it, taking it in hand, lighting it
//!   or putting it on takes `Defs::from_bag` more. Moving a stack between the pockets and the
//!   bag takes `Defs::shift`.
//! - **A bag is put on from the pack** (`Defs::bag_on`). The one you wore comes off into the
//!   pack, and what was in it moves into the new one. A bag you take with your back bare and no
//!   room for it in your pockets goes straight on.
//! - **Dropped, a bag falls with what's in it** (`World::held`), in a moment: the way to fight
//!   unburdened. Taken up again, a full bag goes on your back with all of it; with one on
//!   already, what's in it comes into yours as far as it fits. A body's bag falls full, and a
//!   revenant carries yours.

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::items::{Defs, Item, ItemKind, within_reach};
use crate::world::{Event, World};

/// A bag's numbers, from its definition: the grams it holds, and per mille off the weight of
/// what's in it, on your back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bag {
    pub capacity: u32,
    pub reduce: u32,
}

/// Why a stack can't be taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoTake {
    /// It isn't there any more, or it's out of your reach.
    Gone,
    /// No room for even one of it, in your bag or your pockets.
    NoRoom,
}

impl Defs {
    /// A definition's bag numbers, if it's a bag.
    pub fn bag(&self, def: u16) -> Option<Bag> {
        match self.item(def)?.kind {
            ItemKind::Bag(b) => Some(b),
            _ => None,
        }
    }
}

impl Item {
    /// Out of any bag: what a stack is once it's off its carrier (on a floor, in a container).
    pub fn loose(self) -> Item {
        Item {
            bagged: false,
            ..self
        }
    }
}

/// `x` per mille of `n`.
fn per(n: u32, x: u32) -> u32 {
    (u64::from(n) * u64::from(x) / 1000) as u32
}

/// How many of a stack of `most`, at `unit` grams each, fit in `room` grams: all of a stack that
/// weighs nothing.
fn fits(unit: u32, room: u32, most: u16) -> u16 {
    room.checked_div(unit)
        .map_or(most, |n| n.min(u32::from(most)) as u16)
}

impl World {
    /// The bag on `id`'s back, and its numbers.
    pub fn worn_bag(&self, id: ActorId) -> Option<(Item, Bag)> {
        let bag = self.actors.get(id)?.bag?;
        Some((bag, self.defs.bag(bag.def)?))
    }

    /// What a stack weighs, with what's in it if it's a full bag, in grams.
    pub fn weight_of(&self, item: &Item) -> u32 {
        let inside = match item.holds {
            0 => 0,
            n => self
                .held
                .get(&n)
                .map_or(0, |c| c.iter().map(|i| self.defs.weight(i)).sum::<u32>()),
        };
        self.defs.weight(item) + inside
    }

    /// What's in `id`'s pockets and in its bag, in grams, as they'd weigh in hand.
    pub fn filled(&self, id: ActorId) -> (u32, u32) {
        self.actors.get(id).map_or((0, 0), |a| {
            a.items.iter().fold((0, 0), |(pockets, bag), i| {
                let w = self.weight_of(i);
                if i.bagged {
                    (pockets, bag + w)
                } else {
                    (pockets + w, bag)
                }
            })
        })
    }

    /// What `id`'s pack weighs on it: its pockets, and its bag with what's in it, which weighs
    /// the bag's share less.
    pub(crate) fn pack_weight(&self, id: ActorId) -> u32 {
        let (pockets, bagged) = self.filled(id);
        match self.worn_bag(id) {
            Some((bag, b)) => {
                pockets + per(bagged, 1000u32.saturating_sub(b.reduce)) + self.defs.weight(&bag)
            }
            None => pockets + bagged,
        }
    }

    /// How many of a stack `id` has room for: in its bag, and then in its pockets.
    pub fn room_for(&self, id: ActorId, item: &Item) -> (u16, u16) {
        let (pockets, bagged) = self.filled(id);
        let unit = self.weight_of(&Item { count: 1, ..*item });
        let in_bag = self.worn_bag(id).map_or(0, |(_, b)| {
            fits(unit, b.capacity.saturating_sub(bagged), item.count)
        });
        let left = item.count - in_bag;
        (
            in_bag,
            fits(unit, self.defs.pockets.saturating_sub(pockets), left),
        )
    }

    /// Puts a stack on `id`, never refusing it: into its bag as far as there's room, the rest
    /// into its pockets (over full, if need be).
    pub(crate) fn carry_in(&mut self, id: ActorId, item: Item) {
        let (in_bag, _) = self.room_for(id, &item);
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if in_bag > 0 {
            let bagged = Item {
                count: in_bag,
                bagged: true,
                ..item
            };
            defs.stow(&mut a.items, bagged);
        }
        if item.count > in_bag {
            let loose = Item {
                count: item.count - in_bag,
                ..item.loose()
            };
            defs.stow(&mut a.items, loose);
        }
    }

    /// Whether `id` can take the stack at `slot` of what's on `at` (still `def`), and how many of
    /// it: as many as there's room for. A full bag is taken whole (one): onto your back, or with
    /// a bag on already, as much of what's in it as fits.
    pub fn can_take(&self, id: ActorId, at: TilePos, slot: u16, def: u16) -> Result<u16, NoTake> {
        let a = self
            .actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoTake::Gone)?;
        if !within_reach(&self.grid, a.pos, at) {
            return Err(NoTake::Gone);
        }
        let item = *self
            .store(at)
            .get(usize::from(slot))
            .filter(|i| i.def == def)
            .ok_or(NoTake::Gone)?;
        if item.holds != 0 {
            let room = a.bag.is_none()
                || self.held.get(&item.holds).is_some_and(|c| {
                    c.iter().any(|c| {
                        let (b, p) = self.room_for(id, c);
                        b + p > 0
                    })
                });
            return if room { Ok(1) } else { Err(NoTake::NoRoom) };
        }
        match self.room_for(id, &item) {
            // A bag with no room for it, and a bare back: it goes on (`take`).
            (0, 0) if a.bag.is_none() && self.defs.bag(def).is_some() => Ok(1),
            (0, 0) => Err(NoTake::NoRoom),
            (b, p) => Ok(b + p),
        }
    }

    /// A full bag taken up from `at`: onto `id`'s back with all that's in it; or, with a bag on
    /// already, what's in it into yours as far as it fits, then the bag itself once it's empty, if
    /// there's room. What's left stays on `at`, in the bag.
    pub(crate) fn take_full_bag(&mut self, id: ActorId, at: TilePos, bag: Item) {
        let contents = self.held.remove(&bag.holds).unwrap_or_default();
        let bag = Item {
            holds: 0,
            ..bag.loose()
        };
        let defs = self.defs.clone();
        if let Some(a) = self.actors.get_mut(id)
            && a.bag.is_none()
        {
            a.bag = Some(bag);
            for c in contents {
                defs.stow(&mut a.items, Item { bagged: true, ..c });
            }
            self.events.push(Event::Wore {
                who: id,
                item: bag.def,
            });
            return;
        }
        let mut left = Vec::new();
        for c in contents {
            let (b, p) = self.room_for(id, &c);
            let n = b + p;
            if n > 0 {
                self.carry_in(id, Item { count: n, ..c });
                self.events.push(Event::Took {
                    who: id,
                    item: c.def,
                    count: n,
                });
            }
            if n < c.count {
                left.push(Item {
                    count: c.count - n,
                    ..c
                });
            }
        }
        if left.is_empty() && self.room_for(id, &bag) != (0, 0) {
            self.carry_in(id, bag);
            self.events.push(Event::Took {
                who: id,
                item: bag.def,
                count: 1,
            });
            return;
        }
        let full = self.fill_bag(bag, left);
        self.put(at, full);
    }

    /// A bag with `contents` in it, lying somewhere: `World::held` keeps them, by its `holds`.
    fn fill_bag(&mut self, bag: Item, contents: Vec<Item>) -> Item {
        if contents.is_empty() {
            return Item {
                holds: 0,
                ..bag.loose()
            };
        }
        let n = self.held.last_key_value().map_or(1, |(&k, _)| k + 1);
        self.held
            .insert(n, contents.into_iter().map(Item::loose).collect());
        Item {
            holds: n,
            ..bag.loose()
        }
    }

    /// Takes the bag off `id`'s back, full of what was in it: for dropping, and a body's gear.
    pub(crate) fn unpack(&mut self, id: ActorId) -> Option<Item> {
        let a = self.actors.get_mut(id)?;
        let bag = a.bag.take()?;
        let (contents, rest): (Vec<Item>, Vec<Item>) = std::mem::take(&mut a.items)
            .into_iter()
            .partition(|i| i.bagged);
        a.items = rest;
        Some(self.fill_bag(bag, contents))
    }

    /// Drops the bag on `id`'s back at its feet, with what's in it.
    pub(crate) fn drop_bag(&mut self, id: ActorId) {
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        let Some(bag) = self.unpack(id) else {
            return;
        };
        self.put(pos, bag);
        self.events.push(Event::Dropped {
            who: id,
            item: bag.def,
            count: 1,
        });
    }

    /// Puts a bag on `id`'s back. The one it wore comes off into its pack, and what was in it
    /// moves into the new one (what doesn't fit, into the pockets).
    pub(crate) fn don_bag(&mut self, id: ActorId, bag: Item) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let old = a.bag.replace(Item {
            holds: 0,
            ..bag.loose()
        });
        let (contents, rest): (Vec<Item>, Vec<Item>) = std::mem::take(&mut a.items)
            .into_iter()
            .partition(|i| i.bagged);
        a.items = rest;
        for c in contents {
            self.carry_in(id, c);
        }
        if let Some(old) = old {
            self.carry_in(id, old);
        }
        self.events.push(Event::Wore {
            who: id,
            item: bag.def,
        });
    }

    /// The ticks more it takes to reach the thing at `slot` of `id`'s pack: none from the
    /// pockets, `Defs::from_bag` from the bag.
    pub(crate) fn reach_into(&self, id: ActorId, slot: u16) -> u32 {
        let bagged = self
            .actors
            .get(id)
            .and_then(|a| a.items.get(usize::from(slot)))
            .is_some_and(|i| i.bagged);
        if bagged { self.defs.from_bag } else { 0 }
    }

    /// How many of the stack at `slot` of `id`'s pack (still `def`) would move between its
    /// pockets and its bag: as many as there's room for on the other side, none with no bag on.
    pub fn can_shift(&self, id: ActorId, slot: u16, def: u16) -> u16 {
        let Some(item) = self
            .actors
            .get(id)
            .and_then(|a| a.items.get(usize::from(slot)))
            .filter(|i| i.def == def)
        else {
            return 0;
        };
        let Some((_, bag)) = self.worn_bag(id) else {
            return 0;
        };
        let (pockets, bagged) = self.filled(id);
        let room = if item.bagged {
            self.defs.pockets.saturating_sub(pockets)
        } else {
            bag.capacity.saturating_sub(bagged)
        };
        fits(
            self.weight_of(&Item { count: 1, ..*item }),
            room,
            item.count,
        )
    }

    /// Moves the stack at `slot` of `id`'s pack (still `def`) between its pockets and its bag,
    /// as many as fit.
    pub(crate) fn shift(&mut self, id: ActorId, slot: u16, def: u16) {
        let n = self.can_shift(id, slot, def);
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id).filter(|_| n > 0) else {
            return;
        };
        let slot = usize::from(slot);
        let item = a.items[slot];
        if n >= item.count {
            a.items.remove(slot);
        } else {
            a.items[slot].count -= n;
        }
        let moved = Item {
            count: n,
            bagged: !item.bagged,
            ..item
        };
        defs.stow(&mut a.items, moved);
    }
}
