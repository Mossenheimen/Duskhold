//! Wildlife and hunting (`CONCEPT.md`: the land feeds you, if you can get near it).
//!
//! - **A beast** is a creature that `flees` (`Creature::flees`): it's after nobody, and nobody
//!   dreads it. It grazes and wanders. Seeing one of the living within its `flees` tiles, hearing
//!   a noise, or being hit, it's startled: it runs from there, as far as it can each step, until
//!   it has seen or heard nothing for its `forget_after`. Cornered, it stands. It never strikes.
//! - **Getting near:** a beast sees as the dead do (its cone, the light), so sneaking up from
//!   behind, or from the dark, gets you nearer; a bow reaches it from where it hasn't seen you,
//!   and one unaware takes a sneak attack's harm (`archery.rs`, `stealth.rs`).
//! - **Butchering:** its carcass (a kind with `butcher`) is cut up with a blade or an axe in hand
//!   or pack: a while kneeling over it, and it gives its meat (raw: `cooking.rs`), and there's
//!   nothing left of it. A beast's body never gets up and doesn't burn (it's no threat).
//! - **Beasts aren't the dead:** the region's pressure doesn't count them, the bell doesn't draw
//!   them, and no rare is made of one.
//! - **Wild hunters** (a kind that's `wild` and doesn't flee: the wolf) hunt you as the dead
//!   do, by the same mind, and are just as much a dread. But they're of the living world too:
//!   the pressure doesn't count them, neither the bell nor a fire draws them, and a body is
//!   butchered, not burned.

use crate::actor::{ActionKind, ActorId, ActorKind, Intent, Mind, Remains};
use crate::creature::Creature;
use crate::geom::{Dir8, TilePos};
use crate::hash::{Purpose, below, roll};
use crate::items::{ItemKind, within_reach};
use crate::skills::Skill;
use crate::world::{Event, World};

/// What a carcass gives cut up: so many of an item (its meat), in so many ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Butcher {
    pub item: u16,
    pub count: u16,
    pub time: u32,
}

/// Why you can't butcher now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoButcher {
    /// No carcass beside you.
    Nothing,
    /// No blade or axe, in hand or in the pack.
    NoBlade,
}

impl World {
    /// Whether `id` is of the living world, not the dead: a beast, or a wild hunter (a wolf).
    pub fn is_wild(&self, id: ActorId) -> bool {
        self.stats(id).is_some_and(|c| c.wild || c.flees > 0)
    }

    /// Whether `id` is a beast (it flees, and is after nobody), alive or dead.
    pub fn is_beast(&self, id: ActorId) -> bool {
        self.stats(id).is_some_and(|c| c.flees > 0)
    }

    /// Something at `from` startles beast `id`: it runs from there. News, if it wasn't running
    /// already.
    pub(crate) fn startle(&mut self, id: ActorId, from: TilePos) {
        let tick = self.tick;
        let Some(a) = self.actors.get_mut(id).filter(|a| !a.dead) else {
            return;
        };
        let Mind::Flee { from: was, .. } = a.mind else {
            return;
        };
        a.mind = Mind::Flee {
            from: Some(from),
            since: tick,
        };
        if was.is_none() {
            self.events.push(Event::Startled { who: id });
        }
    }

    /// Beast `id` was hit by `by`: it runs from where the blow came from.
    pub(crate) fn startle_by(&mut self, id: ActorId, by: ActorId) {
        if let Some(from) = self.actors.get(by).map(|a| a.pos) {
            self.startle(id, from);
        }
    }

    /// A beast's turn to think: startled by one of the living it sees near; running while it
    /// remembers the fright; else grazing and wandering.
    pub(crate) fn think_beast(&mut self, id: ActorId, kind: Creature) {
        let tick = self.tick;
        let Some(a) = self.actors.get(id) else { return };
        let pos = a.pos;
        if let Some((_, at)) = self.nearest_visible_character(id)
            && at.dist2(pos) <= i64::from(kind.flees).pow(2)
        {
            self.startle(id, at);
        }
        let Some(a) = self.actors.get(id) else { return };
        let Mind::Flee { mut from, since } = a.mind else {
            return;
        };
        if tick.saturating_sub(since) > kind.forget_after {
            from = None;
        }
        let intent = match from {
            Some(threat) => match self.flight(id, threat) {
                Some(dir) => Intent::Move { dir, run: true },
                None => Intent::Idle,
            },
            None => {
                let who = id.index() as u64;
                if below(roll(self.seed, tick, who, Purpose::Wander), 10_000) < kind.wander {
                    let dir = Dir8::ALL
                        [below(roll(self.seed, tick, who, Purpose::WanderDir), 8) as usize];
                    Intent::Move { dir, run: false }
                } else {
                    Intent::Idle
                }
            }
        };
        if let Some(a) = self.actors.get_mut(id) {
            a.mind = Mind::Flee { from, since };
            a.intent = intent;
        }
    }

    /// The open step that takes `id` farthest from `threat` (and never nearer): None, cornered.
    fn flight(&self, id: ActorId, threat: TilePos) -> Option<Dir8> {
        let a = self.actors.get(id)?;
        // Mid-step, it plans from the tile it's stepping onto.
        let pos = a.step.map_or(a.pos, |s| s.to);
        let now = pos.dist2(threat);
        Dir8::ALL
            .iter()
            .copied()
            .filter(|&d| self.grid.can_step(pos, d) && !self.occupied.contains_key(&pos.step(d)))
            .map(|d| (pos.step(d).dist2(threat), d))
            .filter(|&(far, _)| far >= now)
            .max_by_key(|&(far, d)| (far, std::cmp::Reverse(d as u8)))
            .map(|(_, d)| d)
    }

    /// What a carcass gives cut up, if it's a beast's and still whole.
    fn butchery(&self, body: ActorId) -> Option<crate::hunting::Butcher> {
        let b = self.actors.get(body)?;
        if !b.dead || !matches!(b.remains, Remains::Lying) {
            return None;
        }
        let ActorKind::Creature(n) = b.kind else {
            return None;
        };
        self.defs.creature(n)?.stats.butcher
    }

    /// The carcass `id` would butcher: on the tile it faces, else its own, else the first round
    /// it, clockwise from its facing. Only one it can reach.
    pub fn carcass_beside(&self, id: ActorId) -> Option<ActorId> {
        let a = self.actors.get(id).filter(|a| !a.dead)?;
        let (pos, facing) = (a.pos, a.facing);
        let ahead = std::iter::once(pos.step(facing));
        let round = (1..8).map(|t| pos.step(facing.turned(t)));
        let mut tiles = ahead.chain(std::iter::once(pos)).chain(round);
        tiles.find_map(|at| {
            if !within_reach(&self.grid, pos, at) {
                return None;
            }
            self.actors
                .iter()
                .filter(|(_, b)| b.pos == at)
                .map(|(bid, _)| bid)
                .find(|&bid| self.butchery(bid).is_some())
        })
    }

    /// Whether `id` has a blade or an axe, in hand or in its pack.
    fn has_blade(&self, id: ActorId) -> bool {
        let Some(a) = self.actors.get(id) else {
            return false;
        };
        a.hands.iter().chain(a.items.iter()).any(|i| {
            self.defs.item(i.def).is_some_and(|d| {
                matches!(
                    d.kind,
                    ItemKind::Weapon {
                        skill: Some(Skill::Blade | Skill::Axe),
                        ..
                    }
                )
            })
        })
    }

    /// Whether `id` can butcher now, and which carcass.
    pub fn can_butcher(&self, id: ActorId) -> Result<ActorId, NoButcher> {
        let body = self.carcass_beside(id).ok_or(NoButcher::Nothing)?;
        if !self.has_blade(id) {
            return Err(NoButcher::NoBlade);
        }
        Ok(body)
    }

    /// The butchering's start: which way, and how long.
    pub(crate) fn butcher_action(&self, id: ActorId) -> Option<(ActionKind, Dir8, u32)> {
        let body = self.can_butcher(id).ok()?;
        let cut = self.butchery(body)?;
        let to = self.actors.get(body)?.pos;
        Some((
            ActionKind::Butcher { body },
            self.toward(id, to),
            cut.time.max(1),
        ))
    }

    /// The butchering's end: the meat in your pack, and nothing left of the carcass. Only if it's
    /// still there, whole, and within reach, and you've still a blade.
    pub(crate) fn butcher(&mut self, id: ActorId, body: ActorId) {
        if self.can_butcher(id) != Ok(body) {
            return;
        }
        let Some(cut) = self.butchery(body) else {
            return;
        };
        let defs = self.defs.clone();
        if let Some(a) = self.actors.get_mut(id) {
            defs.stow(&mut a.items, crate::items::Item::new(cut.item, cut.count));
        }
        if let Some(b) = self.actors.get_mut(body) {
            b.remains = Remains::Butchered;
        }
        self.events.push(Event::Butchered {
            who: id,
            body,
            item: cut.item,
            count: cut.count,
        });
    }
}
