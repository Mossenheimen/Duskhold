//! Day and night and the character's eyes (rules.toml's [day] and [sight]), turned into the
//! core's `Day` and `Sight`: hours as game minutes, fractions as per mille, a cone's width as the
//! cosine of half of it.

use std::path::Path;

use dusk_core::light::{Day, Sight};
use dusk_core::senses::Cone;
use serde::Deserialize;

use crate::{DataError, RulesFile, fail};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DayFile {
    /// Hours: the sky lightens from `dawn`, is full from `sunrise` to `sunset`, dark from `dark`.
    dawn: f64,
    sunrise: f64,
    sunset: f64,
    dark: f64,
    /// The sky's light at night, out of 100.
    night: f64,
    /// Under a roof, the share of the sky's light that's left.
    indoors: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SightFile {
    /// Tiles, in full light.
    range: i32,
    /// Degrees across.
    cone: f64,
    /// Tiles all round seen in any light.
    beside: i32,
}

/// A cone's width across, in degrees, as the core's cosine of half of it (in ten-thousandths).
pub(crate) fn cone(degrees: f64) -> Cone {
    Cone {
        cos: ((degrees / 2.0).to_radians().cos() * 10_000.0).round() as i32,
    }
}

/// The day and the character's eyes, from rules.toml.
pub(crate) fn to_light(file: &Path, r: &RulesFile) -> Result<(Day, Sight), DataError> {
    let d = &r.day;
    let hours = [d.dawn, d.sunrise, d.sunset, d.dark];
    if !(hours.windows(2).all(|w| w[0] < w[1]) && d.dawn >= 0.0 && d.dark < 24.0) {
        return Err(fail(
            file,
            "day: dawn, sunrise, sunset and dark are hours from 0 to 24, in that order",
        ));
    }
    if !(0.0..=100.0).contains(&d.night) || !(0.0..=1.0).contains(&d.indoors) {
        return Err(fail(
            file,
            "day.night is out of 100, and day.indoors from 0 to 1",
        ));
    }
    let s = &r.sight;
    if s.range < 1 || s.beside < 0 || !(s.cone > 0.0 && s.cone <= 360.0) {
        return Err(fail(
            file,
            "sight: range is 1 tile or more, beside 0 or more, and the cone above 0 and at most \
             360 degrees",
        ));
    }
    let minutes = |h: f64| (h * 60.0).round() as u32;
    let day = Day {
        dawn: minutes(d.dawn),
        sunrise: minutes(d.sunrise),
        sunset: minutes(d.sunset),
        dark: minutes(d.dark),
        night: d.night.round() as u32,
        indoors: (d.indoors * 1000.0).round() as u32,
    };
    let sight = Sight {
        range: s.range,
        cone: cone(s.cone),
        beside: s.beside,
    };
    Ok((day, sight))
}
