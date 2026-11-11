//! Holdouts (`CONCEPT.md`, the realm): the abbey on the hill, the first. A safe place, a trader
//! and a bed for the night.
//!
//! - **Hallowed ground:** inside a holdout's walls (`World::hallowed`), the dead never set foot,
//!   and take no notice of anything there: what they see, hear or smell on it isn't prey. So
//!   you can rest there with the dead about outside. A bed there is a guest's: sleeping in it
//!   doesn't make it home.
//! - **The counter:** a fixture a holdout trades over (`Defs::counters`). Within reach of it:
//!   - **buy** what it sells, for coin, from a stock that comes back each game day;
//!   - **sell** anything with a worth for a share of it in coin. A find sold there is brought
//!     home as it would be in your stash: it pays its worth in XP, once (`CONCEPT.md`, pillar 5);
//!   - **supply** it: selling what it's short of (`Holdout::wants`) lowers the region's pressure
//!     too, by how much it was worth (`pressure.rs`). A holdout supplied pushes the dusk back;
//!   - **repair** the weapon in your hand, for coin by how worn it is.
//!
//! Coin is an item (`Defs::coin`), carried and weighed like any other. What you pay with is your
//! ordinary coin first, and finds last, so a find keeps its XP for as long as it can.

use crate::actor::ActorId;
use crate::geom::{Dir8, TilePos};
use crate::items::{Defs, Item, ItemKind, NEW, within_reach};
use crate::world::{Event, World};

/// What a holdout sells: an item, its price in coin, and how many it has a game day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sale {
    pub item: u16,
    pub price: u32,
    pub stock: u16,
}

/// A holdout's trade, in the core's units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Holdout {
    pub id: String,
    pub name: String,
    pub sells: Vec<Sale>,
    /// What it pays for an item, per mille of the item's worth, in coin.
    pub buys: u32,
    /// What a full repair costs, per mille of the weapon's worth, in coin.
    pub repair: u32,
    /// The items it's short of.
    pub wants: Vec<u16>,
    /// How far selling it what it's short of lowers the pressure: thousandths of the pressure
    /// (itself in thousandths) for each point of worth.
    pub supply: u32,
    /// Ticks a deal takes.
    pub time: u32,
}

/// A rectangle of tiles on one floor, from (x0, y0) up to, not including, (x1, y1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub z: i8,
}

impl Area {
    pub fn contains(&self, at: TilePos) -> bool {
        at.z == self.z && (self.x0..self.x1).contains(&at.x) && (self.y0..self.y1).contains(&at.y)
    }
}

/// Whether a tile is on hallowed ground, by a list of areas.
pub(crate) fn in_hallowed(hallowed: &[Area], at: TilePos) -> bool {
    hallowed.iter().any(|a| a.contains(at))
}

/// Why a deal can't be done now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoDeal {
    /// No counter within reach, or nothing there (a stale slot or line).
    NoCounter,
    /// It's sold all it has of that today: more tomorrow.
    SoldOut,
    /// Not enough coin.
    TooPoor,
    /// It's worth nothing to them (and coin is coin).
    Worthless,
    /// Nothing in hand that wear has touched.
    NothingToMend,
}

impl Defs {
    /// The holdout a fixture is the counter of.
    pub fn counter(&self, fixture: u16) -> Option<u16> {
        self.counters.get(usize::from(fixture)).copied().flatten()
    }

    pub fn holdout(&self, n: u16) -> Option<&Holdout> {
        self.holdouts.get(usize::from(n))
    }
}

impl World {
    /// Whether a tile is on hallowed ground: inside a holdout's walls.
    pub fn is_hallowed(&self, at: TilePos) -> bool {
        in_hallowed(&self.hallowed, at)
    }

    /// The holdout whose counter is on `at`.
    pub fn counter_at(&self, at: TilePos) -> Option<u16> {
        self.defs.counter(self.grid.tile(at).fixture)
    }

    /// A counter within `id`'s reach, and its holdout: the one it faces, else the first round it.
    pub fn counter_in_reach(&self, id: ActorId) -> Option<(TilePos, u16)> {
        let a = self.actors.get(id).filter(|a| !a.dead)?;
        (0..8)
            .map(|t| a.pos.step(a.facing.turned(t)))
            .find_map(|at| {
                let h = self.counter_at(at)?;
                within_reach(&self.grid, a.pos, at).then_some((at, h))
            })
    }

    /// The game day, for the holdouts' stock: from 0.
    fn trade_day(&self) -> u64 {
        self.clock_minutes() / 1440
    }

    /// How many of a holdout's line it has left today.
    pub fn stock_left(&self, holdout: u16, line: u16) -> u16 {
        let Some(sale) = self
            .defs
            .holdout(holdout)
            .and_then(|h| h.sells.get(usize::from(line)))
        else {
            return 0;
        };
        let sold = match self.sold.get(&(holdout, line)) {
            Some(&(day, n)) if day == self.trade_day() => n,
            _ => 0,
        };
        sale.stock.saturating_sub(sold)
    }

    /// The coin `id` carries.
    pub fn coins(&self, id: ActorId) -> u32 {
        let (Some(a), Some(coin)) = (self.actors.get(id), self.defs.coin) else {
            return 0;
        };
        a.items
            .iter()
            .filter(|i| i.def == coin)
            .map(|i| u32::from(i.count))
            .sum()
    }

    /// The counter on `at`, if `id` can reach it.
    fn reach_counter(&self, id: ActorId, at: TilePos) -> Result<&Holdout, NoDeal> {
        let a = self
            .actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoDeal::NoCounter)?;
        let h = self.counter_at(at).ok_or(NoDeal::NoCounter)?;
        if !within_reach(&self.grid, a.pos, at) {
            return Err(NoDeal::NoCounter);
        }
        self.defs.holdout(h).ok_or(NoDeal::NoCounter)
    }

    /// Whether `id` can buy a holdout's `line` over the counter on `at`, and for how much.
    pub fn can_buy(&self, id: ActorId, at: TilePos, line: u16) -> Result<u32, NoDeal> {
        let h = self.reach_counter(id, at)?;
        let sale = h.sells.get(usize::from(line)).ok_or(NoDeal::NoCounter)?;
        let n = self.counter_at(at).ok_or(NoDeal::NoCounter)?;
        if self.stock_left(n, line) == 0 {
            return Err(NoDeal::SoldOut);
        }
        if self.coins(id) < sale.price {
            return Err(NoDeal::TooPoor);
        }
        Ok(sale.price)
    }

    /// What a holdout pays for a stack, in coin: a share of its worth.
    fn price_of(&self, h: &Holdout, item: &Item) -> u32 {
        let worth = self.defs.item(item.def).map_or(0, |d| d.worth);
        let all = u64::from(worth) * u64::from(item.count) * u64::from(h.buys);
        (all / 1_000_000) as u32
    }

    /// Whether `id` can sell the stack at `slot` of its pack over the counter on `at` (still
    /// `def`), and for how much coin. A find, or what the holdout is short of, sells even when
    /// the coin comes to nothing: it still pays its XP, or pushes the dusk back.
    pub fn can_sell(&self, id: ActorId, at: TilePos, slot: u16, def: u16) -> Result<u32, NoDeal> {
        let h = self.reach_counter(id, at)?;
        let a = self.actors.get(id).ok_or(NoDeal::NoCounter)?;
        let item = a
            .items
            .get(usize::from(slot))
            .filter(|i| i.def == def)
            .ok_or(NoDeal::NoCounter)?;
        if Some(def) == self.defs.coin {
            return Err(NoDeal::Worthless);
        }
        let price = self.price_of(h, item);
        let worth = self.defs.item(def).map_or(0, |d| d.worth);
        let counts = price > 0 || (worth > 0 && (item.found || h.wants.contains(&def)));
        if !counts {
            return Err(NoDeal::Worthless);
        }
        Ok(price)
    }

    /// What mending the weapon in `id`'s hand at the counter on `at` costs, in coin: its share
    /// of the weapon's worth, by how worn it is, and at least one.
    pub fn repair_cost(&self, id: ActorId, at: TilePos) -> Result<u32, NoDeal> {
        let h = self.reach_counter(id, at)?;
        let a = self.actors.get(id).ok_or(NoDeal::NoCounter)?;
        let held = a.hands.ok_or(NoDeal::NothingToMend)?;
        let def = self.defs.item(held.def).ok_or(NoDeal::NothingToMend)?;
        if !matches!(def.kind, ItemKind::Weapon { .. }) || held.condition >= NEW {
            return Err(NoDeal::NothingToMend);
        }
        let worn = u64::from(NEW - held.condition);
        let cost = u64::from(def.worth) * u64::from(h.repair) / 1000 * worn / u64::from(NEW);
        let cost = cost.div_ceil(1000).max(1) as u32;
        if self.coins(id) < cost {
            return Err(NoDeal::TooPoor);
        }
        Ok(cost)
    }

    /// Ticks a deal over the counter on `at` takes.
    pub(crate) fn deal_ticks(&self, at: TilePos) -> u32 {
        let h = self.counter_at(at).and_then(|n| self.defs.holdout(n));
        h.map_or(1, |h| h.time.max(1))
    }

    /// The way to face a counter from where `id` stands.
    pub(crate) fn toward(&self, id: ActorId, at: TilePos) -> Dir8 {
        let a = self.actors.get(id);
        let from = a.map_or(at, |a| a.pos);
        Dir8::toward(at.x - from.x, at.y - from.y).unwrap_or(a.map_or(Dir8::S, |a| a.facing))
    }

    /// Takes `n` coins from `id`'s pack: ordinary coin first, finds last.
    fn pay(&mut self, id: ActorId, mut n: u32) {
        let Some(coin) = self.defs.coin else { return };
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        for finds in [false, true] {
            for i in (0..a.items.len()).rev() {
                if n == 0 {
                    return;
                }
                let it = &mut a.items[i];
                if it.def != coin || it.found != finds {
                    continue;
                }
                let used = u32::from(it.count).min(n);
                it.count -= used as u16;
                n -= used;
                if it.count == 0 {
                    a.items.remove(i);
                }
            }
        }
    }

    /// Puts `n` ordinary coins in `id`'s pack.
    fn earn(&mut self, id: ActorId, n: u32) {
        let (Some(coin), true) = (self.defs.coin, n > 0) else {
            return;
        };
        let defs = self.defs.clone();
        if let Some(a) = self.actors.get_mut(id) {
            let mut left = n;
            while left > 0 {
                let some = left.min(u32::from(u16::MAX)) as u16;
                defs.stow(&mut a.items, Item::new(coin, some));
                left -= u32::from(some);
            }
        }
    }

    /// The deal's end: one of the line bought.
    pub(crate) fn buy(&mut self, id: ActorId, at: TilePos, line: u16) {
        let Ok(price) = self.can_buy(id, at, line) else {
            return;
        };
        let Some(n) = self.counter_at(at) else { return };
        let Some(sale) = self
            .defs
            .holdout(n)
            .and_then(|h| h.sells.get(usize::from(line)))
        else {
            return;
        };
        let item = sale.item;
        self.pay(id, price);
        let day = self.trade_day();
        let sold = self.sold.entry((n, line)).or_insert((day, 0));
        if sold.0 != day {
            *sold = (day, 0);
        }
        sold.1 += 1;
        let defs = self.defs.clone();
        if let Some(a) = self.actors.get_mut(id) {
            defs.stow(&mut a.items, Item::new(item, 1));
        }
        self.events.push(Event::Bought {
            who: id,
            item,
            price,
        });
    }

    /// The deal's end: the stack sold. A find is brought home; what the holdout is short of
    /// supplies it, and the pressure falls.
    pub(crate) fn sell(&mut self, id: ActorId, at: TilePos, slot: u16, def: u16) {
        let Ok(price) = self.can_sell(id, at, slot, def) else {
            return;
        };
        let Some(h) = self.counter_at(at).and_then(|n| self.defs.holdout(n)) else {
            return;
        };
        let (wanted, supply) = (h.wants.contains(&def), h.supply);
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let item = a.items.remove(usize::from(slot));
        let worth = self.defs.item(def).map_or(0, |d| d.worth);
        self.earn(id, price);
        self.events.push(Event::Sold {
            who: id,
            item: def,
            count: item.count,
            price,
        });
        if item.found {
            let xp = worth.saturating_mul(u32::from(item.count));
            self.events.push(Event::BroughtHome {
                who: id,
                item: def,
                count: item.count,
                xp,
            });
            self.gain_deeds(id, xp);
        }
        if wanted {
            // Worth in thousandths of a point, times thousandths of the pressure a point.
            let by = u64::from(worth) * u64::from(item.count) * u64::from(supply) / 1_000_000;
            if by > 0 {
                self.nudge_pressure(-(by as i64));
                self.events.push(Event::Supplied {
                    who: id,
                    by: by as u32,
                });
            }
        }
    }

    /// The deal's end: the weapon in hand mended as new.
    pub(crate) fn mend(&mut self, id: ActorId, at: TilePos) {
        let Ok(cost) = self.repair_cost(id, at) else {
            return;
        };
        self.pay(id, cost);
        if let Some(a) = self.actors.get_mut(id)
            && let Some(held) = a.hands.as_mut()
        {
            held.condition = NEW;
            let item = held.def;
            self.events.push(Event::Mended {
                who: id,
                item,
                price: cost,
            });
        }
    }
}
