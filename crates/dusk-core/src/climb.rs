//! Climbing (`CONCEPT.md`, athletics): through an open window, over a fence.
//!
//! A climb is a step, only slower. It crosses an edge nobody walks through
//! (`Edge::climbable`: an open window with no boards, or a fence), straight across, in its own
//! time whatever your pace. Its seconds cost breath as running does, so armour, a hurt leg and
//! Athletics count. It makes a scrape. Running at a fence, you vault it, quicker. Weighed down,
//! it takes longer.
//!
//! The dead climb too, if their kind does (`Creature::climb`), but only after something: a
//! wandering one stays on its side of the fence. Some tumble over (`Creature::tumble`) and lie
//! on the far side for their `down` time: the window you stand by is where you finish them.

use crate::actor::{ActorId, ActorKind, Mind};
use crate::grid::Edge;
use crate::path::{ClimbCost, STRAIGHT};
use crate::skills::Skill;
use crate::world::{TICKS_PER_SECOND, World};

/// rules.toml's [climb], in the core's units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClimbRules {
    /// Ticks a character takes through a window, and over a fence.
    pub window: u32,
    pub fence: u32,
    /// Running at a fence, you vault it in this share of the time, per mille.
    pub vault: u32,
    /// How far the scrape of a climb carries, in tiles (a character's).
    pub noise: i32,
    /// Weighed down, a climb takes this much longer, per mille.
    pub heavy: u32,
}

impl Default for ClimbRules {
    fn default() -> ClimbRules {
        ClimbRules {
            window: 16,
            fence: 10,
            vault: 600,
            noise: 4,
            heavy: 2000,
        }
    }
}

/// How an actor climbs, worked out as it starts a step: its ticks through a window and over a
/// fence, and the share of that a run at a fence takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Climber {
    pub window: u32,
    pub fence: u32,
    pub vault: u32,
}

impl Climber {
    /// The ticks a climb over `edge` takes, running or not: at least one.
    pub fn ticks(&self, edge: Edge, run: bool) -> u32 {
        let t = match edge {
            Edge::Fence if run => self.fence * self.vault / 1000,
            Edge::Fence => self.fence,
            _ => self.window,
        };
        t.max(1)
    }

    /// What a climb is worth to a search for a way, in its units (a straight step is
    /// `STRAIGHT`): the step, and the ground that could have been covered at `speed` (thousandths
    /// of a tile a second) in the time it takes.
    pub fn costs(&self, speed: u32) -> ClimbCost {
        let cost = |t: u32| STRAIGHT + t * speed / TICKS_PER_SECOND;
        ClimbCost {
            window: cost(self.window),
            fence: cost(self.fence),
        }
    }
}

impl World {
    /// How `id` climbs now, or None if it can't. A character always can: Athletics makes it
    /// quicker (as it makes a run cheaper), and weighed down or out of breath it's slower. One of
    /// the dead climbs if its kind does, and only after something.
    pub(crate) fn climber(&self, id: ActorId) -> Option<Climber> {
        let a = self.actors.get(id)?;
        let r = self.rules.climb;
        match a.kind {
            ActorKind::Character => {
                let s = &self.rules.skills;
                let athletics = a.skills.level(Skill::Athletics, s);
                let heavy = self.weighed_down(id);
                let exhausted = a.body.exhausted;
                let time = |t: u32| {
                    let t = s.run_cost(t as i32, athletics).max(1) as u32;
                    let t = if heavy { t * r.heavy / 1000 } else { t };
                    if exhausted { t * 3 / 2 } else { t }
                };
                Some(Climber {
                    window: time(r.window),
                    fence: time(r.fence),
                    vault: r.vault,
                })
            }
            ActorKind::Creature(n) => {
                if !matches!(
                    a.mind,
                    Mind::Hunt {
                        target: Some(_),
                        ..
                    }
                ) {
                    return None;
                }
                let c = &self.defs.creature(n)?.stats;
                (c.climb > 0).then(|| Climber {
                    window: r.window * c.climb / 1000,
                    fence: r.fence * c.climb / 1000,
                    vault: 1000,
                })
            }
        }
    }

    /// Whether a kind of actor ever climbs: a character does; a creature if its kind's `climb`
    /// says so.
    pub(crate) fn climbs(&self, kind: ActorKind) -> bool {
        match kind {
            ActorKind::Character => true,
            ActorKind::Creature(n) => self.defs.creature(n).is_some_and(|c| c.stats.climb > 0),
        }
    }

    /// A climb is over: the scrape is heard (in the step's footsteps), it may teach Athletics,
    /// and one of a kind that tumbles lands down on the far side.
    pub(crate) fn landed(&mut self, id: ActorId, window: bool) {
        let Some(a) = self.actors.get(id) else {
            return;
        };
        let (kind, to) = (a.kind, a.pos);
        let fell = match kind {
            ActorKind::Character => {
                self.train_run(id);
                None
            }
            ActorKind::Creature(n) => self
                .defs
                .creature(n)
                .filter(|c| c.stats.tumble)
                .map(|c| c.stats.down.max(1)),
        };
        let tick = self.tick;
        if let (Some(down), Some(a)) = (fell, self.actors.get_mut(id)) {
            a.prone_until = tick + u64::from(down);
        }
        self.events.push(crate::world::Event::Climbed {
            who: id,
            to,
            window,
            fell: fell.is_some(),
        });
    }
}
