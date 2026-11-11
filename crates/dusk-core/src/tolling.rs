//! The Tolling (`CONCEPT.md`, the Hushlands' event, and sieges on a calendar): on set nights a
//! bell no one rings tolls, and every dead thing in reach of it walks to it. Zomboid's
//! helicopter, but you know when it's coming, so you can build for it.
//!
//! - **The calendar** is the region's (`Tolling`): the first night, how many days apart, the hour
//!   the bell begins. It rings for a while, a toll every few minutes, and its pull lasts until
//!   dawn. Nothing about it is saved: it's worked out from the clock.
//! - **The pull:** a creature within `reach` of the bell, after nothing, walks to it, by one
//!   flow field from the bell shared by the whole crowd (`flow.rs`): no A* each. A shut door on
//!   the way gets beaten down. The bell's tolls are sounds, which wake sleepers and are drawn,
//!   but the dead don't chase them as noise: the field is how they go.
//! - **They don't give up:** on a Tolling night, what the dead in reach of the bell go after,
//!   they keep after until dawn. Seen going into a house, you're besieged in it: they beat on
//!   the door until it gives, or the sun comes up. (On other nights they give up after their
//!   `forget_after`, so boards and silence keep a lone shambler out.)

use crate::actor::{ActorId, Intent};
use crate::flow::{Flow, FlowStep};
use crate::geom::TilePos;
use crate::world::{Event, Sound, World};

/// The Tolling's calendar and its bell, in the core's units. `every` 0: it never comes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tolling {
    /// The first night: the game day it's on (from 1).
    pub first: u32,
    /// Days from one to the next.
    pub every: u32,
    /// The game minute of the day the bell begins.
    pub start: u32,
    /// Game minutes it rings, and a toll every so many.
    pub rings: u32,
    pub toll_every: u32,
    /// How far a toll carries, as a sound, in tiles.
    pub loudness: i32,
    /// How far from the bell the dead are drawn to it, in tiles.
    pub reach: i32,
    /// What a shut door on the way costs the field, in steps' worth (thousandths of a tile).
    pub door: u32,
}

/// Game minutes in a day.
const DAY: u64 = 1440;
/// While the Tolling lasts, the field is planned again when the map has changed (a door opened,
/// shut or broken, a board nailed), but no oftener than this, in game minutes.
const REPLAN: u64 = 5;

impl World {
    /// The game minute (on the clock) the Tolling on `day` begins, if it's a Tolling day.
    fn tolling_on(&self, day: u64) -> Option<u64> {
        let t = self.region.tolling;
        let first = u64::from(t.first);
        if t.every == 0 || self.region.bell.is_none() || day < first {
            return None;
        }
        (day - first)
            .is_multiple_of(u64::from(t.every))
            .then(|| (day - 1) * DAY + u64::from(t.start))
    }

    /// When the next Tolling begins, or the one under way began: a game minute on the clock.
    pub fn next_tolling(&self) -> Option<u64> {
        let now = self.clock_minutes();
        let today = now / DAY + 1;
        // Last night's may still be on; then today's, and the days after.
        (today.saturating_sub(1).max(1)..today + u64::from(self.region.tolling.every) + 2)
            .filter_map(|d| self.tolling_on(d))
            .find(|&begins| now < self.tolling_ends(begins))
    }

    /// When a Tolling that begins at `begins` ends: dawn the next morning.
    fn tolling_ends(&self, begins: u64) -> u64 {
        let day = begins / DAY;
        let dawn = u64::from(self.rules.day.dawn);
        if begins % DAY < dawn {
            day * DAY + dawn
        } else {
            (day + 1) * DAY + dawn
        }
    }

    /// Whether the Tolling is on now: from the bell's first toll until dawn.
    pub fn is_tolling(&self) -> bool {
        self.next_tolling()
            .is_some_and(|begins| self.clock_minutes() >= begins)
    }

    /// Whether the bell draws one at `pos` now: the Tolling is on, and it's within reach. (The
    /// field stands for "it's on": it's planned as the bell begins and let go at dawn, so every
    /// creature's every thought doesn't work out the calendar again.)
    pub fn drawn_to_bell(&self, pos: TilePos) -> bool {
        let (Some(bell), Some(_)) = (self.region.bell, &self.flow) else {
            return false;
        };
        let r = i64::from(self.region.tolling.reach);
        pos.z == bell.z && pos.dist2(bell) <= r * r
    }

    /// Each game minute: the bell tolls while it rings; the field is planned as it begins and
    /// now and then while it lasts, and let go at dawn.
    pub(crate) fn toll(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        let Some(bell) = self.region.bell else {
            return;
        };
        let t = self.region.tolling;
        let now = self.clock_minutes();
        let Some(begins) = self.next_tolling() else {
            return;
        };
        if now < begins {
            // The field is what says it was on: gone with the dawn.
            if self.flow.take().is_some() {
                self.events.push(Event::TollingEnded);
            }
            // Word of it at dawn on the day.
            if now == begins / DAY * DAY + u64::from(self.rules.day.dawn) {
                self.events.push(Event::TollingTonight);
            }
            return;
        }
        let gone = now - begins;
        if gone == 0 {
            self.events.push(Event::TollingBegan { at: bell });
        }
        let stale = self
            .flow
            .as_ref()
            .is_none_or(|f| f.edits != self.grid.edits() && gone.is_multiple_of(REPLAN));
        if stale {
            self.flow = Some(Flow::build(&self.grid, bell, t.reach, t.door));
        }
        if gone < u64::from(t.rings) && gone.is_multiple_of(u64::from(t.toll_every.max(1))) {
            self.emit(Sound {
                at: bell,
                loudness: t.loudness,
                by: None,
            });
            self.events.push(Event::Tolled { at: bell });
        }
    }

    /// Whether a sound is the bell's: the dead don't chase it as a noise.
    pub(crate) fn is_bell(&self, s: &Sound) -> bool {
        s.by.is_none() && Some(s.at) == self.region.bell
    }

    /// Where the bell's field sends a creature at `pos` that's after nothing: a step, a door to
    /// beat on, or nowhere (at the bell, or no way from here). None if the bell doesn't draw it.
    pub(crate) fn pulled(&self, id: ActorId, pos: TilePos) -> Option<Intent> {
        if !self.drawn_to_bell(pos) {
            return None;
        }
        let flow = self.flow.as_ref()?;
        let free = |p: TilePos| self.occupied.get(&p).is_none_or(|&o| o == id);
        Some(match flow.step(&self.grid, pos, free) {
            FlowStep::Go(dir) => Intent::Move { dir, run: false },
            FlowStep::Door(dir) => Intent::Attack { dir: Some(dir) },
            FlowStep::Stay => Intent::Idle,
        })
    }

    /// The bell's flow field while the Tolling's on: for the tools and the client.
    pub fn bell_field(&self) -> Option<&Flow> {
        self.flow.as_ref()
    }
}
