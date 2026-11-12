//! Classes (`data/classes.toml`) and their talents (`data/talents.toml`), in a designer's units:
//! multipliers, points of health and faith, chances added, skills and items by id.

use std::path::Path;

use dusk_core::skills::Skill;
use dusk_core::{Class, Defs, Item, ItemKind, Kit, Talent};
use serde::Deserialize;

use crate::{DataError, check_unique, fail, points, read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassesFile {
    class: Vec<ClassFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassFile {
    id: String,
    name: String,
    /// What it is, in a line, for whoever chooses (left out: none).
    #[serde(default)]
    line: String,
    /// The skills it learns faster (by id), and how many times as fast.
    skills: Vec<String>,
    learn: f64,
    /// What its running, swings and struggles cost in endurance, as a multiplier.
    endurance: f64,
    /// The chance a creature's hit takes hold of it, as a multiplier.
    grabbed: f64,
    /// Its talents, by id (talents.toml).
    talents: Vec<String>,
    /// The faith it holds, in points (left out: none).
    #[serde(default)]
    faith: f64,
    /// It turns the dead, with faith (left out: it doesn't).
    #[serde(default)]
    turns: bool,
    /// What it starts with, by item id: in hand, and in its pack (name one twice for two).
    /// Left out, both: everyone's kit (rules.toml's [character]).
    #[serde(default)]
    wield: Option<String>,
    #[serde(default)]
    kit: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TalentsFile {
    talent: Vec<TalentFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TalentFile {
    id: String,
    name: String,
    line: String,
    /// Points of health it adds.
    #[serde(default)]
    health: f64,
    /// Multipliers, 1.0 for none: endurance costs; a blow's harm (with any weapon, or only
    /// `skill`'s); the chance a creature's hit takes hold of you.
    #[serde(default = "one")]
    endurance: f64,
    #[serde(default = "one")]
    damage: f64,
    #[serde(default)]
    skill: Option<String>,
    #[serde(default = "one")]
    grabbed: f64,
    /// Chances added, from -1 to 1: a blow landing, a shove knocking down, struggling free.
    #[serde(default)]
    hit: f64,
    #[serde(default)]
    knock_down: f64,
    #[serde(default)]
    struggle: f64,
    /// How many more must hold you to drag you down.
    #[serde(default)]
    hold: u32,
    /// Its hit and grabbed count only beside a door.
    #[serde(default)]
    doorway: bool,
    /// A blow that puts one down carries on into another beside you.
    #[serde(default)]
    cleave: bool,
    /// The Second Wind: once between sleeps, catch your breath.
    #[serde(default)]
    second_wind: bool,
    /// Points of faith it adds (rules.toml's [faith]).
    #[serde(default)]
    faith: f64,
    /// Mending: faith closes the worst bleeding wound. Warding: faith wards a shut door.
    #[serde(default)]
    mending: bool,
    #[serde(default)]
    warding: bool,
    /// Prayer at a shrine cures ghoul fever too.
    #[serde(default)]
    purify: bool,
    /// A multiplier, 1.0 for none: how fast dread rises while you hold a light up.
    #[serde(default = "one")]
    steady: f64,
    /// A multiplier on how long the dead you turn stay turned, and tiles added to its reach.
    #[serde(default = "one")]
    turn_time: f64,
    #[serde(default)]
    turn_reach: i32,
    /// The dead you turn with this many points of health or fewer fall instead.
    #[serde(default)]
    sunder: f64,
    /// The dead you put down stay down.
    #[serde(default)]
    last_rites: bool,
}

fn one() -> f64 {
    1.0
}

/// The talents in `talents.toml` and the classes in `classes.toml` (left out: none, and
/// characters have no class). The items are `defs`', for the classes' kits.
pub(crate) fn load(dir: &Path, defs: &Defs) -> Result<(Vec<Class>, Vec<Talent>), DataError> {
    let talents_file = dir.join("talents.toml");
    let classes_file = dir.join("classes.toml");
    let mut talents = Vec::new();
    if talents_file.exists() {
        let parsed: TalentsFile = read(&talents_file)?;
        check_unique(&talents_file, "talent", parsed.talent.iter().map(|t| &t.id))?;
        if parsed.talent.len() > 250 {
            return Err(fail(&talents_file, "at most 250 talents"));
        }
        for t in &parsed.talent {
            talents.push(to_talent(&talents_file, t)?);
        }
    }
    let mut classes = Vec::new();
    if classes_file.exists() {
        let parsed: ClassesFile = read(&classes_file)?;
        check_unique(&classes_file, "class", parsed.class.iter().map(|c| &c.id))?;
        if parsed.class.len() > 250 {
            return Err(fail(&classes_file, "at most 250 classes"));
        }
        for c in &parsed.class {
            classes.push(to_class(&classes_file, c, &talents, defs)?);
        }
    }
    Ok((classes, talents))
}

fn to_class(
    file: &Path,
    c: &ClassFile,
    talents: &[Talent],
    defs: &Defs,
) -> Result<Class, DataError> {
    let bad = |m: String| fail(file, format!("class {:?}: {m}", c.id));
    if !(1.0..=5.0).contains(&c.learn) || !(0.1..=2.0).contains(&c.endurance) {
        return Err(bad(
            "learn is 1 to 5 times, endurance a multiplier from 0.1 to 2".into(),
        ));
    }
    if !(0.0..=2.0).contains(&c.grabbed) {
        return Err(bad("grabbed is a multiplier from 0 to 2".into()));
    }
    if !(0.0..=500.0).contains(&c.faith) || (c.turns && c.faith <= 0.0) {
        return Err(bad(
            "faith is 0 to 500 points, and a class that turns the dead needs some".into(),
        ));
    }
    let skills = c
        .skills
        .iter()
        .map(|s| Skill::from_id(s).ok_or_else(|| bad(format!("no skill called {s:?}"))))
        .collect::<Result<Vec<_>, _>>()?;
    let picks = c
        .talents
        .iter()
        .map(|t| {
            talents
                .iter()
                .position(|x: &Talent| &x.id == t)
                .map(|n| n as u8)
                .ok_or_else(|| bad(format!("no talent called {t:?} in talents.toml")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if picks.len() < dusk_core::class::OFFERED {
        return Err(bad(format!(
            "needs at least {} talents, for a level's offer",
            dusk_core::class::OFFERED
        )));
    }
    // A talent of faith is no use to a class without it, nor one of turning to a class that
    // doesn't turn the dead.
    for &n in &picks {
        let t = &talents[usize::from(n)];
        if c.faith <= 0.0 && (t.faith > 0 || t.mending || t.warding) {
            return Err(bad(format!(
                "talent {:?} spends faith, and this class has none",
                t.id
            )));
        }
        if !c.turns && (t.turn_time != 1000 || t.turn_reach != 0 || t.sunder > 0) {
            return Err(bad(format!(
                "talent {:?} is for turning the dead, and this class doesn't",
                t.id
            )));
        }
    }
    let item = |name: &str| -> Result<u16, DataError> {
        defs.items
            .iter()
            .position(|i| i.id == name)
            .map(|n| n as u16)
            .ok_or_else(|| bad(format!("no item called {name:?} in items/")))
    };
    let kit = if c.wield.is_none() && c.kit.is_none() {
        None
    } else {
        let wield = match &c.wield {
            None => None,
            Some(name) => {
                let n = item(name)?;
                if !matches!(defs.items[usize::from(n)].kind, ItemKind::Weapon { .. }) {
                    return Err(bad(format!("wield: {name:?} isn't a weapon")));
                }
                Some(n)
            }
        };
        let mut pack = Vec::new();
        for name in c.kit.iter().flatten() {
            defs.stow(&mut pack, Item::new(item(name)?, 1));
        }
        Some(Kit { wield, pack })
    };
    let per = |m: f64| (m * 1000.0).round() as u32;
    Ok(Class {
        id: c.id.clone(),
        name: c.name.clone(),
        line: c.line.clone(),
        skills,
        learn: per(c.learn),
        endurance: per(c.endurance),
        grabbed: per(c.grabbed),
        talents: picks,
        faith: (c.faith * 1000.0).round() as i32,
        turns: c.turns,
        kit,
    })
}

fn to_talent(file: &Path, t: &TalentFile) -> Result<Talent, DataError> {
    let bad = |m: &str| fail(file, format!("talent {:?}: {m}", t.id));
    let multipliers = [t.endurance, t.damage, t.grabbed, t.turn_time];
    if multipliers.iter().any(|m| !(0.0..=5.0).contains(m)) {
        return Err(bad(
            "endurance, damage, grabbed and turn_time are multipliers from 0 to 5",
        ));
    }
    if !(0.0..=1.0).contains(&t.steady) {
        return Err(bad("steady is a multiplier from 0 to 1"));
    }
    let chances = [t.hit, t.knock_down, t.struggle];
    if chances.iter().any(|c| !(-1.0..=1.0).contains(c)) {
        return Err(bad(
            "hit, knock_down and struggle are chances added, from -1 to 1",
        ));
    }
    if !(0.0..=100.0).contains(&t.health) || t.hold > 5 {
        return Err(bad("health is 0 to 100 points, hold 0 to 5"));
    }
    if !(0.0..=100.0).contains(&t.faith)
        || !(0.0..=100.0).contains(&t.sunder)
        || !(0..=20).contains(&t.turn_reach)
    {
        return Err(bad(
            "faith and sunder are 0 to 100 points, turn_reach 0 to 20 tiles",
        ));
    }
    let skill = match &t.skill {
        None => None,
        Some(s) => Some(Skill::from_id(s).ok_or_else(|| bad("no such skill"))?),
    };
    let per = |m: f64| (m * 1000.0).round() as u32;
    let chance = |c: f64| (c * 10_000.0).round() as i32;
    Ok(Talent {
        id: t.id.clone(),
        name: t.name.clone(),
        line: t.line.clone(),
        health: points(t.health),
        endurance: per(t.endurance),
        damage: per(t.damage),
        skill,
        hit: chance(t.hit),
        knock_down: chance(t.knock_down),
        struggle: chance(t.struggle),
        grabbed: per(t.grabbed),
        hold: t.hold,
        doorway: t.doorway,
        cleave: t.cleave,
        second_wind: t.second_wind,
        faith: (t.faith * 1000.0).round() as i32,
        mending: t.mending,
        warding: t.warding,
        purify: t.purify,
        steady: per(t.steady),
        turn_time: per(t.turn_time),
        turn_reach: t.turn_reach,
        sunder: points(t.sunder),
        last_rites: t.last_rites,
    })
}
