//! Faith (rules.toml's [faith]): the Cleric's prayer, turning the dead, consecration, mending and
//! wards, in a designer's units: points out of 100, seconds, tiles and game hours.

use std::path::Path;

use dusk_core::Faith;
use serde::Deserialize;

use crate::{DataError, fail, points, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FaithFile {
    /// Seconds kneeling at a shrine, and the points of dread a prayer takes away.
    pray: f64,
    comfort: f64,
    /// Turning the dead: its faith (points), its wind-up and recovery (seconds), its reach
    /// (tiles), how long the dead stay turned (seconds), and how far the word carries (tiles).
    turn: f64,
    turn_time: f64,
    turn_recovery: f64,
    reach: i32,
    turned: f64,
    noise: i32,
    /// Consecrating a body: its faith without holy water (points), and seconds.
    consecrate: f64,
    consecrate_time: f64,
    /// Mending (a talent's): its faith, the health it gives back (points), and seconds.
    mend: f64,
    mend_health: f64,
    mend_time: f64,
    /// Warding a door (a talent's): its faith, seconds, and the game hours it holds.
    ward: f64,
    ward_time: f64,
    ward_hours: f64,
}

/// Faith's rules, from rules.toml. `ticks_per_minute` is the clock's, for the ward's hours.
pub(crate) fn to_faith(
    file: &Path,
    f: &FaithFile,
    ticks_per_minute: u32,
) -> Result<Faith, DataError> {
    let amounts = [
        f.comfort,
        f.turn,
        f.consecrate,
        f.mend,
        f.mend_health,
        f.ward,
    ];
    if amounts.iter().any(|p| !(0.0..=100.0).contains(p)) {
        return Err(fail(
            file,
            "faith: comfort, turn, consecrate, mend, mend_health and ward are points from 0 to 100",
        ));
    }
    if !(1..=40).contains(&f.reach) || f.noise < 0 || !(0.0..=48.0).contains(&f.ward_hours) {
        return Err(fail(
            file,
            "faith: reach is 1 to 40 tiles, noise 0 or more, ward_hours 0 to 48",
        ));
    }
    let thousandths = |p: f64| (p * 1000.0).round() as i32;
    let t = |what: &str, s: f64| ticks(file, &format!("faith.{what}"), s);
    Ok(Faith {
        pray: (t("pray", f.pray)? as u32).max(1),
        comfort: thousandths(f.comfort),
        turn: thousandths(f.turn),
        turn_wind_up: (t("turn_time", f.turn_time)? as u32).max(1),
        turn_recovery: t("turn_recovery", f.turn_recovery)? as u32,
        reach: f.reach,
        turned: t("turned", f.turned)?,
        noise: f.noise,
        consecrate: thousandths(f.consecrate),
        consecrate_time: (t("consecrate_time", f.consecrate_time)? as u32).max(1),
        mend: thousandths(f.mend),
        mend_health: points(f.mend_health),
        mend_time: (t("mend_time", f.mend_time)? as u32).max(1),
        ward: thousandths(f.ward),
        ward_time: (t("ward_time", f.ward_time)? as u32).max(1),
        ward_lasts: (f.ward_hours * 60.0).round() as u64 * u64::from(ticks_per_minute),
    })
}
