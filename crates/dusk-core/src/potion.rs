//! Potions and the belt (2026-11-16; the user: "a potion belt (life, mana, haste, iron skin,
//! etc.)"): Diablo's belt of potions on the number keys, with Path of Exile's flasks' recovery.
//!
//! - **A potion is data** (`data/items/potions.toml`; `ItemKind::Potion`, a `Draught`): what it
//!   does, how much, for how long, and the ticks it takes to drink. Drunk (`Intent::Use`, a timed
//!   action a blow breaks off, as anything used is), it works on you a while (`Actor::doses`):
//!   - **life** and **mana** come back a little each tick over its time, all of it by the end;
//!     two drunk together both run;
//!   - **haste** (faster on your feet), **iron skin** (a blow on you does less harm, and its wound
//!     bleeds as much less), **vigour** (your breath back at once, and less of it spent) and
//!     **courage** (dread gone at once, and none rises) last their time, a moodle each, and their
//!     end is news (`Event::Waned`). Another of the same makes it the stronger and the longer of
//!     the two.
//!   - A mana potion is no use with no mana to fill: only a Wizard drinks one.
//! - **The belt** (rules.toml's `[belt]`, `Belt`): a few loops at a character's waist, each
//!   holding a few of one potion (`Item::belt`, the loop a stack is in). The stacks stay in the
//!   pack's list, so dropping, selling and putting them away are as for anything. A potion is
//!   quickest to drink from the belt; from the pockets you dig for it first (`Belt::rummage`), from
//!   the bag longer still (`Defs::from_bag`). What's on the belt weighs on you, but takes no room
//!   in the pockets.
//! - **A potion that comes to you goes on the belt first** (Diablo II's): into a loop holding the
//!   same with room, else the first empty one; then the bag and the pockets, as anything goes
//!   (`bag.rs`). `Intent::Belt` puts a stack on, or takes it off into the bag and the pockets.

use crate::actor::ActorId;
use crate::body::FULL;
use crate::items::{Defs, Item, ItemKind};
use crate::needs::Moodle;
use crate::world::{Event, World};

/// What a potion does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Effect {
    /// Health back over its time: `amount` out of `body::FULL` in all.
    Life,
    /// Mana back over its time: `amount` thousandths of a point in all (`spell.rs`).
    Mana,
    /// Faster on your feet, walking and running: `amount` per mille more.
    Haste,
    /// A blow that lands on you does `amount` per mille less harm, and its wound bleeds as much
    /// less.
    IronSkin,
    /// Your breath back at once, and `amount` per mille less of it spent.
    Vigour,
    /// Dread gone at once, and none rises (`dread.rs`).
    Courage,
}

impl Effect {
    pub const ALL: [Effect; 6] = [
        Effect::Life,
        Effect::Mana,
        Effect::Haste,
        Effect::IronSkin,
        Effect::Vigour,
        Effect::Courage,
    ];

    /// Whether it lasts a while and wears off (news when it does), rather than giving something
    /// back over its time.
    pub fn wears_off(self) -> bool {
        !matches!(self, Effect::Life | Effect::Mana)
    }

    /// Its moodle, while it works.
    pub fn moodle(self) -> Moodle {
        match self {
            Effect::Life => Moodle::Reviving,
            Effect::Mana => Moodle::ManaRising,
            Effect::Haste => Moodle::Quickened,
            Effect::IronSkin => Moodle::IronSkin,
            Effect::Vigour => Moodle::Vigorous,
            Effect::Courage => Moodle::Fearless,
        }
    }
}

/// A potion, from its definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draught {
    pub effect: Effect,
    /// How much, in its effect's units (courage: 1000, all of it).
    pub amount: i32,
    /// Ticks it works for (at least one), and ticks to drink it from the belt.
    pub lasts: u32,
    pub time: u32,
}

/// A potion working on whoever drank it (`Actor::doses`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Dose {
    pub effect: Effect,
    pub amount: i32,
    /// Ticks it has left, and had in all.
    pub left: u32,
    pub lasts: u32,
}

impl Dose {
    /// What it gives back on its next tick (life and mana): its share of `amount`, so that all of
    /// it has come back by its last tick, whatever the rounding.
    fn share(&self) -> i32 {
        let (lasts, amount) = (i64::from(self.lasts.max(1)), i64::from(self.amount));
        let gone = i64::from(self.lasts.saturating_sub(self.left));
        (amount * (gone + 1) / lasts - amount * gone / lasts) as i32
    }
}

/// The belt (rules.toml's `[belt]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Belt {
    /// Its loops, and how many of one potion each holds.
    pub loops: u8,
    pub holds: u16,
    /// Ticks more to dig a potion out of the pockets than to take it off the belt.
    pub rummage: u32,
}

impl Default for Belt {
    fn default() -> Belt {
        Belt {
            loops: 4,
            holds: 3,
            rummage: 10,
        }
    }
}

/// Whether two stacks are of the same, made and found alike: what shares a loop.
fn alike(a: &Item, b: &Item) -> bool {
    a.def == b.def
        && a.condition == b.condition
        && a.found == b.found
        && a.magic == b.magic
        && a.quality == b.quality
}

impl Defs {
    /// A definition's potion, if it's one.
    pub fn potion(&self, def: u16) -> Option<Draught> {
        match self.item(def)?.kind {
            ItemKind::Potion(d) => Some(d),
            _ => None,
        }
    }
}

impl World {
    /// Whether `id` could drink `d` now: alive, and for a mana potion, with mana to fill.
    pub fn can_drink(&self, id: ActorId, d: &Draught) -> bool {
        self.actors.get(id).is_some_and(|a| !a.dead)
            && (d.effect != Effect::Mana || self.max_mana(id) > 0)
    }

    /// The ticks to drink the potion at `slot` of `id`'s pack: its own from the belt, and from the
    /// pockets more, as you dig for it (the bag's own on top: `World::reach_into`).
    pub(crate) fn drink_ticks(&self, id: ActorId, slot: u16, d: &Draught) -> Option<u32> {
        if !self.can_drink(id, d) {
            return None;
        }
        let belted = self
            .actors
            .get(id)?
            .items
            .get(usize::from(slot))
            .is_some_and(|i| i.belt > 0);
        let dig = if belted { 0 } else { self.rules.belt.rummage };
        Some(d.time.max(1) + dig)
    }

    /// `id` drinks the potion at `slot` of its pack (still `def`): it starts working, vigour
    /// gives the breath back at once and courage takes the dread away. One of the stack is gone.
    pub(crate) fn drink_potion(&mut self, id: ActorId, slot: u16, def: u16, d: Draught) {
        if !self.can_drink(id, &d) {
            return;
        }
        let was = self.dread_level(id);
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let slot = usize::from(slot);
        if a.items.get(slot).is_none_or(|i| i.def != def) {
            return;
        }
        a.items[slot].count -= 1;
        if a.items[slot].count == 0 {
            a.items.remove(slot);
        }
        let lasts = d.lasts.max(1);
        let same = a
            .doses
            .iter_mut()
            .find(|o| o.effect == d.effect && d.effect.wears_off());
        match same {
            // Another of the same: the stronger and the longer of the two.
            Some(o) => {
                o.amount = o.amount.max(d.amount);
                o.left = o.left.max(lasts);
                o.lasts = o.lasts.max(o.left);
            }
            None => a.doses.push(Dose {
                effect: d.effect,
                amount: d.amount,
                left: lasts,
                lasts,
            }),
        }
        match d.effect {
            Effect::Vigour => {
                a.body.endurance = FULL;
                a.body.exhausted = false;
            }
            Effect::Courage => a.body.dread = 0,
            _ => {}
        }
        self.events.push(Event::Quaffed { who: id, item: def });
        if d.effect == Effect::Courage && was > 0 {
            self.events.push(Event::Dread { who: id, level: 0 });
        }
    }

    /// The doses working on `id`.
    pub fn doses(&self, id: ActorId) -> &[Dose] {
        self.actors.get(id).map_or(&[], |a| a.doses.as_slice())
    }

    /// How strong `effect` is on `id` now: the strongest dose of it working, 0 with none.
    pub fn dose(&self, id: ActorId, effect: Effect) -> i32 {
        self.doses(id)
            .iter()
            .filter(|d| d.effect == effect)
            .map(|d| d.amount)
            .max()
            .unwrap_or(0)
    }

    /// Whether a dose of `effect` is working on `id`.
    pub fn under(&self, id: ActorId, effect: Effect) -> bool {
        self.doses(id).iter().any(|d| d.effect == effect)
    }

    /// Per mille of a blow's harm, and of its wound's bleeding, that gets through `id`'s iron skin.
    pub(crate) fn iron_kept(&self, id: ActorId) -> i32 {
        1000 - self.dose(id, Effect::IronSkin).clamp(0, 1000)
    }

    /// Each tick: each character's doses work and run out. Life and mana come back their share; a
    /// lasting one's end is news.
    pub(crate) fn work_doses(&mut self) {
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            if self
                .actors
                .get(id)
                .is_none_or(|a| a.dead || a.doses.is_empty())
            {
                continue;
            }
            let (health, mana) = (self.max_health(id), self.max_mana(id));
            let Some(a) = self.actors.get_mut(id) else {
                continue;
            };
            let mut waned = Vec::new();
            for d in a.doses.iter_mut() {
                match d.effect {
                    Effect::Life => a.body.health = (a.body.health + d.share()).min(health),
                    Effect::Mana => a.mana = (a.mana + d.share()).min(mana),
                    _ => {}
                }
                d.left = d.left.saturating_sub(1);
                if d.left == 0 && d.effect.wears_off() {
                    waned.push(d.effect);
                }
            }
            a.doses.retain(|d| d.left > 0);
            for effect in waned {
                self.events.push(Event::Waned { who: id, effect });
            }
        }
    }

    /// The stack in each loop of `id`'s belt, the first loop first: its slot in the pack, None for
    /// an empty loop.
    pub fn belt(&self, id: ActorId) -> Vec<Option<u16>> {
        let mut loops = vec![None; usize::from(self.rules.belt.loops)];
        if let Some(a) = self.actors.get(id) {
            for (slot, i) in a.items.iter().enumerate() {
                let k = usize::from(i.belt).wrapping_sub(1);
                if let Some(l) = loops.get_mut(k) {
                    *l = Some(slot as u16);
                }
            }
        }
        loops
    }

    /// The loop of `id`'s belt the next of `item` goes into, and how many of it fit there: one
    /// holding the same with room, else the first empty one. None for what's no potion, or with
    /// no room.
    fn belt_loop(&self, id: ActorId, item: &Item) -> Option<(u8, u16)> {
        self.defs.potion(item.def)?;
        let (a, holds) = (self.actors.get(id)?, self.rules.belt.holds);
        let loops = self.belt(id);
        let held = |slot: Option<u16>| slot.and_then(|s| a.items.get(usize::from(s)));
        let (k, room) = match loops
            .iter()
            .position(|&s| held(s).is_some_and(|h| alike(h, item) && h.count < holds))
        {
            Some(k) => (k, holds - held(loops[k]).map_or(0, |h| h.count)),
            None => (loops.iter().position(Option::is_none)?, holds),
        };
        Some((k as u8 + 1, room.min(item.count)))
    }

    /// How many of `item` `id`'s belt has room for, in all its loops.
    pub fn belt_room(&self, id: ActorId, item: &Item) -> u16 {
        if self.defs.potion(item.def).is_none() {
            return 0;
        }
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        let holds = self.rules.belt.holds;
        let room: u32 = self
            .belt(id)
            .into_iter()
            .map(
                |slot| match slot.and_then(|s| a.items.get(usize::from(s))) {
                    None => u32::from(holds),
                    Some(h) if alike(h, item) => u32::from(holds.saturating_sub(h.count)),
                    Some(_) => 0,
                },
            )
            .sum();
        room.min(u32::from(item.count)) as u16
    }

    /// Puts as many of `item` on `id`'s belt as there's room for; what's left of it, if any.
    pub(crate) fn belt_in(&mut self, id: ActorId, mut item: Item) -> Option<Item> {
        while item.count > 0 {
            let Some((k, n)) = self.belt_loop(id, &item) else {
                break;
            };
            let slot = self.belt(id)[usize::from(k - 1)];
            let Some(a) = self.actors.get_mut(id) else {
                break;
            };
            match slot {
                Some(s) => a.items[usize::from(s)].count += n,
                None => a.items.push(Item {
                    count: n,
                    bagged: false,
                    belt: k,
                    cell: None,
                    ..item
                }),
            }
            item.count -= n;
        }
        (item.count > 0).then_some(item)
    }

    /// What's on `id`'s belt weighs, in grams.
    pub(crate) fn belt_weight(&self, id: ActorId) -> u32 {
        self.actors.get(id).map_or(0, |a| {
            a.items
                .iter()
                .filter(|i| i.belt > 0)
                .map(|i| self.defs.weight(i))
                .sum()
        })
    }

    /// How many of the stack at `slot` of `id`'s pack (still `def`) would move: a potion off the
    /// belt onto it, as many as there's room for; one on it off it, as many as the bag and the
    /// pockets have room for (`pack.rs`).
    pub fn can_belt(&self, id: ActorId, slot: u16, def: u16) -> u16 {
        let Some(i) = self
            .actors
            .get(id)
            .filter(|a| !a.dead)
            .and_then(|a| a.items.get(usize::from(slot)))
            .filter(|i| i.def == def)
        else {
            return 0;
        };
        if i.belt > 0 {
            let (bag, pockets) = self.room_for(id, &Item { belt: 0, ..*i });
            bag + pockets
        } else {
            self.belt_room(id, i)
        }
    }

    /// Moves the stack at `slot` of `id`'s pack (still `def`) onto the belt, as many as fit, or
    /// off it, into the bag and then the pockets, as many as fit.
    pub(crate) fn belt_item(&mut self, id: ActorId, slot: u16, def: u16) {
        let n = self.can_belt(id, slot, def);
        let Some(a) = self.actors.get_mut(id).filter(|_| n > 0) else {
            return;
        };
        let s = usize::from(slot);
        let item = a.items[s];
        if n >= item.count {
            a.items.remove(s);
        } else {
            a.items[s].count -= n;
        }
        let moved = Item {
            count: n,
            bagged: false,
            belt: 0,
            cell: None,
            ..item
        };
        let rest = if item.belt > 0 {
            Some(moved)
        } else {
            self.belt_in(id, moved)
        };
        if let Some(rest) = rest {
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
    }
}
