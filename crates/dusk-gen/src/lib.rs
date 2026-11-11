//! Duskhold's world generation: a patch of a region, from a seed.
//!
//! First the large-scale layout, once: the road and the hamlet's lots. Then every tile is a
//! function of the seed, the layout and its place alone, so a chunk comes out the same whatever
//! order chunks are made in (`ARCHITECTURE.md`, world generation). Last, the dead: some in the
//! buildings, by lot, and some in the open, by chunk.

pub mod noise;

use dusk_core::save::SaveError;
use dusk_core::text::TextGrid;
use dusk_core::{ActorKind, ChunkPos, Edge, Ground, Tile, TilePos, World, geom::CHUNK_SIZE};
use dusk_data::{Data, PrefabCell, Region, Side, doors_on};

use noise::{Layer, fractal, hash, unit};

/// A building on its lot, turned to face the road, in its fenced yard.
#[derive(Clone, Debug)]
pub struct Lot {
    /// The prefab's id.
    pub prefab: String,
    /// The prefab's grid, turned and mirrored as placed.
    pub grid: TextGrid<PrefabCell>,
    /// Where the building's north-west cell is.
    pub origin: TilePos,
    /// The yard inside the fence: from (x0, y0) up to, not including, (x1, y1).
    pub yard: (i32, i32, i32, i32),
    /// North of the road (its front faces south), or south of it.
    pub north: bool,
    /// The tile just inside the front door.
    pub door: TilePos,
    /// The path from the door to the road, door end first.
    pub path: Vec<TilePos>,
}

impl Lot {
    fn in_yard(&self, x: i32, y: i32) -> bool {
        let (x0, y0, x1, y1) = self.yard;
        (x0..x1).contains(&x) && (y0..y1).contains(&y)
    }

    /// The building's cell at a tile, if the tile is inside its grid.
    fn cell(&self, x: i32, y: i32) -> Option<PrefabCell> {
        let (bx, by) = (x - self.origin.x, y - self.origin.y);
        ((0..self.grid.width).contains(&bx) && (0..self.grid.height).contains(&by))
            .then(|| self.grid.cell(bx, by))
    }

    /// Whether the building stands on a tile (a space in its grid doesn't count).
    fn covers(&self, x: i32, y: i32) -> bool {
        self.cell(x, y).is_some_and(|c| c.0.is_some())
    }
}

/// A patch's large-scale layout: everything decided once, before any chunk is made.
#[derive(Clone, Debug)]
pub struct Layout {
    pub seed: u64,
    pub region: Region,
    /// The patch's size in tiles.
    pub width: i32,
    pub height: i32,
    /// The road's middle line: its y at each bend, `bend_every` tiles apart from x = 0.
    bends: Vec<f32>,
    pub lots: Vec<Lot>,
    /// Where you start: on the road, near its west end.
    pub start: TilePos,
    /// A tree, as the data's fixtures have it.
    tree: Tile,
}

impl Layout {
    /// Lays out a patch of `region` from `seed`.
    pub fn new(seed: u64, data: &Data, region: &Region) -> Layout {
        let (width, height) = (
            region.patch.chunks.0 * CHUNK_SIZE,
            region.patch.chunks.1 * CHUNK_SIZE,
        );
        let road = &region.road;
        let bends = (0..=width / road.bend_every + 1)
            .map(|i| {
                let sway =
                    (unit(seed, Layer::Road, i64::from(i), 0) * 2.0 - 1.0) * road.sway as f32;
                height as f32 / 2.0 + sway
            })
            .collect();
        let tree = match data.fixtures.iter().position(|f| f.id == "tree") {
            Some(i) => Tile {
                fixture: (i + 1) as u16,
                solid: data.fixtures[i].solid,
                opaque: data.fixtures[i].opaque,
                ..Tile::GRASS
            },
            None => Tile {
                solid: true,
                opaque: true,
                ..Tile::GRASS
            },
        };
        let mut layout = Layout {
            seed,
            region: region.clone(),
            width,
            height,
            bends,
            lots: Vec::new(),
            start: TilePos::new(0, 0, 0),
            tree,
        };
        layout.lots = layout.place_lots(data);
        // You come along the road from the west, `start` tiles short of the hamlet.
        let hamlet_west = layout
            .lots
            .iter()
            .map(|l| l.yard.0)
            .min()
            .unwrap_or(width / 2);
        let start_x = (hamlet_west - road.start).clamp(1, width - 2);
        layout.start = TilePos::new(start_x, layout.road_rows(start_x).0 + road.width / 2, 0);
        layout
    }

    /// The road's middle line at x (tiles, fractional), eased from bend to bend.
    fn road_middle(&self, x: f32) -> f32 {
        let every = self.region.road.bend_every as f32;
        let i = ((x / every).floor().max(0.0) as usize).min(self.bends.len() - 2);
        let t = ((x - i as f32 * every) / every).clamp(0.0, 1.0);
        let eased = t * t * (3.0 - 2.0 * t);
        self.bends[i] + (self.bends[i + 1] - self.bends[i]) * eased
    }

    /// The road's first and last rows in a column.
    fn road_rows(&self, x: i32) -> (i32, i32) {
        let middle = self.road_middle(x as f32 + 0.5);
        let top = (middle - self.region.road.width as f32 / 2.0).round() as i32;
        (top, top + self.region.road.width - 1)
    }

    fn is_road(&self, x: i32, y: i32) -> bool {
        let (top, bottom) = self.road_rows(x);
        (top..=bottom).contains(&y)
    }

    /// The hamlet: its lots from the middle outward, on alternate sides of the road.
    fn place_lots(&self, data: &Data) -> Vec<Lot> {
        let h = &self.region.hamlet;
        let middle = self.width / 2
            + ((unit(self.seed, Layer::Hamlet, 0, 0) - 0.5) * self.width as f32 / 4.0) as i32;
        // How far each side of the road is taken, west and east: (west edge, east edge).
        let mut spans: [Option<(i32, i32)>; 2] = [None, None];
        let mut placed: [usize; 2] = [0, 0];
        let mut lots = Vec::new();
        for (i, tag) in h.lots.iter().enumerate() {
            let side = i % 2; // 0 north of the road, 1 south
            let roll = |what: i64| hash(self.seed, Layer::Hamlet, i as i64 + 1, what);
            let choices: Vec<_> = data
                .prefabs
                .iter()
                .filter(|p| p.tags.contains(tag))
                .collect();
            let Some(&prefab) = choices.get((roll(1) % choices.len().max(1) as u64) as usize)
            else {
                continue;
            };
            // Mirror it or not, then turn its front to face the road.
            let (mut grid, mut front) = if roll(2) % 2 == 1 {
                (prefab.grid.mirrored(), prefab.front.mirrored())
            } else {
                (prefab.grid.clone(), prefab.front)
            };
            let facing = if side == 0 { Side::South } else { Side::North };
            while front != facing {
                grid = grid.rotated();
                front = front.turned();
            }
            let (w, d) = (grid.width + 2 * h.yard, grid.height + 2 * h.yard);
            // Along the road: the first lot on a side centred, then east and west in turn.
            let x0 = match spans[side] {
                None => middle - w / 2,
                Some((_, east)) if placed[side] % 2 == 1 => east + h.gap,
                Some((west, _)) => west - h.gap - w,
            };
            let x1 = x0 + w;
            // Across the road: clear of it by the setback, wherever it bends.
            let (y0, y1) = if side == 0 {
                let top = (x0..x1).map(|x| self.road_rows(x).0).min().unwrap_or(0);
                (top - h.setback - d, top - h.setback)
            } else {
                let bottom = (x0..x1).map(|x| self.road_rows(x).1).max().unwrap_or(0);
                (bottom + 1 + h.setback, bottom + 1 + h.setback + d)
            };
            if x0 < 1 || y0 < 1 || x1 > self.width - 1 || y1 > self.height - 1 {
                continue; // it doesn't fit in the patch
            }
            spans[side] = Some(match spans[side] {
                None => (x0, x1),
                Some((west, east)) => (west.min(x0), east.max(x1)),
            });
            placed[side] += 1;

            let origin = TilePos::new(x0 + h.yard, y0 + h.yard, 0);
            let doors = doors_on(&grid, facing);
            let along = doors[doors.len() / 2];
            let (door, step) = if side == 0 {
                (origin.offset(along, grid.height - 1), 1)
            } else {
                (origin.offset(along, 0), -1)
            };
            let mut path = Vec::new();
            let mut at = door.offset(0, step);
            while !self.is_road(at.x, at.y) && (0..self.height).contains(&at.y) {
                path.push(at);
                at = at.offset(0, step);
            }
            lots.push(Lot {
                prefab: prefab.id.clone(),
                grid,
                origin,
                yard: (x0, y0, x1, y1),
                north: side == 0,
                door,
                path,
            });
        }
        lots
    }

    /// The land at a tile, before anything is built on it.
    fn land(&self, x: i32, y: i32) -> Tile {
        let t = &self.region.terrain;
        let (fx, fy) = (x as f32, y as f32);
        if fractal(self.seed, Layer::Height, fx, fy, t.hill_size) < t.pond_below {
            return Tile {
                ground: Ground::Water,
                ..Tile::GRASS
            };
        }
        let damp = fractal(self.seed, Layer::Damp, fx, fy, t.hill_size * 0.8);
        let woods = ((damp - t.woods_from) / (t.woods_thick - t.woods_from)).clamp(0.0, 1.0);
        let chance = (t.woods_trees * woods * woods * (3.0 - 2.0 * woods)).max(t.field_trees);
        if unit(self.seed, Layer::Tree, i64::from(x), i64::from(y)) < chance {
            return self.tree;
        }
        Tile::GRASS
    }

    /// The tile at (x, y): the land, then the road, then the lots. A pure function of the seed,
    /// the layout and the place.
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if !(0..self.width).contains(&x) || !(0..self.height).contains(&y) {
            return Tile::VOID;
        }
        if self.is_road(x, y) {
            return Tile {
                ground: Ground::Road,
                ..Tile::GRASS
            };
        }
        // Buildings first, then paths, then the cleared yards: one lot's yard never covers a
        // neighbour's building or path.
        for lot in &self.lots {
            if let Some(PrefabCell(Some(tile))) = lot.cell(x, y) {
                return tile;
            }
        }
        if self
            .lots
            .iter()
            .any(|l| l.path.contains(&TilePos::new(x, y, 0)))
        {
            return Tile {
                ground: Ground::Dirt,
                ..Tile::GRASS
            };
        }
        // A yard and its fence line are cleared of trees and water.
        let near_yard = |l: &Lot| {
            let (x0, y0, x1, y1) = l.yard;
            (x0 - 1..=x1).contains(&x) && (y0 - 1..=y1).contains(&y)
        };
        if self.lots.iter().any(near_yard) {
            return Tile::GRASS;
        }
        self.land(x, y)
    }

    /// The edge on a tile's north side: a building's, else a yard's fence (with a gate where
    /// the path goes through), else open.
    pub fn north_edge(&self, x: i32, y: i32) -> Edge {
        for lot in &self.lots {
            let (bx, by) = (x - lot.origin.x, y - lot.origin.y);
            let g = &lot.grid;
            if (0..g.width).contains(&bx) && (0..=g.height).contains(&by) {
                let edge = g.north(bx, by);
                if edge != Edge::Open || lot.covers(x, y) || lot.covers(x, y - 1) {
                    return edge;
                }
            }
        }
        for lot in &self.lots {
            let (x0, y0, x1, y1) = lot.yard;
            if (x0..x1).contains(&x) && (y == y0 || y == y1) {
                let road_side = if lot.north { y == y1 } else { y == y0 };
                let gate = road_side && lot.path.first().is_some_and(|p| p.x == x);
                return if gate { Edge::Open } else { Edge::Fence };
            }
        }
        Edge::Open
    }

    /// The edge on a tile's west side: a building's, else a yard's fence, else open.
    pub fn west_edge(&self, x: i32, y: i32) -> Edge {
        for lot in &self.lots {
            let (bx, by) = (x - lot.origin.x, y - lot.origin.y);
            let g = &lot.grid;
            if (0..=g.width).contains(&bx) && (0..g.height).contains(&by) {
                let edge = g.west(bx, by);
                if edge != Edge::Open || lot.covers(x, y) || lot.covers(x - 1, y) {
                    return edge;
                }
            }
        }
        for lot in &self.lots {
            let (x0, y0, x1, y1) = lot.yard;
            if (y0..y1).contains(&y) && (x == x0 || x == x1) {
                return Edge::Fence;
            }
        }
        Edge::Open
    }

    /// Writes one chunk of the patch into a grid.
    pub fn fill(&self, chunk: ChunkPos, grid: &mut dusk_core::Grid) {
        let origin = chunk.origin(0);
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let at = origin.offset(lx, ly);
                grid.set_tile(at, self.tile(at.x, at.y));
                grid.set_north(at, self.north_edge(at.x, at.y));
                grid.set_west(at, self.west_edge(at.x, at.y));
            }
        }
    }

    /// Where the dead stand in a chunk at the start: in the buildings whose tiles fall in it,
    /// and in the open by the chunk's own rolls. In reading order.
    pub fn dead_in(&self, chunk: ChunkPos) -> Vec<TilePos> {
        let d = &self.region.dead;
        let in_chunk = |p: TilePos| p.chunk() == chunk;
        let clear_of_start = |p: TilePos| p.dist2(self.start) > i64::from(d.clear_start).pow(2);
        let mut out = Vec::new();
        for (i, lot) in self.lots.iter().enumerate() {
            let (fewest, most) = d.per_house;
            let roll = |what: i64| hash(self.seed, Layer::Dead, -(i as i64) - 1, what);
            let count = fewest + (roll(0) % u64::from(most - fewest + 1)) as u32;
            let mut floors: Vec<TilePos> = (0..lot.grid.height)
                .flat_map(|y| (0..lot.grid.width).map(move |x| (x, y)))
                .filter(|&(x, y)| lot.grid.cell(x, y).0.is_some_and(Tile::walkable))
                .map(|(x, y)| lot.origin.offset(x, y))
                .collect();
            for k in 0..count as usize {
                if floors.is_empty() {
                    break;
                }
                let pick = (roll(k as i64 + 1) % floors.len() as u64) as usize;
                let at = floors.swap_remove(pick);
                if in_chunk(at) && clear_of_start(at) {
                    out.push(at);
                }
            }
        }
        let (cx, cy) = (i64::from(chunk.x), i64::from(chunk.y));
        let expected = d.per_chunk + unit(self.seed, Layer::Dead, cx, cy);
        for k in 0..expected.floor() as i64 {
            // A few tries for a tile in the open.
            for attempt in 0..8 {
                let r = hash(self.seed, Layer::Dead, cx * 4096 + cy, k * 16 + attempt);
                let at = chunk
                    .origin(0)
                    .offset((r % 32) as i32, ((r >> 8) % 32) as i32);
                let tile = self.tile(at.x, at.y);
                let outdoors = tile.walkable() && tile.ground != Ground::Floor;
                if outdoors
                    && clear_of_start(at)
                    && !self.lots.iter().any(|l| l.in_yard(at.x, at.y))
                {
                    out.push(at);
                    break;
                }
            }
        }
        out.sort_by_key(|p| (p.y, p.x));
        out.dedup();
        out
    }

    /// Which of the region's kinds of dead stands at a tile, by their shares: a roll of its own,
    /// so where the dead stand doesn't hang on what they are.
    pub fn dead_kind(&self, at: TilePos) -> &str {
        let kinds = &self.region.dead.kinds;
        let mut left = unit(self.seed, Layer::DeadKind, i64::from(at.x), i64::from(at.y));
        for k in kinds {
            if left < k.share {
                return &k.creature;
            }
            left -= k.share;
        }
        kinds.last().map_or("", |k| k.creature.as_str())
    }

    /// The patch's chunks, row by row.
    pub fn chunks(&self) -> impl Iterator<Item = ChunkPos> {
        let (w, h) = self.region.patch.chunks;
        (0..h).flat_map(move |y| (0..w).map(move |x| ChunkPos { x, y }))
    }
}

/// A patch of a region, ready to play: the land, the road, the hamlet, you at the start, and
/// the dead.
pub fn patch(seed: u64, data: &Data, region: &Region) -> (World, Layout) {
    let layout = Layout::new(seed, data, region);
    let mut world = World::with_defs(seed, data.rules, data.defs.clone());
    for chunk in layout.chunks() {
        layout.fill(chunk, &mut world.grid);
    }
    world.spawn(ActorKind::Character, layout.start);
    // The region names its dead; the loader checked the names.
    for chunk in layout.chunks() {
        for at in layout.dead_in(chunk) {
            if let Some(kind) = data.defs.creature_kind(layout.dead_kind(at)) {
                world.spawn(kind, at);
            }
        }
    }
    // The land is made: from here on, what changes is what a save keeps.
    world.grid.track_changes();
    (world, layout)
}

/// A patch as a save (`dusk_core::save`): its seed and its region, and what play has changed.
/// `memory` is the caller's own bytes to keep with it (the map the character remembers).
pub fn save(world: &World, region: &Region, memory: &[u8]) -> Vec<u8> {
    world.save(&region.id, memory)
}

/// A saved patch, loaded: the world, its layout, and the caller's bytes.
pub struct Loaded {
    pub world: World,
    pub layout: Layout,
    pub memory: Vec<u8>,
}

/// Loads a save: makes its patch afresh from the seed (the header names the region), then puts
/// the saved state onto it.
pub fn load(data: &Data, bytes: &[u8]) -> Result<Loaded, SaveError> {
    let header = dusk_core::save::peek(bytes)?;
    // A region the data no longer has: the save was made with other data.
    let region = data.region(&header.label).ok_or(SaveError::DataChanged)?;
    let (mut world, layout) = patch(header.seed, data, region);
    let memory = world.restore(bytes)?;
    Ok(Loaded {
        world,
        layout,
        memory,
    })
}
