//! Wet and cold (`CONCEPT.md`, the senses: "Heat: your body, your clothes (wet means cold ... on
//! the way to hypothermia), your fires"; and the weather's moodles: wet, cold).
//!
//! - **Wet** (`Body::wet`, out of 100): out in the rain you get wet, faster in heavy rain, and
//!   wading the shallows soaks you. Out of it you dry: slowly in the open, faster under a roof,
//!   fast by a fire.
//! - **Cold** (`Body::chill`, out of 100) moves a game minute at a time toward what the moment is
//!   worth, as dread does: the night out of doors, the wet on you, the rain falling on you; less
//!   under a roof and asleep in a bed, and less for what you wear (each piece's `warmth`, half
//!   of it soaked through). By a lit fire, nothing: it warms you through, and dries you.
//! - **What it does:** chilly is a warning. Cold, you shiver: your aim is worse (a bow's too),
//!   and everything costs more breath. Freezing, worse again: nothing heals and health drains
//!   (hypothermia), and there's no sleeping: the cold wakes you.

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::grid::Ground;
use crate::light::FULL_LIGHT;
use crate::needs::{Woke, level};
use crate::senses::line_of_sight;
use crate::weather::DRY;
use crate::world::{Event, World};

/// Wet and cold's numbers, in points out of 100 (thousandths); the rates are a game minute's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cold {
    /// Wet: out in light and heavy rain, and wading; dried in the open (when it's dry), under a
    /// roof, and by a fire.
    pub soak: [i32; 2],
    pub wade: i32,
    pub dry: i32,
    pub dry_roof: i32,
    pub dry_fire: i32,
    /// What the cold is worth: the night out of doors (by how dark the sky is), soaked through
    /// (by how wet), and the rain on you (light, heavy); and what a roof, and a bed asleep, take
    /// off it.
    pub night: i32,
    pub wet: i32,
    pub rain: [i32; 2],
    pub roof: i32,
    pub bed: i32,
    /// Per mille of what you wear's warmth that's left, soaked through (less, the wetter).
    pub soaked: u32,
    /// Tiles from a lit fire, in sight of it, that it warms.
    pub fire: i32,
    /// How far cold moves toward what it's worth: colder, warmer, and warmer by a fire.
    pub rise: i32,
    pub fall: i32,
    pub fire_fall: i32,
    /// Both moodles' levels (damp, wet, soaked; chilly, cold, freezing). At cold and freezing,
    /// the aim left and what everything costs in breath (per mille); freezing, the health it
    /// costs a game hour (out of `body::FULL`).
    pub levels: [i32; 3],
    pub aim: [u32; 2],
    pub breath: [u32; 2],
    pub harm: i32,
}

impl Default for Cold {
    fn default() -> Cold {
        Cold {
            soak: [2_000, 5_000],
            wade: 10_000,
            dry: 500,
            dry_roof: 1_000,
            dry_fire: 5_000,
            night: 30_000,
            wet: 40_000,
            rain: [10_000, 20_000],
            roof: 20_000,
            bed: 20_000,
            soaked: 500,
            fire: 3,
            rise: 500,
            fall: 1_000,
            fire_fall: 3_000,
            levels: [25_000, 50_000, 75_000],
            aim: [850, 700],
            breath: [1150, 1300],
            harm: 30,
        }
    }
}

/// Wet or cold's moodle level at the top: freezing.
pub const FREEZING: u8 = 3;

/// `n` moved toward `to` by at most `by`.
fn toward(n: i32, to: i32, by: i32) -> i32 {
    if to > n {
        (n + by).min(to)
    } else {
        (n - by).max(to)
    }
}

impl World {
    /// How wet `id` is, and how cold, out of 100 points in thousandths (0 for the dead).
    pub fn wet(&self, id: ActorId) -> i32 {
        self.actors
            .get(id)
            .filter(|a| !a.dead)
            .map_or(0, |a| a.body.wet)
    }

    pub fn chill(&self, id: ActorId) -> i32 {
        self.actors
            .get(id)
            .filter(|a| !a.dead)
            .map_or(0, |a| a.body.chill)
    }

    /// The moodles' levels: 0 (dry, warm) to 3 (soaked, freezing).
    pub fn wet_level(&self, id: ActorId) -> u8 {
        level(self.wet(id), self.rules.cold.levels)
    }

    pub fn cold_level(&self, id: ActorId) -> u8 {
        level(self.chill(id), self.rules.cold.levels)
    }

    /// Whether a lit fire warms `at`: one within the rules' tiles, in sight of it.
    pub fn by_fire(&self, at: TilePos) -> bool {
        let r2 = i64::from(self.rules.cold.fire).pow(2);
        self.hearth_fires()
            .any(|(f, _)| f.z == at.z && f.dist2(at) <= r2 && line_of_sight(&self.grid, at, f))
    }

    /// How warm what `id` wears keeps it, wet as it is: all its pieces' warmth dry, the rules'
    /// share of it soaked through.
    fn worn_warmth(&self, id: ActorId, wet: i32) -> i32 {
        let all: i32 = self.pieces(id).iter().map(|p| p.warmth).sum();
        let lost = i64::from(1000 - self.rules.cold.soaked.min(1000)) * i64::from(wet) / 100_000;
        (i64::from(all) * (1000 - lost) / 1000) as i32
    }

    /// Sets how wet and how cold a character is, in thousandths of a point (at most 100): for
    /// tests and tools, as `set_build` is. In the game they only ever drift.
    pub fn set_weathered(&mut self, id: ActorId, wet: i32, chill: i32) {
        if let Some(a) = self.actors.get_mut(id) {
            a.body.wet = wet.clamp(0, 100_000);
            a.body.chill = chill.clamp(0, 100_000);
        }
    }

    /// What the cold is worth to `id` now, where its cold is heading: the night out of doors, the
    /// wet on it and the rain on it, less a roof, a bed and what it wears. Nothing by a fire.
    pub fn cold_due(&self, id: ActorId) -> i32 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        let c = self.rules.cold;
        if self.by_fire(a.pos) {
            return 0;
        }
        let dark = FULL_LIGHT.saturating_sub(self.daylight()) as i64;
        let mut due = i64::from(c.night) * dark / i64::from(FULL_LIGHT)
            + i64::from(c.wet) * i64::from(a.body.wet) / 100_000;
        let rain = self.rain();
        if !self.outdoors(a.pos) {
            due -= i64::from(c.roof);
        } else if rain != DRY {
            due += i64::from(c.rain[usize::from(rain) - 1]);
        }
        if a.body.asleep && a.body.in_bed {
            due -= i64::from(c.bed);
        }
        due -= i64::from(self.worn_warmth(id, a.body.wet));
        due.clamp(0, 100_000) as i32
    }

    /// A swing's (or a shot's) chance to land, out of 10,000, as the cold shakes `id`.
    pub(crate) fn numbed(&self, id: ActorId, hit: u32) -> u32 {
        match self.cold_level(id) {
            0 | 1 => hit,
            l => hit * self.rules.cold.aim[usize::from(l) - 2] / 1000,
        }
    }

    /// Per mille: what the cold makes every cost in breath come to (`endurance_rate`).
    pub(crate) fn cold_cost(&self, id: ActorId) -> u32 {
        match self.cold_level(id) {
            0 | 1 => 1000,
            l => self.rules.cold.breath[usize::from(l) - 2],
        }
    }

    /// Each game minute: the characters get wetter or drier, then colder or warmer. A new level
    /// of either is news, and freezing wakes the sleeper.
    pub(crate) fn chills(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        let (c, rain) = (self.rules.cold, self.rain());
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
                continue;
            };
            let (pos, wet, chill, asleep) = (a.pos, a.body.wet, a.body.chill, a.body.asleep);
            let (fire, outdoors) = (self.by_fire(pos), self.outdoors(pos));
            let soak = if self.grid.tile(pos).ground == Ground::Shallows {
                c.wade
            } else if outdoors && rain != DRY {
                c.soak[usize::from(rain) - 1]
            } else {
                0
            };
            let dry = match (soak > 0, fire, outdoors) {
                (true, _, _) => 0,
                (_, true, _) => c.dry_fire,
                (_, _, false) => c.dry_roof,
                _ => c.dry,
            };
            let (was_wet, was_cold) = (self.wet_level(id), self.cold_level(id));
            if let Some(a) = self.actors.get_mut(id) {
                a.body.wet = (wet + soak - dry).clamp(0, 100_000);
            }
            // Wet as it is now, the cold moves toward what it's worth.
            let due = self.cold_due(id);
            let rate = if fire {
                c.fire_fall
            } else if due > chill {
                c.rise
            } else {
                c.fall
            };
            if let Some(a) = self.actors.get_mut(id) {
                a.body.chill = toward(chill, due, rate);
            }
            let (wet_now, cold_now) = (self.wet_level(id), self.cold_level(id));
            if wet_now != was_wet {
                self.events.push(Event::Wet {
                    who: id,
                    level: wet_now,
                });
            }
            if cold_now != was_cold {
                self.events.push(Event::Cold {
                    who: id,
                    level: cold_now,
                });
            }
            if asleep && cold_now >= FREEZING {
                self.wake(id, Woke::Cold);
            }
        }
    }
}
