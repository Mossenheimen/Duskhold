//! Bringing it home (rules.toml's [deeds]): levels' XP as thousandths, health as points, seconds
//! as ticks.

use std::path::Path;

use dusk_core::home::{Deeds, MAX_LEVEL};
use serde::Deserialize;

use crate::{DataError, fail, points, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeedsFile {
    /// The XP to reach each level from 2 to 10, rising.
    levels: Vec<f64>,
    /// Health points a level brings.
    health: f64,
    /// Seconds to mark a container as your stash.
    stash: f64,
    /// Tiles from where you sleep that your stash may be.
    home: i32,
}

pub(crate) fn to_deeds(file: &Path, d: &DeedsFile) -> Result<Deeds, DataError> {
    let rising = d.levels.windows(2).all(|w| w[0] < w[1]);
    if d.levels.len() != usize::from(MAX_LEVEL) - 1 || !rising || d.levels[0] <= 0.0 {
        return Err(fail(
            file,
            format!("deeds.levels: the XP for levels 2 to {MAX_LEVEL}, above 0 and rising"),
        ));
    }
    if !(0.0..=100.0).contains(&d.health) || d.home < 1 {
        return Err(fail(
            file,
            "deeds: health is 0 to 100 points a level, and home 1 tile or more",
        ));
    }
    let mut levels = [0; MAX_LEVEL as usize - 1];
    for (to, &xp) in levels.iter_mut().zip(&d.levels) {
        *to = (xp * 1000.0).round() as u32;
    }
    Ok(Deeds {
        levels,
        health: points(d.health),
        stash_time: (ticks(file, "deeds.stash", d.stash)? as u32).max(1),
        home_range: d.home,
    })
}
