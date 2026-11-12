//! Learning by doing (rules.toml's [skills]), turned into the core's `SkillRules`: XP as
//! thousandths, shares as per mille, game hours as ticks.

use std::path::Path;

use dusk_core::skills::{MAX_LEVEL, SKILLS, SkillRules};
use serde::Deserialize;

use crate::{DataError, RulesFile, fail};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillsFile {
    /// The XP to reach each level from 1 to 10, rising.
    levels: Vec<f64>,
    free_cap: u8,
    masteries: u8,
    /// A repeat pays this share of what it paid the time before...
    repeat: f64,
    /// ...until it's been left alone this many game hours.
    forget: f64,
    /// Tiles: running round in a square this size is running in the same spot.
    spot: i32,
    /// Tiles: one of the dead after you this near makes a run count.
    chased: i32,
    /// Or carrying more than this share of what you can.
    loaded: f64,
    xp: XpFile,
    level: LevelFile,
}

/// XP for each act that teaches.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct XpFile {
    hit: f64,
    down: f64,
    run: f64,
    bandage: f64,
    board: f64,
    /// Stealth's: a tile sneaked near the dead, and a sneak attack (left out: none).
    #[serde(default)]
    sneak: f64,
    #[serde(default)]
    sneak_attack: f64,
    /// Shield's: a block or a bash (left out: none).
    #[serde(default)]
    shield: f64,
}

/// What a level does, as a share a level.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelFile {
    hit: f64,
    damage: f64,
    endurance: f64,
    wear: f64,
    run_speed: f64,
    run_endurance: f64,
    bandage: f64,
    board_time: f64,
    board_strength: f64,
    /// Stealth: quicker sneaking, and more harm to a sneak attack (left out: none).
    #[serde(default)]
    sneak_pace: f64,
    #[serde(default)]
    sneak_harm: f64,
    /// Shield: a shield's chance to block, added (left out: none).
    #[serde(default)]
    shield_block: f64,
    /// Cooking: cooking takes less time (left out: no quicker).
    #[serde(default)]
    cook_time: f64,
}

/// The skills' rules, from rules.toml. `ticks_per_minute` is the clock's, for `forget`.
pub(crate) fn to_skills(
    file: &Path,
    r: &RulesFile,
    ticks_per_minute: u32,
) -> Result<SkillRules, DataError> {
    let s = &r.skills;
    let rising = s.levels.windows(2).all(|w| w[0] < w[1]);
    if s.levels.len() != usize::from(MAX_LEVEL) || !rising || s.levels[0] <= 0.0 {
        return Err(fail(
            file,
            format!("skills.levels: the XP for levels 1 to {MAX_LEVEL}, above 0 and rising"),
        ));
    }
    if !(1..MAX_LEVEL).contains(&s.free_cap) || usize::from(s.masteries) > SKILLS {
        return Err(fail(
            file,
            format!(
                "skills: free_cap is a level from 1 to {}, and masteries at most {SKILLS} (one \
                 per skill)",
                MAX_LEVEL - 1
            ),
        ));
    }
    let unit = |x: f64| (0.0..=1.0).contains(&x);
    if !unit(s.repeat) || !unit(s.loaded) || !unit(s.xp.down) {
        return Err(fail(
            file,
            "skills: repeat, loaded and xp.down are shares, from 0 to 1",
        ));
    }
    if !(0.0..=24.0 * 30.0).contains(&s.forget) || s.spot < 1 || s.chased < 0 {
        return Err(fail(
            file,
            "skills: forget is 0 game hours or more (and sensible), spot 1 tile or more, chased \
             0 or more",
        ));
    }
    let x = &s.xp;
    if [x.hit, x.run, x.bandage, x.board]
        .iter()
        .any(|&p| !(0.0..1e6).contains(&p))
    {
        return Err(fail(file, "skills.xp: XP is 0 or more"));
    }
    let l = &s.level;
    // A level can't take away more than all of something by level 10.
    let less = [
        l.endurance,
        l.wear,
        l.run_endurance,
        l.bandage,
        l.board_time,
        l.hit,
    ];
    let tenth = |p: f64| (0.0..=1.0 / f64::from(MAX_LEVEL)).contains(&p);
    if !less.iter().all(|&p| tenth(p))
        || !unit(l.damage)
        || !unit(l.run_speed)
        || !unit(l.board_strength)
    {
        return Err(fail(
            file,
            format!(
                "skills.level: hit, endurance, wear, run_endurance, bandage and board_time are \
                 shares from 0 to {} (all of it by level {MAX_LEVEL}); damage, run_speed and \
                 board_strength from 0 to 1",
                1.0 / f64::from(MAX_LEVEL)
            ),
        ));
    }
    let xp = |p: f64| (p * 1000.0).round() as u32;
    let mille = |p: f64| (p * 1000.0).round() as u32;
    let mut levels = [0; MAX_LEVEL as usize];
    for (to, &from) in levels.iter_mut().zip(&s.levels) {
        *to = xp(from);
    }
    Ok(SkillRules {
        levels,
        free_cap: s.free_cap,
        masteries: s.masteries,
        repeat: mille(s.repeat),
        forget: (s.forget * 60.0).round() as u64 * u64::from(ticks_per_minute),
        spot: s.spot,
        hit: xp(x.hit),
        down: mille(x.down),
        run: xp(x.run),
        bandage: xp(x.bandage),
        board: xp(x.board),
        chased: s.chased,
        loaded: mille(s.loaded),
        weapon_hit: (l.hit * 10_000.0).round() as u32,
        weapon_damage: mille(l.damage),
        weapon_endurance: mille(l.endurance),
        weapon_wear: mille(l.wear),
        run_speed: mille(l.run_speed),
        run_endurance: mille(l.run_endurance),
        bandage_time: mille(l.bandage),
        board_time: mille(l.board_time),
        board_strength: mille(l.board_strength),
        sneak: xp(x.sneak.max(0.0)),
        sneak_attack: xp(x.sneak_attack.max(0.0)),
        sneak_pace: mille(l.sneak_pace.clamp(0.0, 0.1)),
        sneak_harm: mille(l.sneak_harm.clamp(0.0, 1.0)),
        shield: xp(x.shield.max(0.0)),
        shield_block: (l.shield_block.clamp(0.0, 0.1) * 10_000.0).round() as u32,
        cook_time: mille(l.cook_time.clamp(0.0, 0.1)),
    })
}
