//! Positions and directions on the tile grid.

/// Tiles along each side of a chunk.
pub const CHUNK_SIZE: i32 = 32;
/// The lowest floor (cellars are -2 and -1) and the highest (attics up to 3).
pub const Z_MIN: i8 = -2;
pub const Z_MAX: i8 = 3;
/// Floors in a chunk.
pub const Z_LEVELS: usize = (Z_MAX - Z_MIN + 1) as usize;

/// A tile: x east, y south, z the floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
    pub z: i8,
}

impl TilePos {
    pub const fn new(x: i32, y: i32, z: i8) -> TilePos {
        TilePos { x, y, z }
    }

    /// The chunk this tile is in.
    pub fn chunk(self) -> ChunkPos {
        ChunkPos {
            x: self.x.div_euclid(CHUNK_SIZE),
            y: self.y.div_euclid(CHUNK_SIZE),
        }
    }

    /// The tile's place inside its chunk.
    pub fn local(self) -> (usize, usize) {
        (
            self.x.rem_euclid(CHUNK_SIZE) as usize,
            self.y.rem_euclid(CHUNK_SIZE) as usize,
        )
    }

    /// The neighbouring tile in a direction, on the same floor.
    pub fn step(self, dir: Dir8) -> TilePos {
        let (dx, dy) = dir.delta();
        self.offset(dx, dy)
    }

    pub fn offset(self, dx: i32, dy: i32) -> TilePos {
        TilePos {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z,
        }
    }

    /// The squared straight-line distance in tiles (floors aside).
    pub fn dist2(self, other: TilePos) -> i64 {
        let dx = i64::from(other.x - self.x);
        let dy = i64::from(other.y - self.y);
        dx * dx + dy * dy
    }

    /// Steps apart, a diagonal counting as one (the Chebyshev distance).
    pub fn steps_to(self, other: TilePos) -> i32 {
        (other.x - self.x).abs().max((other.y - self.y).abs())
    }
}

/// A chunk's place in the world, counted in chunks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
}

impl ChunkPos {
    /// The chunk's north-west tile on a floor.
    pub fn origin(self, z: i8) -> TilePos {
        TilePos::new(self.x * CHUNK_SIZE, self.y * CHUNK_SIZE, z)
    }
}

/// The eight headings, clockwise from north (north is -y, up the screen).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Dir8 {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Dir8 {
    pub const ALL: [Dir8; 8] = [
        Dir8::N,
        Dir8::NE,
        Dir8::E,
        Dir8::SE,
        Dir8::S,
        Dir8::SW,
        Dir8::W,
        Dir8::NW,
    ];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir8::N => (0, -1),
            Dir8::NE => (1, -1),
            Dir8::E => (1, 0),
            Dir8::SE => (1, 1),
            Dir8::S => (0, 1),
            Dir8::SW => (-1, 1),
            Dir8::W => (-1, 0),
            Dir8::NW => (-1, -1),
        }
    }

    pub fn is_diagonal(self) -> bool {
        matches!(self, Dir8::NE | Dir8::SE | Dir8::SW | Dir8::NW)
    }

    /// The heading that points the way of (dx, dy), by their signs; None for (0, 0).
    pub fn toward(dx: i32, dy: i32) -> Option<Dir8> {
        Some(match (dx.signum(), dy.signum()) {
            (0, -1) => Dir8::N,
            (1, -1) => Dir8::NE,
            (1, 0) => Dir8::E,
            (1, 1) => Dir8::SE,
            (0, 1) => Dir8::S,
            (-1, 1) => Dir8::SW,
            (-1, 0) => Dir8::W,
            (-1, -1) => Dir8::NW,
            _ => return None,
        })
    }

    /// The heading nearest the angle of (dx, dy); None for (0, 0). Unlike [`Dir8::toward`], a
    /// goal 5 east and 1 south is east, not south-east. (A diagonal covers angles whose smaller
    /// side is at least 0.4 of the larger, close to tan 22.5° = 0.414.)
    pub fn nearest(dx: i32, dy: i32) -> Option<Dir8> {
        let (ax, ay) = (dx.abs(), dy.abs());
        if 5 * ay < 2 * ax {
            Dir8::toward(dx, 0)
        } else if 5 * ax < 2 * ay {
            Dir8::toward(0, dy)
        } else {
            Dir8::toward(dx, dy)
        }
    }

    /// Turned by eighths of a turn: clockwise for positive.
    pub fn turned(self, eighths: i32) -> Dir8 {
        Dir8::ALL[(self as i32 + eighths).rem_euclid(8) as usize]
    }

    /// The two orthogonal headings a diagonal is made of (east or west first); None for an
    /// orthogonal heading.
    pub fn split(self) -> Option<(Dir8, Dir8)> {
        match self {
            Dir8::NE => Some((Dir8::E, Dir8::N)),
            Dir8::SE => Some((Dir8::E, Dir8::S)),
            Dir8::SW => Some((Dir8::W, Dir8::S)),
            Dir8::NW => Some((Dir8::W, Dir8::N)),
            _ => None,
        }
    }
}
