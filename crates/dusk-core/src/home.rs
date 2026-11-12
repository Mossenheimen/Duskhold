//! Bring it home (`CONCEPT.md`, pillar 5): most of a character's XP comes from what it carries
//! back, and each find counts once.
//!
//! - **Home is where you slept the night through.** Waking rested makes the tile you slept on
//!   home. Sleep needs tiredness and none of the dead about, so home moves at most once a game
//!   day, and only after a safe night.
//! - **Your stash** is one container you mark, under a roof and near home. Marking another moves
//!   it (the things stay where they are).
//! - **A find** is an item that came out of the world (a container's loot) and hasn't been home
//!   yet. Put into your stash, it pays its worth in XP, once: from then on it's an ordinary item,
//!   wherever it goes. What you started with was never found.
//! - **Character levels** come from that XP, 1 to 10. Each brings a little more health, bounded
//!   (pillar 3: fragile, not weak): a level-10 character is tougher, never safe in a crowd.
//!
//! So every trip asks the loop's question: what's worth its weight, carried how far, past what?

use crate::actor::{ActorId, ActorKind};
use crate::geom::TilePos;
use crate::items::{Item, within_reach};
use crate::world::{Event, World};

/// The highest character level.
pub const MAX_LEVEL: u8 = 10;

/// The numbers of bringing it home, in the core's units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deeds {
    /// The XP (thousandths) to reach each level from 2 to 10, rising. Level 1 is 0.
    pub levels: [u32; MAX_LEVEL as usize - 1],
    /// Health a level brings, out of `body::FULL`.
    pub health: i32,
    /// Ticks to mark a container as your stash.
    pub stash_time: u32,
    /// How far from home your stash may be, in tiles.
    pub home_range: i32,
}

impl Default for Deeds {
    fn default() -> Deeds {
        Deeds {
            levels: [
                30_000, 80_000, 160_000, 300_000, 500_000, 800_000, 1_200_000, 1_800_000, 2_600_000,
            ],
            health: 50,
            stash_time: 10,
            home_range: 8,
        }
    }
}

/// Why a container can't be your stash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoStash {
    /// Not a container, or not within reach.
    NotHere,
    /// Out under the sky: a stash wants a roof over it.
    Outdoors,
    /// You haven't slept a night through anywhere yet.
    NoHome,
    /// Too far from where you sleep.
    FarFromHome,
}

impl Deeds {
    /// The level some XP reaches.
    pub fn level(&self, xp: u32) -> u8 {
        1 + self.levels.iter().filter(|&&l| xp >= l).count() as u8
    }

    /// The XP a level starts at, and the next one's (None at the top).
    pub fn span(&self, level: u8) -> (u32, Option<u32>) {
        let from = match level {
            0 | 1 => 0,
            n => self.levels[usize::from(n.min(MAX_LEVEL)) - 2],
        };
        (
            from,
            self.levels.get(usize::from(level.max(1)) - 1).copied(),
        )
    }
}

impl World {
    /// A character's level.
    pub fn level(&self, id: ActorId) -> u8 {
        self.actors
            .get(id)
            .map_or(1, |a| self.rules.deeds.level(a.deeds))
    }

    /// The most health an actor can have: a character's grows a little with its level.
    pub fn max_health(&self, id: ActorId) -> i32 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        match a.kind {
            ActorKind::Character => {
                let levels = i32::from(self.rules.deeds.level(a.deeds)) - 1;
                self.rules.combat.character_health
                    + levels * self.rules.deeds.health
                    + self.talent_health(id)
            }
            ActorKind::Creature(_) => self.stats(id).map_or(0, |c| c.health),
        }
    }

    /// Whether `id` can make the container on `at` its stash now.
    pub fn can_stash(&self, id: ActorId, at: TilePos) -> Result<(), NoStash> {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return Err(NoStash::NotHere);
        };
        if !self.is_container(at) || !within_reach(&self.grid, a.pos, at) {
            return Err(NoStash::NotHere);
        }
        if !self.grid.tile(at).roof {
            return Err(NoStash::Outdoors);
        }
        let home = a.home.ok_or(NoStash::NoHome)?;
        let r = i64::from(self.rules.deeds.home_range);
        if home.z != at.z || home.dist2(at) > r * r {
            return Err(NoStash::FarFromHome);
        }
        Ok(())
    }

    /// Waking rested: where you slept is home now.
    pub(crate) fn make_home(&mut self, id: ActorId) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if a.kind != ActorKind::Character || a.home == Some(a.pos) {
            return;
        }
        // A holdout's bed is a guest's: it doesn't make a home.
        if crate::holdout::in_hallowed(&self.hallowed, a.pos) {
            return;
        }
        a.home = Some(a.pos);
        let at = a.pos;
        self.events.push(Event::MadeHome { who: id, at });
    }

    /// The stash is marked (the action's end).
    pub(crate) fn mark_stash(&mut self, id: ActorId, at: TilePos) {
        if self.can_stash(id, at).is_err() {
            return;
        }
        if let Some(a) = self.actors.get_mut(id) {
            a.stash = Some(at);
            self.events.push(Event::Stashed { who: id, at });
        }
    }

    /// Ticks to put the stack at `slot` of your pack into the container on `at`, if it can
    /// start: the item is there, and the container within reach.
    pub(crate) fn put_ticks(&self, id: ActorId, at: TilePos, slot: u16, def: u16) -> Option<u32> {
        let a = self.actors.get(id)?;
        let item = a.items.get(usize::from(slot)).filter(|i| i.def == def)?;
        if !self.is_container(at) || !within_reach(&self.grid, a.pos, at) {
            return None;
        }
        let kg = self.defs.weight(item) / 1000;
        Some(self.defs.take + self.defs.take_per_kg * kg)
    }

    /// Puts the stack at `slot` of your pack into the container on `at` (the action's end). Into
    /// your stash, a find is brought home: it pays its worth, once.
    pub(crate) fn put_away(&mut self, id: ActorId, at: TilePos, slot: u16, def: u16) {
        if self.put_ticks(id, at, slot, def).is_none() {
            return;
        }
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let mut item = a.items.remove(usize::from(slot)).loose();
        let home = a.stash == Some(at);
        self.events.push(Event::Put {
            who: id,
            item: def,
            count: item.count,
        });
        if home && item.found {
            item.found = false;
            let worth = self.defs.worth(&item);
            let xp = worth.saturating_mul(u32::from(item.count));
            self.events.push(Event::BroughtHome {
                who: id,
                item: def,
                count: item.count,
                xp,
            });
            self.gain_deeds(id, xp);
        }
        let mut held = self.store(at);
        self.defs.stow(&mut held, item);
        self.stores.insert(at, held);
    }

    /// Adds deed XP; a level gained brings its health at once.
    pub fn gain_deeds(&mut self, id: ActorId, xp: u32) {
        let d = self.rules.deeds;
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let before = d.level(a.deeds);
        a.deeds = a.deeds.saturating_add(xp);
        let after = d.level(a.deeds);
        if after > before {
            a.body.health += i32::from(after - before) * d.health;
            self.events.push(Event::LevelUp {
                who: id,
                level: after,
            });
        }
    }
}

impl Item {
    /// An item as it comes out of the world: a find, worth XP once it's brought home.
    pub fn found(def: u16, count: u16) -> Item {
        Item {
            found: true,
            ..Item::new(def, count)
        }
    }
}
