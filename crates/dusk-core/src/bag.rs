//! Bags (`CONCEPT.md`, the inventory: pockets and bags with room, now as Path of Exile's grids).
//!
//! - **Pockets and a bag.** With no bag you carry what fits in your pockets' grid
//!   (`Defs::pockets`). A bag on your back (`Actor::bag`) is a grid of its own (`Bag::grid`), and
//!   what's in it weighs less on you by its `reduce`: a good pack carries a load well. Each stack
//!   of the pack is in one or the other (`Item::bagged`), on its cells (`Item::cell`, `pack.rs`).
//! - **What you take is bounded:** from a floor, a container or a counter, it goes into the bag
//!   while there's room, then the pockets, as many of a stack as fit; with no room, none. What
//!   comes to you otherwise (made, picked, cut up, coin for a sale, put away from your hand) goes
//!   where there's room, and what doesn't fit falls at your feet (`Event::NoRoom`). A potion goes
//!   on the belt before either, while it has room (`potion.rs`): the belt is a place of its own,
//!   weighing on you but taking none of the grids' room.
//! - **The bag is slower to reach into:** using a thing from it, taking it in hand, lighting it
//!   or putting it on takes `Defs::from_bag` more. Moving a stack between the pockets and the
//!   bag takes `Defs::shift`.
//! - **A bag is put on from the pack** (`Defs::bag_on`). The one you wore comes off into the
//!   pack, and what was in it moves into the new one, where it was if it fits there. A bag you
//!   take with your back bare and no room for it in your pockets goes straight on.
//! - **Dropped, a bag falls with what's in it** (`World::held`), each thing on its cell, in a
//!   moment: the way to fight unburdened. Taken up again, a full bag goes on your back with all
//!   of it, as it was; with one on already, what's in it comes into yours as far as it fits. A
//!   body's bag falls full, and a revenant carries yours.

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::items::{Defs, Item, ItemKind, within_reach};
use crate::pack::{Cell, cells_of, in_grid, place};
use crate::world::{Event, World};

/// A bag's numbers, from its definition: its grid, cells across and down, and per mille off the
/// weight of what's in it, on your back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bag {
    pub grid: (u8, u8),
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
    /// Out of any bag, off any belt, with no cell: what a stack is once it's off its carrier (on
    /// a floor, in a container, before it's laid out there).
    pub fn loose(self) -> Item {
        Item {
            bagged: false,
            belt: 0,
            cell: None,
            ..self
        }
    }
}

/// `x` per mille of `n`.
fn per(n: u32, x: u32) -> u32 {
    (u64::from(n) * u64::from(x) / 1000) as u32
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

    /// What's in `id`'s pockets and in its bag, in grams, as they'd weigh in hand: not what's on
    /// its belt, a place of its own (`World::belt_weight`).
    pub fn filled(&self, id: ActorId) -> (u32, u32) {
        self.actors.get(id).map_or((0, 0), |a| {
            a.items
                .iter()
                .filter(|i| i.belt == 0)
                .fold((0, 0), |(pockets, bag), i| {
                    let w = self.weight_of(i);
                    if i.bagged {
                        (pockets, bag + w)
                    } else {
                        (pockets + w, bag)
                    }
                })
        })
    }

    /// What `id`'s pack weighs on it: its pockets and its belt, and its bag with what's in it,
    /// which weighs the bag's share less.
    pub(crate) fn pack_weight(&self, id: ActorId) -> u32 {
        let (pockets, bagged) = self.filled(id);
        let pockets = pockets + self.belt_weight(id);
        match self.worn_bag(id) {
            Some((bag, b)) => {
                pockets + per(bagged, 1000u32.saturating_sub(b.reduce)) + self.defs.weight(&bag)
            }
            None => pockets + bagged,
        }
    }

    /// How many of a stack `id` has room for: in its bag's grid, and then in its pockets'
    /// (`pack.rs`): onto stacks alike with room, then on free cells.
    pub fn room_for(&self, id: ActorId, item: &Item) -> (u16, u16) {
        let Some(a) = self.actors.get(id) else {
            return (0, 0);
        };
        let mut list = a.items.clone();
        let mut room = [0u16; 2];
        let mut left = item.count;
        for (k, bagged) in [true, false].into_iter().enumerate() {
            let Some(size) = self.pack_grid(id, bagged).filter(|_| left > 0) else {
                continue;
            };
            let mut cells = cells_of(&self.defs, size, &list, in_grid(bagged), None);
            let piece = Item {
                count: left,
                bagged,
                belt: 0,
                cell: None,
                ..*item
            };
            let rest = place(
                &self.defs,
                &mut list,
                &mut cells,
                piece,
                in_grid(bagged),
                None,
            );
            room[k] = left - rest;
            left = rest;
        }
        (room[0], room[1])
    }

    /// Puts a stack on `id`: a potion on the belt while it has room (`potion.rs`), then into its
    /// bag as far as there's room, then its pockets; what doesn't fit falls at its feet, and it's
    /// told (`Event::NoRoom`).
    pub(crate) fn carry_in(&mut self, id: ActorId, item: Item) {
        let Some(rest) = self.belt_in(id, item) else {
            return;
        };
        let left = self.stow_in(id, rest);
        if left > 0 {
            self.fall_at_feet(
                id,
                Item {
                    count: left,
                    ..rest
                },
            );
        }
    }

    /// Lets a stack that came to `id` with no room fall at its feet, and says so.
    pub(crate) fn fall_at_feet(&mut self, id: ActorId, item: Item) {
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        self.put(pos, item);
        self.events.push(Event::NoRoom {
            who: id,
            item: item.def,
            count: item.count,
        });
    }

    /// As `carry_in`, but never onto the belt, and never at the feet: into the bag as far as
    /// there's room, then the pockets. How many didn't fit.
    pub(crate) fn stow_in(&mut self, id: ActorId, item: Item) -> u16 {
        let left = if self.worn_bag(id).is_some() {
            self.stow_grid(id, item, true, None)
        } else {
            item.count
        };
        match left {
            0 => 0,
            n => self.stow_grid(id, Item { count: n, ..item }, false, None),
        }
    }

    /// Whether `id` can take the stack at `slot` of what's on `at` (still `def`), and how many of
    /// it: as many as there's room for (a potion's on the belt too). A full bag is taken whole
    /// (one): onto your back, or with a bag on already, as much of what's in it as fits.
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
        // A potion's room on the belt first (`potion.rs`), then the bag's and the pockets'.
        let belt = self.belt_room(id, &item);
        let rest = Item {
            count: item.count - belt,
            ..item
        };
        match self.room_for(id, &rest) {
            // A bag with no room for it, and a bare back: it goes on (`take`).
            (0, 0) if belt == 0 && a.bag.is_none() && self.defs.bag(def).is_some() => Ok(1),
            (0, 0) if belt == 0 => Err(NoTake::NoRoom),
            (b, p) => Ok(belt + b + p),
        }
    }

    /// Whether `id` can take the stack at `slot` of what's on `at` (still `def`) onto `cell` of
    /// one of its grids (`bagged`: the bag's): how many would go there, onto the stack alike that
    /// covers it or onto free cells; 0 if none.
    pub fn can_take_to(
        &self,
        id: ActorId,
        at: TilePos,
        slot: u16,
        def: u16,
        bagged: bool,
        cell: Cell,
    ) -> u16 {
        let Ok(n) = self.can_take(id, at, slot, def) else {
            return 0;
        };
        let Some(item) = self.store(at).get(usize::from(slot)).copied() else {
            return 0;
        };
        if item.holds != 0 {
            return 0;
        }
        let (Some(a), Some(size)) = (self.actors.get(id), self.pack_grid(id, bagged)) else {
            return 0;
        };
        let list = &a.items;
        let cells = cells_of(&self.defs, size, list, in_grid(bagged), None);
        let stack = self.defs.item(def).map_or(1, |d| d.stack.max(1));
        let piece = Item {
            bagged,
            belt: 0,
            cell: None,
            ..item
        };
        // Only what lands on that cell counts: onto the stack there, or a new one there.
        match crate::pack::stack_at(&self.defs, list, in_grid(bagged), cell) {
            Some(k) if self.defs.alike(&list[k], &piece) => {
                stack.saturating_sub(list[k].count).min(n)
            }
            Some(_) => 0,
            None if cells.fits(cell, self.defs.size(def)) => n.min(stack),
            None => 0,
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
        if self.actors.get(id).is_some_and(|a| a.bag.is_none()) {
            if let Some(a) = self.actors.get_mut(id) {
                a.bag = Some(bag);
            }
            self.refill(id, contents);
            self.events.push(Event::Wore {
                who: id,
                item: bag.def,
            });
            return;
        }
        let mut left = Vec::new();
        for c in contents {
            let piece = Item { cell: None, ..c };
            let n = c.count - self.stow_in(id, piece);
            if n > 0 {
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
            self.stow_in(id, bag);
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

    /// What was in a bag, into the one on `id`'s back: each where it was if it fits there (it's
    /// the same bag, or one as big), else wherever there's room in it, else the pockets, else at
    /// `id`'s feet.
    fn refill(&mut self, id: ActorId, contents: Vec<Item>) {
        for c in contents {
            let piece = Item {
                bagged: true,
                belt: 0,
                ..c
            };
            let left = self.stow_grid(
                id,
                Item {
                    cell: None,
                    ..piece
                },
                true,
                c.cell,
            );
            let left = match left {
                0 => 0,
                n => self.stow_grid(id, Item { count: n, ..piece }, false, None),
            };
            if left > 0 {
                self.fall_at_feet(
                    id,
                    Item {
                        count: left,
                        ..c.loose()
                    },
                );
            }
        }
    }

    /// A bag with `contents` in it, lying somewhere: `World::held` keeps them, by its `holds`,
    /// each on the cell it had in the bag.
    fn fill_bag(&mut self, bag: Item, contents: Vec<Item>) -> Item {
        if contents.is_empty() {
            return Item {
                holds: 0,
                ..bag.loose()
            };
        }
        let n = self.held.last_key_value().map_or(1, |(&k, _)| k + 1);
        let kept = contents
            .into_iter()
            .map(|i| Item {
                bagged: false,
                belt: 0,
                ..i
            })
            .collect();
        self.held.insert(n, kept);
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
    /// moves into the new one (where it was if it fits there; what doesn't fit, into the pockets,
    /// else at its feet).
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
        // From bag to bag, as they were: not onto the belt.
        self.refill(id, contents);
        if let Some(old) = old {
            let left = self.stow_in(id, old);
            if left > 0 {
                self.fall_at_feet(id, old);
            }
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
    /// pockets and its bag: as many as there's room for on the other side (onto `to` there, if
    /// it's asked for: the stack alike that covers it, or free cells), none with no bag on, nor
    /// for one on the belt (`World::can_belt` takes it off).
    pub fn can_shift(&self, id: ActorId, slot: u16, def: u16, to: Option<Cell>) -> u16 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        let Some(item) = a
            .items
            .get(usize::from(slot))
            .filter(|i| i.def == def && i.belt == 0)
        else {
            return 0;
        };
        if self.worn_bag(id).is_none() {
            return 0;
        }
        let other = !item.bagged;
        let Some(size) = self.pack_grid(id, other) else {
            return 0;
        };
        let mut list = a.items.clone();
        let mut cells = cells_of(&self.defs, size, &list, in_grid(other), None);
        let piece = Item {
            bagged: other,
            cell: None,
            ..*item
        };
        if let Some(cell) = to {
            let stack = self.defs.item(def).map_or(1, |d| d.stack.max(1));
            return match crate::pack::stack_at(&self.defs, &list, in_grid(other), cell) {
                Some(k) if self.defs.alike(&list[k], &piece) => {
                    stack.saturating_sub(list[k].count).min(item.count)
                }
                Some(_) => 0,
                None if cells.fits(cell, self.defs.size(def)) => item.count.min(stack),
                None => 0,
            };
        }
        item.count
            - place(
                &self.defs,
                &mut list,
                &mut cells,
                piece,
                in_grid(other),
                None,
            )
    }

    /// Moves the stack at `slot` of `id`'s pack (still `def`) between its pockets and its bag,
    /// as many as fit, onto `to` if it's asked for.
    pub(crate) fn shift(&mut self, id: ActorId, slot: u16, def: u16, to: Option<Cell>) {
        let n = self.can_shift(id, slot, def, to);
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
            cell: None,
            ..item
        };
        let left = self.stow_grid(id, moved, !item.bagged, to);
        // It was counted to fit: anything left goes back where it came from, else at the feet.
        if left > 0 {
            let back = Item {
                count: left,
                ..moved
            };
            let left = self.stow_grid(id, back, item.bagged, None);
            if left > 0 {
                self.fall_at_feet(
                    id,
                    Item {
                        count: left,
                        ..item.loose()
                    },
                );
            }
        }
    }
}
