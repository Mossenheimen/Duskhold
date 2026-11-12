//! Foraging (`dusk_core::forage`): rules.toml's [foraging], the fixtures' `forage` (what's
//! picked from a wild plant), and a region's [plants] (where they grow), in a designer's units:
//! seconds, game days and chances.

use std::path::Path;

use dusk_core::{Defs, Foraging, Plant};
use serde::Deserialize;

use crate::{DataError, Fixture, Region, fail, ticks};

/// A wild plant's (fixtures.toml's `forage`): the item picked from it, how many (the fewest and
/// the most), and the game days before it grows back once it's picked bare.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForageFile {
    pub item: String,
    pub count: (u16, u16),
    pub regrow: f64,
}

/// rules.toml's [foraging]: seconds to pick a plant, and the fixture one becomes once it's
/// picked bare.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForagingFile {
    pick: f64,
    bare: String,
}

/// Foraging's rules, from rules.toml.
pub(crate) fn to_foraging(file: &Path, f: &ForagingFile) -> Result<Foraging, DataError> {
    Ok(Foraging {
        pick: (ticks(file, "foraging.pick", f.pick)? as u32).max(1),
    })
}

/// The wild plants, by fixture number, and the fixture a plant becomes once it's picked bare:
/// from the fixtures' `forage` and rules.toml's [foraging]. `ticks_per_minute` is the clock's,
/// for the days a plant takes to grow back.
pub(crate) fn load(
    rules_file: &Path,
    fixtures_file: &Path,
    foraging: Option<&ForagingFile>,
    fixtures: &[Fixture],
    ticks_per_minute: u32,
    defs: &mut Defs,
) -> Result<(), DataError> {
    let mut plants = vec![None];
    for f in fixtures {
        let Some(p) = &f.forage else {
            plants.push(None);
            continue;
        };
        let Some(item) = defs.items.iter().position(|i| i.id == p.item) else {
            return Err(fail(
                fixtures_file,
                format!(
                    "fixture {:?}: forage.item: no item called {:?} in items/",
                    f.id, p.item
                ),
            ));
        };
        if p.count.0 > p.count.1
            || !(1..=20).contains(&p.count.1)
            || !(p.regrow > 0.0 && p.regrow <= 60.0)
            || f.solid
            || f.opaque
            || f.loot.is_some()
        {
            return Err(fail(
                fixtures_file,
                format!(
                    "fixture {:?}: a plant is walked through and seen past, holds no loot, gives \
                     [fewest, most] (the most 1 to 20), and grows back in above 0 and at most 60 \
                     game days",
                    f.id
                ),
            ));
        }
        let minutes = (p.regrow * 24.0 * 60.0).round() as u64;
        plants.push(Some(Plant {
            item: item as u16,
            count: p.count,
            regrow: minutes * u64::from(ticks_per_minute.max(1)),
        }));
    }
    let any = plants.iter().any(Option::is_some);
    defs.plants = plants;
    match foraging {
        Some(f) => {
            let Some(n) = fixtures.iter().position(|x| x.id == f.bare) else {
                return Err(fail(
                    rules_file,
                    format!("foraging.bare: no fixture called {:?}", f.bare),
                ));
            };
            let bare = &fixtures[n];
            if bare.solid || bare.opaque || bare.forage.is_some() || bare.loot.is_some() {
                return Err(fail(
                    rules_file,
                    format!(
                        "foraging.bare: {:?} is walked through and seen past, and is neither a \
                         plant nor a container",
                        f.bare
                    ),
                ));
            }
            defs.bare = (n + 1) as u16;
        }
        None if any => {
            return Err(fail(
                rules_file,
                "some fixtures are plants (forage), so rules.toml needs [foraging]",
            ));
        }
        None => {}
    }
    Ok(())
}

/// A region's [plants]: each kind a plant (a fixture with `forage`), with a chance above 0 and
/// at most a half.
pub(crate) fn check_plants(
    file: &Path,
    region: &Region,
    fixtures: &[Fixture],
) -> Result<(), DataError> {
    let Some(p) = &region.plants else {
        return Ok(());
    };
    for k in &p.kinds {
        if !fixtures
            .iter()
            .any(|f| f.id == k.plant && f.forage.is_some())
        {
            return Err(fail(
                file,
                format!(
                    "plants: no plant (a fixture with forage) called {:?}",
                    k.plant
                ),
            ));
        }
        if !(k.chance > 0.0 && k.chance <= 0.5) {
            return Err(fail(
                file,
                format!("plants: {:?}'s chance is above 0 and at most 0.5", k.plant),
            ));
        }
    }
    Ok(())
}
