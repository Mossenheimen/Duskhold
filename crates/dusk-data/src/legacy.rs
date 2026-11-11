//! Death and legacy: characters' names (`data/names.toml`) and what an unburned body rises as
//! (rules.toml's `[revenant]`: a creature, and the hour).

use std::path::Path;

use dusk_core::Defs;
use serde::Deserialize;

use crate::{DataError, fail, read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NamesFile {
    given: Vec<String>,
    family: Vec<String>,
    /// Rare monsters' names (`affixes.toml` adds the epithet).
    #[serde(default)]
    rare: Vec<String>,
}

/// rules.toml's `[revenant]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RevenantFile {
    /// The creature (`creatures/*.toml`) a character's body rises as.
    creature: String,
    /// The hour it rises, the night after the death (0.0 is midnight).
    hour: f64,
}

/// Reads `names.toml` (left out: characters go nameless) and the revenant, into the definitions.
pub(crate) fn load(
    dir: &Path,
    rules_file: &Path,
    revenant: Option<&RevenantFile>,
    defs: &mut Defs,
) -> Result<(), DataError> {
    let file = dir.join("names.toml");
    if file.exists() {
        let names: NamesFile = read(&file)?;
        let long = |list: &[String]| list.len() >= usize::from(u16::MAX);
        if names.given.is_empty() || long(&names.given) || long(&names.family) {
            return Err(fail(&file, "given names: at least one (and sensibly many)"));
        }
        if names.rare.len() >= usize::from(u16::MAX) {
            return Err(fail(&file, "rare names: sensibly many"));
        }
        defs.given = names.given;
        defs.family = names.family;
        defs.rare_names = names.rare;
    }
    if let Some(r) = revenant {
        let Some(dusk_core::ActorKind::Creature(n)) = defs.creature_kind(&r.creature) else {
            return Err(fail(
                rules_file,
                format!("revenant.creature: no creature called {:?}", r.creature),
            ));
        };
        if !(0.0..24.0).contains(&r.hour) {
            return Err(fail(rules_file, "revenant.hour is 0 to 24"));
        }
        defs.revenant = Some((n, (r.hour * 60.0).round() as u32));
    }
    Ok(())
}
