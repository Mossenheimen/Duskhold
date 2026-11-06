//! A*: eight directions, no corner cutting, a diagonal costing √2.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::geom::{Dir8, TilePos};
use crate::grid::Grid;
use crate::hash::FastMap;

const STRAIGHT: u32 = 1000;
const DIAGONAL: u32 = 1414;

/// The cheapest cost from `a` to `b` with nothing in the way.
fn octile(a: TilePos, b: TilePos) -> u32 {
    let (dx, dy) = ((a.x - b.x).unsigned_abs(), (a.y - b.y).unsigned_abs());
    let (lo, hi) = if dx < dy { (dx, dy) } else { (dy, dx) };
    hi * STRAIGHT + lo * (DIAGONAL - STRAIGHT)
}

/// The way from `from` to `to` on one floor: the tiles to step onto, the first step first. None
/// when there's no way within `budget` explored tiles. Other actors don't count as obstacles:
/// whoever follows the path waits for them, or plans again.
pub fn find_path(grid: &Grid, from: TilePos, to: TilePos, budget: usize) -> Option<Vec<TilePos>> {
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
        for dir in Dir8::ALL {
            if !grid.can_step(p, dir) {
                continue;
            }
            let next = p.step(dir);
            let next_cost = cost
                + if dir.is_diagonal() {
                    DIAGONAL
                } else {
                    STRAIGHT
                };
            if best.get(&next).is_none_or(|&b| next_cost < b) {
                best.insert(next, next_cost);
                came_from.insert(next, p);
                open.push(Reverse((next_cost + octile(next, to), next_cost, next)));
            }
        }
    }
    None
}
