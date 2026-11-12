//! Sneaking (rules.toml's [sneak]): shares as per mille.

use std::path::Path;

use dusk_core::stealth::Sneak;
use serde::Deserialize;

use crate::{DataError, fail};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SneakFile {
    /// What's left sneaking of your walking pace, of your footsteps' reach, and of how far off
    /// the dead see you.
    pace: f64,
    noise: f64,
    seen: f64,
    /// A sneak attack's harm, as many times a blow's.
    harm: f64,
    /// Tiles: a sneaked tile teaches Stealth with the dead this near.
    near: i32,
}

pub(crate) fn to_sneak(file: &Path, s: &SneakFile) -> Result<Sneak, DataError> {
    let share = |x: f64| x > 0.0 && x <= 1.0;
    if !share(s.pace)
        || !share(s.noise)
        || !share(s.seen)
        || !(1.0..=10.0).contains(&s.harm)
        || s.near < 1
    {
        return Err(fail(
            file,
            "sneak: pace, noise and seen are shares above 0 and at most 1, harm 1 to 10, near 1 or more",
        ));
    }
    let mille = |x: f64| (x * 1000.0).round() as u32;
    Ok(Sneak {
        pace: mille(s.pace),
        noise: mille(s.noise),
        seen: mille(s.seen),
        harm: mille(s.harm),
        near: s.near,
    })
}
