//! Crafting (`recipes.toml`, and rules.toml's [crafting] for breaking furniture down): items by
//! name as the core's numbers, seconds as ticks, XP as thousandths.

use std::collections::BTreeMap;
use std::path::Path;

use dusk_core::craft::{Breaking, Recipe};
use dusk_core::{Defs, Skill};
use serde::Deserialize;

use crate::{DataError, Fixture, check_unique, fail, read, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipesFile {
    recipe: Vec<RecipeFile>,
}

fn one() -> u16 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    id: String,
    /// The item it makes, and how many.
    makes: String,
    #[serde(default = "one")]
    count: u16,
    /// What it uses up: items by name, and how many of each.
    uses: BTreeMap<String, u16>,
    /// Any one of these to hand, kept (none: it wants no tool).
    #[serde(default)]
    tools: Vec<String>,
    /// Cooked: it wants a lit fire beside you.
    #[serde(default)]
    fire: bool,
    /// Seconds it takes.
    time: f64,
    /// The skill it teaches, and the XP it gives.
    #[serde(default)]
    skill: Option<String>,
    #[serde(default)]
    xp: f64,
}

/// rules.toml's [crafting]: breaking furniture down.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CraftingFile {
    /// The item a broken-down piece gives (fixtures.toml's planks: how many).
    plank: String,
    /// Any one of these to hand does it.
    tools: Vec<String>,
    /// Seconds it takes, how far it's heard (tiles), and Carpentry's XP for it.
    time: f64,
    noise: i32,
    xp: f64,
}

/// An item's number by its name.
fn item(file: &Path, defs: &Defs, what: &str, name: &str) -> Result<u16, DataError> {
    defs.items
        .iter()
        .position(|i| i.id == name)
        .map(|n| n as u16)
        .ok_or_else(|| fail(file, format!("{what}: no item called {name:?}")))
}

/// Loads the recipes (`recipes.toml`, if there is one), what breaking furniture down takes, and
/// how many planks each fixture gives, into the core's `Defs`.
pub(crate) fn load(
    dir: &Path,
    rules_file: &Path,
    crafting: Option<&CraftingFile>,
    fixtures: &[Fixture],
    defs: &mut Defs,
) -> Result<(), DataError> {
    if let Some(f) = fixtures.iter().find(|f| f.planks > 0 && f.loot.is_some()) {
        return Err(fail(
            &dir.join("fixtures.toml"),
            format!(
                "fixture {:?}: a container doesn't break down (planks)",
                f.id
            ),
        ));
    }
    defs.planks = std::iter::once(0)
        .chain(fixtures.iter().map(|f| f.planks))
        .collect();
    if let Some(c) = crafting {
        if c.tools.is_empty() || c.noise < 0 || c.xp < 0.0 {
            return Err(fail(
                rules_file,
                "crafting: name at least one tool; noise and xp are 0 or more",
            ));
        }
        defs.breaking = Some(Breaking {
            plank: item(rules_file, defs, "crafting.plank", &c.plank)?,
            tools: c
                .tools
                .iter()
                .map(|t| item(rules_file, defs, "crafting.tools", t))
                .collect::<Result<_, _>>()?,
            time: (ticks(rules_file, "crafting.time", c.time)? as u32).max(1),
            noise: c.noise,
            xp: (c.xp * 1000.0).round() as u32,
        });
    }
    let file = dir.join("recipes.toml");
    if !file.is_file() {
        return Ok(());
    }
    let recipes = read::<RecipesFile>(&file)?.recipe;
    check_unique(&file, "recipe", recipes.iter().map(|r| &r.id))?;
    for r in &recipes {
        let what = format!("recipe {:?}", r.id);
        if r.uses.is_empty() || r.count == 0 || r.uses.values().any(|&n| n == 0) || r.xp < 0.0 {
            return Err(fail(
                &file,
                format!(
                    "{what}: it uses something, at least one of each, makes at least one, and its xp is 0 or more"
                ),
            ));
        }
        let skill = match &r.skill {
            None => None,
            Some(s) => Some(
                Skill::from_id(s)
                    .ok_or_else(|| fail(&file, format!("{what}: no skill called {s:?}")))?,
            ),
        };
        let recipe = Recipe {
            id: r.id.clone(),
            makes: (item(&file, defs, &what, &r.makes)?, r.count),
            uses: r
                .uses
                .iter()
                .map(|(name, &n)| Ok((item(&file, defs, &what, name)?, n)))
                .collect::<Result<_, DataError>>()?,
            tools: r
                .tools
                .iter()
                .map(|t| item(&file, defs, &what, t))
                .collect::<Result<_, _>>()?,
            fire: r.fire,
            time: (ticks(&file, &format!("{what}: time"), r.time)? as u32).max(1),
            skill,
            xp: (r.xp * 1000.0).round() as u32,
        };
        defs.recipes.push(recipe);
    }
    Ok(())
}
