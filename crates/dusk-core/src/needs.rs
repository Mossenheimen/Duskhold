//! The clock, needs, sleep, healing and moodles (`CONCEPT.md`: needs make plans, not chores).
//!
//! - **The clock** runs 24× faster than action time: a game minute is 25 ticks, a game day an
//!   hour at 1×. Needs, healing and sleep run on it, a game minute at a time.
//! - **Needs** (hunger, thirst, fatigue) rise out of 100. Each has three levels, and each level
//!   one clear effect: hungry or thirsty, endurance comes back slower; starving or dying of
//!   thirst, health drains; tired, swings wind up slower. Consequences come on gradually.
//! - **Sleep** (only when tired, and not with the dead about) lets fatigue fall: fully in a bed,
//!   half as fast on the ground. A blow, a grab, a noise that reaches you, or being rested wakes
//!   you; so does choosing to get up.
//! - **Healing:** fed, watered and not bleeding, health comes back slowly (three times as fast
//!   asleep), and wounds close once they've had their time.
//! - **Moodles** say all of this at a glance: Zomboid's column, for the client and later the bot.
//!   Pain and dread have theirs too (`dread.rs`); in agony, there's no sleep.

use crate::actor::{ActorId, ActorKind, Alert, Intent};
use crate::body::Group;
use crate::geom::{Dir8, TilePos};
use crate::grid::Ground;
use crate::items::within_reach;
use crate::world::{Event, World};

/// The world's clock: a game minute is `ticks_per_minute` ticks, and day 1 begins `start`
/// minutes after midnight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    pub ticks_per_minute: u32,
    pub start: u32,
}

/// Hunger or thirst, in thousandths of a point (100 points is the worst): how much it rises a
/// game minute, its three levels, how fast endurance comes back from its second level on (per
/// mille), and the health it costs a game hour at its third (in health's own units, out of
/// `body::FULL`: health moves too slowly to count it by the minute).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Need {
    pub rise: i32,
    pub levels: [i32; 3],
    pub recovery: u32,
    pub harm: i32,
}

/// Fatigue: how much it rises a game minute awake and falls asleep in a bed (half that on the
/// ground), its three levels, and how much longer wind-ups take at its second and third (per
/// mille: 1250 is a quarter longer).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fatigue {
    pub rise: i32,
    pub rest: i32,
    pub levels: [i32; 3],
    pub slow: [u32; 2],
}

/// The needs and what goes with them: sleep and healing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Needs {
    pub hunger: Need,
    pub thirst: Need,
    pub fatigue: Fatigue,
    /// Asleep, hunger and thirst rise this much as fast (per mille).
    pub asleep: u32,
    /// Health back a game hour (out of `body::FULL`), fed, watered and not bleeding; asleep,
    /// `sleep_heal` times as much.
    pub heal: i32,
    pub sleep_heal: i32,
    /// Game minutes each kind of wound takes to close once it's stopped bleeding (by
    /// `WoundKind::ALL`).
    pub wound_heal: [u32; 3],
    /// Ticks to lie down, and to drink your fill from water beside you.
    pub lie_down: u32,
    pub drink_water: u32,
    /// No sleep with one of the dead after you within this many tiles.
    pub sleep_clear: i32,
}

/// What the moodle column shows. Each has a level from 1 (mild) up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Moodle {
    Hungry,
    Thirsty,
    Tired,
    Injured,
    Bleeding,
    OutOfBreath,
    HeavyLoad,
    Held,
    Asleep,
    /// Ghoul fever, by its stage (`fever.rs`). After the others, so they keep their numbers.
    Fever,
    /// Pain from wounds, and dread (`dread.rs`): after the fever, for the same reason.
    Pain,
    Dread,
    /// A hurt arm, leg or head (`injury.rs`): hurt, then badly hurt.
    Arm,
    Leg,
    Head,
    /// Sneaking (`stealth.rs`): a stance, not a trouble.
    Sneaking,
}

/// Why someone woke.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Woke {
    Rested,
    Chose,
    Hurt,
    Noise,
}

/// Why you can't sleep now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoSleep {
    NotTired,
    DeadAbout,
    Held,
    /// In agony (`dread.rs`): dress the wounds first.
    Pain,
}

/// How many of a need's levels a value has reached: 0 to 3.
pub fn level(value: i32, levels: [i32; 3]) -> u8 {
    levels.iter().filter(|&&l| value >= l).count() as u8
}

impl World {
    /// Game minutes since midnight of day 1.
    pub fn clock_minutes(&self) -> u64 {
        let c = self.rules.clock;
        u64::from(c.start) + self.tick / u64::from(c.ticks_per_minute.max(1))
    }

    /// The day (from 1), the hour and the minute.
    pub fn clock(&self) -> (u64, u32, u32) {
        let m = self.clock_minutes();
        (m / 1440 + 1, (m % 1440 / 60) as u32, (m % 60) as u32)
    }

    /// The moodles a character has now, in the column's order, each with its level.
    pub fn moodles(&self, id: ActorId) -> Vec<(Moodle, u8)> {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return Vec::new();
        };
        let (n, b) = (self.rules.needs, &a.body);
        let full = self.max_health(id).max(1);
        let hurt = 3 - (b.health * 4 / full).clamp(0, 3) as u8;
        let bleeding = b.wounds.iter().filter(|w| w.bleeding > 0).count().min(3) as u8;
        let holders = if a.kind == ActorKind::Character {
            self.holders(id).len().min(3) as u8
        } else {
            0
        };
        [
            (Moodle::Hungry, level(b.hunger, n.hunger.levels)),
            (Moodle::Thirsty, level(b.thirst, n.thirst.levels)),
            (Moodle::Tired, level(b.fatigue, n.fatigue.levels)),
            (Moodle::Injured, hurt),
            (Moodle::Bleeding, bleeding),
            (Moodle::Pain, self.pain_level(id)),
            (Moodle::Arm, self.hurt_level(id, Group::Arms)),
            (Moodle::Leg, self.hurt_level(id, Group::Legs)),
            (Moodle::Head, self.hurt_level(id, Group::Head)),
            (Moodle::Fever, self.fever_stage(id)),
            (Moodle::Dread, self.dread_level(id)),
            (Moodle::OutOfBreath, u8::from(b.exhausted)),
            (Moodle::HeavyLoad, u8::from(self.weighed_down(id))),
            (Moodle::Held, holders),
            (Moodle::Asleep, u8::from(b.asleep)),
            (Moodle::Sneaking, u8::from(a.sneaking)),
        ]
        .into_iter()
        .filter(|&(_, l)| l > 0)
        .collect()
    }

    /// How fast a character gets its breath back, per mille: slower hungry and thirsty.
    pub(crate) fn recovery(&self, id: ActorId) -> u32 {
        let Some(a) = self.actors.get(id) else {
            return 1000;
        };
        let n = self.rules.needs;
        let mut rate = 1000;
        if level(a.body.hunger, n.hunger.levels) >= 2 {
            rate = rate * n.hunger.recovery / 1000;
        }
        if level(a.body.thirst, n.thirst.levels) >= 2 {
            rate = rate * n.thirst.recovery / 1000;
        }
        if self.fever_stage(id) >= 2 {
            rate = rate * self.rules.fever.recovery / 1000;
        }
        rate
    }

    /// How much longer a character's wind-ups take for tiredness, per mille.
    pub(crate) fn tired_slow(&self, id: ActorId) -> u32 {
        let f = self.rules.needs.fatigue;
        match self
            .actors
            .get(id)
            .map_or(0, |a| level(a.body.fatigue, f.levels))
        {
            0 | 1 => 1000,
            2 => f.slow[0],
            _ => f.slow[1],
        }
    }

    /// Whether `id` can lie down to sleep now, and if so whether there's a bed beside it.
    pub fn can_sleep(&self, id: ActorId) -> Result<bool, NoSleep> {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return Err(NoSleep::NotTired);
        };
        let n = self.rules.needs;
        if self.is_held(id) {
            return Err(NoSleep::Held);
        }
        if a.body.fatigue < n.fatigue.levels[0] {
            return Err(NoSleep::NotTired);
        }
        if self.pain_level(id) >= 3 {
            return Err(NoSleep::Pain);
        }
        // On hallowed ground the dead outside can't come in, so they don't keep you awake.
        if self.dead_after(id, n.sleep_clear) && !self.is_hallowed(a.pos) {
            return Err(NoSleep::DeadAbout);
        }
        Ok(self.bed_beside(a.pos))
    }

    /// Whether one of the dead is after `id` within `range` tiles: after something it heard or
    /// saw, that near. No sleep then, and running trains Athletics (`skills.rs`).
    pub(crate) fn dead_after(&self, id: ActorId, range: i32) -> bool {
        let Some(a) = self.actors.get(id) else {
            return false;
        };
        let r2 = i64::from(range).pow(2);
        self.actors.iter().any(|(_, o)| {
            !o.dead
                && o.kind.hostile_to(a.kind)
                && o.alert() != Alert::Unaware
                && o.pos.dist2(a.pos) <= r2
        })
    }

    /// Whether a bed is within reach of a tile.
    fn bed_beside(&self, pos: TilePos) -> bool {
        Dir8::ALL.iter().any(|&d| {
            let at = pos.step(d);
            self.defs.is_bed(self.grid.tile(at).fixture) && within_reach(&self.grid, pos, at)
        })
    }

    /// Water within reach of a tile: a side of it that's a pond, a stream, a well...
    pub fn water_beside(&self, pos: TilePos) -> Option<TilePos> {
        // The shallows you stand in, else water beside you, deep or shallow, or a well.
        if self.grid.tile(pos).ground == Ground::Shallows {
            return Some(pos);
        }
        Dir8::ALL.iter().map(|&d| pos.step(d)).find(|&at| {
            let tile = self.grid.tile(at);
            (matches!(tile.ground, Ground::Water | Ground::Shallows)
                || self.defs.is_well(tile.fixture))
                && within_reach(&self.grid, pos, at)
        })
    }

    /// Falls asleep (the sleep action's end).
    pub(crate) fn fall_asleep(&mut self, id: ActorId) {
        let Ok(in_bed) = self.can_sleep(id) else {
            return;
        };
        if let Some(a) = self.actors.get_mut(id) {
            a.body.asleep = true;
            a.body.in_bed = in_bed;
            a.intent = Intent::Idle;
            self.events.push(Event::FellAsleep { who: id, in_bed });
        }
    }

    /// Wakes `id`, if it's asleep.
    pub(crate) fn wake(&mut self, id: ActorId, why: Woke) {
        if let Some(a) = self.actors.get_mut(id)
            && a.body.asleep
        {
            a.body.asleep = false;
            a.body.in_bed = false;
            self.events.push(Event::WokeUp { who: id, why });
            // Slept the night through: this is home now.
            if why == Woke::Rested {
                // Slept the night through: the second wind comes back (`class.rs`).
                if let Some(a) = self.actors.get_mut(id) {
                    a.winded = false;
                }
                self.make_home(id);
            }
        }
    }

    /// Drinks your fill from the water beside you (the action's end).
    pub(crate) fn drink_water(&mut self, id: ActorId) {
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        if self.water_beside(pos).is_none() {
            return;
        }
        if let Some(a) = self.actors.get_mut(id) {
            a.body.thirst = 0;
            self.events.push(Event::DrankWater { who: id });
        }
    }

    /// Each game minute: the characters' needs rise (or fatigue falls, asleep), wounds close,
    /// and the rested wake. On the hour, starving and thirst cost health and the fed and watered
    /// heal. A fever makes you thirstier and stops the healing, and dying of it costs health.
    pub(crate) fn needs(&mut self) {
        let (n, fever) = (self.rules.needs, self.rules.fever);
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        // A fever that's run its course kills first: the day's end, not its last hour's harm.
        self.fevers();
        let on_the_hour = self.clock_minutes().is_multiple_of(60);
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            let full = self.max_health(id);
            let stage = self.fever_stage(id);
            let Some(a) = self.actors.get_mut(id) else {
                continue;
            };
            if a.dead {
                continue;
            }
            let b = &mut a.body;
            let pace = if b.asleep { n.asleep as i32 } else { 1000 };
            let feverish = if stage > 0 { fever.thirst as i32 } else { 1000 };
            b.hunger = (b.hunger + n.hunger.rise * pace / 1000).clamp(0, 100 * 1000);
            b.thirst =
                (b.thirst + n.thirst.rise * pace / 1000 * feverish / 1000).clamp(0, 100 * 1000);
            b.fatigue = if b.asleep {
                let rest = if b.in_bed {
                    n.fatigue.rest
                } else {
                    n.fatigue.rest / 2
                };
                (b.fatigue - rest).max(0)
            } else {
                (b.fatigue + n.fatigue.rise).min(100 * 1000)
            };
            let starving = b.hunger >= n.hunger.levels[2];
            let parched = b.thirst >= n.thirst.levels[2];
            if on_the_hour {
                if starving {
                    b.health -= n.hunger.harm;
                }
                if parched {
                    b.health -= n.thirst.harm;
                }
                if stage >= crate::fever::DYING {
                    b.health -= fever.harm;
                }
                if !starving && !parched && !b.bleeding() && stage == 0 {
                    let heal = n.heal * if b.asleep { n.sleep_heal } else { 1 };
                    b.health = (b.health + heal).min(full);
                }
            }
            for w in &mut b.wounds {
                if w.bleeding == 0 {
                    w.heal_left = w.heal_left.saturating_sub(1);
                }
            }
            b.wounds.retain(|w| w.bleeding > 0 || w.heal_left > 0);
            let rested = b.asleep && b.fatigue == 0;
            let dead = b.health <= 0;
            if dead {
                self.kill(id);
            } else if rested {
                self.wake(id, Woke::Rested);
            }
        }
    }
}
