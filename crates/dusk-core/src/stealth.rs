//! Sneaking (`CONCEPT.md`, senses: stealth is being what they can't sense, and getting behind
//! them for a sneak attack).
//!
//! - **Sneaking** is a stance a character takes and leaves (`World::sneak`), not an action: no
//!   running, a slower step (quicker with Stealth), footsteps that carry a fraction as far, and
//!   the dead see you only a fraction as far off. Right beside them, or within their darkvision,
//!   they see you as ever.
//! - **A sneak attack:** a blow on one of the dead that's after nothing, from outside its sight
//!   cone, does many times the harm, more with Stealth. Sneaking or not: what counts is that it
//!   doesn't know you're there.
//! - **Stealth** grows by real use: a sneak attack that lands, and a tile sneaked with the dead
//!   near and none of them after you.

use crate::actor::{Actor, ActorId, Alert};
use crate::skills::{On, Skill};
use crate::world::{Event, World};

/// Sneaking's numbers, per mille: what's left of your walking pace, of your footsteps' reach, and
/// of how far off the dead see you; a sneak attack's harm; and how near the dead must be for a
/// sneaked tile to teach Stealth (tiles).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sneak {
    pub pace: u32,
    pub noise: u32,
    pub seen: u32,
    pub harm: u32,
    pub near: i32,
}

impl Default for Sneak {
    fn default() -> Sneak {
        Sneak {
            pace: 500,
            noise: 400,
            seen: 500,
            harm: 3000,
            near: 8,
        }
    }
}

impl World {
    /// Takes up sneaking (`on`), or leaves it: a living character's stance.
    pub fn sneak(&mut self, id: ActorId, on: bool) {
        let Some(a) = self
            .actors
            .get_mut(id)
            .filter(|a| !a.dead && a.kind.is_character())
        else {
            return;
        };
        if a.sneaking != on {
            a.sneaking = on;
            self.events.push(Event::Sneaking { who: id, on });
        }
    }

    pub fn is_sneaking(&self, id: ActorId) -> bool {
        self.actors.get(id).is_some_and(|a| a.sneaking)
    }

    /// What's left of `id`'s pace for sneaking, per mille: Stealth quickens it.
    pub(crate) fn sneak_pace(&self, id: ActorId) -> u32 {
        if !self.is_sneaking(id) {
            return 1000;
        }
        let level = u32::from(self.skill_level(id, Skill::Stealth));
        (self.rules.sneak.pace + level * self.rules.skills.sneak_pace).min(1000)
    }

    /// Whether creature `who` makes out character `c`, `d2` off (squared), sneaking: only within
    /// what's left of its sight sneaking, or within its darkvision, or right beside it.
    pub(crate) fn spots(&self, who: ActorId, c: &Actor) -> bool {
        if !c.sneaking {
            return true;
        }
        let (Some(a), Some(kind)) = (self.actors.get(who), self.stats(who)) else {
            return true;
        };
        let steps = a.pos.steps_to(c.pos);
        let near = i64::from(kind.sight) * i64::from(self.rules.sneak.seen) / 1000;
        steps <= kind.darkvision.max(1) || a.pos.dist2(c.pos) <= near * near
    }

    /// Whether `id` strikes `target` unawares: one of the dead that's after nothing, `id` outside
    /// its sight cone. One that's turned knows well enough who's there (`faith.rs`).
    pub(crate) fn unawares(&self, id: ActorId, target: ActorId) -> bool {
        let (Some(a), Some(t)) = (self.actors.get(id), self.actors.get(target)) else {
            return false;
        };
        if !a.kind.is_character() || t.kind.is_character() || t.alert() != Alert::Unaware {
            return false;
        }
        if matches!(t.mind, crate::actor::Mind::Turned { .. }) {
            return false;
        }
        let Some(kind) = self.stats(target) else {
            return false;
        };
        !kind
            .cone
            .contains(t.facing, a.pos.x - t.pos.x, a.pos.y - t.pos.y)
    }

    /// A sneak attack's harm for `id`, per mille: Stealth adds to it.
    pub(crate) fn sneak_harm(&self, id: ActorId) -> u32 {
        let level = u32::from(self.skill_level(id, Skill::Stealth));
        self.rules.sneak.harm + level * self.rules.skills.sneak_harm
    }

    /// A tile sneaked: it trains Stealth with the dead near and none of them after you.
    pub(crate) fn train_sneak(&mut self, id: ActorId) {
        let near = self.rules.sneak.near;
        let Some(a) = self.actors.get(id) else {
            return;
        };
        let (pos, kind) = (a.pos, a.kind);
        let r2 = i64::from(near).pow(2);
        let dead_near = self
            .actors
            .iter()
            .any(|(_, o)| !o.dead && o.kind.hostile_to(kind) && o.pos.dist2(pos) <= r2);
        if !dead_near || self.dead_after(id, near) {
            return;
        }
        let spot = self.rules.skills.spot.max(1);
        let on = On::Spot(
            pos.x.div_euclid(spot),
            pos.y.div_euclid(spot),
            i32::from(pos.z),
        );
        self.train(id, Skill::Stealth, self.rules.skills.sneak, on);
    }
}
