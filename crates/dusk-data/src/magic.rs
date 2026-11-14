//! Magic items: their properties and curses (`data/properties.toml`) in a designer's units
//! (multipliers, a chance added, tiles), rules.toml's `[magic]` (how they're rolled and learnt),
//! and its `[quality]` (how well a weapon found was made).

use std::path::Path;

use dusk_core::{MagicRules, Property, QualityRules};
use serde::Deserialize;

use crate::{DataError, check_unique, fail, read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertiesFile {
    property: Vec<PropertyFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertyFile {
    id: String,
    /// The word before an item's name, and its one line.
    name: String,
    line: String,
    /// "weapon" (in hand) or "light" (held up).
    on: String,
    #[serde(default)]
    curse: bool,
    /// A weapon's, as multipliers (1.0 for none): its noise, harm, wind-up and recovery, and
    /// wear; and a chance added to its blow landing (from -1 to 1).
    #[serde(default = "one")]
    noise: f64,
    #[serde(default = "one")]
    damage: f64,
    #[serde(default = "one")]
    swing: f64,
    #[serde(default = "one")]
    wear: f64,
    #[serde(default)]
    hit: f64,
    /// What it puts down stays down.
    #[serde(default)]
    hallowed: bool,
    /// Tiles of light: round you, in hand; added to a light's reach.
    #[serde(default)]
    glow: i32,
    /// A light's: how fast it burns down, as a multiplier.
    #[serde(default = "one")]
    wane: f64,
}

fn one() -> f64 {
    1.0
}

/// rules.toml's `[magic]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MagicFile {
    /// Of the magic items, the share that's Rare, and the chance one is cursed.
    rare: f64,
    cursed: f64,
    /// The XP a magic item adds to its kind's worth: Uncommon, Rare.
    worth: (f64, f64),
    /// Blows on a foe with an unknown magic weapon before you know what it is.
    learn: u8,
}

pub(crate) fn to_rules(file: &Path, m: &MagicFile) -> Result<MagicRules, DataError> {
    let unit = |x: f64| (0.0..=1.0).contains(&x);
    if !unit(m.rare) || !unit(m.cursed) || m.learn == 0 {
        return Err(fail(
            file,
            "magic: rare and cursed are chances from 0 to 1, learn at least 1 blow",
        ));
    }
    if !(0.0..=10_000.0).contains(&m.worth.0) || !(0.0..=10_000.0).contains(&m.worth.1) {
        return Err(fail(
            file,
            "magic.worth is [Uncommon, Rare], 0 to 10,000 XP",
        ));
    }
    Ok(MagicRules {
        rare: (m.rare * 10_000.0).round() as u32,
        cursed: (m.cursed * 10_000.0).round() as u32,
        worth: [
            (m.worth.0 * 1000.0).round() as u32,
            (m.worth.1 * 1000.0).round() as u32,
        ],
        learn: m.learn,
    })
}

/// rules.toml's `[quality]`: the four grades, worst first.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct QualityFile {
    crude: GradeFile,
    plain: GradeFile,
    fine: GradeFile,
    masterwork: GradeFile,
}

/// A grade: its word before a name ("" for plain), the chance a weapon found is of it (plain's is
/// what's left, so it has none), and multipliers on its harm, its wear and its worth.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GradeFile {
    #[serde(default)]
    name: String,
    #[serde(default)]
    chance: f64,
    damage: f64,
    wear: f64,
    worth: f64,
}

pub(crate) fn quality(file: &Path, q: &QualityFile) -> Result<QualityRules, DataError> {
    let grades = [
        ("crude", &q.crude),
        ("plain", &q.plain),
        ("fine", &q.fine),
        ("masterwork", &q.masterwork),
    ];
    let mut out = QualityRules::default();
    let mut odds = 0.0;
    for (n, (id, g)) in grades.iter().enumerate() {
        let bad = |m: &str| fail(file, format!("quality.{id}: {m}"));
        if *id == "plain" {
            if g.chance != 0.0 {
                return Err(bad("plain has no chance: it's what's left"));
            }
        } else if g.name.trim().is_empty() {
            return Err(bad("a name, the word before a weapon's"));
        }
        if !(0.0..=1.0).contains(&g.chance) {
            return Err(bad("chance is from 0 to 1"));
        }
        if !(0.1..=5.0).contains(&g.damage) || !(0.1..=10.0).contains(&g.wear) {
            return Err(bad("damage is a multiplier from 0.1 to 5, wear 0.1 to 10"));
        }
        if !(0.0..=100.0).contains(&g.worth) {
            return Err(bad("worth is a multiplier from 0 to 100"));
        }
        odds += g.chance;
        let per = |m: f64| (m * 1000.0).round() as u32;
        out.names[n] = g.name.trim().to_string();
        out.chance[n] = (g.chance * 10_000.0).round() as u32;
        out.damage[n] = per(g.damage);
        out.wear[n] = per(g.wear);
        out.worth[n] = per(g.worth);
    }
    if odds > 1.0 + 1e-9 {
        return Err(fail(
            file,
            "quality: crude, fine and masterwork's chances add up to more than 1",
        ));
    }
    Ok(out)
}

/// Every property in `properties.toml` (left out: none, and no magic items).
pub(crate) fn load(dir: &Path) -> Result<Vec<Property>, DataError> {
    let file = dir.join("properties.toml");
    if !file.exists() {
        return Ok(Vec::new());
    }
    let parsed: PropertiesFile = read(&file)?;
    check_unique(&file, "property", parsed.property.iter().map(|p| &p.id))?;
    if parsed.property.len() > 250 {
        return Err(fail(&file, "at most 250 properties"));
    }
    let mut out = Vec::new();
    for p in &parsed.property {
        let bad = |m: &str| fail(&file, format!("property {:?}: {m}", p.id));
        let on_light = match p.on.as_str() {
            "weapon" => false,
            "light" => true,
            _ => return Err(bad("on is \"weapon\" or \"light\"")),
        };
        let multipliers = [p.noise, p.damage, p.swing, p.wear, p.wane];
        if multipliers.iter().any(|m| !(0.0..=10.0).contains(m)) {
            return Err(bad(
                "noise, damage, swing, wear and wane are multipliers from 0 to 10",
            ));
        }
        if !(-1.0..=1.0).contains(&p.hit) || !(0..=12).contains(&p.glow) {
            return Err(bad("hit is a chance from -1 to 1, glow 0 to 12 tiles"));
        }
        let per = |m: f64| (m * 1000.0).round() as u32;
        out.push(Property {
            id: p.id.clone(),
            name: p.name.clone(),
            line: p.line.clone(),
            on_light,
            curse: p.curse,
            noise: per(p.noise),
            damage: per(p.damage),
            swing: per(p.swing),
            hit: (p.hit * 10_000.0).round() as i32,
            wear: per(p.wear),
            hallowed: p.hallowed,
            glow: p.glow,
            wane: per(p.wane),
        });
    }
    Ok(out)
}
