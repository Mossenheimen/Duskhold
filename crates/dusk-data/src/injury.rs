//! Wounds by where they land (rules.toml's [injury]): a part's two levels in points out of 100 as
//! the core's thousandths, and what each does as per mille.

use std::path::Path;

use dusk_core::injury::Injury;
use serde::Deserialize;

use crate::{DataError, fail};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InjuryFile {
    /// Hurt, and badly hurt: the pain of a part's wounds.
    levels: [f64; 2],
    /// Arms: how much longer a swing takes, and what's left of its chance to land.
    swing: [f64; 2],
    arm_hit: [f64; 2],
    /// Legs: what's left of your pace, and how much more running costs.
    pace: [f64; 2],
    run: [f64; 2],
    /// Head: what's left of your chance to land, and of how far you see.
    head_hit: [f64; 2],
    sight: [f64; 2],
    /// How much more a wound to the torso hurts.
    torso: f64,
}

fn per_mille(p: f64) -> u32 {
    (p * 1000.0).round() as u32
}

/// What's left at each level: two shares, falling, from 1 down to 0.
fn left(file: &Path, what: &str, p: [f64; 2]) -> Result<[u32; 2], DataError> {
    if !(0.0 <= p[1] && p[1] <= p[0] && p[0] <= 1.0) {
        return Err(fail(
            file,
            format!("injury.{what} is two falling shares, from 1 down to 0"),
        ));
    }
    Ok(p.map(per_mille))
}

/// How much more at each level: two paces, rising, from 1 to 4.
fn more(file: &Path, what: &str, p: [f64; 2]) -> Result<[u32; 2], DataError> {
    if !(1.0 <= p[0] && p[0] <= p[1] && p[1] <= 4.0) {
        return Err(fail(
            file,
            format!("injury.{what} is two rising paces, from 1 to 4"),
        ));
    }
    Ok(p.map(per_mille))
}

pub(crate) fn to_injury(file: &Path, i: &InjuryFile) -> Result<Injury, DataError> {
    let [hurt, bad] = i.levels;
    if !(0.0 < hurt && hurt < bad && bad <= 100.0) {
        return Err(fail(
            file,
            "injury.levels are two rising points, above 0 and at most 100",
        ));
    }
    if !(1.0..=4.0).contains(&i.torso) {
        return Err(fail(file, "injury.torso is a pace from 1 to 4"));
    }
    Ok(Injury {
        levels: [hurt, bad].map(|p| (p * 1000.0).round() as i32),
        swing: more(file, "swing", i.swing)?,
        arm_hit: left(file, "arm_hit", i.arm_hit)?,
        pace: left(file, "pace", i.pace)?,
        run: more(file, "run", i.run)?,
        head_hit: left(file, "head_hit", i.head_hit)?,
        sight: left(file, "sight", i.sight)?,
        torso: per_mille(i.torso),
    })
}
