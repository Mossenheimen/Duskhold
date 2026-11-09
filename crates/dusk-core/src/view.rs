//! What a character makes out of the map (`CONCEPT.md`, Senses: "the character sees in a cone,
//! and only what's lit; outside it you see what you remember, never a live monster").
//!
//! It sees the tiles in its cone as far as their light allows, and those right beside it all
//! round in any light, in a straight line past no wall. What it once saw it remembers: the fog
//! of war. A view is perception, not simulation: nothing in the world depends on it. So it's
//! worked out when asked (each frame for the client; every few ticks for the bot, later) and kept
//! by whoever asks.

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::light::bright_enough;
use crate::senses::line_of_sight;
use crate::world::World;

/// A tile never seen.
pub const UNSEEN: u8 = 0;
/// A tile seen before, but not now.
pub const REMEMBERED: u8 = 1;
/// A tile seen now.
pub const SEEN: u8 = 2;

/// What a character has made out of the ground floor of a `width × height` map from (0, 0).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub width: i32,
    pub height: i32,
    /// Per tile, row by row: `UNSEEN`, `REMEMBERED` or `SEEN`.
    pub tiles: Vec<u8>,
    /// The tiles seen at the last look, to turn into memories at the next.
    now: Vec<usize>,
}

impl View {
    pub fn new(width: i32, height: i32) -> View {
        View {
            width,
            height,
            tiles: vec![UNSEEN; (width.max(0) * height.max(0)) as usize],
            now: Vec::new(),
        }
    }

    fn index(&self, at: TilePos) -> Option<usize> {
        (at.z == 0 && at.x >= 0 && at.y >= 0 && at.x < self.width && at.y < self.height)
            .then(|| (at.y * self.width + at.x) as usize)
    }

    /// What the view holds for a tile (`UNSEEN` off the map).
    pub fn at(&self, at: TilePos) -> u8 {
        self.index(at).map_or(UNSEEN, |i| self.tiles[i])
    }

    /// Whether the tile is seen now.
    pub fn sees(&self, at: TilePos) -> bool {
        self.at(at) == SEEN
    }
}

impl World {
    /// Whether character `id` makes out a tile now: its own; one beside it all round, in any
    /// light; or one in its cone that's lit enough for how far off it is. Always in a straight
    /// line past no wall.
    pub fn makes_out(&self, id: ActorId, at: TilePos) -> bool {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return false;
        };
        if a.pos.z != at.z {
            return false;
        }
        if a.pos == at {
            return true;
        }
        let s = self.rules.sight;
        let d2 = a.pos.dist2(at);
        if d2 > i64::from(s.range).pow(2) {
            return false;
        }
        let beside = a.pos.steps_to(at) <= s.beside;
        let ahead = || {
            s.cone.contains(a.facing, at.x - a.pos.x, at.y - a.pos.y)
                && bright_enough(d2, self.light_at(at), s.range)
        };
        (beside || ahead()) && line_of_sight(&self.grid, a.pos, at)
    }

    /// Looks: what `id` saw before becomes a memory, and what it makes out now is seen. A dead
    /// character sees nothing more.
    pub fn look(&self, id: ActorId, view: &mut View) {
        for &i in &view.now {
            view.tiles[i] = REMEMBERED;
        }
        view.now.clear();
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return;
        };
        let (pos, r) = (a.pos, self.rules.sight.range);
        for y in pos.y - r..=pos.y + r {
            for x in pos.x - r..=pos.x + r {
                let at = TilePos::new(x, y, pos.z);
                let Some(i) = view.index(at) else { continue };
                if self.makes_out(id, at) {
                    view.tiles[i] = SEEN;
                    view.now.push(i);
                }
            }
        }
    }
}
