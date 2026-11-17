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

use crate::actor::ActorId;
use crate::geom::Dir8;
use crate::world::World;

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

    /// What's left of `id`'s pace on guard, per mille.
    pub(crate) fn guard_pace(&self, id: ActorId) -> u32 {
        if self.is_guarding(id) {
            self.rules.guard_pace
        } else {
            1000
        }
    }
}
