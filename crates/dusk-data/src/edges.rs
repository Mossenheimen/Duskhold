//! Each class of weapon's own edge (`edge.rs` in the core): rules.toml's [edges] (optional: with
//! none, no class has one).

use std::path::Path;

use dusk_core::{Edges, WeaponEdge};
use serde::Deserialize;

use crate::{DataError, chance, fail, ticks};

/// One class's edge: each left out is none.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdgeFile {
    /// More harm to one that's down, a share on top of the double any blow does there.
    #[serde(default)]
    down: f64,
    /// The chance a blow that lands knocks it down.
    #[serde(default)]
    fell: f64,
    /// Seconds a blow that isn't turned leaves it bleeding.
    #[serde(default)]
    bleed: f64,
    /// The chance a blow that lands drives it back a tile.
    #[serde(default)]
    push: f64,
}

/// rules.toml's [edges]: one for each weapon skill.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EdgesFile {
    #[serde(default)]
    blade: EdgeFile,
    #[serde(default)]
    blunt: EdgeFile,
    #[serde(default)]
    axe: EdgeFile,
    #[serde(default)]
    spear: EdgeFile,
}

fn to_edge(file: &Path, class: &str, e: &EdgeFile) -> Result<WeaponEdge, DataError> {
    if !(0.0..=2.0).contains(&e.down) || !(0.0..=30.0).contains(&e.bleed) {
        return Err(fail(
            file,
            format!("edges.{class}: down is a share more harm, 0 to 2, and bleed seconds, 0 to 30"),
        ));
    }
    Ok(WeaponEdge {
        down: (e.down * 1000.0).round() as u32,
        fell: chance(file, &format!("edges.{class}.fell"), e.fell)?,
        bleed: ticks(file, &format!("edges.{class}.bleed"), e.bleed)? as u32,
        push: chance(file, &format!("edges.{class}.push"), e.push)?,
    })
}

pub(crate) fn to_edges(file: &Path, e: &EdgesFile) -> Result<Edges, DataError> {
    Ok(Edges {
        blade: to_edge(file, "blade", &e.blade)?,
        blunt: to_edge(file, "blunt", &e.blunt)?,
        axe: to_edge(file, "axe", &e.axe)?,
        spear: to_edge(file, "spear", &e.spear)?,
    })
}
