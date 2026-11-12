//! The region's pressure (`CONCEPT.md`, the Long Dusk): one number, and you can move it.
//!
//! - **What it is:** how many of the dead the patch holds, in thousandths of how many it began
//!   with. At 1000 it holds as many as at the start; at 1500, half as many again.
//! - **It rises** with time, to a ceiling (Zomboid's population peaks and stops), and with the
//!   region's own rule: in the Hushlands, every body that gets up (`remains.rs`). Bodies that get
//!   up can push it past the ceiling, up to the most it can be.
//! - **It falls** with deeds: each body burned. (Bounties and a holdout supplied come with the
//!   holdout; a wardstone relit, which drops the ceiling for good, with the dungeons.)
//! - **What it does:** while the patch holds fewer of the dead than the pressure says, one walks
//!   in every so often. It comes into a chunk that's asleep, out where you aren't, so the dead
//!   you put down are made up for out of sight, never in front of you. Nothing else: no creature
//!   gets tougher.

use crate::actor::ActorKind;
use crate::geom::{CHUNK_SIZE, TilePos};
use crate::grid::Ground;
use crate::hash::{Purpose, below, roll};
use crate::world::World;

/// A region's pressure numbers, in thousandths of the dead the patch began with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pressure {
    /// Where it starts.
    pub start: u32,
    /// It never falls below this, nor rises above `most`.
    pub least: u32,
    pub most: u32,
    /// Time raises it this much a game day, until it reaches the ceiling.
    pub per_day: u32,
    pub ceiling: u32,
    /// Each body that gets up raises it this much, and each body burned lowers it this much.
    pub per_rise: u32,
    pub per_burn: u32,
    /// While the patch is short of the dead, one walks in every this many game minutes.
    pub every: u32,
}

/// What the world knows of the region it's a patch of. Not saved: it comes from the data, and
/// the patch's making, each time the world is made. The default is no region: nothing walks
/// in and the pressure goes nowhere.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Region {
    pub pressure: Pressure,
    /// The kinds of its dead, with their shares out of 10,000: who walks in.
    pub kinds: Vec<(ActorKind, u32)>,
    /// The patch's size, in chunks from (0, 0): where they can walk in.
    pub chunks: (i32, i32),
    /// How many of the dead the patch began with: what a pressure of 1000 means.
    pub dead: u32,
    /// The region's event on a calendar (`tolling.rs`), and where its bell is in the patch.
    pub tolling: crate::tolling::Tolling,
    pub bell: Option<TilePos>,
    /// Where a character comes into the patch: the next one after a death too (`legacy.rs`).
    pub start: Option<TilePos>,
    /// Its weather (`weather.rs`).
    pub weather: crate::weather::Weather,
}

/// Rolls a tile can take before an arrival gives up for this time.
const TRIES: u64 = 24;

impl World {
    /// The region's pressure, in thousandths of the dead the patch began with.
    pub fn pressure(&self) -> u32 {
        self.pressure
    }

    /// Sets the region the world is a patch of, and its pressure where the region starts it.
    pub fn set_region(&mut self, region: Region) {
        self.pressure = region.pressure.start;
        self.region = region;
    }

    /// How many of the dead the pressure says the patch holds now.
    pub fn dead_wanted(&self) -> u32 {
        (u64::from(self.region.dead) * u64::from(self.pressure) / 1000) as u32
    }

    /// The dead the patch holds: those on their feet, and the bodies that will get up.
    pub fn dead_about(&self) -> u32 {
        self.actors
            .iter()
            .filter(|(_, a)| !a.kind.is_character())
            // Beasts aren't the dead (`hunting.rs`).
            .filter(|(_, a)| !matches!(a.mind, crate::actor::Mind::Flee { .. }))
            .filter(|(_, a)| !a.dead || matches!(a.remains, crate::actor::Remains::GetsUp { .. }))
            .count() as u32
    }

    /// Raises or lowers the pressure, between the least and the most it can be.
    pub(crate) fn nudge_pressure(&mut self, by: i64) {
        let p = self.region.pressure;
        let now = (i64::from(self.pressure) + by).clamp(i64::from(p.least), i64::from(p.most));
        self.pressure = now as u32;
    }

    /// Each game minute: time raises the pressure toward its ceiling, and while the patch is
    /// short of the dead, one walks in now and then.
    pub(crate) fn press(&mut self) {
        let p = self.region.pressure;
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        let minute = self.tick / per;
        if self.pressure < p.ceiling {
            // Worked out from the minute, not added up, so it comes out exact over a day.
            let day = u64::from(p.per_day);
            let rise = day * minute / 1440 - day * (minute - 1) / 1440;
            self.pressure = (self.pressure + rise as u32).min(p.ceiling);
        }
        if p.every > 0
            && minute.is_multiple_of(u64::from(p.every))
            && self.dead_about() < self.dead_wanted()
        {
            self.walk_in();
        }
    }

    /// One of the region's dead walks into the patch: onto open ground, free, in a chunk that's
    /// asleep. If none of a few tries finds such a tile, it comes another time.
    fn walk_in(&mut self) {
        let (w, h) = self.region.chunks;
        if w <= 0 || h <= 0 || self.region.kinds.is_empty() {
            return;
        }
        let (seed, tick) = (self.seed, self.tick);
        let mut left = below(roll(seed, tick, 0, Purpose::ArrivalKind), 10_000);
        let kinds = &self.region.kinds;
        let mut kind = kinds[kinds.len() - 1].0;
        for &(k, share) in kinds {
            if left < share {
                kind = k;
                break;
            }
            left -= share;
        }
        let (wide, high) = ((w * CHUNK_SIZE) as u32, (h * CHUNK_SIZE) as u32);
        for attempt in 0..TRIES {
            let r = roll(seed, tick, attempt, Purpose::Arrival);
            let at = TilePos::new(below(r, wide) as i32, below(r << 32, high) as i32, 0);
            let tile = self.grid.tile(at);
            let open = tile.ground != Ground::Floor && !self.is_hallowed(at);
            if open && !self.is_awake(at) && self.spawn(kind, at).is_some() {
                return;
            }
        }
    }
}
