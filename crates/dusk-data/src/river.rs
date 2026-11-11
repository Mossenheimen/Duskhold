//! Water you cross: a region's river (`[river]`, for the generator) and rules.toml's
//! `[shallows]` (wading: how fast, how loud).

use std::path::Path;

use dusk_core::world::Shallows;
use serde::Deserialize;

use crate::{DataError, fail};

/// A region's river, as the generator reads it: north to south across a patch, beside the
/// hamlet, crossed by the road on a bridge and by fords.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiverFile {
    /// Tiles of deep water across, with a tile of shallows at each bank.
    pub width: i32,
    /// The most it strays east or west of its line, in tiles; and tiles between its bends.
    pub sway: i32,
    pub bend: f32,
    /// Fords: rows where it's shallow all the way across.
    pub fords: u32,
    /// Tiles it keeps clear of the hamlet's lots.
    pub margin: i32,
}

impl RiverFile {
    pub(crate) fn problem(&self) -> Option<String> {
        if !(1..=15).contains(&self.width) || !(0..=30).contains(&self.sway) {
            return Some("river: width is 1 to 15 tiles, sway 0 to 30".into());
        }
        if self.bend < 4.0 || self.fords > 6 || self.margin < 0 {
            return Some("river: bend at least 4 tiles, 0 to 6 fords, margin 0 or more".into());
        }
        None
    }
}

/// rules.toml's `[shallows]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShallowsFile {
    /// How fast you wade, and how far a splash carries, as multipliers of a step's.
    pace: f64,
    noise: f64,
}

pub(crate) fn to_shallows(file: &Path, s: &ShallowsFile) -> Result<Shallows, DataError> {
    if !(0.1..=1.0).contains(&s.pace) || !(0.0..=5.0).contains(&s.noise) {
        return Err(fail(
            file,
            "shallows: pace is 0.1 to 1, noise a multiplier from 0 to 5",
        ));
    }
    Ok(Shallows {
        pace: (s.pace * 1000.0).round() as u32,
        noise: (s.noise * 1000.0).round() as u32,
    })
}
