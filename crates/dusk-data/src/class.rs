//! Classes (`data/classes.toml`) and their talents (`data/talents.toml`), in a designer's units:
//! multipliers, points of health, chances added, skills by id.

use std::path::Path;

use dusk_core::skills::Skill;
use dusk_core::{Class, Talent};
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
    /// The skills it learns faster (by id), and how many times as fast.
    skills: Vec<String>,
    learn: f64,
    /// What its running, swings and struggles cost in endurance, as a multiplier.
    endurance: f64,
    /// The chance a creature's hit takes hold of it, as a multiplier.
    grabbed: f64,
    /// Its talents, by id (talents.toml).
    talents: Vec<String>,
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
}

fn one() -> f64 {
    1.0
}

/// The talents in `talents.toml` and the classes in `classes.toml` (left out: none, and
/// characters have no class).
pub(crate) fn load(dir: &Path) -> Result<(Vec<Class>, Vec<Talent>), DataError> {
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
        for c in &parsed.class {
            let bad = |m: String| fail(&classes_file, format!("class {:?}: {m}", c.id));
            if !(1.0..=5.0).contains(&c.learn) || !(0.1..=2.0).contains(&c.endurance) {
                return Err(bad(
                    "learn is 1 to 5 times, endurance a multiplier from 0.1 to 2".into(),
                ));
            }
            if !(0.0..=2.0).contains(&c.grabbed) {
                return Err(bad("grabbed is a multiplier from 0 to 2".into()));
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
            let per = |m: f64| (m * 1000.0).round() as u32;
            classes.push(Class {
                id: c.id.clone(),
                name: c.name.clone(),
                skills,
                learn: per(c.learn),
                endurance: per(c.endurance),
                grabbed: per(c.grabbed),
                talents: picks,
            });
        }
    }
    Ok((classes, talents))
}

fn to_talent(file: &Path, t: &TalentFile) -> Result<Talent, DataError> {
    let bad = |m: &str| fail(file, format!("talent {:?}: {m}", t.id));
    let multipliers = [t.endurance, t.damage, t.grabbed];
    if multipliers.iter().any(|m| !(0.0..=5.0).contains(m)) {
        return Err(bad(
            "endurance, damage and grabbed are multipliers from 0 to 5",
        ));
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
    })
}
