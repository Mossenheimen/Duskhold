//! Tiles, the edges between them, and the chunks that hold both.

use std::collections::{BTreeMap, BTreeSet};

use crate::geom::{CHUNK_SIZE, ChunkPos, Dir8, TilePos, Z_LEVELS, Z_MAX, Z_MIN};

/// What a tile has to stand on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Ground {
    /// Nothing: outside the world, or a hole. Blocks movement and sight.
    #[default]
    Void,
    /// Grass and wild ground, outdoors.
    Grass,
    /// Bare earth: paths, yards, ploughed fields.
    Dirt,
    /// A made road.
    Road,
    /// Water too deep to wade: nobody walks it, but it doesn't stop sight.
    Water,
    /// A built floor.
    Floor,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Tile {
    pub ground: Ground,
    /// What stands on it, by its number in the data's fixture list; 0 for nothing.
    pub fixture: u16,
    /// Nobody walks through it: furniture, a tree, rock.
    pub solid: bool,
    /// Nobody sees through it: a tree, a wardrobe, rock.
    pub opaque: bool,
    /// Under a roof: indoors, for sunlight and rain.
    pub roof: bool,
}

impl Tile {
    pub const VOID: Tile = Tile {
        ground: Ground::Void,
        fixture: 0,
        solid: false,
        opaque: false,
        roof: false,
    };
    pub const GRASS: Tile = Tile {
        ground: Ground::Grass,
        ..Tile::VOID
    };
    pub const FLOOR: Tile = Tile {
        ground: Ground::Floor,
        roof: true,
        ..Tile::VOID
    };
    /// Something with no fixture of its own that blocks movement and sight: test maps' `#`.
    pub const ROCK: Tile = Tile {
        solid: true,
        opaque: true,
        ..Tile::FLOOR
    };

    pub fn walkable(self) -> bool {
        !matches!(self.ground, Ground::Void | Ground::Water) && !self.solid
    }

    pub fn blocks_sight(self) -> bool {
        self.ground == Ground::Void || self.opaque
    }
}

/// What's on the edge between two tiles: Zomboid's walls, doors and windows. Doors and windows
/// carry the boards nailed across them (`barricade.rs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Edge {
    #[default]
    Open,
    Wall,
    /// A boarded door can't be opened (only shut doors are boarded).
    Door {
        open: bool,
        boards: u8,
    },
    /// Nobody climbs through a window yet. A boarded one can't be seen through.
    Window {
        open: bool,
        boards: u8,
    },
    /// Stops walking (climbing comes later), but not sight or sound.
    Fence,
}

impl Edge {
    pub const SHUT_DOOR: Edge = Edge::door(false);

    /// A door with no boards on it.
    pub const fn door(open: bool) -> Edge {
        Edge::Door { open, boards: 0 }
    }

    /// A window with no boards on it.
    pub const fn window(open: bool) -> Edge {
        Edge::Window { open, boards: 0 }
    }

    /// Can you walk through it?
    pub fn walkable(self) -> bool {
        matches!(self, Edge::Open | Edge::Door { open: true, .. })
    }

    /// Does it stop sight? Walls, shut doors and boards do; glass doesn't.
    pub fn blocks_sight(self) -> bool {
        matches!(
            self,
            Edge::Wall | Edge::Door { open: false, .. } | Edge::Window { boards: 1.., .. }
        )
    }

    pub fn is_shut_door(self) -> bool {
        matches!(self, Edge::Door { open: false, .. })
    }

    /// The boards across a door or window (0 for anything else).
    pub fn boards(self) -> u8 {
        match self {
            Edge::Door { boards, .. } | Edge::Window { boards, .. } => boards,
            _ => 0,
        }
    }

    /// The same door or window with `n` boards across it (anything else as it is).
    pub fn with_boards(self, n: u8) -> Edge {
        match self {
            Edge::Door { open, .. } => Edge::Door { open, boards: n },
            Edge::Window { open, .. } => Edge::Window { open, boards: n },
            other => other,
        }
    }
}

const CELLS: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize * Z_LEVELS;

/// 32 × 32 tiles on every floor, each with its north and west edge.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Chunk {
    tiles: Vec<Tile>,
    north: Vec<Edge>,
    west: Vec<Edge>,
}

impl Default for Chunk {
    fn default() -> Chunk {
        Chunk {
            tiles: vec![Tile::VOID; CELLS],
            north: vec![Edge::Open; CELLS],
            west: vec![Edge::Open; CELLS],
        }
    }
}

/// Where a tile's data sits inside its chunk; None for a floor out of range.
fn index(pos: TilePos) -> Option<usize> {
    if !(Z_MIN..=Z_MAX).contains(&pos.z) {
        return None;
    }
    let (lx, ly) = pos.local();
    let size = CHUNK_SIZE as usize;
    Some(((pos.z - Z_MIN) as usize * size + ly) * size + lx)
}

/// The tile that owns the edge on a tile's side, and which of its own sides that is (north or
/// west): how doors are named in events and in the door damage the world keeps.
pub fn edge_owner(pos: TilePos, dir: Dir8) -> (TilePos, Dir8) {
    match dir {
        Dir8::S => (pos.step(Dir8::S), Dir8::N),
        Dir8::E => (pos.step(Dir8::E), Dir8::W),
        Dir8::N | Dir8::W => (pos, dir),
        _ => panic!("a diagonal ({dir:?}) crosses two edges, not one"),
    }
}

/// The world's tiles, chunk by chunk. A tile in a chunk that doesn't exist is void.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Grid {
    chunks: BTreeMap<ChunkPos, Chunk>,
    /// Whether changes are being tracked (from `track_changes` on), and the chunks changed
    /// since: what a save keeps, the rest coming back from the seed.
    tracking: bool,
    changed: BTreeSet<ChunkPos>,
    edits: Edits,
}

/// How many times the grid has been changed: for caches built from it (the Tolling's flow
/// field) to know they're stale. Not part of the grid's state: it neither hashes nor compares,
/// so a loaded world is the same world as the one saved.
#[derive(Clone, Copy, Debug, Default)]
struct Edits(u64);

impl PartialEq for Edits {
    fn eq(&self, _: &Edits) -> bool {
        true
    }
}

impl Eq for Edits {}

impl std::hash::Hash for Edits {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

impl Grid {
    pub fn new() -> Grid {
        Grid::default()
    }

    pub fn tile(&self, pos: TilePos) -> Tile {
        match (self.chunks.get(&pos.chunk()), index(pos)) {
            (Some(chunk), Some(i)) => chunk.tiles[i],
            _ => Tile::VOID,
        }
    }

    pub fn set_tile(&mut self, pos: TilePos, tile: Tile) {
        if let Some(i) = index(pos) {
            self.chunks.entry(pos.chunk()).or_default().tiles[i] = tile;
            self.touch(pos);
        }
    }

    /// From now on, the grid notes which chunks change. The generator calls this once the land
    /// is made, so a save holds only what play changed.
    pub fn track_changes(&mut self) {
        self.tracking = true;
        self.changed.clear();
    }

    /// How many times the grid has changed (see `Edits`).
    pub fn edits(&self) -> u64 {
        self.edits.0
    }

    fn touch(&mut self, pos: TilePos) {
        self.edits.0 += 1;
        if self.tracking {
            self.changed.insert(pos.chunk());
        }
    }

    /// The chunks changed since `track_changes`, in order: every chunk, for a grid that never
    /// began tracking (a hand-made map has no seed to come back from).
    pub fn changed_chunks(&self) -> Vec<(ChunkPos, Chunk)> {
        self.chunks
            .iter()
            .filter(|(at, _)| !self.tracking || self.changed.contains(at))
            .map(|(&at, chunk)| (at, chunk.clone()))
            .collect()
    }

    /// Puts a whole chunk in place (a saved one, over the one the seed made).
    pub fn put_chunk(&mut self, at: ChunkPos, chunk: Chunk) {
        self.chunks.insert(at, chunk);
        self.edits.0 += 1;
        if self.tracking {
            self.changed.insert(at);
        }
    }

    /// The edge on a tile's north side.
    pub fn north(&self, pos: TilePos) -> Edge {
        match (self.chunks.get(&pos.chunk()), index(pos)) {
            (Some(chunk), Some(i)) => chunk.north[i],
            _ => Edge::Open,
        }
    }

    /// The edge on a tile's west side.
    pub fn west(&self, pos: TilePos) -> Edge {
        match (self.chunks.get(&pos.chunk()), index(pos)) {
            (Some(chunk), Some(i)) => chunk.west[i],
            _ => Edge::Open,
        }
    }

    pub fn set_north(&mut self, pos: TilePos, edge: Edge) {
        if let Some(i) = index(pos) {
            self.chunks.entry(pos.chunk()).or_default().north[i] = edge;
            self.touch(pos);
        }
    }

    pub fn set_west(&mut self, pos: TilePos, edge: Edge) {
        if let Some(i) = index(pos) {
            self.chunks.entry(pos.chunk()).or_default().west[i] = edge;
            self.touch(pos);
        }
    }

    /// The edge between a tile and its neighbour in an orthogonal direction. (A diagonal crosses
    /// two edges; see [`Dir8::split`].)
    pub fn edge(&self, pos: TilePos, dir: Dir8) -> Edge {
        match dir {
            Dir8::N => self.north(pos),
            Dir8::S => self.north(pos.step(Dir8::S)),
            Dir8::W => self.west(pos),
            Dir8::E => self.west(pos.step(Dir8::E)),
            _ => panic!("a diagonal ({dir:?}) crosses two edges, not one"),
        }
    }

    pub fn set_edge(&mut self, pos: TilePos, dir: Dir8, edge: Edge) {
        match dir {
            Dir8::N => self.set_north(pos, edge),
            Dir8::S => self.set_north(pos.step(Dir8::S), edge),
            Dir8::W => self.set_west(pos, edge),
            Dir8::E => self.set_west(pos.step(Dir8::E), edge),
            _ => panic!("a diagonal ({dir:?}) crosses two edges, not one"),
        }
    }

    /// Can an actor step from `pos` to its neighbour in `dir`? Other actors aren't counted. A
    /// diagonal step needs both of its sides open, so nobody cuts a corner past a wall.
    pub fn can_step(&self, pos: TilePos, dir: Dir8) -> bool {
        if !self.tile(pos.step(dir)).walkable() {
            return false;
        }
        match dir.split() {
            None => self.edge(pos, dir).walkable(),
            Some((a, b)) => {
                let (via_a, via_b) = (pos.step(a), pos.step(b));
                self.tile(via_a).walkable()
                    && self.tile(via_b).walkable()
                    && self.edge(pos, a).walkable()
                    && self.edge(via_a, b).walkable()
                    && self.edge(pos, b).walkable()
                    && self.edge(via_b, a).walkable()
            }
        }
    }

    /// How many chunks exist.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}
