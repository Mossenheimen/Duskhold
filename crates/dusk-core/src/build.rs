//! Building (`CONCEPT.md`, base-building: "or build one (carpentry...): walls, doors"): a door of
//! your own, hung in a gap in a wall, and a wall of planks, put up anywhere.
//!
//! - **Made first, then put up:** each is an item (`ItemKind::Piece`), made from planks and nails
//!   with a hammer (`data/recipes.toml`), heavy to carry; used, it's put up on the side of your
//!   tile you face, with a hammer to hand, in `Building::time`, as loud as boarding up.
//! - **A wall of planks** (`Edge::PlankWall`) goes on any open edge. It stops walking and sight as
//!   a wall does, but the dead beat on it as on a shut door (`World::thump`), and past
//!   `Building::wall` it gives way, as a door does. Beaten, it's mended as a door is
//!   (`barricade.rs`).
//! - **A door** goes only in a gap: an open edge framed at both its ends by a wall, a wall of
//!   planks, a door or a window (`Edge::frames`). A door the dead beat down leaves just such a
//!   gap, and so does one left in a line of your own walls. Hung, it's a door like any: shut,
//!   opened, boarded up, warded, beaten and mended.
//! - Not while someone's stepping through.

use crate::actor::ActorId;
use crate::geom::{Dir8, TilePos};
use crate::grid::{Edge, edge_owner};
use crate::skills::{On, Skill};
use crate::world::{Event, Sound, World};

/// What can be put up (`ItemKind::Piece`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Piece {
    Door,
    Wall,
}

/// Building's numbers, in the core's units (rules.toml's `[building]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Building {
    /// Ticks to put a door or a wall up.
    pub time: u32,
    /// What a wall of planks takes before it gives way (out of `body::FULL`, as a door's
    /// strength).
    pub wall: i32,
}

impl Default for Building {
    fn default() -> Building {
        Building {
            time: 100,
            wall: 800,
        }
    }
}

/// Why it can't be put up here and now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoPutUp {
    /// No open edge on the side you face.
    Nothing,
    /// A door goes only in a gap framed at both ends; there's none you face.
    NoGap,
    /// Someone's stepping through.
    Crossing,
    NoHammer,
}

impl World {
    /// Whether the open edge on side `d` of `pos` is framed at both its ends, as a doorway is: at
    /// each end, the line goes on in something that frames it, or something meets it there.
    fn framed(&self, pos: TilePos, d: Dir8) -> bool {
        let framed_toward = |end: Dir8| {
            [
                self.grid.edge(pos, end),
                self.grid.edge(pos.step(end), d),
                self.grid.edge(pos.step(d), end),
            ]
            .into_iter()
            .any(Edge::frames)
        };
        framed_toward(d.turned(2)) && framed_toward(d.turned(-2))
    }

    /// Whether `id` can put `piece` up now, and on which side: the side it faces (facing a
    /// diagonal, either of its two), open, and for a door framed as a doorway.
    pub fn can_put_up(&self, id: ActorId, piece: Piece) -> Result<Dir8, NoPutUp> {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return Err(NoPutUp::Nothing);
        };
        let faced = match a.facing.split() {
            Some((x, y)) => vec![x, y],
            None => vec![a.facing],
        };
        let open: Vec<Dir8> = faced
            .into_iter()
            .filter(|&d| self.grid.edge(a.pos, d) == Edge::Open)
            .collect();
        if open.is_empty() {
            return Err(NoPutUp::Nothing);
        }
        let side = match piece {
            Piece::Wall => open[0],
            Piece::Door => *open
                .iter()
                .find(|&&d| self.framed(a.pos, d))
                .ok_or(NoPutUp::NoGap)?,
        };
        let (from, to) = (a.pos, a.pos.step(side));
        let crossing = self.actors.iter().any(|(_, b)| {
            b.step
                .is_some_and(|s| (s.from == from && s.to == to) || (s.from == to && s.to == from))
        });
        if crossing {
            return Err(NoPutUp::Crossing);
        }
        if !self.has_hammer(id) {
            return Err(NoPutUp::NoHammer);
        }
        Ok(side)
    }

    /// `id` puts `piece` (the item at `slot`) up: the use's strike. A door is hung shut; the edge
    /// starts whole, unwarded. Loud, as hammering is; Carpentry learns from it.
    pub(crate) fn put_up(&mut self, id: ActorId, slot: u16, def: u16, piece: Piece) {
        let Ok(side) = self.can_put_up(id, piece) else {
            return;
        };
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let n = usize::from(slot);
        if a.items.get(n).is_none_or(|i| i.def != def) {
            return;
        }
        a.items[n].count -= 1;
        if a.items[n].count == 0 {
            a.items.remove(n);
        }
        let pos = a.pos;
        let edge = match piece {
            Piece::Door => Edge::door(false),
            Piece::Wall => Edge::PlankWall,
        };
        self.grid.set_edge(pos, side, edge);
        let (at, side) = edge_owner(pos, side);
        self.door_damage.remove(&(at, side));
        self.wards.remove(&(at, side));
        self.emit(Sound {
            at: pos,
            loudness: self.rules.barricade.noise,
            by: Some(id),
        });
        self.events.push(Event::PutUp {
            who: id,
            at,
            side,
            door: piece == Piece::Door,
        });
        self.train(
            id,
            Skill::Carpentry,
            self.rules.skills.board,
            On::Edge(at, side),
        );
    }
}
