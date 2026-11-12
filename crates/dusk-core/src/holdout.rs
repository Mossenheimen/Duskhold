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
//!   - **repair** the weapon in your hand, for coin by how worn it is;
//!   - **claim a bounty** on a rare that's been put down (`rare.rs`): coin, XP, and the
//!     pressure falls. Each is claimed once;
//!   - **have a magic item read** (`magic.rs`): its properties, and any curse, known;
//!   - **have a curse lifted** from the weapon that clings to your hand.
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

/// What a holdout pays for each rare put down: coin, XP (thousandths), and how far the
/// pressure falls (thousandths). All 0: it posts no bounties.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bounty {
    pub coin: u32,
    pub xp: u32,
    pub pressure: u32,
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
    /// What it pays for each rare put down.
    pub bounty: Bounty,
    /// Coin to read a magic item, and to lift a curse (0: it doesn't).
    pub identify: u32,
    pub uncurse: u32,
}

/// A slot that means the weapon in hand, where a deal takes a pack's slot.
pub const HANDS: u16 = u16::MAX;

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
    /// The rare's still about: put it down first.
    StillAbout,
    /// That bounty's been paid.
    Claimed,
    /// Nothing unknown to read there.
    NothingToRead,
    /// No curse in hand to lift.
    NoCurse,
    /// No room in your bag or your pockets for what you'd buy (`bag.rs`).
    NoRoom,
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
        if self.room_for(id, &Item::new(sale.item, 1)) == (0, 0) {
            return Err(NoDeal::NoRoom);
        }
        Ok(sale.price)
    }

    /// What a holdout pays for a stack, in coin: a share of its worth.
    fn price_of(&self, h: &Holdout, item: &Item) -> u32 {
        let worth = self.defs.worth(item);
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
        let worth = self.defs.worth(item);
        let counts = price > 0 || (worth > 0 && (item.found || h.wants.contains(&def)));
        if !counts {
            return Err(NoDeal::Worthless);
        }
        Ok(price)
    }

    /// What mending the weapon in `id`'s hand and the armour it wears (`armour.rs`) at the
    /// counter on `at` costs, in coin: each one's share of its worth, by how worn it is, and at
    /// least one in all.
    pub fn repair_cost(&self, id: ActorId, at: TilePos) -> Result<u32, NoDeal> {
        let h = self.reach_counter(id, at)?;
        let mending = self.to_mend(id);
        if mending.is_empty() {
            return Err(NoDeal::NothingToMend);
        }
        let cost: u64 = mending
            .iter()
            .map(|item| {
                let worn = u64::from(NEW - item.condition);
                let worth = u64::from(self.defs.worth(item));
                worth * u64::from(h.repair) / 1000 * worn / u64::from(NEW)
            })
            .sum();
        let cost = cost.div_ceil(1000).max(1) as u32;
        if self.coins(id) < cost {
            return Err(NoDeal::TooPoor);
        }
        Ok(cost)
    }

    /// Whether `id` can claim the bounty on `rare` over the counter on `at`, and what it pays.
    pub fn can_claim(&self, id: ActorId, at: TilePos, rare: ActorId) -> Result<Bounty, NoDeal> {
        let h = self.reach_counter(id, at)?;
        let bounty = h.bounty;
        if bounty == Bounty::default() || !self.is_rare(rare) {
            return Err(NoDeal::NoCounter);
        }
        if self.claimed.contains(&rare) {
            return Err(NoDeal::Claimed);
        }
        if !self.actors.get(rare).is_some_and(|r| r.dead) {
            return Err(NoDeal::StillAbout);
        }
        Ok(bounty)
    }

    /// The item at a pack's `slot`, or in hand (`HANDS`).
    fn item_at(&self, id: ActorId, slot: u16) -> Option<Item> {
        let a = self.actors.get(id)?;
        if slot == HANDS {
            a.hands
        } else {
            a.items.get(usize::from(slot)).copied()
        }
    }

    /// Whether `id` can have the item at `slot` (or `HANDS`) read over the counter on `at`,
    /// and for how much: a magic item it doesn't know.
    pub fn can_identify(&self, id: ActorId, at: TilePos, slot: u16) -> Result<u32, NoDeal> {
        let price = self.reach_counter(id, at)?.identify;
        let item = self.item_at(id, slot).ok_or(NoDeal::NothingToRead)?;
        if price == 0 || !item.magic.is_magic() || item.magic.known {
            return Err(NoDeal::NothingToRead);
        }
        if self.coins(id) < price {
            return Err(NoDeal::TooPoor);
        }
        Ok(price)
    }

    /// Whether `id` can have the curse on the weapon in its hand lifted, and for how much.
    pub fn can_uncurse(&self, id: ActorId, at: TilePos) -> Result<u32, NoDeal> {
        let price = self.reach_counter(id, at)?.uncurse;
        let item = self.item_at(id, HANDS).ok_or(NoDeal::NoCurse)?;
        if price == 0 || !item.magic.cursed() {
            return Err(NoDeal::NoCurse);
        }
        if self.coins(id) < price {
            return Err(NoDeal::TooPoor);
        }
        Ok(price)
    }

    /// The deal's end: the item read.
    pub(crate) fn read_item(&mut self, id: ActorId, at: TilePos, slot: u16) {
        let Ok(price) = self.can_identify(id, at, slot) else {
            return;
        };
        self.pay(id, price);
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let item = if slot == HANDS {
            a.hands.as_mut()
        } else {
            a.items.get_mut(usize::from(slot))
        };
        if let Some(item) = item {
            item.magic.known = true;
            item.magic.curse_shown = true;
            let def = item.def;
            self.events.push(Event::Identified { who: id, item: def });
        }
    }

    /// The deal's end: the curse lifted. The rest of its magic stays.
    pub(crate) fn lift_curse(&mut self, id: ActorId, at: TilePos) {
        let Ok(price) = self.can_uncurse(id, at) else {
            return;
        };
        self.pay(id, price);
        if let Some(held) = self.actors.get_mut(id).and_then(|a| a.hands.as_mut()) {
            held.magic.curse = crate::magic::NONE;
            let item = held.def;
            self.events.push(Event::CurseLifted { who: id, item });
        }
    }

    /// Whether the bounty on a rare has been claimed.
    pub fn bounty_claimed(&self, rare: ActorId) -> bool {
        self.claimed.contains(&rare)
    }

    /// The deal's end: the bounty paid.
    pub(crate) fn claim(&mut self, id: ActorId, at: TilePos, rare: ActorId) {
        let Ok(b) = self.can_claim(id, at, rare) else {
            return;
        };
        self.claimed.insert(rare);
        self.earn(id, b.coin);
        self.gain_deeds(id, b.xp);
        self.nudge_pressure(-i64::from(b.pressure));
        self.events.push(Event::BountyClaimed {
            who: id,
            rare,
            coin: b.coin,
        });
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
        let mut left = n;
        while left > 0 {
            let some = left.min(u32::from(u16::MAX)) as u16;
            self.carry_in(id, Item::new(coin, some));
            left -= u32::from(some);
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
        self.carry_in(id, Item::new(item, 1));
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
        let worth = self.defs.worth(&item);
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

    /// What there is to mend on `id`: the weapon in its hand and the armour it wears, those
    /// that aren't as new.
    fn to_mend(&self, id: ActorId) -> Vec<Item> {
        let Some(a) = self.actors.get(id) else {
            return Vec::new();
        };
        let weapon = a.hands.filter(|held| {
            self.defs
                .item(held.def)
                .is_some_and(|d| matches!(d.kind, ItemKind::Weapon { .. }))
        });
        weapon
            .iter()
            .chain(&a.worn)
            .filter(|i| i.condition < NEW)
            .copied()
            .collect()
    }

    /// The deal's end: the weapon in hand and the armour worn mended as new.
    pub(crate) fn mend(&mut self, id: ActorId, at: TilePos) {
        let Ok(cost) = self.repair_cost(id, at) else {
            return;
        };
        let Some(first) = self.to_mend(id).first().map(|i| i.def) else {
            return;
        };
        let weapon = self
            .actors
            .get(id)
            .and_then(|a| a.hands)
            .is_some_and(|held| {
                self.defs
                    .item(held.def)
                    .is_some_and(|d| matches!(d.kind, ItemKind::Weapon { .. }))
            });
        self.pay(id, cost);
        if let Some(a) = self.actors.get_mut(id) {
            if weapon && let Some(held) = a.hands.as_mut() {
                held.condition = NEW;
            }
            for piece in &mut a.worn {
                piece.condition = NEW;
            }
            self.events.push(Event::Mended {
                who: id,
                item: first,
                price: cost,
            });
        }
    }
}
