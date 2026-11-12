//! Strength and Fitness (`CONCEPT.md`, athletics): the body's two slow stats, grown over days of
//! real exertion, as Zomboid's are.
//!
//! - **What counts:** every point of breath spent counts toward the day's Fitness
//!   (`Body::spent`); blows that land, shoves and steps under a heavy load count toward its
//!   Strength (`Actor::strain`).
//! - **At midnight** the day's work turns into growth (`Growth`), the more the less each: a day
//!   gives half its most at `half`, and the higher the stat, the less a day adds. So a week of
//!   real use beats an hour of jumping on the spot.
//! - **What they do:** Strength carries more, hits harder and shoves down more often; Fitness
//!   makes every cost in breath less and brings it back faster. Everyone starts at 0, which is
//!   the game's balance; they rise to 10, and don't fall back.
//!
//! Both are kept in thousandths of a level, and what they do grows with every thousandth.

use crate::actor::{ActorId, ActorKind};
use crate::world::{Event, World};

/// The top of each stat, in thousandths of a level: level 10.
pub const TOP: u32 = 10_000;

/// How a day's work turns into a stat: at most `most` thousandths of a level a day, half of
/// that for `half` of work, and less the higher the stat already is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Growth {
    pub most: u32,
    pub half: u32,
}

impl Growth {
    /// What a day's `work` adds to a stat at `now`, in thousandths of a level.
    pub fn gain(self, work: u32, now: u32) -> u32 {
        let w = u64::from(work);
        let g = u64::from(self.most) * w / (w + u64::from(self.half.max(1)));
        (g * u64::from(TOP.saturating_sub(now)) / u64::from(TOP)) as u32
    }
}

/// rules.toml's [fitness], in the core's units: its growth (work in breath, out of
/// `body::FULL` a bar), and per mille a level, what's taken off every cost in breath and added
/// to how fast it comes back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FitnessRules {
    pub growth: Growth,
    pub cost: u32,
    pub recover: u32,
}

impl Default for FitnessRules {
    fn default() -> FitnessRules {
        FitnessRules {
            growth: Growth {
                most: 600,
                half: 15_000,
            },
            cost: 30,
            recover: 50,
        }
    }
}

/// rules.toml's [strength], in the core's units: its growth (work in strain, thousandths of a
/// point), the strain of a blow that lands (a heavy one, twice), a shove that does and a step
/// under a heavy load (over half what you can carry); and a level's worth: grams more you carry,
/// per mille more harm, and more chance out of 10,000 that a shove knocks down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrengthRules {
    pub growth: Growth,
    pub blow: u32,
    pub shove: u32,
    pub haul: u32,
    pub carry: u32,
    pub harm: u32,
    pub knock_down: u32,
}

impl Default for StrengthRules {
    fn default() -> StrengthRules {
        StrengthRules {
            growth: Growth {
                most: 600,
                half: 100_000,
            },
            blow: 1000,
            shove: 1000,
            haul: 500,
            carry: 1000,
            harm: 30,
            knock_down: 200,
        }
    }
}

/// Which of the two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Stat {
    Strength,
    Fitness,
}

/// `value` thousandths of a level at `per` a level.
fn by(value: u32, per: u32) -> u32 {
    (u64::from(value) * u64::from(per) / 1000) as u32
}

impl World {
    /// A character's Strength and Fitness, in thousandths of a level (0 for anyone else).
    pub fn strength(&self, id: ActorId) -> u32 {
        self.actors.get(id).map_or(0, |a| a.strength)
    }

    pub fn fitness(&self, id: ActorId) -> u32 {
        self.actors.get(id).map_or(0, |a| a.fitness)
    }

    /// Sets a character's Strength and Fitness, in thousandths of a level (at most 10): for
    /// tests and tools, as `creature_mut` is. In the game they're only ever grown.
    pub fn set_build(&mut self, id: ActorId, strength: u32, fitness: u32) {
        if let Some(a) = self.actors.get_mut(id) {
            a.strength = strength.min(TOP);
            a.fitness = fitness.min(TOP);
        }
    }

    /// What `id` can carry before it's weighed down, in grams: the data's, and more with
    /// Strength.
    pub fn carry(&self, id: ActorId) -> u32 {
        let s = self.rules.strength;
        self.defs.carry + by(self.strength(id), s.carry)
    }

    /// Work toward a character's Strength, today.
    pub(crate) fn strain(&mut self, id: ActorId, amount: u32) {
        if let Some(a) = self.actors.get_mut(id)
            && a.kind == ActorKind::Character
        {
            a.strain = a.strain.saturating_add(amount);
        }
    }

    /// A character's blow, as its Strength makes it.
    pub(crate) fn strong_harm(&self, id: ActorId, harm: i32) -> i32 {
        let more = by(self.strength(id), self.rules.strength.harm);
        (i64::from(harm) * i64::from(1000 + more) / 1000) as i32
    }

    /// What Strength adds to a shove's chance to knock down, out of 10,000.
    pub(crate) fn strength_knock_down(&self, id: ActorId) -> i32 {
        by(self.strength(id), self.rules.strength.knock_down) as i32
    }

    /// Per mille: what's left of a cost in breath with `id`'s Fitness (`endurance_rate`).
    pub(crate) fn fitness_cost(&self, id: ActorId) -> u32 {
        1000u32.saturating_sub(by(self.fitness(id), self.rules.fitness.cost))
    }

    /// Per mille: how fast `id`'s breath comes back with its Fitness (`recovery`).
    pub(crate) fn fitness_recovery(&self, id: ActorId) -> u32 {
        1000 + by(self.fitness(id), self.rules.fitness.recover)
    }

    /// Midnight: each living character's day of work turns into Strength and Fitness, and the
    /// day's count starts again. A whole level reached is news.
    pub(crate) fn grow(&mut self) {
        let (s, f) = (self.rules.strength.growth, self.rules.fitness.growth);
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            let Some(a) = self.actors.get_mut(id) else {
                continue;
            };
            if a.dead {
                continue;
            }
            let mut grew = Vec::new();
            let was = a.strength;
            a.strength = (was + s.gain(a.strain, was)).min(TOP);
            if a.strength / 1000 > was / 1000 {
                grew.push((Stat::Strength, a.strength / 1000));
            }
            let was = a.fitness;
            a.fitness = (was + f.gain(a.body.spent, was)).min(TOP);
            if a.fitness / 1000 > was / 1000 {
                grew.push((Stat::Fitness, a.fitness / 1000));
            }
            a.strain = 0;
            a.body.spent = 0;
            for (stat, level) in grew {
                self.events.push(Event::Grew {
                    who: id,
                    stat,
                    level: level as u8,
                });
            }
        }
    }
}
