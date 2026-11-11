//! Ghoul fever (rules.toml's [fever]): game hours as game minutes, paces as per mille, health as
//! points.

use std::path::Path;

use dusk_core::fever::Fever;
use serde::Deserialize;

use crate::{DataError, fail, points};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FeverFile {
    /// Game hours each of the three stages lasts.
    stage: f64,
    /// Feverish, thirst rises this many times as fast.
    thirst: f64,
    /// Weak, endurance comes back at this pace.
    recovery: f64,
    /// Dying, the health points it costs a game hour.
    harm: f64,
}

pub(crate) fn to_fever(file: &Path, f: &FeverFile) -> Result<Fever, DataError> {
    if !(1.0..=240.0).contains(&f.stage) {
        return Err(fail(file, "fever.stage is 1 to 240 game hours"));
    }
    if !(1.0..=5.0).contains(&f.thirst) || !(0.0..=1.0).contains(&f.recovery) {
        return Err(fail(
            file,
            "fever: thirst is a pace from 1 to 5, and recovery one from 0 to 1",
        ));
    }
    if !(0.0..=100.0).contains(&f.harm) {
        return Err(fail(file, "fever.harm is 0 to 100 points a game hour"));
    }
    Ok(Fever {
        stage: (f.stage * 60.0).round() as u32,
        thirst: (f.thirst * 1000.0).round() as u32,
        recovery: (f.recovery * 1000.0).round() as u32,
        harm: points(f.harm),
    })
}
