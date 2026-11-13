//! Wet and cold (rules.toml's [cold]): points out of 100 as the core's thousandths, rates a game
//! minute, shares as per mille.

use std::path::Path;

use dusk_core::Cold;
use serde::Deserialize;

use crate::needs::{levels, pace};
use crate::{DataError, fail};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ColdFile {
    /// Points of wet a game minute: out in light and heavy rain, and wading; and dried in the
    /// open (when it's dry), under a roof, and by a fire.
    soak: [f64; 2],
    wade: f64,
    dry: f64,
    dry_roof: f64,
    dry_fire: f64,
    /// What the cold is worth, in points: the night out of doors, soaked through, the rain on
    /// you (light, heavy); and what a roof and a bed take off.
    night: f64,
    wet: f64,
    rain: [f64; 2],
    roof: f64,
    bed: f64,
    /// The share of what you wear's warmth that's left, soaked through.
    soaked: f64,
    /// Tiles from a lit fire that it warms.
    fire: i32,
    /// Points a game minute toward what the cold is worth: colder, warmer, by a fire.
    rise: f64,
    fall: f64,
    fire_fall: f64,
    /// The moodles' three levels; at cold and freezing, the aim left and the pace breath costs
    /// come to; and freezing, the health lost a game hour.
    levels: [f64; 3],
    aim: [f64; 2],
    breath: [f64; 2],
    harm: f64,
}

/// Points out of 100 as thousandths.
fn points(file: &Path, what: &str, p: f64) -> Result<i32, DataError> {
    if !(0.0..=100.0).contains(&p) {
        return Err(fail(file, format!("cold.{what} is points, from 0 to 100")));
    }
    Ok((p * 1000.0).round() as i32)
}

pub(crate) fn to_cold(file: &Path, c: &ColdFile) -> Result<Cold, DataError> {
    if c.rise <= 0.0 || c.fall <= 0.0 || c.fire_fall <= 0.0 || !(0..=12).contains(&c.fire) {
        return Err(fail(
            file,
            "cold: rise, fall and fire_fall are above 0, and a fire warms 0 to 12 tiles",
        ));
    }
    let shaking = |b: f64| (1.0..=3.0).contains(&b);
    if !(shaking(c.breath[0]) && shaking(c.breath[1]) && c.breath[0] <= c.breath[1])
        || c.aim[0] < c.aim[1]
        || !(0.0..=100.0).contains(&c.harm)
    {
        return Err(fail(
            file,
            "cold: breath is two rising paces from 1 to 3, aim two falling shares (cold, then \
             freezing), harm health points from 0 to 100",
        ));
    }
    Ok(Cold {
        soak: [
            points(file, "soak", c.soak[0])?,
            points(file, "soak", c.soak[1])?,
        ],
        wade: points(file, "wade", c.wade)?,
        dry: points(file, "dry", c.dry)?,
        dry_roof: points(file, "dry_roof", c.dry_roof)?,
        dry_fire: points(file, "dry_fire", c.dry_fire)?,
        night: points(file, "night", c.night)?,
        wet: points(file, "wet", c.wet)?,
        rain: [
            points(file, "rain", c.rain[0])?,
            points(file, "rain", c.rain[1])?,
        ],
        roof: points(file, "roof", c.roof)?,
        bed: points(file, "bed", c.bed)?,
        soaked: pace(file, "cold.soaked", c.soaked)?,
        fire: c.fire,
        rise: points(file, "rise", c.rise)?,
        fall: points(file, "fall", c.fall)?,
        fire_fall: points(file, "fire_fall", c.fire_fall)?,
        levels: levels(file, "cold", c.levels)?,
        aim: [
            pace(file, "cold.aim", c.aim[0])?,
            pace(file, "cold.aim", c.aim[1])?,
        ],
        breath: c.breath.map(|b| (b * 1000.0).round() as u32),
        // Health is counted in tenths of a point (`body::FULL`).
        harm: (c.harm * 10.0).round() as i32,
    })
}
