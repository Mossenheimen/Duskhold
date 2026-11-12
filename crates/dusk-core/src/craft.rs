//! Crafting (`CONCEPT.md`: base-building, and the things you make): recipes from the data, made
//! from what you carry with a tool to hand, and furniture broken down for planks.
//!
//! - **A recipe** uses up its makings (from your pack, then your hand: a club starts as the cudgel
//!   you hold) and wants one of its tools to hand, kept. It takes its time, less for the skilled
//!   (Carpentry, as boarding up), and teaches its skill. What you make is yours, not a find.
//! - **Breaking furniture down** (a chair, a table, a bed, a pew: a fixture with `planks` in the
//!   data) takes a hammer or an axe, and it's loud. It gives its planks, the tile is clear, and
//!   Carpentry learns from it. A container isn't broken up, nor anything on hallowed ground.

use crate::actor::{ActionKind, ActorId};
use crate::barricade::spend;
use crate::geom::{Dir8, TilePos};
use crate::grid::Tile;
use crate::items::{Item, within_reach};
use crate::skills::{On, Skill};
use crate::world::{Event, Sound, World};

/// A recipe: what it makes and how many, what it uses up, the tools it wants to hand (any one;
/// none if empty), whether it wants a lit fire beside you (cooking: `cooking.rs`), the ticks it
/// takes, and the skill it teaches with its XP (thousandths).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub id: String,
    pub makes: (u16, u16),
    pub uses: Vec<(u16, u16)>,
    pub tools: Vec<u16>,
    pub fire: bool,
    pub time: u32,
    pub skill: Option<Skill>,
    pub xp: u32,
}

/// Breaking furniture down: the plank it gives, the tools that do it (any one), the ticks it
/// takes, how far it's heard (tiles), and Carpentry's XP for it (thousandths).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Breaking {
    pub plank: u16,
    pub tools: Vec<u16>,
    pub time: u32,
    pub noise: i32,
    pub xp: u32,
}

/// Why you can't make something now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoCraft {
    /// No such recipe.
    NoRecipe,
    /// None of its tools to hand.
    NoTool,
    /// Short of this item.
    Lacks(u16),
    /// It's cooked, and there's no lit fire beside you.
    NoFire,
}

/// Why you can't break furniture down here and now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoBreak {
    /// Nothing beside you that breaks down.
    Nothing,
    /// No hammer or axe to hand.
    NoTool,
}

impl World {
    /// How many of an item `id` carries, in its pack and in its hand.
    fn carried(&self, id: ActorId, item: u16) -> u32 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        let pack: u32 = a
            .items
            .iter()
            .filter(|i| i.def == item)
            .map(|i| u32::from(i.count))
            .sum();
        pack + a
            .hands
            .filter(|h| h.def == item)
            .map_or(0, |h| u32::from(h.count))
    }

    /// Whether `id` has one of `tools` to hand: in its pack or its hand.
    fn has_tool(&self, id: ActorId, tools: &[u16]) -> bool {
        tools.iter().any(|&t| self.carried(id, t) > 0)
    }

    /// Whether `id` can make recipe `recipe` now.
    pub fn can_craft(&self, id: ActorId, recipe: u16) -> Result<(), NoCraft> {
        let r = self
            .defs
            .recipes
            .get(usize::from(recipe))
            .ok_or(NoCraft::NoRecipe)?;
        if self.actors.get(id).is_none_or(|a| a.dead) {
            return Err(NoCraft::NoRecipe);
        }
        if !r.tools.is_empty() && !self.has_tool(id, &r.tools) {
            return Err(NoCraft::NoTool);
        }
        match r
            .uses
            .iter()
            .find(|&&(item, n)| self.carried(id, item) < u32::from(n))
        {
            Some(&(item, _)) => Err(NoCraft::Lacks(item)),
            None if r.fire && !self.by_a_fire(id) => Err(NoCraft::NoFire),
            None => Ok(()),
        }
    }

    /// Ticks a job of `ticks` takes `id` with `skill`: Carpentry makes it quicker, as it does
    /// boarding up, and Cooking a meal.
    fn job_ticks(&self, id: ActorId, ticks: u32, skill: Option<Skill>) -> u32 {
        match skill {
            Some(Skill::Carpentry) => {
                let level = self.skill_level(id, Skill::Carpentry);
                self.rules.skills.board_ticks(ticks, level)
            }
            Some(Skill::Cooking) => {
                let level = self.skill_level(id, Skill::Cooking);
                self.rules.skills.cook_ticks(ticks, level)
            }
            _ => ticks.max(1),
        }
    }

    /// The making's start.
    pub(crate) fn craft_action(&self, id: ActorId, recipe: u16) -> Option<(ActionKind, u32)> {
        self.can_craft(id, recipe).ok()?;
        let r = &self.defs.recipes[usize::from(recipe)];
        Some((
            ActionKind::Craft { recipe },
            self.job_ticks(id, r.time, r.skill),
        ))
    }

    /// The making's end: its makings used up (the pack's first, then the hand's), what it makes
    /// stowed, and its skill taught. Only if it still can be made.
    pub(crate) fn craft(&mut self, id: ActorId, recipe: u16) {
        if self.can_craft(id, recipe).is_err() {
            return;
        }
        let defs = self.defs.clone();
        let r = &defs.recipes[usize::from(recipe)];
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        for &(item, n) in &r.uses {
            let in_pack: u32 = a
                .items
                .iter()
                .filter(|i| i.def == item)
                .map(|i| u32::from(i.count))
                .sum();
            let from_pack = in_pack.min(u32::from(n)) as u16;
            spend(&mut a.items, item, from_pack);
            if from_pack < n && a.hands.is_some_and(|h| h.def == item) {
                a.hands = None;
            }
        }
        let (made, count) = r.makes;
        self.carry_in(id, Item::new(made, count));
        self.events.push(Event::Crafted { who: id, recipe });
        if let Some(skill) = r.skill {
            self.train(id, skill, r.xp, On::Once);
        }
    }

    /// How many planks the fixture on `at` breaks down into: 0 if it doesn't.
    fn planks_at(&self, at: TilePos) -> u16 {
        let fixture = self.grid.tile(at).fixture;
        self.defs
            .planks
            .get(usize::from(fixture))
            .copied()
            .unwrap_or(0)
    }

    /// The furniture `id` would break down: the one it faces, else the first round it, clockwise
    /// from its facing, within reach and not on hallowed ground.
    pub fn furniture_beside(&self, id: ActorId) -> Option<TilePos> {
        let a = self.actors.get(id).filter(|a| !a.dead)?;
        let (pos, facing) = (a.pos, a.facing);
        (0..8).map(|t| pos.step(facing.turned(t))).find(|&at| {
            self.planks_at(at) > 0 && within_reach(&self.grid, pos, at) && !self.is_hallowed(at)
        })
    }

    /// Whether `id` can break furniture down now, and which.
    pub fn can_break(&self, id: ActorId) -> Result<TilePos, NoBreak> {
        let at = self.furniture_beside(id).ok_or(NoBreak::Nothing)?;
        let tools = self
            .defs
            .breaking
            .as_ref()
            .map_or(&[][..], |b| &b.tools[..]);
        if !self.has_tool(id, tools) {
            return Err(NoBreak::NoTool);
        }
        Ok(at)
    }

    /// The breaking's start: which way, and how long. The work is loud from the first blow.
    pub(crate) fn break_action(&mut self, id: ActorId) -> Option<(ActionKind, Dir8, u32)> {
        let at = self.can_break(id).ok()?;
        let b = self.defs.breaking.clone()?;
        let ticks = self.job_ticks(id, b.time, Some(Skill::Carpentry));
        self.emit(Sound {
            at,
            loudness: b.noise,
            by: Some(id),
        });
        Some((ActionKind::BreakDown { at }, self.toward(id, at), ticks))
    }

    /// The breaking's end: the furniture's gone, its planks in your pack, and it's heard again.
    pub(crate) fn break_down(&mut self, id: ActorId, at: TilePos) {
        if self.can_break(id) != Ok(at) {
            return;
        }
        let Some(b) = self.defs.breaking.clone() else {
            return;
        };
        let planks = self.planks_at(at);
        let tile = self.grid.tile(at);
        self.grid.set_tile(
            at,
            Tile {
                fixture: 0,
                solid: false,
                opaque: false,
                ..tile
            },
        );
        self.carry_in(id, Item::new(b.plank, planks));
        self.emit(Sound {
            at,
            loudness: b.noise,
            by: Some(id),
        });
        self.events.push(Event::BrokeDown {
            who: id,
            at,
            planks,
        });
        self.train(id, Skill::Carpentry, b.xp, On::Once);
    }
}
