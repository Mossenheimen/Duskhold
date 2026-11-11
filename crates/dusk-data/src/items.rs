//! Items and loot tables (`data/items/*.toml`, `data/loot/*.toml`), the containers that use them
//! (`fixtures.toml`), and the start kit (`rules.toml`), turned into the core's `Defs`.

use std::path::Path;

use dusk_core::items::NEW;
use dusk_core::world::Swing;
use dusk_core::{Defs, Item, ItemDef, ItemKind, LootEntry, LootTable, Skill};
use serde::Deserialize;

use crate::{DataError, Fixture, chance, check_unique, fail, points, read, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemsFile {
    item: Vec<ItemFile>,
}

fn one() -> u16 {
    1
}

fn one_each() -> (u16, u16) {
    (1, 1)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemFile {
    id: String,
    name: String,
    /// Kilograms, for one.
    weight: f64,
    #[serde(default = "one")]
    stack: u16,
    #[serde(default)]
    weapon: Option<WeaponFile>,
    #[serde(default)]
    bandage: Option<BandageFile>,
    #[serde(default)]
    food: Option<FoodFile>,
    #[serde(default)]
    drink: Option<DrinkFile>,
    #[serde(default)]
    light: Option<LightFile>,
    #[serde(default)]
    cure: Option<CureFile>,
    /// XP one brings home the first time (`rules.toml`, [deeds]); 0 if left out.
    #[serde(default)]
    worth: f64,
}

/// A weapon's swing (as rules.toml's [attack]), the condition, out of 100, a blow costs it, and
/// the skill it trains and uses.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeaponFile {
    wind_up: f64,
    recovery: f64,
    endurance: f64,
    hit: f64,
    damage: (f64, f64),
    noise: i32,
    wear: f64,
    skill: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BandageFile {
    time: f64,
}

/// The cure for ghoul fever: seconds to take it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CureFile {
    time: f64,
}

/// Food: the hunger it takes away (points, out of 100) and seconds to eat it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FoodFile {
    nutrition: f64,
    time: f64,
}

/// Drink: the thirst it takes away and seconds to drink it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DrinkFile {
    water: f64,
    time: f64,
}

/// A light: the tiles it lights round it, and the game hours it burns.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LightFile {
    radius: i32,
    burn: f64,
}

/// A light's burn time as the condition it loses a game minute (out of `NEW`): at least one, so
/// a light burns 1000 game minutes at most.
fn light(file: &Path, id: &str, l: &LightFile) -> Result<ItemKind, DataError> {
    if !(1..=20).contains(&l.radius) || !(l.burn > 0.0 && l.burn <= 16.0) {
        return Err(fail(
            file,
            format!(
                "item {id:?}: a light's radius is 1 to 20 tiles, and it burns above 0 and at \
                 most 16 game hours"
            ),
        ));
    }
    let wane = (f64::from(NEW) / (l.burn * 60.0)).round().max(1.0) as u16;
    Ok(ItemKind::Light {
        radius: l.radius,
        wane,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LootsFile {
    loot: Vec<LootFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LootFile {
    id: String,
    rolls: (u32, u32),
    items: Vec<LootLine>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LootLine {
    item: String,
    weight: u32,
    #[serde(default = "one_each")]
    count: (u16, u16),
}

/// rules.toml's [items]: seconds each item action takes.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ItemTimesFile {
    take: f64,
    per_kg: f64,
    drop: f64,
    wield: f64,
    unwield: f64,
    light: f64,
    douse: f64,
}

/// What a character starts with and can carry: rules.toml's [character].
pub(crate) struct Kit<'a> {
    pub carry: f64,
    pub wield: Option<&'a str>,
    pub kit: &'a [String],
}

/// The `.toml` files in a folder that may not be there (a data set with no items).
fn files_maybe(dir: &Path) -> Result<Vec<std::path::PathBuf>, DataError> {
    if dir.is_dir() {
        crate::files_in(dir)
    } else {
        Ok(Vec::new())
    }
}

fn weapon(file: &Path, id: &str, w: &WeaponFile) -> Result<ItemKind, DataError> {
    let what = |part: &str| format!("item {id:?}: weapon.{part}");
    let wind_up = ticks(file, &what("wind_up"), w.wind_up)? as u32;
    if wind_up == 0 || w.damage.0 < 0.0 || w.damage.0 > w.damage.1 || w.noise < 0 {
        return Err(fail(
            file,
            format!(
                "item {id:?}: a weapon's wind-up must be at least a tick (0.1 s), its damage \
                 [least, most], its noise 0 or more"
            ),
        ));
    }
    if !(0.0..=100.0).contains(&w.wear) || w.endurance < 0.0 {
        return Err(fail(
            file,
            format!("item {id:?}: wear is out of 100, and endurance can't be negative"),
        ));
    }
    let Some(skill) = Skill::from_id(&w.skill).filter(|s| s.is_weapon()) else {
        let skills: Vec<&str> = Skill::ALL
            .iter()
            .filter(|s| s.is_weapon())
            .map(|s| s.id())
            .collect();
        return Err(fail(
            file,
            format!(
                "item {id:?}: weapon.skill {:?} isn't a weapon skill ({})",
                w.skill,
                skills.join(", ")
            ),
        ));
    };
    Ok(ItemKind::Weapon {
        swing: Swing {
            wind_up,
            recovery: ticks(file, &what("recovery"), w.recovery)? as u32,
            endurance: points(w.endurance),
            hit: chance(file, &what("hit"), w.hit)?,
            damage: (points(w.damage.0), points(w.damage.1)),
            noise: w.noise,
        },
        wear: (w.wear * 10.0).round() as u16,
        skill: Some(skill),
    })
}

/// Food or drink: what it takes off its need (points out of 100, as thousandths) and the ticks
/// it takes.
fn eaten(file: &Path, id: &str, what: &str, off: f64, time: f64) -> Result<(i32, u32), DataError> {
    if !(off > 0.0 && off <= 100.0) {
        return Err(fail(
            file,
            format!("item {id:?}: what {what} takes away is above 0 and at most 100"),
        ));
    }
    let t = ticks(file, &format!("item {id:?}: {what}.time"), time)?;
    Ok(((off * 1000.0).round() as i32, (t as u32).max(1)))
}

/// Loads the items and loot tables and puts the `Defs` together.
pub(crate) fn load_defs(
    dir: &Path,
    rules_file: &Path,
    kit: Kit,
    times: &ItemTimesFile,
    fixtures: &[Fixture],
) -> Result<Defs, DataError> {
    let mut items: Vec<ItemDef> = Vec::new();
    for file in files_maybe(&dir.join("items"))? {
        for i in read::<ItemsFile>(&file)?.item {
            let kinds = [
                i.weapon.is_some(),
                i.bandage.is_some(),
                i.food.is_some(),
                i.drink.is_some(),
                i.light.is_some(),
                i.cure.is_some(),
            ];
            if kinds.iter().filter(|&&k| k).count() > 1 {
                return Err(fail(
                    &file,
                    format!(
                        "item {:?} is more than one of weapon, bandage, food, drink, light and cure",
                        i.id
                    ),
                ));
            }
            if !(i.weight > 0.0 && i.weight < 1000.0)
                || i.stack == 0
                || !(0.0..1e6).contains(&i.worth)
            {
                return Err(fail(
                    &file,
                    format!(
                        "item {:?}: its weight must be above 0 kg, its stack at least 1, its \
                         worth 0 or more",
                        i.id
                    ),
                ));
            }
            let kind = if let Some(w) = &i.weapon {
                weapon(&file, &i.id, w)?
            } else if let Some(b) = &i.bandage {
                let time = ticks(&file, &format!("item {:?}: bandage.time", i.id), b.time)?;
                ItemKind::Bandage {
                    time: (time as u32).max(1),
                }
            } else if let Some(f) = &i.food {
                let (nutrition, time) = eaten(&file, &i.id, "food", f.nutrition, f.time)?;
                ItemKind::Food { nutrition, time }
            } else if let Some(d) = &i.drink {
                let (water, time) = eaten(&file, &i.id, "drink", d.water, d.time)?;
                ItemKind::Drink { water, time }
            } else if let Some(l) = &i.light {
                light(&file, &i.id, l)?
            } else if let Some(c) = &i.cure {
                let time = ticks(&file, &format!("item {:?}: cure.time", i.id), c.time)?;
                ItemKind::Cure {
                    time: (time as u32).max(1),
                }
            } else {
                ItemKind::Other
            };
            items.push(ItemDef {
                weight: (i.weight * 1000.0).round() as u32,
                id: i.id,
                name: i.name,
                stack: i.stack,
                worth: (i.worth * 1000.0).round() as u32,
                kind,
            });
        }
    }
    check_unique(&dir.join("items"), "item", items.iter().map(|i| &i.id))?;
    let item = |file: &Path, name: &str| -> Result<u16, DataError> {
        items
            .iter()
            .position(|i| i.id == name)
            .map(|n| n as u16)
            .ok_or_else(|| fail(file, format!("no item called {name:?} in items/")))
    };

    let mut loot: Vec<LootTable> = Vec::new();
    for file in files_maybe(&dir.join("loot"))? {
        for t in read::<LootsFile>(&file)?.loot {
            if t.rolls.0 > t.rolls.1 || t.items.iter().all(|l| l.weight == 0) {
                return Err(fail(
                    &file,
                    format!(
                        "loot {:?}: rolls are [fewest, most], and some line needs a weight",
                        t.id
                    ),
                ));
            }
            let mut entries = Vec::new();
            for line in &t.items {
                if line.count.0 > line.count.1 {
                    return Err(fail(
                        &file,
                        format!("loot {:?}: count is [fewest, most]", t.id),
                    ));
                }
                entries.push(LootEntry {
                    item: item(&file, &line.item)?,
                    weight: line.weight,
                    count: line.count,
                });
            }
            loot.push(LootTable {
                id: t.id,
                rolls: t.rolls,
                entries,
            });
        }
    }
    check_unique(&dir.join("loot"), "loot table", loot.iter().map(|t| &t.id))?;

    let fixtures_file = dir.join("fixtures.toml");
    let mut containers = vec![None];
    for f in fixtures {
        containers.push(match &f.loot {
            None => None,
            Some(name) => Some(
                loot.iter()
                    .position(|t| &t.id == name)
                    .map(|n| n as u16)
                    .ok_or_else(|| {
                        fail(
                            &fixtures_file,
                            format!("fixture {:?}: no loot table called {name:?}", f.id),
                        )
                    })?,
            ),
        });
    }

    let wield = match kit.wield {
        None => None,
        Some(name) => {
            let n = item(rules_file, name)?;
            if !matches!(items[usize::from(n)].kind, ItemKind::Weapon { .. }) {
                return Err(fail(
                    rules_file,
                    format!("character.wield: {name:?} isn't a weapon"),
                ));
            }
            Some(n)
        }
    };
    let mut defs = Defs {
        // Filled in by the loader, from creatures/.
        creatures: Vec::new(),
        items: Vec::new(),
        loot,
        containers,
        beds: std::iter::once(false)
            .chain(fixtures.iter().map(|f| f.bed))
            .collect(),
        wield,
        kit: Vec::new(),
        carry: (kit.carry * 1000.0).round().max(0.0) as u32,
        take: ticks(rules_file, "items.take", times.take)? as u32,
        take_per_kg: ticks(rules_file, "items.per_kg", times.per_kg)? as u32,
        drop: ticks(rules_file, "items.drop", times.drop)? as u32,
        wield_time: (ticks(rules_file, "items.wield", times.wield)? as u32).max(1),
        unwield_time: (ticks(rules_file, "items.unwield", times.unwield)? as u32).max(1),
        light_time: (ticks(rules_file, "items.light", times.light)? as u32).max(1),
        douse_time: (ticks(rules_file, "items.douse", times.douse)? as u32).max(1),
        // Filled in by the loader, from rules.toml's [barricade].
        boarding: None,
    };
    let mut start = Vec::new();
    for name in kit.kit {
        start.push(Item::new(item(rules_file, name)?, 1));
    }
    defs.items = items;
    for i in start {
        let mut kit = std::mem::take(&mut defs.kit);
        defs.stow(&mut kit, i);
        defs.kit = kit;
    }
    Ok(defs)
}
