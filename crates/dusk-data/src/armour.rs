//! Armour (`armour.rs` in the core): a piece's numbers in an item's `armour = { ... }`, and the
//! rules' [armour] (optional: with none, the core's defaults).

use std::path::Path;

use dusk_core::armour::{Armour, ArmourRules};
use dusk_core::{Group, ItemKind};
use serde::Deserialize;

use crate::{DataError, chance, fail, ticks};

/// A piece of armour: the places it covers ("head", "torso", "arms", "legs"), its chance to
/// turn each kind of wound, the condition (out of 100) a blow turned takes off it, the tiles it
/// adds to a footstep, the share more every cost in breath comes to, the share of your sight it
/// leaves you, and the seconds it takes to put on.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArmourFile {
    covers: Vec<String>,
    turn: TurnFile,
    wear: f64,
    #[serde(default)]
    noise: i32,
    #[serde(default)]
    endurance: f64,
    #[serde(default = "all")]
    sight: f64,
    time: f64,
    /// Points it takes off the cold (`cold.rs` in the core); left out, none.
    #[serde(default)]
    warmth: f64,
}

fn all() -> f64 {
    1.0
}

/// A piece's chance to turn a scratch, a laceration and a bite.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TurnFile {
    scratch: f64,
    laceration: f64,
    bite: f64,
}

pub(crate) fn armour(file: &Path, id: &str, a: &ArmourFile) -> Result<ItemKind, DataError> {
    let what = |part: &str| format!("item {id:?}: armour.{part}");
    let mut covers = 0u8;
    for name in &a.covers {
        let Some(group) = Group::ALL.iter().find(|g| g.name() == name) else {
            return Err(fail(
                file,
                format!(
                    "{} {name:?}: it's \"head\", \"torso\", \"arms\" or \"legs\"",
                    what("covers")
                ),
            ));
        };
        if covers & group.bit() != 0 {
            return Err(fail(
                file,
                format!("{} names {name:?} twice", what("covers")),
            ));
        }
        covers |= group.bit();
    }
    if covers == 0
        || !(0.0..=100.0).contains(&a.wear)
        || !(0..=10).contains(&a.noise)
        || !(0.0..=2.0).contains(&a.endurance)
        || !(a.sight > 0.0 && a.sight <= 1.0)
    {
        return Err(fail(
            file,
            format!(
                "item {id:?}: armour covers at least one place; its wear is out of 100, its noise \
                 0 to 10 tiles, its endurance a share more from 0 to 2, its sight a share above 0 \
                 and at most 1"
            ),
        ));
    }
    let time = ticks(file, &what("time"), a.time)? as u32;
    if time == 0 {
        return Err(fail(file, format!("{} is at least a tick", what("time"))));
    }
    if !(0.0..=50.0).contains(&a.warmth) {
        return Err(fail(
            file,
            format!("{} is points, from 0 to 50", what("warmth")),
        ));
    }
    Ok(ItemKind::Armour(Armour {
        covers,
        turn: [
            chance(file, &what("turn.scratch"), a.turn.scratch)?,
            chance(file, &what("turn.laceration"), a.turn.laceration)?,
            chance(file, &what("turn.bite"), a.turn.bite)?,
        ],
        wear: (a.wear * 10.0).round() as u16,
        noise: a.noise,
        endurance: (a.endurance * 1000.0).round() as u32,
        sight: (a.sight * 1000.0).round() as u32,
        time,
        warmth: (a.warmth * 1000.0).round() as i32,
    }))
}

/// rules.toml's [armour]: shares, from 0 to 1.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArmourRulesFile {
    /// What a blow turned still does of its harm.
    bruise: f64,
    /// What a piece weighs worn, of its weight.
    worn_weight: f64,
    /// How much of a piece's chance holds against each kind of weapon.
    pierce: PierceFile,
    /// How long taking a piece off takes, of putting it on.
    off: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PierceFile {
    blade: f64,
    blunt: f64,
    axe: f64,
    spear: f64,
}

pub(crate) fn to_armour_rules(file: &Path, a: &ArmourRulesFile) -> Result<ArmourRules, DataError> {
    let p = &a.pierce;
    let shares = [a.bruise, a.worn_weight, p.blade, p.blunt, p.axe, p.spear];
    if shares.iter().any(|s| !(0.0..=1.0).contains(s)) || !(a.off > 0.0 && a.off <= 1.0) {
        return Err(fail(
            file,
            "armour: bruise, worn_weight and the pierce shares are from 0 to 1, off above 0 and at most 1",
        ));
    }
    let mille = |x: f64| (x * 1000.0).round() as u32;
    Ok(ArmourRules {
        bruise: mille(a.bruise),
        worn_weight: mille(a.worn_weight),
        pierce: [mille(p.blade), mille(p.blunt), mille(p.axe), mille(p.spear)],
        off: mille(a.off),
    })
}
