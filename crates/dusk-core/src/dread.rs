//! Pain and dread (`CONCEPT.md`, body, moodles and afflictions: each injury has one clear effect
//! and one treatment, and dread is the one stat for the mind).
//!
//! - **Pain** comes from wounds. Each hurts by its kind, half as much dressed (a bandage on it),
//!   and less the nearer it is to closing. Nothing's kept: it's worked out from the wounds when
//!   asked. In pain, everything you do costs more breath; in agony, more again, and you can't
//!   sleep.
//! - **Dread** is kept, out of 100. Each game minute it moves toward what the moment is worth:
//!   the dark, each of the dead near you (seen, or after something), your pain, your hunger. It
//!   rises fast and falls slowly. A light takes the dark's share away. Asleep, or on hallowed
//!   ground among the sisters, it ebbs away to nothing.
//! - **What dread does:** uneasy, your aim shakes; afraid, it shakes more; frightened, most, and
//!   you can't make yourself step nearer the dead (D&D's frightened).

use crate::actor::{ActorId, Alert};
use crate::body::{Part, Wound};
use crate::geom::TilePos;
use crate::needs::level;
use crate::senses::line_of_sight;
use crate::world::{Event, World};

/// Pain's numbers, in points out of 100 (thousandths): what a fresh wound of each kind hurts (by
/// `WoundKind::ALL`), the share of that a dressed wound still hurts (per mille), the moodle's
/// three levels, and what endurance costs at the second and the third (per mille).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pain {
    pub wound: [i32; 3],
    pub dressed: u32,
    pub levels: [i32; 3],
    pub cost: [u32; 2],
}

impl Default for Pain {
    fn default() -> Pain {
        Pain {
            wound: [10_000, 20_000, 30_000],
            dressed: 500,
            levels: [10_000, 30_000, 60_000],
            cost: [1250, 1500],
        }
    }
}

/// Dread's numbers, in points out of 100 (thousandths).
///
/// What the moment is worth: `dark` with your tile lit below `dark_below` (out of 100);
/// `per_dead` for each of the dead within `near` tiles that you see or that's after something,
/// up to `most` of them; `per_pain` a level of pain; `hungry` a level of hunger from the second.
/// How far it moves toward that a game minute: `rise` up, `fall` down, and `calm` down to nothing
/// asleep or on hallowed ground. The moodle's three levels, and the aim each leaves (per mille).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dread {
    pub dark_below: u32,
    pub dark: i32,
    pub near: i32,
    pub per_dead: i32,
    pub most: u32,
    pub per_pain: i32,
    pub hungry: i32,
    pub rise: i32,
    pub fall: i32,
    pub calm: i32,
    pub levels: [i32; 3],
    pub aim: [u32; 3],
}

impl Default for Dread {
    fn default() -> Dread {
        Dread {
            dark_below: 25,
            dark: 30_000,
            near: 8,
            per_dead: 20_000,
            most: 3,
            per_pain: 10_000,
            hungry: 10_000,
            rise: 6_000,
            fall: 1_000,
            calm: 3_000,
            levels: [25_000, 50_000, 75_000],
            aim: [900, 800, 700],
        }
    }
}

/// The level at which dread frightens you.
pub const FRIGHTENED: u8 = 3;

impl World {
    /// `id`'s pain now, in thousandths of a point: each of its wounds' (`wound_pain`).
    pub fn pain(&self, id: ActorId) -> i32 {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return 0;
        };
        a.body.wounds.iter().map(|w| self.wound_pain(w)).sum()
    }

    /// What a wound hurts, in thousandths of a point: by its kind, half that dressed, less the
    /// nearer it is to closing, and more on the torso (`injury.rs`).
    pub(crate) fn wound_pain(&self, w: &Wound) -> i32 {
        let (p, heal) = (self.rules.pain, self.rules.needs.wound_heal);
        let k = w.kind as usize;
        let fresh = i64::from(p.wound[k]);
        // How much of its healing is still to come, per mille: all of it while it bleeds.
        let left = (i64::from(w.heal_left) * 1000 / i64::from(heal[k].max(1))).min(1000);
        let dressed = if w.bandaged {
            i64::from(p.dressed)
        } else {
            1000
        };
        let torso = if w.part == Part::Torso {
            i64::from(self.rules.injury.torso)
        } else {
            1000
        };
        (fresh * left / 1000 * dressed / 1000 * torso / 1000) as i32
    }

    /// Pain's moodle level: 0 (none) to 3 (agony).
    pub fn pain_level(&self, id: ActorId) -> u8 {
        level(self.pain(id), self.rules.pain.levels)
    }

    /// What `id`'s endurance costs come to for its pain, per mille.
    pub(crate) fn pain_cost(&self, id: ActorId) -> u32 {
        match self.pain_level(id) {
            0 | 1 => 1000,
            2 => self.rules.pain.cost[0],
            _ => self.rules.pain.cost[1],
        }
    }

    /// `id`'s dread, out of 100 points in thousandths.
    pub fn dread(&self, id: ActorId) -> i32 {
        self.actors
            .get(id)
            .filter(|a| !a.dead)
            .map_or(0, |a| a.body.dread)
    }

    /// Dread's moodle level: 0 (calm) to 3 (frightened).
    pub fn dread_level(&self, id: ActorId) -> u8 {
        level(self.dread(id), self.rules.dread.levels)
    }

    /// A swing's chance to land (out of 10,000), as `id`'s dread shakes its aim.
    pub(crate) fn shaken(&self, id: ActorId, hit: u32) -> u32 {
        match self.dread_level(id) {
            0 => hit,
            l => hit * self.rules.dread.aim[usize::from(l) - 1] / 1000,
        }
    }

    /// Where the dead `id` dreads are: alive, within `near` tiles, and in its sight (a line of
    /// sight, lit or not: you know what's there) or after something.
    fn dreaded(&self, id: ActorId) -> Vec<TilePos> {
        let Some(a) = self.actors.get(id) else {
            return Vec::new();
        };
        let (pos, kind) = (a.pos, a.kind);
        let r2 = i64::from(self.rules.dread.near).pow(2);
        self.actors
            .iter()
            .filter(|(_, o)| {
                !o.dead
                    && o.kind.hostile_to(kind)
                    && o.pos.z == pos.z
                    && o.pos.dist2(pos) <= r2
                    && (o.alert() != Alert::Unaware || line_of_sight(&self.grid, pos, o.pos))
            })
            .map(|(_, o)| o.pos)
            .collect()
    }

    /// What the moment is worth in dread to `id`: the dark, the dead near it, its pain and its
    /// hunger.
    fn dread_due(&self, id: ActorId) -> i32 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        let d = self.rules.dread;
        let dark = if self.light_at(a.pos) < d.dark_below {
            d.dark
        } else {
            0
        };
        let dead = self.dreaded(id).len().min(d.most as usize) as i32;
        let pain = i32::from(self.pain_level(id));
        let hunger = level(a.body.hunger, self.rules.needs.hunger.levels);
        let hungry = i32::from(hunger.saturating_sub(1));
        (dark + d.per_dead * dead + d.per_pain * pain + d.hungry * hungry).min(100_000)
    }

    /// The one of the dead `id` is frightened of, if dread has frightened it: the nearest it
    /// dreads. It won't step nearer that one.
    pub(crate) fn fear(&self, id: ActorId) -> Option<TilePos> {
        if self.dread_level(id) < FRIGHTENED {
            return None;
        }
        let pos = self.actors.get(id)?.pos;
        self.dreaded(id).into_iter().min_by_key(|p| p.dist2(pos))
    }

    /// Each game minute: each character's dread moves toward what the moment is worth, and
    /// asleep or on hallowed ground ebbs away. A change of level is news.
    pub(crate) fn minds(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        let d = self.rules.dread;
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
                continue;
            };
            let (now, calm) = (a.body.dread, a.body.asleep || self.is_hallowed(a.pos));
            let (due, rate) = if calm {
                (0, d.calm)
            } else {
                let due = self.dread_due(id);
                (due, if due > now { d.rise } else { d.fall })
            };
            let next = if due > now {
                (now + rate).min(due)
            } else {
                (now - rate).max(due)
            };
            let was = self.dread_level(id);
            if let Some(a) = self.actors.get_mut(id) {
                a.body.dread = next;
            }
            let level = self.dread_level(id);
            if level != was {
                self.events.push(Event::Dread { who: id, level });
            }
        }
    }
}
