//! Climbing (`climb.rs` in the core): rules.toml's [climb] (optional: with none, the core's
//! defaults). A kind of creature's own `climb` and `tumble` are read with the rest of it.

use std::path::Path;

use dusk_core::climb::ClimbRules;
use serde::Deserialize;

use crate::{DataError, fail, ticks};

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
}

pub(crate) fn to_climb(file: &Path, c: &ClimbFile) -> Result<ClimbRules, DataError> {
    let window = ticks(file, "climb.window", c.window)? as u32;
    let fence = ticks(file, "climb.fence", c.fence)? as u32;
    if window == 0
        || fence == 0
        || !(c.vault > 0.0 && c.vault <= 1.0)
        || !(0..=20).contains(&c.noise)
        || !(1.0..=5.0).contains(&c.heavy)
    {
        return Err(fail(
            file,
            "climb: window and fence are at least a tick, vault a share above 0 and at most 1, \
             noise 0 to 20 tiles, heavy 1 to 5 times",
        ));
    }
    Ok(ClimbRules {
        window,
        fence,
        vault: (c.vault * 1000.0).round() as u32,
        noise: c.noise,
        heavy: (c.heavy * 1000.0).round() as u32,
    })
}
