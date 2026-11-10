//! Lines of sight, and how far sounds carry, on a grid with walls on its edges.

use crate::geom::{Dir8, TilePos};
use crate::grid::{Edge, Grid};

/// Loudness lost passing an edge, in tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Muffling {
    pub wall: i32,
    pub shut_door: i32,
    pub shut_window: i32,
}

impl Muffling {
    pub fn of(self, edge: Edge) -> i32 {
        match edge {
            Edge::Open
            | Edge::Door { open: true, .. }
            | Edge::Window { open: true, .. }
            | Edge::Fence => 0,
            Edge::Wall => self.wall,
            Edge::Door { open: false, .. } => self.shut_door,
            Edge::Window { open: false, .. } => self.shut_window,
        }
    }
}

/// How far either side of its facing a creature sees: the cosine of that angle, in ten-thousandths.
/// 5000 is 60° either side (a 120° cone), 0 is 90° (all that's in front), -10000 all the way round.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cone {
    pub cos: i32,
}

impl Cone {
    /// Is the offset (dx, dy) inside the cone round `facing`? The creature's own tile isn't. Whole
    /// numbers only, so every machine agrees.
    pub fn contains(self, facing: Dir8, dx: i32, dy: i32) -> bool {
        if (dx, dy) == (0, 0) {
            return false;
        }
        let (fx, fy) = facing.delta();
        let dot = i128::from(fx * dx + fy * dy);
        let (dx, dy) = (i128::from(dx), i128::from(dy));
        // dot / (|f| |d|) >= cos / 10,000, squared: the comparison flips where the signs differ.
        let lhs = (dot * 10_000).pow(2);
        let rhs = i128::from(self.cos).pow(2) * i128::from(fx * fx + fy * fy) * (dx * dx + dy * dy);
        match (dot >= 0, self.cos >= 0) {
            (true, true) => lhs >= rhs,
            (true, false) => true,
            (false, true) => false,
            (false, false) => lhs <= rhs,
        }
    }
}

/// Walks the straight line from `a` to `b` (Bresenham), one step between neighbours at a time.
/// `visit(from, dir)` returns false to stop; the walk returns whether it got to `b`.
fn walk_line(a: TilePos, b: TilePos, mut visit: impl FnMut(TilePos, Dir8) -> bool) -> bool {
    let (dx, dy) = ((b.x - a.x).abs(), -(b.y - a.y).abs());
    let (sx, sy) = ((b.x - a.x).signum(), (b.y - a.y).signum());
    let mut err = dx + dy;
    let mut p = a;
    while (p.x, p.y) != (b.x, b.y) {
        let e2 = 2 * err;
        let (mut mx, mut my) = (0, 0);
        if e2 >= dy {
            err += dy;
            mx = sx;
        }
        if e2 <= dx {
            err += dx;
            my = sy;
        }
        let dir = Dir8::toward(mx, my).expect("a line always moves");
        if !visit(p, dir) {
            return false;
        }
        p = p.step(dir);
    }
    true
}

/// Is the route `p` → `p + first` → `p + first + second` clear for sight?
fn side_clear(grid: &Grid, p: TilePos, first: Dir8, second: Dir8) -> bool {
    let via = p.step(first);
    !grid.edge(p, first).blocks_sight()
        && !grid.tile(via).blocks_sight()
        && !grid.edge(via, second).blocks_sight()
}

/// Can `from` see `to`? Solid tiles, walls, shut doors and boarded windows block sight; glass
/// doesn't. A diagonal
/// step on the line gets past a corner if either of its sides is clear. (Light and the vision
/// cone come at build step 5.)
pub fn line_of_sight(grid: &Grid, from: TilePos, to: TilePos) -> bool {
    if from.z != to.z {
        return false;
    }
    walk_line(from, to, |p, dir| {
        let next = p.step(dir);
        let clear = match dir.split() {
            None => !grid.edge(p, dir).blocks_sight(),
            Some((a, b)) => side_clear(grid, p, a, b) || side_clear(grid, p, b, a),
        };
        clear && (next == to || !grid.tile(next).blocks_sight())
    })
}

/// The loudness a sound loses between `from` and `to` to the edges on the way (walls, shut doors
/// and windows). A diagonal step takes the quieter of its two sides. Stops counting past `cap`.
pub fn muffling(grid: &Grid, from: TilePos, to: TilePos, m: Muffling, cap: i32) -> i32 {
    let mut lost = 0;
    walk_line(from, to, |p, dir| {
        lost += match dir.split() {
            None => m.of(grid.edge(p, dir)),
            Some((a, b)) => {
                let (via_a, via_b) = (p.step(a), p.step(b));
                let side_a = m.of(grid.edge(p, a)) + m.of(grid.edge(via_a, b));
                let side_b = m.of(grid.edge(p, b)) + m.of(grid.edge(via_b, a));
                side_a.min(side_b)
            }
        };
        lost <= cap
    });
    lost
}

/// Does a sound made at `from`, as loud as `loudness` (how many tiles it carries in the open),
/// reach a listener at `to`?
pub fn hears(grid: &Grid, from: TilePos, to: TilePos, loudness: i32, m: Muffling) -> bool {
    if from.z != to.z || loudness <= 0 {
        return false;
    }
    let d2 = from.dist2(to);
    if d2 > i64::from(loudness).pow(2) {
        return false;
    }
    let lost = muffling(grid, from, to, m, loudness);
    lost < loudness && d2 <= i64::from(loudness - lost).pow(2)
}
