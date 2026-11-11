//! Items: what they are (definitions, from the data), what a container holds, and the timed
//! actions that move them (`ARCHITECTURE.md`: items are values; containers fill from the seed).
//!
//! An item is a value: a definition's number, a count and a condition. A container's contents
//! are a function of the seed, its place and its loot table, worked out whenever someone looks.
//! They're stored only once something changes, so an unopened cupboard costs nothing. The floor
//! of a tile holds what's dropped there.

use crate::actor::{ActorId, ActorKind};
use crate::barricade::Boarding;
use crate::creature::CreatureDef;
use crate::geom::{Dir8, TilePos};
use crate::grid::Grid;
use crate::hash::{Purpose, below, mix64, roll};
use crate::skills::{On, Skill};
use crate::world::{Event, Swing, World};

/// A full condition: a new weapon.
pub const NEW: u16 = 1000;

/// One item, or a stack of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Item {
    /// Its definition's number: its place in `Defs::items`.
    pub def: u16,
    pub count: u16,
    /// Out of `NEW`. Weapons wear with every blow and break at 0.
    pub condition: u16,
    /// A find: it came out of the world and hasn't been home yet (`home.rs`).
    pub found: bool,
}

impl Item {
    /// An item that isn't a find: what you start with, or what a test or a tool puts down.
    pub fn new(def: u16, count: u16) -> Item {
        Item {
            def,
            count,
            condition: NEW,
            found: false,
        }
    }
}

/// What an item does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ItemKind {
    /// Swung in hand: its swing, the condition it loses a blow, and the skill it trains and uses
    /// (`skills.rs`; None for one that needs no skill).
    Weapon {
        swing: Swing,
        wear: u16,
        skill: Option<Skill>,
    },
    /// Put on a bleeding wound, it stops the bleeding: `time` ticks to put on.
    Bandage { time: u32 },
    /// Eaten: hunger falls by `nutrition` (thousandths of a point); `time` ticks to eat.
    Food { nutrition: i32, time: u32 },
    /// Drunk: thirst falls by `water`; `time` ticks to drink.
    Drink { water: i32, time: u32 },
    /// Taken with ghoul fever, it cures it, at any stage (`fever.rs`): `time` ticks to take.
    Cure { time: u32 },
    /// Held up lit (`light.rs`): it lights `radius` tiles round it, and loses `wane` of its
    /// condition a game minute until it burns out.
    Light { radius: i32, wane: u16 },
    /// Poured on a body and set alight, it burns it (`remains.rs`): one a body.
    Oil,
    /// Everything else: materials, junk.
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    /// Grams, for one.
    pub weight: u32,
    /// How many make one stack.
    pub stack: u16,
    pub kind: ItemKind,
    /// The XP one brings home, the first time (thousandths: `home.rs`).
    pub worth: u32,
}

/// One line of a loot table: an item, how likely it is against the others, how many.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LootEntry {
    pub item: u16,
    pub weight: u32,
    pub count: (u16, u16),
}

/// What a kind of container holds: so many draws, each an entry picked by weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LootTable {
    pub id: String,
    pub rolls: (u32, u32),
    pub entries: Vec<LootEntry>,
}

/// The game's definitions that the core needs while it runs, shared read-only by every world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Defs {
    /// The kinds of creature (`creature.rs`): `ActorKind::Creature(n)` is `creatures[n]`.
    pub creatures: Vec<CreatureDef>,
    pub items: Vec<ItemDef>,
    pub loot: Vec<LootTable>,
    /// By fixture number: the loot table of a container (None for a fixture that isn't one).
    pub containers: Vec<Option<u16>>,
    /// By fixture number: whether it's a bed (sleep beside one rests you twice as fast).
    pub beds: Vec<bool>,
    /// What a character starts with: in hand, and carried.
    pub wield: Option<u16>,
    pub kit: Vec<Item>,
    /// Grams a character carries before it's weighed down (no running).
    pub carry: u32,
    /// How long item actions take, in ticks: taking (plus `take_per_kg` for each kilogram),
    /// dropping, taking something in hand, putting it away, lighting a light and holding it up,
    /// and putting it out.
    pub take: u32,
    pub take_per_kg: u32,
    pub drop: u32,
    pub wield_time: u32,
    pub unwield_time: u32,
    pub light_time: u32,
    pub douse_time: u32,
    /// What boarding up takes (`barricade.rs`); None: nobody can.
    pub boarding: Option<Boarding>,
    /// The holdouts' trade (`holdout.rs`), and by fixture number, the holdout a counter is
    /// the counter of.
    pub holdouts: Vec<crate::holdout::Holdout>,
    pub counters: Vec<Option<u16>>,
    /// The item that's coin.
    pub coin: Option<u16>,
}

impl Default for Defs {
    /// The fixed test set: one creature, the test shambler (`Creature::default`), and no items.
    fn default() -> Defs {
        Defs {
            creatures: vec![CreatureDef::shambler()],
            items: Vec::new(),
            loot: Vec::new(),
            containers: Vec::new(),
            beds: Vec::new(),
            wield: None,
            kit: Vec::new(),
            carry: 0,
            take: 0,
            take_per_kg: 0,
            drop: 0,
            wield_time: 0,
            unwield_time: 0,
            light_time: 0,
            douse_time: 0,
            boarding: None,
            holdouts: Vec::new(),
            counters: Vec::new(),
            coin: None,
        }
    }
}

impl Defs {
    pub fn item(&self, def: u16) -> Option<&ItemDef> {
        self.items.get(usize::from(def))
    }

    /// Whether a fixture is a bed.
    pub fn is_bed(&self, fixture: u16) -> bool {
        self.beds
            .get(usize::from(fixture))
            .copied()
            .unwrap_or(false)
    }

    /// Whether a fixture is a container, and its loot table.
    pub fn container(&self, fixture: u16) -> Option<Option<u16>> {
        if fixture == 0 {
            return None;
        }
        self.containers
            .get(usize::from(fixture))
            .copied()
            .flatten()
            .map(Some)
    }

    /// The weight of an item or stack, in grams.
    pub fn weight(&self, item: &Item) -> u32 {
        self.item(item.def)
            .map_or(0, |d| d.weight * u32::from(item.count))
    }

    /// How a stack goes into a list: onto a stack of the same, in the same condition, that has
    /// room (a half-burnt candle doesn't join new ones), else on its own.
    pub fn stow(&self, list: &mut Vec<Item>, item: Item) {
        let stack = self.item(item.def).map_or(1, |d| d.stack.max(1));
        let mut left = item.count;
        if stack > 1 {
            for held in list.iter_mut() {
                let same = held.def == item.def
                    && held.condition == item.condition
                    && held.found == item.found;
                if same && held.count < stack && left > 0 {
                    let room = (stack - held.count).min(left);
                    held.count += room;
                    left -= room;
                }
            }
        }
        while left > 0 {
            let n = left.min(stack);
            list.push(Item { count: n, ..item });
            left -= n;
        }
    }
}

/// Where a container or a floor can be reached from: the tile itself, or one beside it with no
/// wall or shut door between (furniture is solid, so this isn't `reaches`).
pub(crate) fn within_reach(grid: &Grid, from: TilePos, to: TilePos) -> bool {
    if from == to {
        return true;
    }
    if from.z != to.z || from.steps_to(to) != 1 {
        return false;
    }
    let Some(dir) = Dir8::toward(to.x - from.x, to.y - from.y) else {
        return false;
    };
    match dir.split() {
        None => grid.edge(from, dir).walkable(),
        // Round either corner, if that way is open.
        Some((a, b)) => [(a, b), (b, a)].iter().any(|&(first, second)| {
            let via = from.step(first);
            grid.edge(from, first).walkable()
                && grid.tile(via).walkable()
                && grid.edge(via, second).walkable()
        }),
    }
}

/// A tile's place as one number, for the rolls that fill its container.
fn place_key(at: TilePos) -> u64 {
    mix64((at.x as u32 as u64) | ((at.y as u32 as u64) << 32)) ^ (at.z as u8 as u64)
}

impl World {
    /// What a tile holds: its container's contents (worked out from the seed if nobody has
    /// changed them), or what lies on its floor.
    pub fn store(&self, at: TilePos) -> Vec<Item> {
        if let Some(held) = self.stores.get(&at) {
            return held.clone();
        }
        let fixture = self.grid.tile(at).fixture;
        match self.defs.container(fixture) {
            Some(Some(table)) => self.fill(at, table),
            _ => Vec::new(),
        }
    }

    /// Whether a tile's fixture is a container.
    pub fn is_container(&self, at: TilePos) -> bool {
        self.defs.container(self.grid.tile(at).fixture).is_some()
    }

    /// A container's contents from its loot table: a function of the seed and its place.
    fn fill(&self, at: TilePos, table: u16) -> Vec<Item> {
        let Some(t) = self.defs.loot.get(usize::from(table)) else {
            return Vec::new();
        };
        let key = place_key(at);
        let r = |k: u64, why| roll(self.seed, key, k, why);
        let (lo, hi) = t.rolls;
        let draws = lo + below(r(0, Purpose::Loot), hi.saturating_sub(lo) + 1);
        let total: u32 = t.entries.iter().map(|e| e.weight).sum();
        let mut out = Vec::new();
        if total == 0 {
            return out;
        }
        for k in 1..=u64::from(draws) {
            let mut pick = below(r(k, Purpose::Loot), total);
            let Some(entry) = t.entries.iter().find(|e| {
                let hit = pick < e.weight;
                pick = pick.saturating_sub(e.weight);
                hit
            }) else {
                continue;
            };
            let (clo, chi) = entry.count;
            let span = u32::from(chi.saturating_sub(clo)) + 1;
            let count = clo + below(r(k, Purpose::LootCount), span) as u16;
            if count > 0 {
                self.defs.stow(&mut out, Item::found(entry.item, count));
            }
        }
        out
    }

    /// The containers and floors `id` can reach, its own tile first, then clockwise from north:
    /// each with its fixture (0 for a bare floor) and what it holds. Floors with nothing on them
    /// are left out; containers aren't, empty or not.
    pub fn stores_in_reach(&self, id: ActorId) -> Vec<(TilePos, u16, Vec<Item>)> {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for at in std::iter::once(a.pos).chain(Dir8::ALL.iter().map(|&d| a.pos.step(d))) {
            if !within_reach(&self.grid, a.pos, at) {
                continue;
            }
            let container = self.is_container(at);
            let held = self.store(at);
            if container || !held.is_empty() {
                let fixture = if container {
                    self.grid.tile(at).fixture
                } else {
                    0
                };
                out.push((at, fixture, held));
            }
        }
        out
    }

    /// The tiles with something lying on the floor (not in a container), in order.
    pub fn floors(&self) -> Vec<TilePos> {
        self.stores
            .iter()
            .filter(|(at, held)| !held.is_empty() && !self.is_container(**at))
            .map(|(&at, _)| at)
            .collect()
    }

    /// Puts an item on a tile's floor (or into its container): for tests, tools and scripted
    /// events.
    pub fn put(&mut self, at: TilePos, item: Item) {
        let mut held = self.store(at);
        self.defs.stow(&mut held, item);
        self.stores.insert(at, held);
    }

    /// What an actor carries, in hand and in its pack, in grams.
    pub fn load(&self, id: ActorId) -> u32 {
        self.actors.get(id).map_or(0, |a| {
            a.items
                .iter()
                .chain(&a.hands)
                .chain(&a.light)
                .map(|i| self.defs.weight(i))
                .sum()
        })
    }

    /// Weighed down: carrying more than a character can and still run.
    pub fn weighed_down(&self, id: ActorId) -> bool {
        self.actors
            .get(id)
            .is_some_and(|a| a.kind == ActorKind::Character)
            && self.load(id) > self.defs.carry
    }

    /// The swing an actor makes: its weapon's, as its skill with it makes it, else bare hands
    /// (for a creature, its own).
    pub(crate) fn swing_of(&self, id: ActorId) -> Swing {
        let c = self.rules.combat;
        let Some(a) = self.actors.get(id) else {
            return c.attack;
        };
        if let Some(creature) = self.creature(a.kind) {
            return creature.attack;
        }
        match a.hands.and_then(|h| self.defs.item(h.def)).map(|d| &d.kind) {
            Some(ItemKind::Weapon { swing, skill, .. }) => {
                let level = skill.map_or(0, |s| self.skill_level(id, s));
                self.rules.skills.swing(*swing, level)
            }
            _ => c.attack,
        }
    }

    /// A blow landed: the weapon in hand wears (less in skilled hands), and breaks at 0.
    pub(crate) fn wear(&mut self, id: ActorId) {
        let defs = self.defs.clone();
        let level = self.weapon_skill(id).map_or(0, |s| self.skill_level(id, s));
        let rules = self.rules.skills;
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let Some(mut held) = a.hands else { return };
        let Some(ItemKind::Weapon { wear, .. }) = defs.item(held.def).map(|d| &d.kind) else {
            return;
        };
        held.condition = held.condition.saturating_sub(rules.wear(*wear, level));
        if held.condition == 0 {
            a.hands = None;
            self.events.push(Event::WeaponBroke {
                who: id,
                item: held.def,
            });
        } else {
            a.hands = Some(held);
        }
    }

    /// Takes the stack at `slot` of what's on `at` (it must still be `def`, and within reach).
    pub(crate) fn take(&mut self, id: ActorId, at: TilePos, slot: u16, def: u16) {
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        if !within_reach(&self.grid, pos, at) {
            return;
        }
        let mut held = self.store(at);
        let slot = usize::from(slot);
        if held.get(slot).is_none_or(|i| i.def != def) {
            return;
        }
        let item = held.remove(slot);
        self.stores.insert(at, held);
        let defs = self.defs.clone();
        if let Some(a) = self.actors.get_mut(id) {
            defs.stow(&mut a.items, item);
        }
        self.events.push(Event::Took {
            who: id,
            item: def,
            count: item.count,
        });
    }

    /// Drops the stack at `slot` onto the actor's own tile.
    pub(crate) fn drop_item(&mut self, id: ActorId, slot: u16, def: u16) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let slot = usize::from(slot);
        if a.items.get(slot).is_none_or(|i| i.def != def) {
            return;
        }
        let (item, pos) = (a.items.remove(slot), a.pos);
        self.put(pos, item);
        self.events.push(Event::Dropped {
            who: id,
            item: def,
            count: item.count,
        });
    }

    /// Takes the weapon at `slot` in hand; what was in hand goes into the pack.
    pub(crate) fn wield(&mut self, id: ActorId, slot: u16, def: u16) {
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let slot = usize::from(slot);
        if a.items.get(slot).is_none_or(|i| i.def != def) {
            return;
        }
        let mut item = a.items.remove(slot);
        if item.count > 1 {
            // One in hand, the rest stay in the pack.
            let rest = Item {
                count: item.count - 1,
                ..item
            };
            a.items.insert(slot, rest);
            item.count = 1;
        }
        if let Some(was) = a.hands.replace(item) {
            defs.stow(&mut a.items, was);
        }
        self.events.push(Event::Wielded { who: id, item: def });
    }

    /// Puts what's in hand into the pack.
    pub(crate) fn unwield(&mut self, id: ActorId) {
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if let Some(was) = a.hands.take() {
            defs.stow(&mut a.items, was);
        }
    }

    /// Uses the item at `slot`: a bandage goes on the worst wound that's bleeding; food is eaten
    /// and drink drunk; the cure is taken for a fever. One of the stack is used up.
    pub(crate) fn use_item(&mut self, id: ActorId, slot: u16, def: u16) {
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let slot = usize::from(slot);
        if a.items.get(slot).is_none_or(|i| i.def != def) {
            return;
        }
        let event = match defs.item(def).map(|d| &d.kind) {
            Some(ItemKind::Bandage { .. }) => {
                let Some(wound) = a
                    .body
                    .wounds
                    .iter_mut()
                    .filter(|w| w.bleeding > 0)
                    .max_by_key(|w| (w.kind, w.bleeding))
                else {
                    return;
                };
                wound.bleeding = 0;
                wound.bandaged = true;
                Event::Bandaged {
                    who: id,
                    part: wound.part,
                }
            }
            Some(ItemKind::Food { nutrition, .. }) => {
                a.body.hunger = (a.body.hunger - nutrition).max(0);
                Event::Ate { who: id, item: def }
            }
            Some(ItemKind::Drink { water, .. }) => {
                a.body.thirst = (a.body.thirst - water).max(0);
                Event::Drank { who: id, item: def }
            }
            Some(ItemKind::Cure { .. }) => {
                if a.body.fever.take().is_none() {
                    return;
                }
                Event::Cured { who: id }
            }
            _ => return,
        };
        a.items[slot].count -= 1;
        if a.items[slot].count == 0 {
            a.items.remove(slot);
        }
        let bandaged = matches!(event, Event::Bandaged { .. });
        self.events.push(event);
        // A wound that was really bleeding: First Aid learns from every one.
        if bandaged {
            self.train(id, Skill::FirstAid, self.rules.skills.bandage, On::Once);
        }
    }

    /// The ticks it takes to use the item at `slot` (`using`) or take it in hand, if that can
    /// start: it's there (as `def`), a weapon to take in hand, a bandage with a wound bleeding,
    /// food or drink, or the cure with a fever.
    pub(crate) fn item_ticks(&self, id: ActorId, slot: u16, def: u16, using: bool) -> Option<u32> {
        let a = self.actors.get(id)?;
        a.items.get(usize::from(slot)).filter(|i| i.def == def)?;
        match (using, &self.defs.item(def)?.kind) {
            (false, ItemKind::Weapon { .. }) => Some(self.defs.wield_time),
            (true, ItemKind::Bandage { time }) if a.body.bleeding() => {
                let level = a.skills.level(Skill::FirstAid, &self.rules.skills);
                Some(self.rules.skills.bandage(*time, level))
            }
            (true, ItemKind::Food { time, .. } | ItemKind::Drink { time, .. }) => {
                Some((*time).max(1))
            }
            (true, ItemKind::Cure { time }) if a.body.fever.is_some() => Some((*time).max(1)),
            _ => None,
        }
    }

    /// Ticks to take a stack: a moment, and more the heavier it is.
    pub(crate) fn take_ticks(&self, at: TilePos, slot: u16, def: u16) -> Option<u32> {
        let item = *self
            .store(at)
            .get(usize::from(slot))
            .filter(|i| i.def == def)?;
        let kg = self.defs.weight(&item) / 1000;
        Some(self.defs.take + self.defs.take_per_kg * kg)
    }
}
