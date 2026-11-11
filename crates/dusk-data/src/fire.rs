//! A fire for your base (rules.toml's [fire]): what a fire holds, in game hours as game minutes,
//! and how long tending one takes, in seconds as ticks. What burns is the items' own `fuel`, and
//! where a fire can be lit the fixtures' own `fire` (`items.rs`).

use std::path::Path;

use dusk_core::fire::Fire;
use serde::Deserialize;

use crate::{DataError, fail, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FireFile {
    /// Game hours of fuel a fire holds at most.
    most: f64,
    /// Seconds to put a piece of fuel on, to light it, and to put it out.
    lay: f64,
    light: f64,
    smother: f64,
}

/// The fire's numbers.
pub(crate) fn to_fire(file: &Path, f: &FireFile) -> Result<Fire, DataError> {
    if !(f.most > 0.0 && f.most <= 48.0) {
        return Err(fail(file, "fire.most is above 0 and at most 48 game hours"));
    }
    Ok(Fire {
        most: (f.most * 60.0).round().max(1.0) as u32,
        lay: (ticks(file, "fire.lay", f.lay)? as u32).max(1),
        light: (ticks(file, "fire.light", f.light)? as u32).max(1),
        smother: (ticks(file, "fire.smother", f.smother)? as u32).max(1),
    })
}
