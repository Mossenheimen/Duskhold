//! Flow fields (`ARCHITECTURE.md`, creature AI): one map of the way to a place, shared by a whole
//! crowd going there. Each tile holds what it costs to get from it to the goal, so a creature
//! only looks at the eight tiles round it and steps downhill. Planning it once for the crowd
//! costs what one long A* search costs; a crowd of five hundred then costs nothing more.
//!
//! A shut door is a way through, at a cost: the dead beat doors down. A wall, a window or a
//! solid tile isn't.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::geom::{Dir8, TilePos};
use crate::grid::Grid;

const STRAIGHT: u32 = 1000;
const DIAGONAL: u32 = 1414;
/// No way to the goal from here.
pub const NOWHERE: u32 = u32::MAX;

/// The way to one goal, over a rectangle of one floor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flow {
    pub goal: TilePos,
    /// The grid's `edits` when it was planned: different now, and it may be stale.
    pub edits: u64,
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
    cost: Vec<u32>,
}

/// How a creature goes on from a tile, by the field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowStep {
    /// Step this way.
    Go(Dir8),
    /// Beat on the shut door on this side: the way goes through it.
    Door(Dir8),
    /// Nowhere better: at the goal, or no way from here.
    Stay,
}

impl Flow {
    /// Plans the field from `goal` over the tiles within `reach` of it (a square), a shut door
    /// costing `door` more than a step.
    pub fn build(grid: &Grid, goal: TilePos, reach: i32, door: u32) -> Flow {
        let (x0, y0) = (goal.x - reach, goal.y - reach);
        let (w, h) = (2 * reach + 1, 2 * reach + 1);
        let mut flow = Flow {
            goal,
            edits: grid.edits(),
            x0,
            y0,
            w,
            h,
            cost: vec![NOWHERE; (w * h) as usize],
        };
        let mut open = BinaryHeap::new();
        flow.set(goal, 0);
        open.push(Reverse((0u32, goal.y, goal.x)));
        while let Some(Reverse((cost, y, x))) = open.pop() {
            let p = TilePos::new(x, y, goal.z);
            if cost > flow.cost_at(p) {
                continue;
            }
            // Coming the other way: from each tile round `p` that could step onto it.
            for dir in Dir8::ALL {
                let from = p.step(dir);
                let Some(i) = flow.index(from) else { continue };
                // Nobody stands on furniture or rock, so it needs no way.
                if !grid.tile(from).walkable() {
                    continue;
                }
                let back = dir.turned(4);
                let step = if grid.steps_from(from) & back.bit() != 0 {
                    if back.is_diagonal() {
                        DIAGONAL
                    } else {
                        STRAIGHT
                    }
                } else if back.split().is_none()
                    && grid.tile(p).walkable()
                    && grid.edge(from, back).is_shut_door()
                {
                    STRAIGHT + door
                } else {
                    continue;
                };
                let next = cost.saturating_add(step);
                if next < flow.cost[i] {
                    flow.cost[i] = next;
                    open.push(Reverse((next, from.y, from.x)));
                }
            }
        }
        flow
    }

    fn index(&self, p: TilePos) -> Option<usize> {
        let (x, y) = (p.x - self.x0, p.y - self.y0);
        (p.z == self.goal.z && (0..self.w).contains(&x) && (0..self.h).contains(&y))
            .then(|| (y * self.w + x) as usize)
    }

    fn set(&mut self, p: TilePos, cost: u32) {
        if let Some(i) = self.index(p) {
            self.cost[i] = cost;
        }
    }

    /// What it costs to get from a tile to the goal: `NOWHERE` with no way, or outside the field.
    pub fn cost_at(&self, p: TilePos) -> u32 {
        self.index(p).map_or(NOWHERE, |i| self.cost[i])
    }

    /// Downhill from `p`: the cheapest way on, with `free` saying which tiles nobody stands on.
    /// A step to a tile someone's on is passed over for the next best that's still downhill;
    /// if there's none, it waits.
    pub fn step(&self, grid: &Grid, p: TilePos, free: impl Fn(TilePos) -> bool) -> FlowStep {
        let here = self.cost_at(p);
        if here == 0 || here == NOWHERE {
            return FlowStep::Stay;
        }
        let mut ways: Vec<(u32, FlowStep)> = Vec::with_capacity(8);
        let open = grid.steps_from(p);
        for dir in Dir8::ALL {
            let next = p.step(dir);
            let cost = self.cost_at(next);
            if cost >= here {
                continue;
            }
            if open & dir.bit() != 0 {
                ways.push((cost, FlowStep::Go(dir)));
            } else if dir.split().is_none() && grid.edge(p, dir).is_shut_door() {
                ways.push((cost, FlowStep::Door(dir)));
            }
        }
        // Cheapest first; ties in the fixed order of the directions.
        ways.sort_by_key(|&(cost, _)| cost);
        for &(_, way) in &ways {
            match way {
                FlowStep::Go(dir) if !free(p.step(dir)) => continue,
                _ => return way,
            }
        }
        FlowStep::Stay
    }
}
