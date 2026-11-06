//! Actors (the character, NPCs, creatures) and what their controllers ask of them.

use crate::geom::{Dir8, TilePos};

pub type ActorId = crate::arena::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActorKind {
    Character,
    Shambler,
}

/// What a controller (the player, a bot, an NPC's or a creature's AI) asks an actor to do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Intent {
    #[default]
    Idle,
    /// Keep walking (or running) this way.
    Move { dir: Dir8, run: bool },
    /// Walk (or run) to a tile, finding the way there.
    WalkTo { to: TilePos, run: bool },
}

/// A step under way from one tile to the next. `progress` and `length` are in thousandths of a
/// tile: 1000 for a straight step, 1414 for a diagonal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    pub from: TilePos,
    pub to: TilePos,
    pub dir: Dir8,
    pub progress: u32,
    pub length: u32,
    pub run: bool,
}

/// A creature's thoughts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Mind {
    #[default]
    None,
    /// Zomboid's zombie: goes after the last thing it heard or saw, and gives up after a while.
    Shambler {
        target: Option<TilePos>,
        /// The tick it last heard or saw something.
        since: u64,
        /// Next to its prey right now.
        touching: bool,
        /// It saw its prey when it last thought.
        sees: bool,
    },
}

/// What a creature knows of its prey: the mark the client draws over its head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Alert {
    /// Not after anything.
    Unaware,
    /// After something it heard, or the place it last saw prey: "?".
    Curious,
    /// It sees its prey: "!".
    Hunting,
}

/// How fast a kind of actor moves, in thousandths of a tile a second. (These go into the data
/// files at build step 4.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Speeds {
    pub walk: u32,
    pub run: u32,
}

impl ActorKind {
    pub fn speeds(self) -> Speeds {
        match self {
            ActorKind::Character => Speeds {
                walk: 1500,
                run: 3000,
            },
            // Shamblers shuffle when they wander and lurch when they're after something.
            ActorKind::Shambler => Speeds {
                walk: 500,
                run: 800,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Actor {
    pub kind: ActorKind,
    pub pos: TilePos,
    pub facing: Dir8,
    pub intent: Intent,
    pub step: Option<Step>,
    /// The way to a `WalkTo` goal, the next tile last.
    pub path: Vec<TilePos>,
    /// The goal `path` leads to.
    pub path_goal: Option<TilePos>,
    /// The last `WalkTo` goal no way was found to.
    pub no_way: Option<TilePos>,
    /// Ticks spent waiting on the way ahead (a tile someone is in, a door that shut).
    pub blocked: u32,
    /// Progress left over from the last step, carried into the next, so speeds come out exact.
    pub carry: u32,
    pub mind: Mind,
}

impl Actor {
    pub fn new(kind: ActorKind, pos: TilePos) -> Actor {
        let mind = match kind {
            ActorKind::Shambler => Mind::Shambler {
                target: None,
                since: 0,
                touching: false,
                sees: false,
            },
            ActorKind::Character => Mind::None,
        };
        Actor {
            kind,
            pos,
            facing: Dir8::S,
            intent: Intent::Idle,
            step: None,
            path: Vec::new(),
            path_goal: None,
            no_way: None,
            blocked: 0,
            carry: 0,
            mind,
        }
    }

    /// Where the actor is in tiles, part way between two while a step is under way. For drawing
    /// it smoothly; game logic uses `pos`.
    pub fn position(&self) -> (f32, f32) {
        match self.step {
            Some(s) => {
                let t = (s.progress as f32 / s.length as f32).min(1.0);
                (
                    s.from.x as f32 + (s.to.x - s.from.x) as f32 * t,
                    s.from.y as f32 + (s.to.y - s.from.y) as f32 * t,
                )
            }
            None => (self.pos.x as f32, self.pos.y as f32),
        }
    }

    /// What the actor knows of its prey (a creature's; the character is always `Unaware`).
    pub fn alert(&self) -> Alert {
        match self.mind {
            Mind::Shambler { sees: true, .. } => Alert::Hunting,
            Mind::Shambler {
                target: Some(_), ..
            } => Alert::Curious,
            _ => Alert::Unaware,
        }
    }
}
