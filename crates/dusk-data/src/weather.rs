//! A region's weather (`[weather]`): spells in game hours, shares and multipliers by light and
//! heavy rain, and the chance a minute a light outdoors goes out.

use dusk_core::Weather;
use serde::Deserialize;

/// A region's rain (`dusk_core::weather`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeatherFile {
    /// Game hours a spell of weather lasts.
    pub spell: f64,
    /// The share of spells that are rain, and of those, the share that's heavy.
    pub rain: f64,
    pub heavy: f64,
    /// [light rain, heavy rain], as multipliers: how far a sound made outdoors carries, how far
    /// blood outdoors is smelled, and the sky's light.
    pub sound: (f64, f64),
    pub scent: (f64, f64),
    pub sky: (f64, f64),
    /// [light rain, heavy rain]: the chance a minute a light held up outdoors goes out.
    pub douse: (f64, f64),
}

impl WeatherFile {
    pub(crate) fn problem(&self) -> Option<String> {
        let unit = |x: f64| (0.0..=1.0).contains(&x);
        let pairs = [self.sound, self.scent, self.sky, self.douse];
        if !(0.25..=48.0).contains(&self.spell) || !unit(self.rain) || !unit(self.heavy) {
            return Some("weather: spell is 0.25 to 48 game hours, rain and heavy shares".into());
        }
        if pairs.iter().any(|&(a, b)| !unit(a) || !unit(b)) {
            return Some(
                "weather: sound, scent, sky and douse are each [light, heavy], 0 to 1".into(),
            );
        }
        None
    }

    pub fn to_weather(&self) -> Weather {
        let per = |x: f64| (x * 1000.0).round() as u32;
        let chance = |x: f64| (x * 10_000.0).round() as u32;
        Weather {
            spell: (self.spell * 60.0).round() as u32,
            rain: chance(self.rain),
            heavy: chance(self.heavy),
            sound: [per(self.sound.0), per(self.sound.1)],
            scent: [per(self.scent.0), per(self.scent.1)],
            sky: [per(self.sky.0), per(self.sky.1)],
            douse: [chance(self.douse.0), chance(self.douse.1)],
        }
    }
}
