//! Barricades (`CONCEPT.md`, base-building: the Hushlands want barricades and strong doors, for
//! the dead thump).
//!
//! - **Boarding up** a shut door or a window takes a plank and two nails a board, and a hammer in
//!   hand or in the pack, up to four boards. It's loud: everything in earshot hears the hammering
//!   start, and the last nail go in. Board up by day.
//! - **A boarded door** can't be opened; **a boarded window** can't be seen through, either way.
//! - **The dead beat on a boarded door** as on any shut door, and the boards take it first, the
//!   last one nailed first, each as strong as the hand that nailed it (Carpentry). Then the door.
//! - **Prying a board off** takes the hammer too. You get the plank back; the nails are bent.
//!
//! The edge holds how many boards there are (`grid::Edge`), so sight and walking need nothing
//! else; the world keeps how much each board has left, in its hash.

use crate::actor::ActorId;
use crate::geom::{Dir8, TilePos};
use crate::grid::{Edge, edge_owner};
use crate::items::Item;
use crate::skills::{On, Skill};
use crate::world::{Event, Sound, World};

/// The barricade's numbers, in the core's units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Barricade {
    /// The most boards on one door or window.
    pub most: u8,
    /// What a board takes before it breaks (out of `body::FULL`, as a door's strength).
    pub strength: i32,
    /// Ticks to nail a board on, and how far the hammering is heard (tiles).
    pub time: u32,
    pub noise: i32,
    /// Ticks to pry one off, and how far that's heard.
    pub remove_time: u32,
    pub remove_noise: i32,
}

impl Default for Barricade {
    fn default() -> Barricade {
        Barricade {
            most: 4,
            strength: 500,
            time: 40,
            noise: 12,
            remove_time: 30,
            remove_noise: 6,
        }
    }
}

/// What boarding up takes, from the data's items: a board, so many nails, and a hammer (any of
/// `hammers`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Boarding {
    pub plank: u16,
    pub nails: u16,
    pub nails_each: u16,
    pub hammers: Vec<u16>,
}

/// Why you can't board up (or pry a board off) here and now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoBoard {
    /// No door or window beside you (to pry: none with boards on it).
    Nothing,
    /// The door is open: shut it first.
    Open,
    /// It has all the boards it can take.
    Full,
    NoHammer,
    NoPlank,
    NoNails,
}

/// How many of an item a list holds, over all its stacks.
fn count(items: &[Item], def: u16) -> u32 {
    items
        .iter()
        .filter(|i| i.def == def)
        .map(|i| u32::from(i.count))
        .sum()
}

/// Takes `n` of an item out of a list, from its last stacks first.
pub(crate) fn spend(items: &mut Vec<Item>, def: u16, mut n: u16) {
    for i in (0..items.len()).rev() {
        if n == 0 {
            break;
        }
        if items[i].def != def {
            continue;
        }
        let used = items[i].count.min(n);
        items[i].count -= used;
        n -= used;
        if items[i].count == 0 {
            items.remove(i);
        }
    }
}

impl World {
    /// The sides of a tile to look at for a door or window: the one faced (or, facing a
    /// diagonal, both of its sides), then north, east, south and west.
    pub(crate) fn sides_from(facing: Dir8) -> impl Iterator<Item = Dir8> {
        let faced = match facing.split() {
            Some((a, b)) => [Some(a), Some(b)],
            None => [Some(facing), None],
        };
        faced
            .into_iter()
            .flatten()
            .chain([Dir8::N, Dir8::E, Dir8::S, Dir8::W])
    }

    fn has_hammer(&self, id: ActorId) -> bool {
        let (Some(a), Some(b)) = (self.actors.get(id), &self.defs.boarding) else {
            return false;
        };
        a.items
            .iter()
            .chain(&a.hands)
            .any(|i| b.hammers.contains(&i.def))
    }

    /// Whether `id` can nail a board on now, and on which side of its tile: the door or window
    /// it faces, else the first beside it.
    pub fn can_board(&self, id: ActorId) -> Result<Dir8, NoBoard> {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return Err(NoBoard::Nothing);
        };
        let side = World::sides_from(a.facing)
            .find(|&d| {
                matches!(
                    self.grid.edge(a.pos, d),
                    Edge::Door { .. } | Edge::Window { .. }
                )
            })
            .ok_or(NoBoard::Nothing)?;
        let edge = self.grid.edge(a.pos, side);
        if matches!(edge, Edge::Door { open: true, .. }) {
            return Err(NoBoard::Open);
        }
        if edge.boards() >= self.rules.barricade.most {
            return Err(NoBoard::Full);
        }
        let Some(b) = &self.defs.boarding else {
            return Err(NoBoard::NoHammer);
        };
        if !self.has_hammer(id) {
            return Err(NoBoard::NoHammer);
        }
        if count(&a.items, b.plank) == 0 {
            return Err(NoBoard::NoPlank);
        }
        if count(&a.items, b.nails) < u32::from(b.nails_each) {
            return Err(NoBoard::NoNails);
        }
        Ok(side)
    }

    /// Whether `id` can pry a board off now, and on which side: the boarded door or window it
    /// faces, else the first beside it.
    pub fn can_unboard(&self, id: ActorId) -> Result<Dir8, NoBoard> {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return Err(NoBoard::Nothing);
        };
        let side = World::sides_from(a.facing)
            .find(|&d| self.grid.edge(a.pos, d).boards() > 0)
            .ok_or(NoBoard::Nothing)?;
        if !self.has_hammer(id) {
            return Err(NoBoard::NoHammer);
        }
        Ok(side)
    }

    /// What each board on an edge has left, the last nailed last.
    pub fn boards(&self, pos: TilePos, dir: Dir8) -> Vec<i32> {
        let n = usize::from(self.grid.edge(pos, dir).boards());
        let mut list = self
            .board_strength
            .get(&edge_owner(pos, dir))
            .cloned()
            .unwrap_or_default();
        // Boards the world didn't nail (a test's, a scenario's) are as strong as new.
        list.resize(n, self.rules.barricade.strength);
        list
    }

    /// Sets the boards on an edge: the edge's count and each board's strength together.
    fn set_boards(&mut self, pos: TilePos, dir: Dir8, list: Vec<i32>) {
        let edge = self.grid.edge(pos, dir);
        self.grid
            .set_edge(pos, dir, edge.with_boards(list.len() as u8));
        let key = edge_owner(pos, dir);
        if list.is_empty() {
            self.board_strength.remove(&key);
        } else {
            self.board_strength.insert(key, list);
        }
    }

    /// The hammering starts: heard all round.
    pub(crate) fn start_hammering(&mut self, id: ActorId) {
        if let Some(pos) = self.actors.get(id).map(|a| a.pos) {
            let loudness = self.rules.barricade.noise;
            self.emit(Sound {
                at: pos,
                loudness,
                by: Some(id),
            });
        }
    }

    /// The board goes on (the boarding action's end): a plank and its nails are used up, and the
    /// board is as strong as Carpentry makes it.
    pub(crate) fn nail_board(&mut self, id: ActorId, dir: Dir8) {
        if self.can_board(id) != Ok(dir) {
            return;
        }
        let (r, s) = (self.rules.barricade, self.rules.skills);
        let level = self.skill_level(id, Skill::Carpentry);
        let Some(b) = self.defs.boarding.clone() else {
            return;
        };
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let pos = a.pos;
        spend(&mut a.items, b.plank, 1);
        spend(&mut a.items, b.nails, b.nails_each);
        let mut list = self.boards(pos, dir);
        list.push(s.board_strength(r.strength, level));
        let boards = list.len() as u8;
        self.set_boards(pos, dir, list);
        self.emit(Sound {
            at: pos,
            loudness: r.noise,
            by: Some(id),
        });
        let (at, side) = edge_owner(pos, dir);
        self.events.push(Event::Boarded {
            who: id,
            at,
            side,
            boards,
        });
        // A board on a real door or window is a useful thing made; the same one pays less.
        self.train(id, Skill::Carpentry, s.board, On::Edge(at, side));
    }

    /// A board comes off (the prying action's end): the last one nailed, and its plank back.
    pub(crate) fn pry_board(&mut self, id: ActorId, dir: Dir8) {
        if self.can_unboard(id) != Ok(dir) {
            return;
        }
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        let mut list = self.boards(pos, dir);
        list.pop();
        let boards = list.len() as u8;
        self.set_boards(pos, dir, list);
        let defs = self.defs.clone();
        if let (Some(a), Some(b)) = (self.actors.get_mut(id), &defs.boarding) {
            defs.stow(&mut a.items, Item::new(b.plank, 1));
        }
        self.emit(Sound {
            at: pos,
            loudness: self.rules.barricade.remove_noise,
            by: Some(id),
        });
        let (at, side) = edge_owner(pos, dir);
        self.events.push(Event::Unboarded {
            who: id,
            at,
            side,
            boards,
        });
    }

    /// A blow on a shut door from `pos`: the boards across it take it, if it has any (true), the
    /// last nailed first. A board beaten to nothing breaks.
    pub(crate) fn beat_boards(&mut self, pos: TilePos, dir: Dir8, harm: i32) -> bool {
        let mut list = self.boards(pos, dir);
        let Some(top) = list.last_mut() else {
            return false;
        };
        *top -= harm;
        let broke = *top <= 0;
        if broke {
            list.pop();
        }
        let boards = list.len() as u8;
        self.set_boards(pos, dir, list);
        if broke {
            let (at, side) = edge_owner(pos, dir);
            self.events.push(Event::BoardBroke { at, side, boards });
        }
        true
    }
}
