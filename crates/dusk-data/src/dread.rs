//! Pain and dread (rules.toml's [pain] and [dread]): points out of 100 as the core's thousandths,
//! shares as per mille, and paces a game minute.

use std::path::Path;

use dusk_core::dread::{Dread, Pain};
use serde::Deserialize;

use crate::needs::{levels, pace};
use crate::{DataError, fail};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PainFile {
    /// What a fresh wound of each kind hurts, in points out of 100.
    scratch: f64,
    laceration: f64,
    bite: f64,
    /// The share a dressed wound still hurts.
    dressed: f64,
    /// Sore, in pain, agony.
    levels: [f64; 3],
    /// In pain, and in agony: what endurance costs come to.
    cost: [f64; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DreadFile {
    /// The dark's share, with your tile lit below `dark_below` (out of 100).
    dark: f64,
    dark_below: u32,
    /// Each of the dead within `near` tiles that you see or that's after something, up to `most`.
    per_dead: f64,
    near: i32,
    most: u32,
    /// Each level of pain, and each of hunger past the first.
    per_pain: f64,
    hungry: f64,
    /// Points a game minute: up toward what the moment is worth, down, and asleep or on hallowed
    /// ground, down to nothing.
    rise: f64,
    fall: f64,
    calm: f64,
    /// Uneasy, afraid, frightened, and what's left of your chance to hit at each.
    levels: [f64; 3],
    aim: [f64; 3],
}

/// Points out of 100 as thousandths.
fn points(file: &Path, what: &str, p: f64) -> Result<i32, DataError> {
    if !(0.0..=100.0).contains(&p) {
        return Err(fail(file, format!("{what} is points, from 0 to 100")));
    }
    Ok((p * 1000.0).round() as i32)
}

pub(crate) fn to_pain(file: &Path, p: &PainFile) -> Result<Pain, DataError> {
    if !(1.0 <= p.cost[0] && p.cost[0] <= p.cost[1] && p.cost[1] <= 4.0) {
        return Err(fail(
            file,
            "pain.cost is two rising paces, from 1 to 4: in pain, then in agony",
        ));
    }
    Ok(Pain {
        wound: [
            points(file, "pain.scratch", p.scratch)?,
            points(file, "pain.laceration", p.laceration)?,
            points(file, "pain.bite", p.bite)?,
        ],
        dressed: pace(file, "pain.dressed", p.dressed)?,
        levels: levels(file, "pain", p.levels)?,
        cost: p.cost.map(|c| (c * 1000.0).round() as u32),
    })
}

pub(crate) fn to_dread(file: &Path, d: &DreadFile) -> Result<Dread, DataError> {
    if d.dark_below > 100 || d.near < 0 || d.rise <= 0.0 || d.fall <= 0.0 || d.calm <= 0.0 {
        return Err(fail(
            file,
            "dread: dark_below is out of 100, near 0 or more, and rise, fall and calm above 0",
        ));
    }
    let aim = [
        pace(file, "dread.aim", d.aim[0])?,
        pace(file, "dread.aim", d.aim[1])?,
        pace(file, "dread.aim", d.aim[2])?,
    ];
    Ok(Dread {
        dark_below: d.dark_below,
        dark: points(file, "dread.dark", d.dark)?,
        near: d.near,
        per_dead: points(file, "dread.per_dead", d.per_dead)?,
        most: d.most,
        per_pain: points(file, "dread.per_pain", d.per_pain)?,
        hungry: points(file, "dread.hungry", d.hungry)?,
        rise: points(file, "dread.rise", d.rise)?,
        fall: points(file, "dread.fall", d.fall)?,
        calm: points(file, "dread.calm", d.calm)?,
        levels: levels(file, "dread", d.levels)?,
        aim,
    })
}
