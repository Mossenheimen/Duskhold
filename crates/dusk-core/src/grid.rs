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
    /// Water shallow enough to wade: a ford, a river's edge. Slow going, and loud
    /// (`Rules::shallows`), and you can drink from it.
    Shallows,
    /// Planks over water: a bridge, walked as a road is.
    Bridge,
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
    /// Climbed through when it's open and has no boards on it (`climb.rs`). A boarded one can't
    /// be seen through. A broken one (its glass beaten out) is open for good: it can't be shut,
    /// and what's left in the frame cuts whoever climbs through.
    Window {
        open: bool,
        boards: u8,
        broken: bool,
    },
    /// Stops walking, but not sight or sound: it's climbed over (`climb.rs`).
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
        Edge::Window {
            open,
            boards: 0,
            broken: false,
        }
    }

    /// A window with its glass beaten out, and no boards on it.
    pub const fn broken_window() -> Edge {
        Edge::Window {
            open: true,
            boards: 0,
            broken: true,
        }
    }

    /// Can you walk through it?
    pub fn walkable(self) -> bool {
        matches!(self, Edge::Open | Edge::Door { open: true, .. })
    }

    /// Can you climb through it or over it (`climb.rs`)? An open window with no boards across
    /// it, and a fence.
    pub fn climbable(self) -> bool {
        matches!(
            self,
            Edge::Fence
                | Edge::Window {
                    open: true,
                    boards: 0,
                    ..
                }
        )
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

    /// Is something across it that a blow beats on (`World::thump`)? A shut door; a window
    /// shut on its glass, or with boards across it.
    pub fn beatable(self) -> bool {
        match self {
            Edge::Door { open, .. } => !open,
            Edge::Window { open, boards, .. } => !open || boards > 0,
            _ => false,
        }
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
            Edge::Window { open, broken, .. } => Edge::Window {
                open,
                boards: n,
                broken,
            },
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
///
/// Every rule reads the grid (each step, each line of sight, each sound carried), so finding a
/// tile's chunk has to be cheap: the chunks sit in a list, and a dense table over the box they
/// cover says which is where, one array read. The sorted index keeps the grid's order for
/// comparing, hashing and saving, whatever order the chunks were made in.
#[derive(Clone, Debug, Default)]
pub struct Grid {
    /// The chunks, in the order they were made.
    store: Vec<Chunk>,
    /// Each chunk's place in `store`, by position: sorted.
    order: BTreeMap<ChunkPos, u32>,
    /// Which chunk is where, over the box the chunks cover.
    dense: Dense,
    /// From `index_steps` on: for each chunk (by its place in `store`), each tile's open steps,
    /// a bit for each of `Dir8::ALL` (`steps_from`), and above them its climbs (`climbs_from`).
    /// Kept up to date as the grid changes.
    steps: Option<Vec<Vec<u16>>>,
    /// Whether changes are being tracked (from `track_changes` on), and the chunks changed
    /// since: what a save keeps, the rest coming back from the seed.
    tracking: bool,
    changed: BTreeSet<ChunkPos>,
    edits: Edits,
}

/// A table over the box of chunk positions from (`x0`, `y0`), `w` × `h`: each holds its chunk's
/// place in the grid's list, or `NO_CHUNK`. Empty (0 × 0) when the chunks are too scattered for
/// a table to be worth its memory: then the sorted index is asked instead.
#[derive(Clone, Debug, Default)]
struct Dense {
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    slots: Vec<u32>,
}

const NO_CHUNK: u32 = u32::MAX;
/// The most entries the dense table may have (a box of 512 × 512 chunks: 1 MB).
const DENSE_MOST: i64 = 1 << 18;

impl PartialEq for Grid {
    /// The same chunks in the same places, tracked the same way: however they came to be made.
    fn eq(&self, other: &Grid) -> bool {
        self.tracking == other.tracking
            && self.changed == other.changed
            && self.order.len() == other.order.len()
            && self.chunks().zip(other.chunks()).all(|(a, b)| a == b)
    }
}

impl Eq for Grid {}

impl std::hash::Hash for Grid {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.order.len().hash(h);
        for (at, chunk) in self.chunks() {
            at.hash(h);
            chunk.hash(h);
        }
        self.tracking.hash(h);
        self.changed.hash(h);
    }
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

    /// Every chunk and where it is, in order of position.
    fn chunks(&self) -> impl Iterator<Item = (&ChunkPos, &Chunk)> {
        self.order
            .iter()
            .map(|(at, &slot)| (at, &self.store[slot as usize]))
    }

    /// The chunk at a position, if there is one: by the dense table, else the sorted index.
    #[inline]
    fn chunk(&self, at: ChunkPos) -> Option<&Chunk> {
        self.slot(at).map(|slot| &self.store[slot])
    }

    /// A chunk's place in `store`, if there's a chunk at that position.
    #[inline]
    fn slot(&self, at: ChunkPos) -> Option<usize> {
        let d = &self.dense;
        if d.w > 0 {
            // Off the table's box, the difference wraps round to a huge number.
            let (dx, dy) = (
                at.x.wrapping_sub(d.x0) as u32,
                at.y.wrapping_sub(d.y0) as u32,
            );
            if dx >= d.w as u32 || dy >= d.h as u32 {
                return None;
            }
            let slot = d.slots[(dy * d.w as u32 + dx) as usize];
            return (slot != NO_CHUNK).then_some(slot as usize);
        }
        self.order.get(&at).map(|&slot| slot as usize)
    }

    /// The chunk at a position, made (void) if there isn't one yet.
    fn chunk_mut(&mut self, at: ChunkPos) -> &mut Chunk {
        let slot = match self.order.get(&at) {
            Some(&slot) => slot,
            None => {
                let slot = self.store.len() as u32;
                self.store.push(Chunk::default());
                if let Some(steps) = &mut self.steps {
                    steps.push(vec![0; CELLS]);
                }
                self.order.insert(at, slot);
                self.redo_table();
                slot
            }
        };
        &mut self.store[slot as usize]
    }

    /// Lays the dense table out again over the box the chunks cover (a new chunk was made).
    fn redo_table(&mut self) {
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for at in self.order.keys() {
            (x0, y0) = (x0.min(at.x), y0.min(at.y));
            (x1, y1) = (x1.max(at.x), y1.max(at.y));
        }
        let (w, h) = (
            i64::from(x1) - i64::from(x0) + 1,
            i64::from(y1) - i64::from(y0) + 1,
        );
        if self.order.is_empty() || w * h > DENSE_MOST {
            self.dense = Dense::default();
            return;
        }
        let (w, h) = (w as i32, h as i32);
        let mut slots = vec![NO_CHUNK; (w * h) as usize];
        for (at, &slot) in &self.order {
            slots[((at.y - y0) * w + (at.x - x0)) as usize] = slot;
        }
        self.dense = Dense {
            x0,
            y0,
            w,
            h,
            slots,
        };
    }

    pub fn tile(&self, pos: TilePos) -> Tile {
        match (self.chunk(pos.chunk()), index(pos)) {
            (Some(chunk), Some(i)) => chunk.tiles[i],
            _ => Tile::VOID,
        }
    }

    pub fn set_tile(&mut self, pos: TilePos, tile: Tile) {
        if let Some(i) = index(pos) {
            self.chunk_mut(pos.chunk()).tiles[i] = tile;
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
        if self.steps.is_some() {
            // A tile or an edge here changes the steps of the tiles round it: two tiles out
            // covers a diagonal's corners.
            self.restep(pos.x - 2, pos.y - 2, pos.x + 2, pos.y + 2, pos.z);
        }
    }

    /// From now on, the grid keeps each tile's open steps (`steps_from`), so a search reads a
    /// byte a tile instead of looking up its neighbours and their edges. The generator calls
    /// this once the land is made; a grid that never does works the steps out each time, the
    /// same answer, slower.
    pub fn index_steps(&mut self) {
        self.steps = Some(vec![vec![0; CELLS]; self.store.len()]);
        let all: Vec<ChunkPos> = self.order.keys().copied().collect();
        for at in all {
            let o = at.origin(0);
            for z in Z_MIN..=Z_MAX {
                self.restep(o.x, o.y, o.x + CHUNK_SIZE - 1, o.y + CHUNK_SIZE - 1, z);
            }
        }
    }

    /// Works out again the open steps of the tiles from (x0, y0) to (x1, y1) on floor `z`,
    /// both corners in. A tile nobody can stand on has none.
    fn restep(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, z: i8) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                let p = TilePos::new(x, y, z);
                let (Some(slot), Some(i)) = (self.slot(p.chunk()), index(p)) else {
                    continue;
                };
                let open = self.work_out_steps(p);
                if let Some(steps) = &mut self.steps {
                    steps[slot][i] = open;
                }
            }
        }
    }

    /// A tile's open steps, and its climbs a byte above, worked out from the tiles and edges
    /// round it.
    fn work_out_steps(&self, p: TilePos) -> u16 {
        if !self.tile(p).walkable() {
            return 0;
        }
        Dir8::ALL.iter().enumerate().fold(0, |bits, (k, &d)| {
            if self.can_step(p, d) {
                bits | 1 << k
            } else if self.can_climb(p, d) {
                bits | 1 << (k + 8)
            } else {
                bits
            }
        })
    }

    /// Which steps from a tile are open: bit k for `Dir8::ALL[k]`, as `can_step` answers for
    /// each. Nothing from a tile nobody can stand on.
    #[inline]
    pub fn steps_from(&self, p: TilePos) -> u8 {
        self.ways_from(p) as u8
    }

    /// Which steps from a tile are climbs, as `can_climb` answers for each: bit k for
    /// `Dir8::ALL[k]` (only the four straight ones ever are).
    #[inline]
    pub fn climbs_from(&self, p: TilePos) -> u8 {
        (self.ways_from(p) >> 8) as u8
    }

    /// The steps from a tile, and its climbs a byte above: from the cache, else worked out.
    #[inline]
    pub(crate) fn ways_from(&self, p: TilePos) -> u16 {
        if let Some(steps) = &self.steps {
            return match (self.slot(p.chunk()), index(p)) {
                (Some(slot), Some(i)) => steps[slot][i],
                _ => 0,
            };
        }
        self.work_out_steps(p)
    }

    /// The chunks changed since `track_changes`, in order: every chunk, for a grid that never
    /// began tracking (a hand-made map has no seed to come back from).
    pub fn changed_chunks(&self) -> Vec<(ChunkPos, Chunk)> {
        self.chunks()
            .filter(|(at, _)| !self.tracking || self.changed.contains(at))
            .map(|(&at, chunk)| (at, chunk.clone()))
            .collect()
    }

    /// Puts a whole chunk in place (a saved one, over the one the seed made).
    pub fn put_chunk(&mut self, at: ChunkPos, chunk: Chunk) {
        *self.chunk_mut(at) = chunk;
        self.edits.0 += 1;
        if self.steps.is_some() {
            // The whole chunk, and the tiles round it that step into it.
            let o = at.origin(0);
            for z in Z_MIN..=Z_MAX {
                self.restep(
                    o.x - 2,
                    o.y - 2,
                    o.x + CHUNK_SIZE + 1,
                    o.y + CHUNK_SIZE + 1,
                    z,
                );
            }
        }
        if self.tracking {
            self.changed.insert(at);
        }
    }

    /// The edge on a tile's north side.
    pub fn north(&self, pos: TilePos) -> Edge {
        match (self.chunk(pos.chunk()), index(pos)) {
            (Some(chunk), Some(i)) => chunk.north[i],
            _ => Edge::Open,
        }
    }

    /// The edge on a tile's west side.
    pub fn west(&self, pos: TilePos) -> Edge {
        match (self.chunk(pos.chunk()), index(pos)) {
            (Some(chunk), Some(i)) => chunk.west[i],
            _ => Edge::Open,
        }
    }

    pub fn set_north(&mut self, pos: TilePos, edge: Edge) {
        if let Some(i) = index(pos) {
            self.chunk_mut(pos.chunk()).north[i] = edge;
            self.touch(pos);
        }
    }

    pub fn set_west(&mut self, pos: TilePos, edge: Edge) {
        if let Some(i) = index(pos) {
            self.chunk_mut(pos.chunk()).west[i] = edge;
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

    /// Can an actor climb from `pos` to its neighbour in `dir` (`climb.rs`): through an open
    /// window, or over a fence? Only straight across: nobody climbs a corner.
    pub fn can_climb(&self, pos: TilePos, dir: Dir8) -> bool {
        !dir.is_diagonal() && self.edge(pos, dir).climbable() && self.tile(pos.step(dir)).walkable()
    }

    /// How many chunks exist.
    pub fn chunk_count(&self) -> usize {
        self.order.len()
    }
}
