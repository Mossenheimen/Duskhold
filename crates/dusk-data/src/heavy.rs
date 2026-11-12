//! A heavy swing (`combat::Heavy` in the core): rules.toml's [heavy] (optional: with none, the
//! core's defaults).

use std::path::Path;

use dusk_core::combat::Heavy;
use serde::Deserialize;

use crate::{DataError, chance, fail};

/// rules.toml's [heavy]: each a multiple of a swing's own.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HeavyFile {
    wind_up: f64,
    recovery: f64,
    endurance: f64,
    harm: f64,
    /// The chance it knocks down what it hits.
    knock_down: f64,
    /// How many blows' wear it puts on the weapon.
    wear: u8,
}

pub(crate) fn to_heavy(file: &Path, h: &HeavyFile) -> Result<Heavy, DataError> {
    let times = [h.wind_up, h.recovery, h.endurance, h.harm];
    if times.iter().any(|x| !(1.0..=5.0).contains(x)) || !(1..=5).contains(&h.wear) {
        return Err(fail(
            file,
            "heavy: wind_up, recovery, endurance and harm are 1 to 5 times a swing's, wear 1 to 5 blows",
        ));
    }
    let mille = |x: f64| (x * 1000.0).round() as u32;
    Ok(Heavy {
        wind_up: mille(h.wind_up),
        recovery: mille(h.recovery),
        endurance: mille(h.endurance),
        harm: mille(h.harm),
        knock_down: chance(file, "heavy.knock_down", h.knock_down)?,
        wear: h.wear,
    })
}
