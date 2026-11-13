//! A fire for your base (`CONCEPT.md`, base-building: a bed, storage, a fire). A hearth, a stove
//! or a forge (a fixture with a `fire` reach in the data) takes one.
//!
//! - **Laying it:** fuel from your pack goes on a piece at a time, the longest-burning first
//!   (split logs before planks), up to what a fire holds (`Fire::most`).
//! - **Lighting it** takes fuel in it and a flame: the light you hold up, as for burning a body.
//! - **It burns down** a game minute at a time, and goes out when its fuel's gone. Put out, what's
//!   left stays in it for the next time.
//! - **It lights** the tiles round it as a carried light does, as far as its fixture says (an
//!   open hearth further than a stove), so a lit room needs no torch in your hand.
//! - **Light is bait:** at night, the dead that are after nothing and see a fire (a burning body
//!   too) come to it, as to a noise. A boarded window can't be seen through, so a fire behind one
//!   draws nobody.
//! - **A Lantern-Eater** (`rare.rs`) puts out a fire it passes, as it does a light.

use crate::actor::{ActionKind, ActorId, Mind};
use crate::barricade::spend;
use crate::geom::{Dir8, TilePos};
use crate::items::within_reach;
use crate::world::{Event, World};

/// The fire's numbers, in the core's units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fire {
    /// Game minutes of fuel a fire holds at most.
    pub most: u32,
    /// Ticks to put a piece of fuel on, to light it, and to put it out.
    pub lay: u32,
    pub light: u32,
    pub smother: u32,
}

impl Default for Fire {
    fn default() -> Fire {
        Fire {
            most: 480,
            lay: 20,
            light: 30,
            smother: 20,
        }
    }
}

/// A fire laid in a hearth, a stove or a forge: the fuel in it (game minutes of burning), and
/// whether it's alight.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Hearth {
    pub fuel: u32,
    pub lit: bool,
}

/// What tending a fire does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Tend {
    /// Puts a piece of fuel (this item) on.
    Lay { item: u16 },
    /// Lights what's laid, from the light held up.
    Light,
}

/// Why you can't tend a fire, or put one out, here and now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoFire {
    /// No hearth, stove or forge beside you (to put out: none alight).
    Nothing,
    /// It holds all the fuel it can.
    Full,
    /// Nothing in your pack that burns.
    NoFuel,
    /// It's laid, but there's no light held up to light it with.
    NoFlame,
}

impl World {
    /// How far a fire in the fixture on `at` lights, in tiles: 0 if it takes none.
    pub fn hearth_reach(&self, at: TilePos) -> i32 {
        self.defs.hearth(self.grid.tile(at).fixture)
    }

    /// The fire laid on `at` (cold and empty if none is).
    pub fn hearth(&self, at: TilePos) -> Hearth {
        self.hearths.get(&at).copied().unwrap_or_default()
    }

    /// The fires alight in hearths, stoves and forges: where each is, and how far it lights.
    pub fn hearth_fires(&self) -> impl Iterator<Item = (TilePos, i32)> + '_ {
        self.hearths
            .iter()
            .filter(|(_, h)| h.lit)
            .map(|(&at, _)| (at, self.hearth_reach(at)))
    }

    /// The hearth, stove or forge `id` would tend (`lit`: only one alight): the one it faces,
    /// else the first round it, clockwise from its facing. Only one it can reach (no wall
    /// between).
    pub fn hearth_beside(&self, id: ActorId, lit: bool) -> Option<TilePos> {
        let a = self.actors.get(id).filter(|a| !a.dead)?;
        let (pos, facing) = (a.pos, a.facing);
        (0..8).map(|t| pos.step(facing.turned(t))).find(|&at| {
            self.hearth_reach(at) > 0
                && (!lit || self.hearth(at).lit)
                && within_reach(&self.grid, pos, at)
        })
    }

    /// The fuel `id` would put on next: the longest-burning in its pack, and how long it burns.
    fn fuel_in_pack(&self, id: ActorId) -> Option<(u16, u32)> {
        let a = self.actors.get(id)?;
        self.defs
            .fuel
            .iter()
            .copied()
            .find(|&(item, _)| a.items.iter().any(|i| i.def == item))
    }

    /// What tending the fire beside `id` would do now, and where: light it if it's laid and you
    /// hold up a light, else put fuel on if it has room.
    pub fn can_tend(&self, id: ActorId) -> Result<(TilePos, Tend), NoFire> {
        let at = self.hearth_beside(id, false).ok_or(NoFire::Nothing)?;
        let fire = self.hearth(at);
        let flame = self.actors.get(id).is_some_and(|a| a.light.is_some());
        let laid = !fire.lit && fire.fuel > 0;
        if laid && flame {
            return Ok((at, Tend::Light));
        }
        match self.fuel_in_pack(id) {
            Some((item, burns)) if fire.fuel + burns <= self.rules.fire.most => {
                Ok((at, Tend::Lay { item }))
            }
            // Laid, and nothing more will go on: what it wants is a flame.
            _ if laid => Err(NoFire::NoFlame),
            Some(_) => Err(NoFire::Full),
            None => Err(NoFire::NoFuel),
        }
    }

    /// The fire alight beside `id`, to put out.
    pub fn can_smother(&self, id: ActorId) -> Result<TilePos, NoFire> {
        self.hearth_beside(id, true).ok_or(NoFire::Nothing)
    }

    /// The tending's start: what it does, which way, and how long.
    pub(crate) fn tend_action(&self, id: ActorId) -> Option<(ActionKind, Dir8, u32)> {
        let (at, tend) = self.can_tend(id).ok()?;
        let ticks = match tend {
            Tend::Lay { .. } => self.rules.fire.lay,
            Tend::Light => self.rules.fire.light,
        };
        Some((
            ActionKind::Tend { at, tend },
            self.toward(id, at),
            ticks.max(1),
        ))
    }

    /// The putting out's start.
    pub(crate) fn smother_action(&self, id: ActorId) -> Option<(ActionKind, Dir8, u32)> {
        let at = self.can_smother(id).ok()?;
        let ticks = self.rules.fire.smother.max(1);
        Some((ActionKind::Smother { at }, self.toward(id, at), ticks))
    }

    /// The tending's end: the fuel put on, or the fire lit, if that's still what tending it
    /// would do (the fuel still in the pack, the flame still held up).
    pub(crate) fn tend(&mut self, id: ActorId, at: TilePos, tend: Tend) {
        if self.can_tend(id) != Ok((at, tend)) {
            return;
        }
        match tend {
            Tend::Lay { item } => {
                let burns = self
                    .defs
                    .fuel
                    .iter()
                    .find(|f| f.0 == item)
                    .map_or(0, |f| f.1);
                if let Some(a) = self.actors.get_mut(id) {
                    spend(&mut a.items, item, 1);
                }
                self.hearths.entry(at).or_default().fuel += burns;
                self.events.push(Event::Laid { who: id, at, item });
            }
            Tend::Light => {
                self.hearths.entry(at).or_default().lit = true;
                self.events.push(Event::FireLit { who: id, at });
            }
        }
    }

    /// The putting out's end: still alight, and still within reach.
    pub(crate) fn smother(&mut self, id: ActorId, at: TilePos) {
        let near = self
            .actors
            .get(id)
            .is_some_and(|a| !a.dead && within_reach(&self.grid, a.pos, at));
        if near {
            self.put_out(at, id);
        }
    }

    /// Puts out the fire on `at`, if it's alight: what fuel's left stays in it. `by`: who did.
    pub(crate) fn put_out(&mut self, at: TilePos, by: ActorId) {
        if let Some(h) = self.hearths.get_mut(&at).filter(|h| h.lit) {
            h.lit = false;
            self.events.push(Event::Smothered { who: by, at });
        }
    }

    /// Each game minute: the fires burn down and go out at nothing, and at night the dead that
    /// see one come to it.
    pub(crate) fn burn_fires(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        let mut out = Vec::new();
        for (&at, h) in self.hearths.iter_mut().filter(|(_, h)| h.lit) {
            h.fuel = h.fuel.saturating_sub(1);
            if h.fuel == 0 {
                out.push(at);
            }
        }
        for at in out {
            self.hearths.remove(&at);
            self.events.push(Event::FireOut { at });
        }
        self.draw_to_fires();
    }

    /// At night, the dead that are after nothing and see a fire come to it, as to a noise: the
    /// nearest fire each sees. On hallowed ground, a fire draws nobody.
    fn draw_to_fires(&mut self) {
        if !self.is_night() {
            return;
        }
        let fires: Vec<TilePos> = self
            .hearth_fires()
            .chain(self.fires())
            // A spell's light lures them too (`spell.rs`).
            .chain(self.spell_lights())
            .map(|(at, _)| at)
            .filter(|&at| !crate::holdout::in_hallowed(&self.hallowed, at))
            .collect();
        if fires.is_empty() {
            return;
        }
        let drawn: Vec<(ActorId, TilePos, TilePos)> = self
            .actors
            .iter()
            .filter(|&(id, a)| {
                !a.dead
                    && !a.kind.is_character()
                    && !self.is_wild(id)
                    && matches!(a.mind, Mind::Hunt { target: None, .. })
                    && self.is_awake(a.pos)
            })
            .filter_map(|(id, a)| {
                let fire = fires
                    .iter()
                    .copied()
                    .filter(|&f| self.sees(id, f))
                    .min_by_key(|f| f.dist2(a.pos))?;
                Some((id, fire, self.fire_side(fire, a.pos)))
            })
            .collect();
        let tick = self.tick;
        for (id, fire, goal) in drawn {
            if let Some(a) = self.actors.get_mut(id)
                && let Mind::Hunt { touching, sees, .. } = a.mind
            {
                a.mind = Mind::Hunt {
                    target: Some(goal),
                    since: tick,
                    touching,
                    sees,
                };
                self.events.push(Event::SawFire { who: id, at: fire });
            }
        }
    }

    /// Where the dead go to look at a fire: its own tile if it can be stood on (a burning body's),
    /// else the tile beside it, within reach of it, nearest `from`.
    fn fire_side(&self, fire: TilePos, from: TilePos) -> TilePos {
        if self.grid.tile(fire).walkable() {
            return fire;
        }
        Dir8::ALL
            .iter()
            .map(|&d| fire.step(d))
            .filter(|&p| self.grid.tile(p).walkable() && within_reach(&self.grid, fire, p))
            .min_by_key(|p| p.dist2(from))
            .unwrap_or(fire)
    }
}
