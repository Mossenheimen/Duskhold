//! Decoys (`CONCEPT.md`, base-building: the Hushlands' "decoy noise-makers to draw them off"): a
//! clockwork thing you wind and set down, that rattles on a beat for a while, and the dead that
//! hear it go to it, as they go to any sound they hear (`World::hear`).
//!
//! - **An item** (`ItemKind::Decoy`, `data/items/`): how long it takes to wind, how long before it
//!   starts (time to get away), how often it rattles, for how long, and how far it's heard.
//! - **Set down** (used: `Intent::Use`, a timed action, its winding the tell): it lies on the
//!   floor where you stood, wound (`World::decoys`, saved), and rattles on its beat from the time
//!   it was set to.
//! - **Taken up** (as anything on the floor is), it stops. Run down, it lies there, to be taken up,
//!   wound and set again.
//! - **Its rattle is a sound like any other:** walls muffle it, the rain carries it less far, a
//!   Silence hushes it, it wakes a sleeper, and on hallowed ground the dead pay it no heed.

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::items::{Item, ItemKind};
use crate::world::{Event, Sound, World};

/// A decoy as made (`ItemKind::Decoy`), in ticks and tiles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decoy {
    /// Winding it and setting it down: the action's time.
    pub wind: u32,
    /// From set down to its first rattle.
    pub delay: u32,
    /// Between rattles.
    pub every: u32,
    /// How long it rattles.
    pub lasts: u32,
    /// How far each rattle is heard.
    pub noise: i32,
}

/// A decoy set down, ticking: the tick it starts and the tick it's run down, its beat and how loud.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Ticking {
    pub starts: u64,
    pub until: u64,
    pub every: u32,
    pub noise: i32,
}

impl World {
    /// Whether item `def` is a decoy.
    fn is_decoy(&self, def: u16) -> bool {
        matches!(
            self.defs.item(def).map(|d| &d.kind),
            Some(ItemKind::Decoy(_))
        )
    }

    /// The decoys wound and lying about, by where: each still to start, or rattling.
    pub fn decoys(&self) -> impl Iterator<Item = (TilePos, &Ticking)> {
        self.decoys.iter().map(|(&at, w)| (at, w))
    }

    /// `id` sets the decoy at `slot` down where it stands, wound: the use's strike.
    pub(crate) fn set_decoy(&mut self, id: ActorId, slot: u16, def: u16, decoy: Decoy) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let n = usize::from(slot);
        let Some(item) = a.items.get(n).filter(|i| i.def == def).copied() else {
            return;
        };
        a.items[n].count -= 1;
        if a.items[n].count == 0 {
            a.items.remove(n);
        }
        let at = a.pos;
        self.put(at, Item { count: 1, ..item });
        let starts = self.tick + u64::from(decoy.delay);
        self.decoys.insert(
            at,
            Ticking {
                starts,
                until: starts + u64::from(decoy.lasts),
                every: decoy.every.max(1),
                noise: decoy.noise,
            },
        );
        self.events.push(Event::DecoySet {
            who: id,
            at,
            delay: decoy.delay,
        });
    }

    /// Each tick: the decoys rattle on their beats, and stop once they've run down or been taken
    /// up (no decoy left on their tile).
    pub(crate) fn rattle_decoys(&mut self) {
        if self.decoys.is_empty() {
            return;
        }
        let tick = self.tick;
        let set: Vec<(TilePos, Ticking)> = self.decoys.iter().map(|(&at, &w)| (at, w)).collect();
        for (at, w) in set {
            let still_there = self.store(at).iter().any(|i| self.is_decoy(i.def));
            if tick >= w.until || !still_there {
                self.decoys.remove(&at);
                continue;
            }
            if tick >= w.starts && (tick - w.starts).is_multiple_of(u64::from(w.every)) {
                self.emit(Sound {
                    at,
                    loudness: w.noise,
                    by: None,
                });
            }
        }
    }
}
