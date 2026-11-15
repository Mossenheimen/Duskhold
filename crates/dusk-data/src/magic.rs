//! Magic items: their properties and curses (`data/properties.toml`) in a designer's units
//! (multipliers, a chance added, tiles), the uniques (`data/uniques.toml`: weapons made by hand),
//! rules.toml's `[magic]` (how they're rolled and learnt), and its `[quality]` (how well a weapon
//! found was made).

use std::path::Path;

use dusk_core::quality::{CRUDE, FINE, MASTERWORK, PLAIN};
use dusk_core::{Defs, ItemKind, MagicRules, Property, QualityRules, Raises, Skill, Unique};
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
    /// A unique's effect has none: it's the unique's.
    #[serde(default)]
    id: String,
    /// The word before an item's name, and its line on the card, in words: for a property that's
    /// more than numbers (left out, the card says its numbers: "+80% damage").
    name: String,
    #[serde(default)]
    line: String,
    /// "weapon" (in hand) or "light" (held up); a unique's effect is a weapon's, and needn't say.
    #[serde(default)]
    on: String,
    /// Its family (left out: its own), and its tier in it, 1 to 3 (left out: none, found
    /// anywhere): a thing has at most one of a family, and a find rolls a tier only as far out as
    /// it reaches.
    #[serde(default)]
    family: String,
    #[serde(default)]
    tier: u8,
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
    /// A weapon's: the breath a blow costs, as a multiplier.
    #[serde(default = "one")]
    endurance: f64,
    /// What it puts down stays down.
    #[serde(default)]
    hallowed: bool,
    /// Tiles of light: round you, in hand; added to a light's reach.
    #[serde(default)]
    glow: i32,
    /// A light's: how fast it burns down, as a multiplier.
    #[serde(default = "one")]
    wane: f64,
    /// A weapon's, what a unique's effect does: a kill carries on into the next one beside you;
    /// the share of a blow's harm that comes back as health (0 to 1); the dead it puts down
    /// burn; the chance a blow sends the dead fleeing, and knocks down (0 to 1); and the dread a
    /// kill eases, in points.
    #[serde(default)]
    cleave: bool,
    #[serde(default)]
    drain: f64,
    #[serde(default)]
    pyre: bool,
    #[serde(default)]
    terror: f64,
    #[serde(default)]
    fell: f64,
    #[serde(default)]
    hearten: f64,
    /// A weapon's, in hand: the levels it adds to Strength and to Fitness (0 to 5), and to a
    /// skill: `skill` "own" (the one its weapon trains) or a skill's id, and `levels` (1 to 5).
    #[serde(default)]
    strength: f64,
    #[serde(default)]
    fitness: f64,
    #[serde(default)]
    skill: String,
    #[serde(default)]
    levels: u8,
    /// A weapon's: the harm its blows add, [least, most] in points: fire's on anything, holy
    /// harm on the dead only.
    #[serde(default)]
    fire: (f64, f64),
    #[serde(default)]
    holy: (f64, f64),
}

/// `uniques.toml`: weapons made by hand, each with a name of its own.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniquesFile {
    unique: Vec<UniqueFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UniqueFile {
    id: String,
    name: String,
    /// The kind of weapon it is (items/), and how well it was made: "crude", "plain", "fine" or
    /// "masterwork" (left out: plain).
    item: String,
    #[serde(default)]
    grade: String,
    /// Properties from properties.toml, by id: a weapon's; a curse's too, as a cost it carries.
    #[serde(default)]
    properties: Vec<String>,
    /// Its own effect, written as a property is (no id, and on a weapon).
    effect: PropertyFile,
    /// A line of its story.
    #[serde(default)]
    lore: String,
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
    /// The XP a magic item adds to its kind's worth: Uncommon, Rare, Unique.
    worth: (f64, f64, f64),
    /// Blows on a foe with an unknown magic weapon before you know what it is.
    learn: u8,
    /// How many properties: an Uncommon's fewest and most, and a Rare's.
    count: [(u8, u8); 2],
    /// Tiles from where you come in at which finds reach the second tier, and the third.
    far: (u32, u32),
}

pub(crate) fn to_rules(file: &Path, m: &MagicFile) -> Result<MagicRules, DataError> {
    let unit = |x: f64| (0.0..=1.0).contains(&x);
    if !unit(m.rare) || !unit(m.cursed) || m.learn == 0 {
        return Err(fail(
            file,
            "magic: rare and cursed are chances from 0 to 1, learn at least 1 blow",
        ));
    }
    let counts = |(fewest, most): (u8, u8)| fewest >= 1 && fewest <= most && most <= 4;
    if !counts(m.count[0]) || !counts(m.count[1]) {
        return Err(fail(
            file,
            "magic.count is [[fewest, most], [fewest, most]]: an Uncommon's, a Rare's, 1 to 4",
        ));
    }
    if m.far.0 > m.far.1 {
        return Err(fail(
            file,
            "magic.far is [second tier, third]: nearer first",
        ));
    }
    let xp = |x: f64| (0.0..=10_000.0).contains(&x);
    if !xp(m.worth.0) || !xp(m.worth.1) || !xp(m.worth.2) {
        return Err(fail(
            file,
            "magic.worth is [Uncommon, Rare, Unique], 0 to 10,000 XP",
        ));
    }
    Ok(MagicRules {
        rare: (m.rare * 10_000.0).round() as u32,
        cursed: (m.cursed * 10_000.0).round() as u32,
        worth: [
            (m.worth.0 * 1000.0).round() as u32,
            (m.worth.1 * 1000.0).round() as u32,
            (m.worth.2 * 1000.0).round() as u32,
        ],
        learn: m.learn,
        count: m.count,
        far: [m.far.0, m.far.1],
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
        if p.id.trim().is_empty() {
            return Err(fail(&file, "a property needs an id"));
        }
        let on_light = match p.on.as_str() {
            "weapon" => false,
            "light" => true,
            _ => return Err(bad("on is \"weapon\" or \"light\"")),
        };
        out.push(to_property(p, on_light).map_err(|m| bad(&m))?);
    }
    Ok(out)
}

/// A property (or a unique's effect) in the core's units, or what's wrong with it.
fn to_property(p: &PropertyFile, on_light: bool) -> Result<Property, String> {
    let multipliers = [p.noise, p.damage, p.swing, p.wear, p.wane, p.endurance];
    if multipliers.iter().any(|m| !(0.0..=10.0).contains(m)) {
        return Err(
            "noise, damage, swing, wear, wane and endurance are multipliers from 0 to 10".into(),
        );
    }
    if p.tier > 3 {
        return Err("tier is 1 to 3 (left out: none)".into());
    }
    if !(-1.0..=1.0).contains(&p.hit) || !(0..=12).contains(&p.glow) {
        return Err("hit is a chance from -1 to 1, glow 0 to 12 tiles".into());
    }
    let unit = |x: f64| (0.0..=1.0).contains(&x);
    if !unit(p.drain) || !unit(p.terror) || !unit(p.fell) || !(0.0..=100.0).contains(&p.hearten) {
        return Err("drain, terror and fell are from 0 to 1, hearten 0 to 100 points".into());
    }
    let effect = p.cleave || p.pyre || p.drain > 0.0 || p.terror > 0.0 || p.fell > 0.0;
    if on_light && (effect || p.hearten > 0.0) {
        return Err("cleave, drain, pyre, terror, fell and hearten are a weapon's".into());
    }
    let level = |x: f64| (0.0..=5.0).contains(&x);
    if !level(p.strength) || !level(p.fitness) {
        return Err("strength and fitness are levels, from 0 to 5".into());
    }
    let raises = match p.skill.as_str() {
        "" if p.levels == 0 => Raises::Nothing,
        "" => return Err("levels go to a skill: skill = \"own\", or a skill's id".into()),
        _ if !(1..=5).contains(&p.levels) => {
            return Err("a skill's levels are 1 to 5".into());
        }
        "own" => Raises::Own,
        id => Raises::Skill(
            Skill::from_id(id)
                .ok_or_else(|| format!("skill {id:?} is \"own\", or a skill's id"))?,
        ),
    };
    let harm = |(lo, hi): (f64, f64)| 0.0 <= lo && lo <= hi && hi <= 50.0;
    if !harm(p.fire) || !harm(p.holy) {
        return Err("fire and holy are [least, most], from 0 to 50 points".into());
    }
    let body = p.strength > 0.0 || p.fitness > 0.0 || p.levels > 0;
    if on_light && (body || p.fire.1 > 0.0 || p.holy.1 > 0.0) {
        return Err("strength, fitness, skill, fire and holy are a weapon's".into());
    }
    let numbers = [p.noise, p.damage, p.swing, p.wear, p.wane, p.endurance]
        .iter()
        .any(|&m| m != 1.0)
        || p.hit != 0.0
        || p.glow != 0
        || p.hallowed
        || effect
        || p.hearten > 0.0
        || body
        || p.fire.1 > 0.0
        || p.holy.1 > 0.0;
    if p.line.trim().is_empty() && !numbers {
        return Err("a property does something: numbers, or a line in words".into());
    }
    let per = |m: f64| (m * 1000.0).round() as u32;
    let chance = |x: f64| (x * 10_000.0).round() as u32;
    let points = |(lo, hi): (f64, f64)| ((lo * 10.0).round() as i32, (hi * 10.0).round() as i32);
    Ok(Property {
        id: p.id.clone(),
        name: p.name.clone(),
        line: p.line.trim().to_string(),
        family: p.family.trim().to_string(),
        tier: p.tier,
        on_light,
        curse: p.curse,
        noise: per(p.noise),
        damage: per(p.damage),
        swing: per(p.swing),
        hit: (p.hit * 10_000.0).round() as i32,
        wear: per(p.wear),
        endurance: per(p.endurance),
        hallowed: p.hallowed,
        glow: p.glow,
        wane: per(p.wane),
        cleave: p.cleave,
        drain: per(p.drain),
        pyre: p.pyre,
        terror: chance(p.terror),
        hearten: (p.hearten * 1000.0).round() as i32,
        fell: chance(p.fell),
        strength: per(p.strength),
        fitness: per(p.fitness),
        raises,
        levels: p.levels,
        fire: points(p.fire),
        holy: points(p.holy),
    })
}

/// Every unique in `uniques.toml` (left out: none), made on the items and properties loaded.
pub(crate) fn uniques(dir: &Path, defs: &Defs) -> Result<Vec<Unique>, DataError> {
    let file = dir.join("uniques.toml");
    if !file.exists() {
        return Ok(Vec::new());
    }
    let parsed: UniquesFile = read(&file)?;
    check_unique(&file, "unique", parsed.unique.iter().map(|u| &u.id))?;
    if parsed.unique.len() > 250 {
        return Err(fail(&file, "at most 250 uniques"));
    }
    let mut out = Vec::new();
    for u in &parsed.unique {
        let bad = |m: &str| fail(&file, format!("unique {:?}: {m}", u.id));
        if u.name.trim().is_empty() {
            return Err(bad("a name, its own"));
        }
        let item = defs
            .items
            .iter()
            .position(|d| d.id == u.item && matches!(d.kind, ItemKind::Weapon { .. }))
            .ok_or_else(|| bad(&format!("{:?} is no weapon in items/", u.item)))?;
        let quality = match u.grade.as_str() {
            "crude" => CRUDE,
            "" | "plain" => PLAIN,
            "fine" => FINE,
            "masterwork" => MASTERWORK,
            _ => return Err(bad("grade is crude, plain, fine or masterwork")),
        };
        let mut props = Vec::new();
        for id in &u.properties {
            let n = defs
                .properties
                .iter()
                .position(|p| &p.id == id && !p.on_light)
                .ok_or_else(|| {
                    bad(&format!(
                        "{id:?} is no weapon's property in properties.toml"
                    ))
                })?;
            if props.contains(&(n as u8)) {
                return Err(bad(&format!("{id:?} twice")));
            }
            props.push(n as u8);
        }
        let e = &u.effect;
        if !matches!(e.on.as_str(), "" | "weapon") || e.curse {
            return Err(bad("its effect is a weapon's, and no curse"));
        }
        if e.name.trim().is_empty() || e.line.trim().is_empty() {
            return Err(bad("its effect needs a name and a line"));
        }
        let mut effect = to_property(e, false).map_err(|m| bad(&format!("effect: {m}")))?;
        effect.id = u.id.clone();
        out.push(Unique {
            id: u.id.clone(),
            name: u.name.trim().to_string(),
            item: item as u16,
            quality,
            props,
            effect,
            lore: u.lore.trim().to_string(),
        });
    }
    Ok(out)
}
