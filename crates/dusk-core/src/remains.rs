//! The dead get up (`CONCEPT.md`, the Hushlands): a body left lying gets up again after a while,
//! unless it's burned.
//!
//! - **When it falls,** its kind says whether it will get up and when (`Creature::get_up`), and
//!   that's settled there and then: a body that will get up keeps the tick it does
//!   (`Remains::GetsUp`). It gets up once. Put down a second time, it stays down.
//! - **Getting up,** it's whole again, and it takes its kind's knock-down time to stand: a
//!   window to finish it while it's down. Each body that gets up raises the region's pressure
//!   (`pressure.rs`). One waits while someone stands on it.
//! - **Burning** a body takes oil and a flame: oil from your pack, poured on, and set alight from
//!   the light you hold up. A body that burns never gets up. Each one lowers the pressure.
//! - **A pyre:** fire goes from a burning body to the bodies on the tiles round it (not through
//!   walls), one tile a game minute. So the dead you put down at a door, piled up, burn from one
//!   flask. It doesn't catch on anything else yet: fire as a system comes later.
//! - **A burning body lights** the tiles round it as a torch does, then it's ash.

use crate::actor::{ActionKind, ActorId, Mind, Remains};
use crate::body::Body;
use crate::geom::{Dir8, TilePos};
use crate::hash::{Purpose, below, roll};
use crate::items::{ItemKind, within_reach};
use crate::world::{Event, World};

/// Burning's numbers, in the core's units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Burning {
    /// Ticks to pour the oil on and set it alight.
    pub time: u32,
    /// Game minutes a body burns before it's ash.
    pub burn: u32,
    /// How far a burning body lights round it, in tiles.
    pub light: i32,
}

impl Default for Burning {
    fn default() -> Burning {
        Burning {
            time: 30,
            burn: 20,
            light: 4,
        }
    }
}

/// Why a body can't be burned now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoBurn {
    /// No body beside you that can burn.
    Nothing,
    /// No oil in your pack.
    NoOil,
    /// No light held up to set it alight with.
    NoFlame,
}

impl World {
    /// When a creature falls: whether its body will get up, and when. Its kind says; and it gets
    /// up once.
    pub(crate) fn settle_remains(&mut self, id: ActorId) {
        let Some(a) = self.actors.get(id) else {
            return;
        };
        let risen = a.risen;
        let Some(c) = self.stats(id) else {
            return;
        };
        let (g, who, tick) = (c.get_up, id.index() as u64, self.tick);
        // Once for its kind, and once more for each time its affixes say (the Thrice-Risen).
        if risen > self.extra_rises(id) || g.chance == 0 {
            return;
        }
        if below(roll(self.seed, tick, who, Purpose::GetUp), 10_000) >= g.chance {
            return;
        }
        let (soonest, latest) = (g.after.0, g.after.1.max(g.after.0));
        let span = latest - soonest + 1;
        let minutes = soonest + below(roll(self.seed, tick, who, Purpose::GetUpWhen), span);
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if let Some(a) = self.actors.get_mut(id) {
            a.remains = Remains::GetsUp {
                at: tick + u64::from(minutes) * per,
            };
        }
    }

    /// Whether a body can burn: not burning yet, nor burnt, nor risen.
    fn burnable(&self, id: ActorId) -> bool {
        self.actors
            .get(id)
            .is_some_and(|a| a.dead && matches!(a.remains, Remains::Lying | Remains::GetsUp { .. }))
    }

    /// The bodies that can burn on a tile, in id order.
    fn bodies_on(&self, at: TilePos) -> impl Iterator<Item = ActorId> + '_ {
        self.actors
            .iter()
            .filter(move |(_, a)| a.pos == at)
            .map(|(id, _)| id)
            .filter(|&id| self.burnable(id))
    }

    /// The body `id` would burn: on the tile it faces, else its own, else the first round it,
    /// clockwise from its facing. Only one it can reach (no wall between).
    pub fn body_beside(&self, id: ActorId) -> Option<ActorId> {
        let a = self.actors.get(id)?;
        let (pos, facing) = (a.pos, a.facing);
        let ahead = std::iter::once(pos.step(facing));
        let round = (1..8).map(|t| pos.step(facing.turned(t)));
        let mut tiles = ahead.chain(std::iter::once(pos)).chain(round);
        tiles.find_map(|at| {
            within_reach(&self.grid, pos, at)
                .then(|| self.bodies_on(at).next())
                .flatten()
        })
    }

    /// Whether `id` can burn a body now, and which: one beside it, oil in its pack, and a light
    /// held up.
    pub fn can_burn(&self, id: ActorId) -> Result<ActorId, NoBurn> {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return Err(NoBurn::Nothing);
        };
        let body = self.body_beside(id).ok_or(NoBurn::Nothing)?;
        if self.oil_slot(id).is_none() {
            return Err(NoBurn::NoOil);
        }
        if a.light.is_none() {
            return Err(NoBurn::NoFlame);
        }
        Ok(body)
    }

    /// The first stack of oil in `id`'s pack.
    fn oil_slot(&self, id: ActorId) -> Option<usize> {
        let a = self.actors.get(id)?;
        a.items.iter().position(|i| {
            self.defs
                .item(i.def)
                .is_some_and(|d| matches!(d.kind, ItemKind::Oil))
        })
    }

    /// The action's start: what it takes, and how long.
    pub(crate) fn burn_action(&self, id: ActorId) -> Option<(ActionKind, Dir8, u32)> {
        let body = self.can_burn(id).ok()?;
        let (from, to) = (self.actors.get(id)?.pos, self.actors.get(body)?.pos);
        let dir = Dir8::toward(to.x - from.x, to.y - from.y).unwrap_or(self.actors.get(id)?.facing);
        Some((
            ActionKind::Burn { body },
            dir,
            self.rules.burning.time.max(1),
        ))
    }

    /// The action's end: the oil poured on and lit. The body must still be there, within reach,
    /// and still able to burn; the oil and the flame still in hand.
    pub(crate) fn kindle(&mut self, id: ActorId, body: ActorId) {
        let (Some(a), Some(b)) = (self.actors.get(id), self.actors.get(body)) else {
            return;
        };
        if a.dead || a.light.is_none() || !within_reach(&self.grid, a.pos, b.pos) {
            return;
        }
        if !self.burnable(body) {
            return;
        }
        let Some(slot) = self.oil_slot(id) else {
            return;
        };
        if let Some(a) = self.actors.get_mut(id) {
            a.items[slot].count -= 1;
            if a.items[slot].count == 0 {
                a.items.remove(slot);
            }
        }
        self.set_alight(body);
        self.events.push(Event::Kindled { who: id, body });
    }

    /// A body catches fire: it'll never get up now, and the pressure falls.
    fn set_alight(&mut self, body: ActorId) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        let until = self.tick + u64::from(self.rules.burning.burn) * per;
        // A character's gear doesn't burn: it falls where the body lies.
        if self.actors.get(body).is_some_and(|b| b.kind.is_character()) {
            self.shed_gear(body);
        }
        if let Some(b) = self.actors.get_mut(body) {
            b.remains = Remains::Burning { until };
            self.fires.push(body);
            let by = self.region.pressure.per_burn;
            self.nudge_pressure(-i64::from(by));
        }
    }

    /// The bodies burning now: where each is, and how far it lights.
    pub fn fires(&self) -> impl Iterator<Item = (TilePos, i32)> + '_ {
        let light = self.rules.burning.light;
        self.fires
            .iter()
            .filter_map(move |&id| Some((self.actors.get(id)?.pos, light)))
    }

    /// Works out again which bodies are burning (after a load).
    #[cfg(feature = "save")]
    pub(crate) fn find_fires(&mut self) {
        self.fires = self
            .actors
            .iter()
            .filter(|(_, a)| matches!(a.remains, Remains::Burning { .. }))
            .map(|(id, _)| id)
            .collect();
    }

    /// Each game minute: fires burn out and spread, and the bodies whose time has come get up.
    pub(crate) fn remains(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        let tick = self.tick;
        // Burnt out: ash.
        let burning = std::mem::take(&mut self.fires);
        let mut still = Vec::with_capacity(burning.len());
        for id in burning {
            let Some(a) = self.actors.get_mut(id) else {
                continue;
            };
            match a.remains {
                Remains::Burning { until } if until <= tick => {
                    a.remains = Remains::Ash;
                    self.events.push(Event::Burnt { body: id });
                }
                Remains::Burning { .. } => still.push(id),
                _ => {}
            }
        }
        self.fires = still;
        // Spread: from each body that was burning, to the bodies on its tile and round it, a
        // tile a minute.
        let from: Vec<TilePos> = self.fires().map(|(at, _)| at).collect();
        for at in from {
            let round = Dir8::ALL.map(|d| at.step(d));
            for next in std::iter::once(at).chain(round) {
                if !within_reach(&self.grid, at, next) {
                    continue;
                }
                let catching: Vec<ActorId> = self.bodies_on(next).collect();
                for body in catching {
                    self.set_alight(body);
                    self.events.push(Event::Caught { body });
                }
            }
        }
        // Getting up.
        let due: Vec<ActorId> = self
            .actors
            .iter()
            .filter(|(_, a)| a.dead && matches!(a.remains, Remains::GetsUp { at } if at <= tick))
            .map(|(id, _)| id)
            .collect();
        for id in due {
            if self.actors.get(id).is_some_and(|a| a.kind.is_character()) {
                self.rise_as_revenant(id);
            } else {
                self.get_up(id);
            }
        }
    }

    /// A body gets up: whole again, on its back for its kind's knock-down time, and knowing
    /// nothing. Not while someone stands on it: then it waits.
    fn get_up(&mut self, id: ActorId) {
        let Some(a) = self.actors.get(id) else {
            return;
        };
        let pos = a.pos;
        let Some(c) = self.stats(id) else {
            return;
        };
        if self.occupied.contains_key(&pos) {
            return;
        }
        let tick = self.tick;
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        a.dead = false;
        a.remains = Remains::Lying;
        a.risen = a.risen.saturating_add(1);
        a.body = Body::new(c.health);
        a.prone_until = tick + u64::from(c.down.max(1));
        a.mind = Mind::Hunt {
            target: None,
            since: tick,
            touching: false,
            sees: false,
        };
        a.intent = crate::actor::Intent::Idle;
        a.action = None;
        a.step = None;
        a.path.clear();
        a.path_goal = None;
        self.occupied.insert(pos, id);
        let by = self.region.pressure.per_rise;
        self.nudge_pressure(i64::from(by));
        self.events.push(Event::GotUp { who: id });
    }
}
