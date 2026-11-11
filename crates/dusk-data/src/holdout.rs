//! The holdouts' trade (`data/holdouts.toml`): what each sells and pays, what it's short of, and
//! which item is coin. Prices are in coin; shares are from 0 to 1, as the core's per mille.

use std::path::Path;

use dusk_core::{Bounty, Defs, Holdout, Sale};
use serde::Deserialize;

use crate::{DataError, Fixture, check_unique, fail, read, ticks};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HoldoutsFile {
    /// The item that's coin.
    coin: String,
    holdout: Vec<HoldoutFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HoldoutFile {
    id: String,
    name: String,
    /// The share of an item's worth it pays, in coin.
    buys: f64,
    /// The share of a weapon's worth a full repair costs, in coin.
    repair: f64,
    /// Seconds a deal takes.
    time: f64,
    /// What it's short of: selling it these supplies it.
    #[serde(default)]
    wants: Vec<String>,
    /// How far supplying it lowers the pressure: per cent for each point of worth.
    #[serde(default)]
    supply: f64,
    sells: Vec<SaleFile>,
    /// What it pays for each rare put down (left out: it posts no bounties).
    #[serde(default)]
    bounty: Option<BountyFile>,
    /// Coin to read a magic item, and to lift a curse (left out: it doesn't).
    #[serde(default)]
    identify: u32,
    #[serde(default)]
    uncurse: u32,
}

/// A bounty: coin, XP, and how far the pressure falls, in per cent.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BountyFile {
    coin: u32,
    xp: f64,
    pressure: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SaleFile {
    item: String,
    /// In coin.
    price: u32,
    /// How many it has a game day.
    stock: u16,
}

/// Reads `holdouts.toml` into the definitions, and ties each counter fixture to its holdout. A
/// data set without the file has no holdouts, and no counter may name one.
pub(crate) fn load(dir: &Path, defs: &mut Defs, fixtures: &[Fixture]) -> Result<(), DataError> {
    let file = dir.join("holdouts.toml");
    let fixtures_file = dir.join("fixtures.toml");
    let parsed: Option<HoldoutsFile> = if file.exists() {
        Some(read(&file)?)
    } else {
        None
    };
    let item = |name: &str| {
        defs.items
            .iter()
            .position(|i| i.id == name)
            .map(|n| n as u16)
            .ok_or_else(|| fail(&file, format!("no item called {name:?} in items/")))
    };
    let mut holdouts = Vec::new();
    let mut coin = None;
    if let Some(p) = &parsed {
        coin = Some(item(&p.coin)?);
        check_unique(&file, "holdout", p.holdout.iter().map(|h| &h.id))?;
        for h in &p.holdout {
            let bad = |m: &str| fail(&file, format!("holdout {:?}: {m}", h.id));
            if !(0.0..=1.0).contains(&h.buys) || !(0.0..=10.0).contains(&h.repair) {
                return Err(bad(
                    "buys is a share from 0 to 1, and repair one from 0 to 10",
                ));
            }
            if !(0.0..=10.0).contains(&h.supply) {
                return Err(bad("supply is 0 to 10 per cent a point of worth"));
            }
            let mut sells = Vec::new();
            for s in &h.sells {
                if s.price == 0 || s.stock == 0 {
                    return Err(bad("each line it sells has a price and a stock above 0"));
                }
                sells.push(Sale {
                    item: item(&s.item)?,
                    price: s.price,
                    stock: s.stock,
                });
            }
            let wants = h.wants.iter().map(|w| item(w)).collect::<Result<_, _>>()?;
            let bounty = match &h.bounty {
                Some(b) => {
                    if !(0.0..=10_000.0).contains(&b.xp) || !(0.0..=100.0).contains(&b.pressure) {
                        return Err(bad("bounty: xp 0 to 10,000, pressure 0 to 100 per cent"));
                    }
                    Bounty {
                        coin: b.coin,
                        xp: (b.xp * 1000.0).round() as u32,
                        pressure: (b.pressure * 10.0).round() as u32,
                    }
                }
                None => Bounty::default(),
            };
            holdouts.push(Holdout {
                id: h.id.clone(),
                name: h.name.clone(),
                sells,
                buys: (h.buys * 1000.0).round() as u32,
                repair: (h.repair * 1000.0).round() as u32,
                wants,
                // Per cent a point, as thousandths of the pressure's thousandths a point.
                supply: (h.supply * 10_000.0).round() as u32,
                time: (ticks(&file, "time", h.time)? as u32).max(1),
                bounty,
                identify: h.identify,
                uncurse: h.uncurse,
            });
        }
    }
    let mut counters = vec![None];
    for f in fixtures {
        counters.push(match &f.trade {
            None => None,
            Some(name) => Some(
                holdouts
                    .iter()
                    .position(|h: &Holdout| &h.id == name)
                    .map(|n| n as u16)
                    .ok_or_else(|| {
                        fail(
                            &fixtures_file,
                            format!("fixture {:?}: no holdout called {name:?}", f.id),
                        )
                    })?,
            ),
        });
    }
    defs.holdouts = holdouts;
    defs.counters = counters;
    defs.coin = coin;
    Ok(())
}
