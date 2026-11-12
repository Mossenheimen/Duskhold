//! A*: eight directions, no corner cutting, a diagonal costing √2, and, for one that climbs,
//! a climb through a window or over a fence costing what it takes (`climb.rs`).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::geom::{Dir8, TilePos};
use crate::grid::{Edge, Grid};
use crate::hash::FastMap;

pub(crate) const STRAIGHT: u32 = 1000;
const DIAGONAL: u32 = 1414;

/// What a climb costs a search, for one that climbs, in its units (a straight step is 1000):
/// through a window, and over a fence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClimbCost {
    pub window: u32,
    pub fence: u32,
}

/// The cheapest cost from `a` to `b` with nothing in the way.
fn octile(a: TilePos, b: TilePos) -> u32 {
    let (dx, dy) = ((a.x - b.x).unsigned_abs(), (a.y - b.y).unsigned_abs());
    let (lo, hi) = if dx < dy { (dx, dy) } else { (dy, dx) };
    hi * STRAIGHT + lo * (DIAGONAL - STRAIGHT)
}

/// The way from `from` to `to` on one floor: the tiles to step onto, the first step first. None
/// when there's no way within `budget` explored tiles. Other actors don't count as obstacles:
/// whoever follows the path waits for them, or plans again. Nothing is climbed.
pub fn find_path(grid: &Grid, from: TilePos, to: TilePos, budget: usize) -> Option<Vec<TilePos>> {
    find_way(grid, from, to, budget, None)
}

/// `find_path` for one that climbs: a way may go through an open window or over a fence, at
/// the cost `climb` says (None: as `find_path`).
pub fn find_way(
    grid: &Grid,
    from: TilePos,
    to: TilePos,
    budget: usize,
    climb: Option<ClimbCost>,
) -> Option<Vec<TilePos>> {
    if from == to {
        return Some(Vec::new());
    }
    if from.z != to.z || !grid.tile(to).walkable() {
        return None;
    }
    let mut open = BinaryHeap::new();
    let mut best: FastMap<TilePos, u32> = FastMap::default();
    let mut came_from: FastMap<TilePos, TilePos> = FastMap::default();
    best.insert(from, 0);
    open.push(Reverse((octile(from, to), 0u32, from)));
    let mut explored = 0;
    while let Some(Reverse((_, cost, p))) = open.pop() {
        if p == to {
            let mut path = vec![to];
            let mut at = to;
            while let Some(&prev) = came_from.get(&at) {
                if prev == from {
                    break;
                }
                path.push(prev);
                at = prev;
            }
            path.reverse();
            return Some(path);
        }
        if best.get(&p).is_some_and(|&b| cost > b) {
            continue; // a stale entry: a cheaper way here was found after it was queued
        }
        explored += 1;
        if explored > budget {
            return None;
        }
        // Which ways on are open: a byte from the grid's cache (from the start, which may be a
        // tile nobody else could stand on, worked out).
        let (ways, climbs) = if p == from {
            let bits = |open: &dyn Fn(Dir8) -> bool| {
                Dir8::ALL
                    .iter()
                    .filter(|&&d| open(d))
                    .fold(0u8, |bits, &d| bits | d.bit())
            };
            let climbs = if climb.is_some() {
                bits(&|d| grid.can_climb(p, d))
            } else {
                0
            };
            (bits(&|d| grid.can_step(p, d)), climbs)
        } else if climb.is_some() {
            let w = grid.ways_from(p);
            (w as u8, (w >> 8) as u8)
        } else {
            (grid.steps_from(p), 0)
        };
        for dir in Dir8::ALL {
            let bit = dir.bit();
            let length = if ways & bit != 0 {
                if dir.is_diagonal() {
                    DIAGONAL
                } else {
                    STRAIGHT
                }
            } else if let (true, Some(c)) = (climbs & bit != 0, climb) {
                match grid.edge(p, dir) {
                    Edge::Fence => c.fence,
                    _ => c.window,
                }
            } else {
                continue;
            };
            let next = p.step(dir);
            let next_cost = cost + length;
            if best.get(&next).is_none_or(|&b| next_cost < b) {
                best.insert(next, next_cost);
                came_from.insert(next, p);
                open.push(Reverse((next_cost + octile(next, to), next_cost, next)));
            }
        }
    }
    None
}
