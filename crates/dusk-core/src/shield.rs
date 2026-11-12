//! Shields (`CONCEPT.md`, the Fighter: Brace and Shield Bash). A shield is carried on the off
//! arm, where a light would be held up: taking one up puts out the light you held, and holding
//! up a light puts the shield in your pack.
//!
//! - **Bracing** is a stance, held: a slower step and no running, and a blow from ahead of you
//!   (the attacker in the half you face) may be blocked, by the shield's chance, more with the
//!   Shield skill. A blocked blow does nothing, no wound and no hold, but it costs breath and
//!   wears the shield, which breaks at nothing. Held, you can block nothing.
//! - **A shove with a shield on your arm is a bash:** it knocks down more often.
//! - **Shield,** the ninth skill, learns from blocks and bashes, and a level blocks more often.

use crate::actor::ActorId;
use crate::hash::{Purpose, below, roll};
use crate::items::{Item, ItemKind};
use crate::skills::{On, Skill};
use crate::world::{Event, World};

/// A shield's numbers, from its definition: the chance it blocks a blow from ahead while you
/// brace (out of 10,000), the breath a block costs (out of `body::FULL`), the condition a block
/// takes off it (out of `items::NEW`), and what it adds to a shove's chance to knock down (out of
/// 10,000).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guard {
    pub block: u32,
    pub endurance: i32,
    pub wear: u16,
    pub bash: u32,
}

impl World {
    /// The shield on `id`'s arm and its numbers, if it carries one.
    pub(crate) fn guard(&self, id: ActorId) -> Option<(Item, Guard)> {
        let shield = self.actors.get(id)?.shield?;
        match self.defs.item(shield.def)?.kind {
            ItemKind::Shield(g) => Some((shield, g)),
            _ => None,
        }
    }

    /// Braces behind a shield (`on`), or stops: a stance, as sneaking is.
    pub fn brace(&mut self, id: ActorId, on: bool) {
        let Some(a) = self
            .actors
            .get_mut(id)
            .filter(|a| !a.dead && a.kind.is_character())
        else {
            return;
        };
        if a.bracing != on {
            a.bracing = on;
            self.events.push(Event::Bracing { who: id, on });
        }
    }

    pub fn is_bracing(&self, id: ActorId) -> bool {
        self.actors.get(id).is_some_and(|a| a.bracing)
    }

    /// What's left of `id`'s pace for bracing, per mille.
    pub(crate) fn brace_pace(&self, id: ActorId) -> u32 {
        if self.is_bracing(id) {
            self.rules.shield_pace
        } else {
            1000
        }
    }

    /// Whether `id`, bracing behind a shield and not held, blocks a blow from `by`: it has to
    /// come from the half `id` faces, and the shield's chance (with Shield's) has to come up.
    pub(crate) fn blocks(&self, id: ActorId, by: ActorId) -> bool {
        let (Some(a), Some(b)) = (self.actors.get(id), self.actors.get(by)) else {
            return false;
        };
        if !a.bracing || self.is_held(id) {
            return false;
        }
        let Some((_, g)) = self.guard(id) else {
            return false;
        };
        let (fx, fy) = a.facing.delta();
        if fx * (b.pos.x - a.pos.x) + fy * (b.pos.y - a.pos.y) <= 0 {
            return false;
        }
        let level = u32::from(self.skill_level(id, Skill::Shield));
        let chance = g.block + level * self.rules.skills.shield_block;
        below(
            roll(self.seed, self.tick, id.index() as u64, Purpose::Block),
            10_000,
        ) < chance
    }

    /// A blow blocked: it costs breath, wears the shield (breaking it at nothing), and teaches
    /// Shield.
    pub(crate) fn block(&mut self, id: ActorId, by: ActorId) {
        let Some((mut shield, g)) = self.guard(id) else {
            return;
        };
        shield.condition = shield.condition.saturating_sub(g.wear);
        if let Some(a) = self.actors.get_mut(id) {
            a.body.spend(g.endurance);
            a.shield = (shield.condition > 0).then_some(shield);
        }
        self.events.push(Event::Blocked { who: id, by });
        if shield.condition == 0 {
            self.events.push(Event::WeaponBroke {
                who: id,
                item: shield.def,
            });
        }
        self.train(id, Skill::Shield, self.rules.skills.shield, On::Foe(by));
    }

    /// What a shield on `id`'s arm adds to a shove's chance to knock down (out of 10,000): a bash.
    pub(crate) fn bash_bonus(&self, id: ActorId) -> i32 {
        self.guard(id).map_or(0, |(_, g)| g.bash as i32)
    }
}
