//! A region's Tolling (`[tolling]`): its calendar in game days and hours, its bell's reach in
//! tiles, and the building its bell hangs in, by prefab.

use dusk_core::Tolling;
use serde::Deserialize;

/// The region's event on a calendar (`dusk_core::tolling`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TollingFile {
    /// The prefab the bell hangs in: the first lot of it in the hamlet. The bell is just inside
    /// its door.
    pub bell: String,
    /// The first Tolling's game day (from 1), and days from one to the next.
    pub first: u32,
    pub every: u32,
    /// The hour it begins (22.0 is 22:00). It lasts until dawn.
    pub hour: f64,
    /// Game minutes the bell rings, and a toll every so many.
    pub rings: u32,
    pub toll_every: u32,
    /// Tiles a toll carries, as a sound.
    pub loudness: i32,
    /// Tiles from the bell the dead are drawn from.
    pub reach: i32,
    /// What a shut door on the way is worth to the dead, in tiles of walking round it.
    pub door: f64,
}

impl TollingFile {
    /// What's wrong with these numbers, if anything.
    pub(crate) fn problem(&self) -> Option<String> {
        if self.first == 0 || self.every == 0 {
            return Some("tolling: first is a day from 1, and every at least 1".to_string());
        }
        if !(0.0..24.0).contains(&self.hour) {
            return Some("tolling.hour is 0 to 24".to_string());
        }
        if self.rings == 0 || self.toll_every == 0 || self.rings > 600 {
            return Some(
                "tolling: rings is 1 to 600 game minutes, toll_every at least 1".to_string(),
            );
        }
        if !(1..=256).contains(&self.reach)
            || self.loudness < 1
            || !(0.0..=100.0).contains(&self.door)
        {
            return Some(
                "tolling: reach is 1 to 256 tiles, loudness at least 1, door 0 to 100 tiles"
                    .to_string(),
            );
        }
        None
    }

    pub fn to_tolling(&self) -> Tolling {
        Tolling {
            first: self.first,
            every: self.every,
            start: (self.hour * 60.0).round() as u32,
            rings: self.rings,
            toll_every: self.toll_every,
            loudness: self.loudness,
            reach: self.reach,
            door: (self.door * 1000.0).round() as u32,
        }
    }
}
