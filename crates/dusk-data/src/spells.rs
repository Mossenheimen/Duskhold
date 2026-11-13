//! Spells (`data/spells.toml`), in a designer's units: seconds, tiles, points of health, the
//! component by item id.

use std::path::Path;

use dusk_core::{Defs, SpellDef, SpellKind};
use serde::Deserialize;

use crate::{DataError, check_unique, fail, points, read, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellsFile {
    spell: Vec<SpellFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellFile {
    id: String,
    name: String,
    line: String,
    /// What it does: light, silence, fog or fireball.
    kind: String,
    /// Slots it costs (0: a cantrip).
    #[serde(default)]
    slots: u8,
    /// Seconds of words.
    cast: f64,
    /// Tiles to the spot.
    range: i32,
    /// Tiles it reaches round the spot, and tiles it lights.
    #[serde(default)]
    radius: i32,
    #[serde(default)]
    light: i32,
    /// Seconds it lasts on the spot (its light too).
    #[serde(default)]
    lasts: f64,
    /// Tiles its words carry, and its burst.
    #[serde(default)]
    noise: i32,
    #[serde(default)]
    boom: i32,
    /// The harm its burst does, least and most, in points.
    #[serde(default)]
    damage: (f64, f64),
    /// The item it uses up, by id (left out: none).
    #[serde(default)]
    component: Option<String>,
}

/// The spells in `spells.toml` (left out: none). The items are `defs`', for the components.
pub(crate) fn load(dir: &Path, defs: &Defs) -> Result<Vec<SpellDef>, DataError> {
    let file = dir.join("spells.toml");
    if !file.exists() {
        return Ok(Vec::new());
    }
    let parsed: SpellsFile = read(&file)?;
    check_unique(&file, "spell", parsed.spell.iter().map(|s| &s.id))?;
    parsed
        .spell
        .iter()
        .map(|s| to_spell(&file, s, defs))
        .collect()
}

fn to_spell(file: &Path, s: &SpellFile, defs: &Defs) -> Result<SpellDef, DataError> {
    let bad = |m: &str| fail(file, format!("spell {:?}: {m}", s.id));
    let kind = match s.kind.as_str() {
        "light" => SpellKind::Light,
        "silence" => SpellKind::Silence,
        "fog" => SpellKind::Fog,
        "fireball" => SpellKind::Fireball,
        _ => return Err(bad("kind is light, silence, fog or fireball")),
    };
    let tiles = [s.range, s.radius, s.light];
    if tiles.iter().any(|t| !(0..=20).contains(t)) || s.range < 1 {
        return Err(bad("range is 1 to 20 tiles, and radius and light 0 to 20"));
    }
    if !(0..=60).contains(&s.noise) || !(0..=60).contains(&s.boom) || s.slots > 9 {
        return Err(bad("noise and boom are 0 to 60 tiles, slots 0 to 9"));
    }
    if s.damage.0 < 0.0 || s.damage.0 > s.damage.1 || s.damage.1 > 200.0 {
        return Err(bad("damage is [least, most] points, from 0 to 200"));
    }
    // What each kind needs to do anything.
    let empty = match kind {
        SpellKind::Light => s.light == 0 || s.lasts <= 0.0,
        SpellKind::Silence | SpellKind::Fog => s.radius == 0 || s.lasts <= 0.0,
        SpellKind::Fireball => s.radius == 0 || s.damage.1 <= 0.0,
    };
    if empty {
        return Err(bad(
            "it does nothing: a light lights and lasts, a silence or fog reaches and lasts, a \
             fireball reaches and harms",
        ));
    }
    let component = match &s.component {
        None => None,
        Some(name) => Some(
            defs.items
                .iter()
                .position(|i| &i.id == name)
                .map(|n| n as u16)
                .ok_or_else(|| bad(&format!("component: no item called {name:?} in items/")))?,
        ),
    };
    Ok(SpellDef {
        id: s.id.clone(),
        name: s.name.clone(),
        line: s.line.clone(),
        kind,
        slots: s.slots,
        cast: (ticks(file, &format!("spell {:?}: cast", s.id), s.cast)? as u32).max(1),
        range: s.range,
        radius: s.radius,
        light: s.light,
        lasts: ticks(file, &format!("spell {:?}: lasts", s.id), s.lasts)? as u32,
        noise: s.noise,
        boom: s.boom,
        damage: (points(s.damage.0), points(s.damage.1)),
        component,
    })
}
