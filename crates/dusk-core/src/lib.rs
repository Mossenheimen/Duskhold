//! Duskhold's simulation core: the world and every rule of the game, with no graphics, no files
//! and no threads. The game client, the tools and, later, the bot all drive it the same way:
//! [`World::intent`] in, [`World::step`] to advance, the world's state and [`Event`]s out.
//!
//! Coordinates: x grows east, y grows south (down the screen, so north is -y), z is the floor
//! (0 is the ground, below it cellars). A tile owns its north and west edges, which hold walls,
//! doors and windows, as in Project Zomboid.

pub mod actor;
pub mod arena;
pub mod geom;
pub mod grid;
pub mod hash;
pub mod path;
pub mod senses;
pub mod text;
pub mod world;

pub use actor::{Actor, ActorId, ActorKind, Intent, Mind};
pub use geom::{ChunkPos, Dir8, TilePos};
pub use grid::{Edge, Grid, Ground, Tile};
pub use world::{Event, Rules, World};
