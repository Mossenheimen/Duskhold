//! Herbs and foraging (`CONCEPT.md`: cures are quests, "a rare herb from somewhere unpleasant";
//! Zomboid's foraging, as plants you can see).
//!
//! - **Wild plants** are fixtures with something to pick (`Defs::plants`, by fixture number:
//!   `Plant`). The land grows them (yarrow in the open, chamomile by the water, brambles in the
//!   woods), and the yards round some places grow their own (a prefab's `grows`): feverfew round
//!   the barrows and the graveyard, where the dead lie.
//! - **Picking** (`Intent::Pick`): a few quiet seconds bent over the plant beside you (the one you
//!   face, else the one you stand on, else the first round you). It gives a few of its herb, and
//!   the plant is picked bare (`Defs::bare`, the fixture it becomes) until it grows back, days
//!   later: `World::regrowing`, looked at each game hour (`Event::Regrew`). One that's been built
//!   over or broken up by then stays gone.
//! - **Foraging**, the skill, grows by picking (picking in one spot again and again pays less): a
//!   level makes one more of the herb likelier.
//! - **The remedies** are recipes (`recipes.toml`), some at a fire, and their effects are the
//!   items' own: a poultice is a bandage with a salve (`ItemKind::Bandage`'s `salve`: the wound
//!   it dresses closes sooner, and so hurts less: `dread.rs`), a tea is a drink with comfort, and
//!   feverfew's tincture is the fever's cure (`fever.rs`).

use crate::actor::{ActionKind, ActorId};
use crate::geom::{Dir8, TilePos};
use crate::grid::Tile;
use crate::hash::{Purpose, below, roll};
use crate::items::{Item, within_reach};
use crate::skills::{On, Skill};
use crate::world::{Event, World};

/// A wild plant, as its fixture has it: the item picked from it, how many (the fewest and the
/// most), and the ticks before it grows back once it's picked bare.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plant {
    pub item: u16,
    pub count: (u16, u16),
    pub regrow: u64,
}

/// Foraging's rules: the ticks a plant takes to pick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Foraging {
    pub pick: u32,
}

impl Default for Foraging {
    fn default() -> Foraging {
        Foraging { pick: 30 }
    }
}

impl World {
    /// The plant on `at`, if there's one there to pick (not one picked bare).
    pub fn plant_at(&self, at: TilePos) -> Option<Plant> {
        let f = usize::from(self.grid.tile(at).fixture);
        self.defs.plants.get(f).copied().flatten()
    }

    /// The plant `id` would pick: on the tile it faces, else its own, else the first round it,
    /// clockwise from its facing. Only one it can reach.
    pub fn plant_beside(&self, id: ActorId) -> Option<TilePos> {
        let a = self.actors.get(id).filter(|a| !a.dead)?;
        let (pos, facing) = (a.pos, a.facing);
        let ahead = std::iter::once(pos.step(facing));
        let round = (1..8).map(|t| pos.step(facing.turned(t)));
        ahead
            .chain(std::iter::once(pos))
            .chain(round)
            .find(|&at| self.plant_at(at).is_some() && within_reach(&self.grid, pos, at))
    }

    /// The picking's start: which way, and how long.
    pub(crate) fn pick_action(&self, id: ActorId) -> Option<(ActionKind, Dir8, u32)> {
        let at = self.plant_beside(id)?;
        Some((
            ActionKind::Pick { at },
            self.toward(id, at),
            self.rules.forage.pick.max(1),
        ))
    }

    /// The picking's end: a few of the plant's herb in your pack (one more, maybe, with
    /// Foraging), the plant picked bare until it grows back, and Foraging taught. Only if it's
    /// still there, unpicked, and within reach.
    pub(crate) fn pick(&mut self, id: ActorId, at: TilePos) {
        let Some(pos) = self.actors.get(id).filter(|a| !a.dead).map(|a| a.pos) else {
            return;
        };
        let Some(plant) = self.plant_at(at) else {
            return;
        };
        if !within_reach(&self.grid, pos, at) {
            return;
        }
        let (tick, who) = (self.tick, id.index() as u64);
        let (fewest, most) = (plant.count.0, plant.count.1.max(plant.count.0));
        let spread = u32::from(most - fewest) + 1;
        let mut count = fewest + below(roll(self.seed, tick, who, Purpose::Pick), spread) as u16;
        let level = self.skill_level(id, Skill::Foraging);
        let more = self.rules.skills.forage_more(level);
        if below(roll(self.seed, tick, who, Purpose::PickMore), 1000) < more {
            count += 1;
        }
        let tile = self.grid.tile(at);
        self.grid.set_tile(
            at,
            Tile {
                fixture: self.defs.bare,
                solid: false,
                opaque: false,
                ..tile
            },
        );
        self.regrowing
            .insert(at, (tile.fixture, tick + plant.regrow));
        let defs = self.defs.clone();
        if count > 0
            && let Some(a) = self.actors.get_mut(id)
        {
            defs.stow(&mut a.items, Item::new(plant.item, count));
        }
        self.events.push(Event::Picked {
            who: id,
            at,
            item: plant.item,
            count,
        });
        let spot = self.rules.skills.spot.max(1);
        let on = On::Spot(
            at.x.div_euclid(spot),
            at.y.div_euclid(spot),
            i32::from(at.z),
        );
        self.train(id, Skill::Foraging, self.rules.skills.pick, on);
    }

    /// Each game hour: the plants picked bare whose time has come grow back, unless something
    /// else stands there by now.
    pub(crate) fn regrow(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1)) * 60;
        if self.regrowing.is_empty() || !self.tick.is_multiple_of(per) {
            return;
        }
        let tick = self.tick;
        let grown: Vec<(TilePos, u16)> = self
            .regrowing
            .iter()
            .filter(|&(_, &(_, until))| until <= tick)
            .map(|(&at, &(fixture, _))| (at, fixture))
            .collect();
        for (at, fixture) in grown {
            self.regrowing.remove(&at);
            let tile = self.grid.tile(at);
            if tile.fixture != self.defs.bare {
                continue;
            }
            self.grid.set_tile(at, Tile { fixture, ..tile });
            self.events.push(Event::Regrew { at });
        }
    }
}
