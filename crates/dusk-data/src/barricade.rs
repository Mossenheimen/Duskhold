//! Boarding up (rules.toml's [barricade]): its numbers into the core's `Barricade` (seconds as
//! ticks, points as thousandths), and the items it takes into `Boarding`.

use std::path::Path;

use dusk_core::ItemDef;
use dusk_core::barricade::{Barricade, Boarding};
use serde::Deserialize;

use crate::{DataError, fail, points, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BarricadeFile {
    /// The items a board takes: a plank and so many nails, and any one of the hammers.
    plank: String,
    nails: String,
    nails_each: u16,
    hammers: Vec<String>,
    /// The most boards on a door or window.
    most: u8,
    /// Points of beating a board takes before it breaks.
    strength: f64,
    /// Seconds to nail one on, and how far it's heard (tiles); to pry one off, and its noise.
    time: f64,
    noise: i32,
    remove: f64,
    remove_noise: i32,
}

/// The barricade's numbers.
pub(crate) fn to_barricade(file: &Path, b: &BarricadeFile) -> Result<Barricade, DataError> {
    // The client packs the board count into four bits of an edge's code.
    if !(1..=15).contains(&b.most) || b.strength <= 0.0 || b.noise < 0 || b.remove_noise < 0 {
        return Err(fail(
            file,
            "barricade: most is 1 to 15 boards, strength above 0, and noises 0 or more",
        ));
    }
    Ok(Barricade {
        most: b.most,
        strength: points(b.strength),
        time: (ticks(file, "barricade.time", b.time)? as u32).max(1),
        noise: b.noise,
        remove_time: (ticks(file, "barricade.remove", b.remove)? as u32).max(1),
        remove_noise: b.remove_noise,
    })
}

/// rules.toml's [building]: doors and walls of your own (`build.rs` in the core).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuildingFile {
    /// Seconds to put a door or a wall up.
    time: f64,
    /// Points of beating a wall of planks takes before it gives way.
    wall: f64,
}

/// Building's numbers.
pub(crate) fn to_building(file: &Path, b: &BuildingFile) -> Result<dusk_core::Building, DataError> {
    if b.wall <= 0.0 || b.time <= 0.0 {
        return Err(fail(
            file,
            "building: time is seconds above 0, and a wall's strength points above 0",
        ));
    }
    Ok(dusk_core::Building {
        time: (ticks(file, "building.time", b.time)? as u32).max(1),
        wall: points(b.wall),
    })
}

/// The items boarding up takes, each of them checked.
pub(crate) fn to_boarding(
    file: &Path,
    b: &BarricadeFile,
    items: &[ItemDef],
) -> Result<Boarding, DataError> {
    let item = |name: &str, what: &str| -> Result<u16, DataError> {
        items
            .iter()
            .position(|i| i.id == name)
            .map(|n| n as u16)
            .ok_or_else(|| fail(file, format!("barricade.{what}: no item called {name:?}")))
    };
    if b.hammers.is_empty() {
        return Err(fail(file, "barricade.hammers: name at least one"));
    }
    Ok(Boarding {
        plank: item(&b.plank, "plank")?,
        nails: item(&b.nails, "nails")?,
        nails_each: b.nails_each,
        hammers: b
            .hammers
            .iter()
            .map(|h| item(h, "hammers"))
            .collect::<Result<_, _>>()?,
    })
}
