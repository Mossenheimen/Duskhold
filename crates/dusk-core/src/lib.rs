//! Duskhold's simulation core: the world and every rule of the game, with no graphics, no files
//! and no threads. The game client, the tools and, later, the bot all drive it the same way:
//! [`World::intent`] in, [`World::step`] to advance, the world's state and [`Event`]s out.
//!
//! Coordinates: x grows east, y grows south (down the screen, so north is -y), z is the floor
//! (0 is the ground, below it cellars). A tile owns its north and west edges, which hold walls,
//! doors and windows, as in Project Zomboid.

pub mod actor;
pub mod arena;
pub mod barricade;
pub mod body;
mod combat;
pub mod creature;
pub mod demo;
pub mod fever;
pub mod flow;
pub mod geom;
mod grab;
pub mod grid;
pub mod hash;
pub mod holdout;
pub mod home;
pub mod items;
pub mod legacy;
pub mod light;
pub mod needs;
pub mod path;
pub mod pressure;
pub mod rare;
pub mod remains;
#[cfg(feature = "save")]
pub mod save;
pub mod senses;
pub mod skills;
pub mod text;
pub mod tolling;
pub mod view;
pub mod world;

pub use actor::{Action, ActionKind, Actor, ActorId, ActorKind, Alert, Intent, Mind, Remains};
pub use barricade::NoBoard;
pub use body::{Body, Part, Wound, WoundKind};
pub use creature::{Creature, CreatureDef, GetUp};
pub use geom::{ChunkPos, Dir8, TilePos};
pub use grid::{Edge, Grid, Ground, Tile};
pub use holdout::{Area, Bounty, Holdout, NoDeal, Sale};
pub use home::NoStash;
pub use items::{Defs, Item, ItemDef, ItemKind, LootEntry, LootTable};
pub use needs::{Moodle, NoSleep, Woke};
pub use pressure::{Pressure, Region};
pub use rare::Affix;
pub use remains::NoBurn;
pub use skills::{Rise, Skill, Skills, Standing};
pub use tolling::Tolling;
pub use view::View;
pub use world::{Event, Rules, World};
