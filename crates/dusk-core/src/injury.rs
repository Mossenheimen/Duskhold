//! Wounds by where they land (`CONCEPT.md`, body, moodles and afflictions: each kind of injury
//! has one clear effect and one treatment, and heals over days).
//!
//! - **How hurt a part of you is:** the pain of the wounds on it (`dread.rs`: by kind, half
//!   dressed, less as they heal), the arms counted with the hands and the legs with the feet.
//!   Two levels: hurt, and badly hurt. Nothing's kept: it's worked out from the wounds.
//! - **What each does,** to the living (the dead feel nothing):
//!   - arms: your swings take longer, wind-up and recovery, and land less often;
//!   - legs: you're slower on your feet, and running costs more breath;
//!   - head: your aim is worse, and you see less far;
//!   - torso: its wounds hurt more, so pain, and what pain costs, come sooner.
//! - **Its treatment is the bandage,** as pain's is: a dressed wound hurts half, so it hampers
//!   half. And it all eases as the wound closes.

use crate::actor::ActorId;
use crate::body::Group;
use crate::world::{Swing, World};

/// What wounds do where they land. A part's two levels (the pain of its wounds, in thousandths of
/// a point), and at each, per mille: how much longer a hurt arm's swing takes and what's left of
/// its chance to land; what's left of a hurt leg's pace, and how much more running costs; what's
/// left of a hurt head's chance to land and of how far it sees. And how much more a wound to the
/// torso hurts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Injury {
    pub levels: [i32; 2],
    pub swing: [u32; 2],
    pub arm_hit: [u32; 2],
    pub pace: [u32; 2],
    pub run: [u32; 2],
    pub head_hit: [u32; 2],
    pub sight: [u32; 2],
    pub torso: u32,
}

impl Default for Injury {
    fn default() -> Injury {
        Injury {
            levels: [10_000, 25_000],
            swing: [1150, 1300],
            arm_hit: [900, 800],
            pace: [850, 600],
            run: [1250, 1500],
            head_hit: [900, 800],
            sight: [850, 700],
            torso: 1500,
        }
    }
}

impl World {
    /// How hurt a part of `id` is: the pain of its wounds there, in thousandths of a point.
    pub fn hurt(&self, id: ActorId, group: Group) -> i32 {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return 0;
        };
        a.body
            .wounds
            .iter()
            .filter(|w| w.part.group() == group)
            .map(|w| self.wound_pain(w))
            .sum()
    }

    /// Its level: 0 whole, 1 hurt, 2 badly hurt.
    pub fn hurt_level(&self, id: ActorId, group: Group) -> u8 {
        let hurt = self.hurt(id, group);
        self.rules
            .injury
            .levels
            .iter()
            .filter(|&&l| hurt >= l)
            .count() as u8
    }

    /// One of a pair of numbers (per mille) by how hurt a part of `id` is: 1000 whole, and
    /// always for the dead, who feel nothing.
    fn hampered_by(&self, id: ActorId, group: Group, pair: [u32; 2]) -> u32 {
        let living = self.actors.get(id).is_some_and(|a| a.kind.is_character());
        match self.hurt_level(id, group) {
            _ if !living => 1000,
            0 => 1000,
            l => pair[usize::from(l) - 1],
        }
    }

    /// A swing as `id`'s wounds hamper it: hurt arms swing slower and land less often, and a
    /// hurt head lands less often.
    pub(crate) fn hampered(&self, id: ActorId, mut swing: Swing) -> Swing {
        let i = self.rules.injury;
        let slow = self.hampered_by(id, Group::Arms, i.swing);
        swing.wind_up = (swing.wind_up * slow / 1000).max(1);
        swing.recovery = swing.recovery * slow / 1000;
        let arms = self.hampered_by(id, Group::Arms, i.arm_hit);
        let head = self.hampered_by(id, Group::Head, i.head_hit);
        swing.hit = swing.hit * arms / 1000 * head / 1000;
        swing
    }

    /// What's left of `id`'s pace for its legs, per mille.
    pub(crate) fn leg_pace(&self, id: ActorId) -> u32 {
        self.hampered_by(id, Group::Legs, self.rules.injury.pace)
    }

    /// How much more running costs `id` for its legs, per mille.
    pub(crate) fn leg_run(&self, id: ActorId) -> u32 {
        self.hampered_by(id, Group::Legs, self.rules.injury.run)
    }

    /// How far `id` sees in full light: less with a hurt head.
    pub fn sight_range(&self, id: ActorId) -> i32 {
        let range = i64::from(self.rules.sight.range);
        let left = i64::from(self.hampered_by(id, Group::Head, self.rules.injury.sight));
        (range * left / 1000) as i32
    }
}
