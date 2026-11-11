//! The dead getting up, burning them, and a region's pressure: a creature's `get_up` (game hours
//! as game minutes), rules.toml's [burning] (seconds as ticks, game minutes, tiles), and a
//! region's [pressure] (per cent of the dead it began with, as thousandths).

use std::path::Path;

use dusk_core::creature::GetUp;
use dusk_core::remains::Burning;
use dusk_core::{ActorKind, Defs, Pressure};
use serde::Deserialize;

use crate::{DataError, Region, chance, fail, ticks};

/// A creature's `get_up`: the chance its body gets up again, and how many game hours after it
/// fell, the soonest and the latest.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GetUpFile {
    chance: f64,
    after: (f64, f64),
}

pub(crate) fn to_get_up(file: &Path, g: &GetUpFile) -> Result<GetUp, String> {
    let chance = chance(file, "get_up.chance", g.chance).map_err(|e| e.message)?;
    let (soonest, latest) = g.after;
    if !(soonest > 0.0 && soonest <= latest && latest <= 24.0 * 30.0) {
        return Err("get_up.after is [soonest, latest] in game hours, above 0".to_string());
    }
    Ok(GetUp {
        chance,
        after: (
            (soonest * 60.0).round() as u32,
            (latest * 60.0).round() as u32,
        ),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BurningFile {
    /// Seconds to pour the oil on and set it alight.
    time: f64,
    /// Game minutes a body burns.
    burn: u32,
    /// Tiles a burning body lights round it.
    light: i32,
}

pub(crate) fn to_burning(file: &Path, b: &BurningFile) -> Result<Burning, DataError> {
    if b.burn == 0 || !(0..=12).contains(&b.light) {
        return Err(fail(
            file,
            "burning: burn is at least 1 game minute, and light 0 to 12 tiles",
        ));
    }
    Ok(Burning {
        time: (ticks(file, "burning.time", b.time)? as u32).max(1),
        burn: b.burn,
        light: b.light,
    })
}

/// A region's pressure, in per cent of the dead its patch began with (`dusk_core::pressure`).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PressureFile {
    /// Where it starts, and the least and the most it can be.
    pub start: f64,
    pub least: f64,
    pub most: f64,
    /// Time raises it this much a game day, up to the ceiling.
    pub per_day: f64,
    pub ceiling: f64,
    /// Each body that gets up raises it this much; each burned lowers it this much.
    pub per_rise: f64,
    pub per_burn: f64,
    /// While the patch is short of the dead, one walks in every this many game minutes.
    pub every: u32,
}

/// Per cent as thousandths.
fn mille(p: f64) -> u32 {
    (p * 10.0).round() as u32
}

impl PressureFile {
    /// What's wrong with these numbers, if anything.
    pub(crate) fn problem(&self) -> Option<String> {
        let all = [
            self.start,
            self.least,
            self.most,
            self.per_day,
            self.ceiling,
            self.per_rise,
            self.per_burn,
        ];
        if all.iter().any(|p| !(0.0..=1000.0).contains(p)) {
            return Some("pressure: every number is 0 to 1000 per cent".to_string());
        }
        if !(self.least <= self.start && self.start <= self.ceiling && self.ceiling <= self.most) {
            return Some("pressure: least ≤ start ≤ ceiling ≤ most".to_string());
        }
        None
    }

    pub fn to_pressure(&self) -> Pressure {
        Pressure {
            start: mille(self.start),
            least: mille(self.least),
            most: mille(self.most),
            per_day: mille(self.per_day),
            ceiling: mille(self.ceiling),
            per_rise: mille(self.per_rise),
            per_burn: mille(self.per_burn),
            every: self.every,
        }
    }
}

impl Region {
    /// What the core knows of this region, for a patch of it that began with `dead` of the dead.
    pub fn to_core(&self, defs: &Defs, dead: u32) -> dusk_core::Region {
        let kinds = self.dead.kinds.iter().filter_map(|k| {
            let ActorKind::Creature(n) = defs.creature_kind(&k.creature)? else {
                return None;
            };
            Some((ActorKind::Creature(n), (k.share * 10_000.0).round() as u32))
        });
        dusk_core::Region {
            pressure: self.pressure.to_pressure(),
            kinds: kinds.collect(),
            chunks: self.patch.chunks,
            dead,
            tolling: self
                .tolling
                .as_ref()
                .map(|t| t.to_tolling())
                .unwrap_or_default(),
            // Where the bell hangs, and where you come in, are the patch's to say (`dusk-gen`).
            bell: None,
            start: None,
        }
    }
}
