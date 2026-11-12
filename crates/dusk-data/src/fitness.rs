//! Strength and Fitness (`fitness.rs` in the core): rules.toml's [strength] and [fitness]
//! (optional: with none, the core's defaults).

use std::path::Path;

use dusk_core::fitness::{FitnessRules, Growth, StrengthRules};
use serde::Deserialize;

use crate::{DataError, chance, fail, points};

/// How a day's work turns into a stat: at most `most` levels a day, half that for `half` work.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FitnessFile {
    most: f64,
    /// Points of breath spent in a day.
    half: f64,
    /// Each level: what's taken off every cost in breath, and added to how fast it comes back.
    cost: f64,
    recover: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StrengthFile {
    most: f64,
    /// Points of strain in a day.
    half: f64,
    /// The strain of a blow that lands (a heavy one, twice), a shove that does, and a step
    /// carrying more than half what you can.
    blow: f64,
    shove: f64,
    haul: f64,
    /// Each level: kg more you carry, the share more harm, and the chance more a shove knocks
    /// down.
    carry: f64,
    harm: f64,
    knock_down: f64,
}

fn growth(file: &Path, what: &str, most: f64, half: f64, unit: f64) -> Result<Growth, DataError> {
    if !(most > 0.0 && most <= 2.0) || half <= 0.0 {
        return Err(fail(
            file,
            format!("{what}: most is above 0 and at most 2 levels a day, half above 0"),
        ));
    }
    Ok(Growth {
        most: (most * 1000.0).round() as u32,
        half: (half * unit).round() as u32,
    })
}

pub(crate) fn to_fitness(file: &Path, f: &FitnessFile) -> Result<FitnessRules, DataError> {
    if !(0.0..=0.09).contains(&f.cost) || !(0.0..=0.5).contains(&f.recover) {
        return Err(fail(
            file,
            "fitness: cost is a share from 0 to 0.09 a level, recover 0 to 0.5",
        ));
    }
    Ok(FitnessRules {
        // Breath is out of body::FULL: ten to a point.
        growth: growth(file, "fitness", f.most, f.half, f64::from(points(1.0)))?,
        cost: (f.cost * 1000.0).round() as u32,
        recover: (f.recover * 1000.0).round() as u32,
    })
}

pub(crate) fn to_strength(file: &Path, s: &StrengthFile) -> Result<StrengthRules, DataError> {
    let work = [s.blow, s.shove, s.haul];
    if work.iter().any(|w| !(0.0..=10.0).contains(w))
        || !(0.0..=5.0).contains(&s.carry)
        || !(0.0..=0.2).contains(&s.harm)
    {
        return Err(fail(
            file,
            "strength: blow, shove and haul are 0 to 10 points of strain, carry 0 to 5 kg a \
             level, harm a share from 0 to 0.2 a level",
        ));
    }
    let strain = |x: f64| (x * 1000.0).round() as u32;
    Ok(StrengthRules {
        growth: growth(file, "strength", s.most, s.half, 1000.0)?,
        blow: strain(s.blow),
        shove: strain(s.shove),
        haul: strain(s.haul),
        carry: (s.carry * 1000.0).round() as u32,
        harm: (s.harm * 1000.0).round() as u32,
        knock_down: chance(file, "strength.knock_down", s.knock_down)?,
    })
}
