//! The clock, needs, sleep and healing (rules.toml's [clock], [hunger], [thirst], [fatigue],
//! [sleep], [healing] and [water]), turned into the core's `Clock` and `Needs`.
//!
//! Needs are out of 100 in the file and thousandths of a point in the core; rates are a game hour
//! in the file and a game minute in the core (the clock's step). Health is the core's tenths.

use std::path::Path;

use dusk_core::needs::{Clock, Fatigue, Need, Needs};
use dusk_core::world::TICKS_PER_SECOND;
use serde::Deserialize;

use crate::{DataError, RulesFile, fail, points, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClockFile {
    /// How many times as fast as action time the clock runs.
    rate: f64,
    /// The hour day 1 begins.
    start: f64,
}

/// Hunger or thirst.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NeedFile {
    /// Points a game hour.
    rise: f64,
    levels: [f64; 3],
    /// From the second level on, endurance comes back at this pace.
    recovery: f64,
    /// At the third level, health lost a game hour.
    harm: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FatigueFile {
    rise: f64,
    rest: f64,
    levels: [f64; 3],
    slow: [f64; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SleepFile {
    lie_down: f64,
    needs: f64,
    clear: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HealingFile {
    health: f64,
    asleep: i32,
    scratch: f64,
    laceration: f64,
    bite: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WaterFile {
    drink: f64,
}

/// Points a game hour, as thousandths of a point a game minute.
fn per_minute(file: &Path, what: &str, per_hour: f64) -> Result<i32, DataError> {
    if !(0.0..=1000.0).contains(&per_hour) {
        return Err(fail(
            file,
            format!("{what} must be 0 or more points a game hour"),
        ));
    }
    Ok((per_hour * 1000.0 / 60.0).round() as i32)
}

/// Three levels out of 100, rising, as thousandths of a point.
fn levels(file: &Path, what: &str, l: [f64; 3]) -> Result<[i32; 3], DataError> {
    if !(0.0 < l[0] && l[0] < l[1] && l[1] < l[2] && l[2] <= 100.0) {
        return Err(fail(
            file,
            format!("{what}.levels are three rising points, above 0 and at most 100"),
        ));
    }
    Ok(l.map(|p| (p * 1000.0).round() as i32))
}

/// A pace from 0 to 1, per mille.
fn pace(file: &Path, what: &str, p: f64) -> Result<u32, DataError> {
    if !(0.0..=1.0).contains(&p) {
        return Err(fail(file, format!("{what} is a pace, from 0 to 1")));
    }
    Ok((p * 1000.0).round() as u32)
}

fn need(file: &Path, what: &str, n: &NeedFile) -> Result<Need, DataError> {
    if n.harm < 0.0 {
        return Err(fail(file, format!("{what}.harm can't be negative")));
    }
    Ok(Need {
        rise: per_minute(file, &format!("{what}.rise"), n.rise)?,
        levels: levels(file, what, n.levels)?,
        recovery: pace(file, &format!("{what}.recovery"), n.recovery)?,
        harm: points(n.harm),
    })
}

/// Game hours, as game minutes.
fn minutes(file: &Path, what: &str, hours: f64) -> Result<u32, DataError> {
    if !(0.0..1e5).contains(&hours) {
        return Err(fail(file, format!("{what} must be 0 game hours or more")));
    }
    Ok((hours * 60.0).round() as u32)
}

/// The clock and the needs, from rules.toml.
pub(crate) fn to_needs(file: &Path, r: &RulesFile) -> Result<(Clock, Needs), DataError> {
    let c = &r.clock;
    let minute = 60.0 * f64::from(TICKS_PER_SECOND) / c.rate;
    if !(1.0..1e6).contains(&minute) || !(0.0..24.0).contains(&c.start) {
        return Err(fail(
            file,
            "clock.rate must leave a game minute at least a tick long, and clock.start is an \
             hour from 0 to 24",
        ));
    }
    let clock = Clock {
        ticks_per_minute: minute.round() as u32,
        start: (c.start * 60.0).round() as u32,
    };

    let f = &r.fatigue;
    if f.rest <= 0.0 || f.slow[0] < 1.0 || f.slow[0] > f.slow[1] || f.slow[1] > 10.0 {
        return Err(fail(
            file,
            "fatigue.rest must be above 0, and fatigue.slow two rising paces from 1",
        ));
    }
    let (s, h) = (&r.sleep, &r.healing);
    if s.clear < 0 || h.health < 0.0 || h.asleep < 1 {
        return Err(fail(
            file,
            "sleep.clear and healing.health can't be negative, and healing.asleep is 1 or more",
        ));
    }
    let needs = Needs {
        hunger: need(file, "hunger", &r.hunger)?,
        thirst: need(file, "thirst", &r.thirst)?,
        fatigue: Fatigue {
            rise: per_minute(file, "fatigue.rise", f.rise)?,
            rest: per_minute(file, "fatigue.rest", f.rest)?,
            levels: levels(file, "fatigue", f.levels)?,
            slow: f.slow.map(|p| (p * 1000.0).round() as u32),
        },
        asleep: pace(file, "sleep.needs", s.needs)?,
        heal: points(h.health),
        sleep_heal: h.asleep,
        wound_heal: [
            minutes(file, "healing.scratch", h.scratch)?,
            minutes(file, "healing.laceration", h.laceration)?,
            minutes(file, "healing.bite", h.bite)?,
        ],
        lie_down: (ticks(file, "sleep.lie_down", s.lie_down)? as u32).max(1),
        drink_water: (ticks(file, "water.drink", r.water.drink)? as u32).max(1),
        sleep_clear: s.clear,
    };
    Ok((clock, needs))
}
