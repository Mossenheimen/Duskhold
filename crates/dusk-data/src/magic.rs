//! Magic items: their properties and curses (`data/properties.toml`) in a designer's units
//! (multipliers, a chance added, tiles), the uniques (`data/uniques.toml`: things made by hand),
//! rules.toml's `[magic]` (how they're rolled and learnt), and its `[quality]` (how well a weapon
//! found was made).

use std::path::Path;

use dusk_core::quality::{CRUDE, FINE, MASTERWORK, PLAIN};
use dusk_core::{Defs, Fits, MagicRules, Property, QualityRules, Raises, Skill, Unique};
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
    /// "weapon" (in hand), "light" (held up), "armour" (worn), "bow" (in hand) or "shield" (on
    /// the arm); a unique's effect is a weapon's, and needn't say.
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
    /// What a unique's effect does. A weapon's or a bow's: a kill carries on into the next one
    /// beside you (a bow's arrow, on along its line); the share of a blow's harm that comes back
    /// as health (0 to 1); the dead it puts down burn; the chance a blow sends the dead fleeing,
    /// and knocks down (0 to 1: a shield's, a blow it blocks); and the dread a kill eases, in
    /// points.
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
    /// Armour's, worn: a chance added to turn every kind of blow (0 to 0.5), warmth added (in
    /// points), and tiles added to every footstep (a curse's: mail that clanks).
    #[serde(default)]
    ward: f64,
    #[serde(default)]
    warmth: f64,
    #[serde(default)]
    steps: i32,
    /// A bow's: tiles added to its range (-5 to 10), and how often its arrows break on a hit, as
    /// a multiplier.
    #[serde(default)]
    range: i32,
    #[serde(default = "one")]
    arrows: f64,
    /// A shield's: a chance added to its block, and to a bash's knocking down (0 to 0.5).
    #[serde(default)]
    block: f64,
    #[serde(default)]
    bash: f64,
    /// A unique's effect: harm back, [least, most] in points, on whatever's blow lands on you
    /// (armour's) or whatever's blow you block (a shield's); and how fast your dread rises, as a
    /// multiplier (armour's).
    #[serde(default)]
    thorns: (f64, f64),
    #[serde(default = "one")]
    calm: f64,
}

/// `uniques.toml`: weapons, armour, bows and shields made by hand, each with a name of its own.
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
    /// The kind of thing it is (items/: a weapon, armour, a bow or a shield), and how well it was
    /// made: "crude", "plain", "fine" or "masterwork" (left out: plain).
    item: String,
    #[serde(default)]
    grade: String,
    /// Properties from properties.toml, by id: its kind's; a curse's too, as a cost it carries.
    #[serde(default)]
    properties: Vec<String>,
    /// Its own effect, written as a property is (no id, and on what it's made on).
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
    /// How far a tiered property's numbers roll either way of its data's, as a share of each
    /// (0 to 0.5; left out, 0.2: Keen's +50% damage is +40% to +60%).
    #[serde(default = "spread")]
    spread: f64,
}

fn spread() -> f64 {
    0.2
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
    if !(0.0..=0.5).contains(&m.spread) {
        return Err(fail(file, "magic.spread is a share, from 0 to 0.5"));
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
        spread: (m.spread * 1000.0).round() as u32,
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

/// A grade: its word before a name ("" for plain), the chance a weapon or a piece of armour found
/// is of it (plain's is what's left, so it has none), multipliers on its harm, its wear and its
/// worth, and the chance added to armour's turning a blow (-0.5 to 0.5; left out, none).
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
    #[serde(default)]
    turn: f64,
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
        if !(-0.5..=0.5).contains(&g.turn) {
            return Err(bad("turn is a chance added, from -0.5 to 0.5"));
        }
        odds += g.chance;
        let per = |m: f64| (m * 1000.0).round() as u32;
        out.names[n] = g.name.trim().to_string();
        out.chance[n] = (g.chance * 10_000.0).round() as u32;
        out.damage[n] = per(g.damage);
        out.wear[n] = per(g.wear);
        out.worth[n] = per(g.worth);
        out.turn[n] = (g.turn * 10_000.0).round() as i32;
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
        let fits = fits_of(&p.on)
            .ok_or_else(|| bad("on is \"weapon\", \"light\", \"armour\", \"bow\" or \"shield\""))?;
        out.push(to_property(p, fits).map_err(|m| bad(&m))?);
    }
    Ok(out)
}

/// What a property's `on` names: a weapon, a light, armour, a bow or a shield.
fn fits_of(on: &str) -> Option<Fits> {
    Some(match on {
        "weapon" => Fits::Weapon,
        "light" => Fits::Light,
        "armour" => Fits::Armour,
        "bow" => Fits::Bow,
        "shield" => Fits::Shield,
        _ => return None,
    })
}

/// "a weapon, a bow or a shield": what a number goes on, in words.
fn kinds(on: &[Fits]) -> String {
    let words: Vec<&str> = on.iter().map(|f| f.word()).collect();
    match words.split_last() {
        Some((last, [])) => (*last).to_string(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
        None => "nothing".into(),
    }
}

/// A property (or a unique's effect) in the core's units, or what's wrong with it.
fn to_property(p: &PropertyFile, fits: Fits) -> Result<Property, String> {
    let multipliers = [
        p.noise,
        p.damage,
        p.swing,
        p.wear,
        p.wane,
        p.endurance,
        p.arrows,
        p.calm,
    ];
    if multipliers.iter().any(|m| !(0.0..=10.0).contains(m)) {
        return Err(
            "noise, damage, swing, wear, wane, endurance, arrows and calm are multipliers from 0 \
             to 10"
                .into(),
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
    if !harm(p.fire) || !harm(p.holy) || !harm(p.thorns) {
        return Err("fire, holy and thorns are [least, most], from 0 to 50 points".into());
    }
    if !(0.0..=0.5).contains(&p.ward) || !(-30.0..=30.0).contains(&p.warmth) {
        return Err("ward is a chance from 0 to 0.5, warmth -30 to 30 points".into());
    }
    if !(-5..=5).contains(&p.steps) || !(-5..=10).contains(&p.range) {
        return Err("steps are tiles, -5 to 5; range -5 to 10".into());
    }
    if !(0.0..=0.5).contains(&p.block) || !(0.0..=0.5).contains(&p.bash) {
        return Err("block and bash are chances from 0 to 0.5".into());
    }
    // What each number does, and what it goes on (Path of Exile's mods by item class): a blow's
    // is a weapon's, a shot's a bow's, a block's a shield's, the body and a skill anything worn
    // or held in a fight. A unique's effect is a blow's on a weapon, a shot's on a bow (an arrow
    // drinks, burns what it puts down, knocks down, flies on), a block's on a shield (harm back,
    // the dead put to flight or knocked down), and a blow's that lands on you on armour.
    use Fits::{Armour, Bow, Light, Shield, Weapon};
    let held: &[Fits] = &[Weapon, Armour, Bow, Shield];
    let numbers: [(&str, bool, &[Fits]); 29] = [
        ("noise", p.noise != 1.0, &[Weapon, Bow]),
        ("damage", p.damage != 1.0, &[Weapon, Bow]),
        ("swing", p.swing != 1.0, &[Weapon, Bow]),
        ("hit", p.hit != 0.0, &[Weapon, Bow]),
        ("endurance", p.endurance != 1.0, &[Weapon, Bow, Shield]),
        ("wear", p.wear != 1.0, &[Weapon, Armour, Shield]),
        ("hallowed", p.hallowed, &[Weapon]),
        ("glow", p.glow != 0, &[Weapon, Light]),
        ("wane", p.wane != 1.0, &[Light]),
        ("cleave", p.cleave, &[Weapon, Bow]),
        ("drain", p.drain > 0.0, &[Weapon, Bow]),
        ("pyre", p.pyre, &[Weapon, Bow]),
        ("terror", p.terror > 0.0, &[Weapon, Bow, Shield]),
        ("fell", p.fell > 0.0, &[Weapon, Bow, Shield]),
        ("hearten", p.hearten > 0.0, &[Weapon, Bow]),
        ("thorns", p.thorns.1 > 0.0, &[Armour, Shield]),
        ("calm", p.calm != 1.0, &[Armour]),
        ("strength", p.strength > 0.0, held),
        ("fitness", p.fitness > 0.0, held),
        ("skill", p.levels > 0, held),
        ("fire", p.fire.1 > 0.0, &[Weapon, Bow]),
        ("holy", p.holy.1 > 0.0, &[Weapon, Bow]),
        ("ward", p.ward > 0.0, &[Armour]),
        ("warmth", p.warmth != 0.0, &[Armour]),
        ("steps", p.steps != 0, &[Armour]),
        ("range", p.range != 0, &[Bow]),
        ("arrows", p.arrows != 1.0, &[Bow]),
        ("block", p.block > 0.0, &[Shield]),
        ("bash", p.bash > 0.0, &[Shield]),
    ];
    for (name, does, on) in numbers {
        if does && !on.contains(&fits) {
            let (on, not) = (kinds(on), fits.word());
            return Err(format!("{name} goes on {on}, not {not}"));
        }
    }
    // Armour trains no skill of its own: it names one.
    if fits == Armour && raises == Raises::Own {
        return Err("armour trains no skill of its own: skill names one".into());
    }
    if p.line.trim().is_empty() && !numbers.iter().any(|n| n.1) {
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
        fits,
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
        ward: chance(p.ward),
        warmth: (p.warmth * 1000.0).round() as i32,
        steps: p.steps,
        range: p.range,
        arrows: per(p.arrows),
        block: chance(p.block),
        bash: chance(p.bash),
        thorns: points(p.thorns),
        calm: per(p.calm),
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
        // Made on a weapon, a piece of armour, a bow or a shield: not a light (yet).
        let (item, kind) = (0..defs.items.len())
            .find_map(|n| {
                let kind = defs.fits(n as u16).filter(|&k| k != Fits::Light)?;
                (defs.items[n].id == u.item).then_some((n, kind))
            })
            .ok_or_else(|| {
                bad(&format!(
                    "{:?} is no weapon, armour, bow or shield in items/",
                    u.item
                ))
            })?;
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
                .position(|p| &p.id == id && p.fits == kind)
                .ok_or_else(|| {
                    bad(&format!(
                        "{id:?} is no property of {} in properties.toml",
                        kind.word()
                    ))
                })?;
            if props.contains(&(n as u8)) {
                return Err(bad(&format!("{id:?} twice")));
            }
            props.push(n as u8);
        }
        let e = &u.effect;
        if !(e.on.is_empty() || fits_of(&e.on) == Some(kind)) || e.curse {
            return Err(bad(&format!(
                "its effect goes on {}, as it's made, and is no curse",
                kind.word()
            )));
        }
        if e.name.trim().is_empty() || e.line.trim().is_empty() {
            return Err(bad("its effect needs a name and a line"));
        }
        let mut effect = to_property(e, kind).map_err(|m| bad(&format!("effect: {m}")))?;
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
