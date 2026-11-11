//! Rare monsters: their affixes (`data/affixes.toml`) in a designer's units (multipliers, tiles,
//! chances), and a region's `[rares]`: how many a patch holds and where.

use std::path::Path;

use dusk_core::Affix;
use serde::Deserialize;

use crate::{DataError, WoundOddsFile, chance, check_unique, fail, read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffixesFile {
    affix: Vec<AffixFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffixFile {
    id: String,
    /// What it adds to a rare's name: "the Lantern-Eater".
    epithet: String,
    /// Multipliers, 1.0 for none: its pace, its health, its wind-up and recovery, and how long
    /// it keeps after what it heard or saw.
    #[serde(default = "one")]
    speed: f64,
    #[serde(default = "one")]
    health: f64,
    #[serde(default = "one")]
    swing: f64,
    #[serde(default = "one")]
    forget: f64,
    /// Tiles added to its groan, and to how far it smells blood for each wound.
    #[serde(default)]
    groan: i32,
    #[serde(default)]
    blood: i32,
    /// At least this chance each kind of wound it leaves carries ghoul fever.
    #[serde(default)]
    fever: Option<WoundOddsFile>,
    /// Times more it gets up after it's put down.
    #[serde(default)]
    rises: u8,
    /// Lights held up within this many tiles of it go out.
    #[serde(default)]
    douses: i32,
}

fn one() -> f64 {
    1.0
}

/// Every affix in `affixes.toml` (left out: there are none, and no rares).
pub(crate) fn load(dir: &Path) -> Result<Vec<Affix>, DataError> {
    let file = dir.join("affixes.toml");
    if !file.exists() {
        return Ok(Vec::new());
    }
    let parsed: AffixesFile = read(&file)?;
    check_unique(&file, "affix", parsed.affix.iter().map(|a| &a.id))?;
    if parsed.affix.len() > 255 {
        return Err(fail(&file, "at most 255 affixes"));
    }
    let mut out = Vec::new();
    for a in &parsed.affix {
        let bad = |m: &str| fail(&file, format!("affix {:?}: {m}", a.id));
        let multipliers = [a.speed, a.health, a.swing, a.forget];
        if multipliers.iter().any(|m| !(0.1..=10.0).contains(m)) {
            return Err(bad(
                "speed, health, swing and forget are multipliers from 0.1 to 10",
            ));
        }
        if !(0..=60).contains(&a.groan)
            || !(0..=60).contains(&a.blood)
            || !(0..=20).contains(&a.douses)
        {
            return Err(bad("groan and blood are 0 to 60 tiles, douses 0 to 20"));
        }
        if a.rises > 5 {
            return Err(bad("rises is 0 to 5"));
        }
        let fever = match &a.fever {
            Some(f) => [
                chance(&file, "fever.scratch", f.scratch)?,
                chance(&file, "fever.laceration", f.laceration)?,
                chance(&file, "fever.bite", f.bite)?,
            ],
            None => [0; 3],
        };
        let per = |m: f64| (m * 1000.0).round() as u32;
        out.push(Affix {
            id: a.id.clone(),
            epithet: a.epithet.clone(),
            speed: per(a.speed),
            health: per(a.health),
            swing: per(a.swing),
            forget: per(a.forget),
            groan: a.groan,
            blood: a.blood,
            fever,
            rises: a.rises,
            douses: a.douses,
        });
    }
    Ok(out)
}

/// A region's rares: how many a patch holds, how many affixes each, how far from where you start
/// at the least, and the loot table their packs are filled from.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaresFile {
    pub count: u32,
    pub affixes: (u32, u32),
    pub far: i32,
    pub loot: String,
}

impl RaresFile {
    pub(crate) fn problem(&self, loot: &[String]) -> Option<String> {
        if self.count > 20
            || self.affixes.0 == 0
            || self.affixes.0 > self.affixes.1
            || self.affixes.1 > 3
        {
            return Some("rares: count 0 to 20, affixes [fewest, most] from 1 to 3".to_string());
        }
        if self.far < 0 {
            return Some("rares.far can't be negative".to_string());
        }
        (!loot.contains(&self.loot))
            .then(|| format!("rares.loot: no loot table called {:?}", self.loot))
    }
}
