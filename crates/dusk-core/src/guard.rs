//! The combat stance, as Zomboid's (2026-11-17; the first playtest: "Similar to zomboid, holding
//! right click should put you in a combat stance, where your character will be on guard, slower,
//! face whatever way the cursor is, allow movement/backstepping, and swing with left click").
//!
//! - **On guard** is a stance a character takes and leaves (`World::on_guard`), as sneaking and
//!   bracing are: a slower step (`Rules::guard_pace`) and no running. It faces where it's turned
//!   (`World::face`: the client turns it to its cursor), not where it walks, so it steps back or
//!   aside with its eyes on the dead.
//! - **A swing on guard goes where it faces** (`combat.rs`'s `aim`), not to the nearest of the
//!   dead beside it.
//! - A shield on the arm braces in the stance too: the client asks for both (`shield.rs`).
//! - **In a fight** (`World::in_combat`): held, or seen by one of the dead near you. Out of one,
//!   taking a thing in hand, putting it away or putting armour on or off is done at once (the
//!   second playtest: "Equipping items shouldnt take time out of combat, but should take time when
//!   in combat"; `World::equip_ticks`).

use crate::actor::{ActorId, Mind};
use crate::geom::Dir8;
use crate::world::World;

/// Tiles within which one of the dead that sees you puts you in a fight.
pub const IN_COMBAT: i32 = 8;

impl World {
    /// Takes up the stance (`on`), or leaves it: a living character's.
    pub fn on_guard(&mut self, id: ActorId, on: bool) {
        if let Some(a) = self
            .actors
            .get_mut(id)
            .filter(|a| !a.dead && a.kind.is_character())
        {
            a.guarding = on;
        }
    }

    pub fn is_guarding(&self, id: ActorId) -> bool {
        self.actors.get(id).is_some_and(|a| a.guarding)
    }

    /// On guard, `id` turns to face `dir`: only on guard, and not while a timed action keeps it
    /// facing its own way (a swing's).
    pub fn face(&mut self, id: ActorId, dir: Dir8) {
        if let Some(a) = self
            .actors
            .get_mut(id)
            .filter(|a| a.guarding && !a.dead && a.action.is_none())
        {
            a.facing = dir;
        }
    }

    /// Whether `id` is in a fight: held, or one of the dead within `IN_COMBAT` tiles sees it.
    pub fn in_combat(&self, id: ActorId) -> bool {
        let Some(a) = self.actors.get(id) else {
            return false;
        };
        if self.is_held(id) {
            return true;
        }
        let r2 = i64::from(IN_COMBAT).pow(2);
        self.actors.iter().any(|(_, o)| {
            !o.dead
                && o.kind.hostile_to(a.kind)
                && matches!(o.mind, Mind::Hunt { sees: true, .. })
                && o.pos.z == a.pos.z
                && o.pos.dist2(a.pos) <= r2
        })
    }

    /// The ticks taking a thing in hand, putting it away or putting armour on or off takes `id`:
    /// `ticks` in a fight, a moment (one) out of one.
    pub(crate) fn equip_ticks(&self, id: ActorId, ticks: u32) -> u32 {
        if self.in_combat(id) { ticks } else { 1 }
    }

    /// What's left of `id`'s pace on guard, per mille.
    pub(crate) fn guard_pace(&self, id: ActorId) -> u32 {
        if self.is_guarding(id) {
            self.rules.guard_pace
        } else {
            1000
        }
    }
}
