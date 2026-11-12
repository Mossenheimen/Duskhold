//! Climbing (`climb.rs` in the core): rules.toml's [climb] (optional: with none, the core's
//! defaults), with the windows' glass. A kind of creature's own `climb` and `tumble` are read
//! with the rest of it.

use std::path::Path;

use dusk_core::climb::ClimbRules;
use serde::Deserialize;

use crate::{DataError, chance, fail, points, ticks};

/// rules.toml's [climb].
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClimbFile {
    /// Seconds a character takes through an open window, and over a fence.
    window: f64,
    fence: f64,
    /// Running at a fence, you vault it in this share of the time.
    vault: f64,
    /// How far the scrape of a climb carries, in tiles.
    noise: i32,
    /// Weighed down, a climb takes this many times as long.
    heavy: f64,
    /// What a window's glass takes before it breaks, in health points of beating (a door's
    /// [door] strength), and how far the crash carries, in tiles.
    glass: f64,
    smash: i32,
    /// The chance the glass left in a broken window cuts you climbing through.
    cut: f64,
}

pub(crate) fn to_climb(file: &Path, c: &ClimbFile) -> Result<ClimbRules, DataError> {
    let window = ticks(file, "climb.window", c.window)? as u32;
    let fence = ticks(file, "climb.fence", c.fence)? as u32;
    if window == 0
        || fence == 0
        || !(c.vault > 0.0 && c.vault <= 1.0)
        || !(0..=20).contains(&c.noise)
        || !(1.0..=5.0).contains(&c.heavy)
        || !(c.glass > 0.0 && c.glass <= 100.0)
        || !(0..=40).contains(&c.smash)
    {
        return Err(fail(
            file,
            "climb: window and fence are at least a tick, vault a share above 0 and at most 1, \
             noise 0 to 20 tiles, heavy 1 to 5 times, glass above 0 and at most 100 points, \
             smash 0 to 40 tiles",
        ));
    }
    Ok(ClimbRules {
        window,
        fence,
        vault: (c.vault * 1000.0).round() as u32,
        noise: c.noise,
        heavy: (c.heavy * 1000.0).round() as u32,
        glass: points(c.glass),
        smash: c.smash,
        cut: chance(file, "climb.cut", c.cut)?,
    })
}
