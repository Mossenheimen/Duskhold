//! Magic items (`CONCEPT.md`, loot): few, and each one an event.
//!
//! - **Rarity:** Uncommon (one property) or Rare (two). The rarer tiers wait for the dungeons.
//! - **Properties are data** (`data/properties.toml`): each reads in one line and changes how
//!   you play. On a weapon, while it's in your hand: quieter swings, harder or surer blows, a
//!   quicker tell, less wear, a glow round you (a trade: the dead see it too), and Hallowed: what
//!   it puts down stays down. On a light, held up: farther, or slower to burn down.
//! - **Curses** are properties too, the bad kind. A cursed weapon clings: taken in hand, it
//!   can't be put away or changed for another until its curse is lifted (the abbey does it, for
//!   coin). Its curse shows itself the moment you take it in hand: the hard way.
//! - **Unknown until identified:** a magic item found works from the first, but says only that
//!   it's magic. The abbey reads it for coin; or strike enough blows with it in hand and you
//!   know it; a light, you know as soon as you light it.
//! - **Where they come from:** a loot table's line can say how often its item is magic
//!   (`LootEntry::magic`), rolled with the container's seed like everything in it. Rares carry
//!   better.

use crate::actor::ActorId;
use crate::hash::{Purpose, below, roll};
use crate::items::{Defs, Item, ItemKind};
use crate::world::{Event, Swing, World};

/// No property in this slot.
pub const NONE: u8 = u8::MAX;

/// What makes an item magic, if anything. Mundane is rarity 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Magic {
    /// 0 mundane, 1 Uncommon, 2 Rare.
    pub rarity: u8,
    /// Its properties and its curse, by their numbers in `Defs::properties` (`NONE`: none).
    pub props: [u8; 2],
    pub curse: u8,
    /// Whether you know what it is (its properties and any curse).
    pub known: bool,
    /// Whether its curse has shown itself (it clung to your hand).
    pub curse_shown: bool,
    /// Blows struck with it while unknown: `MagicRules::learn` of them, and you know it.
    pub uses: u8,
}

impl Magic {
    pub const MUNDANE: Magic = Magic {
        rarity: 0,
        props: [NONE; 2],
        curse: NONE,
        known: true,
        curse_shown: true,
        uses: 0,
    };

    pub fn is_magic(self) -> bool {
        self.rarity > 0
    }

    pub fn cursed(self) -> bool {
        self.curse != NONE
    }

    /// Its properties, its curse among them.
    pub fn all(self) -> impl Iterator<Item = u8> {
        self.props
            .into_iter()
            .chain(std::iter::once(self.curse))
            .filter(|&p| p != NONE)
    }
}

impl Default for Magic {
    fn default() -> Magic {
        Magic::MUNDANE
    }
}

/// A property (or a curse): what it does, in the core's units. Per mille is 1000 for none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub id: String,
    /// The word it puts before an item's name, and its one line.
    pub name: String,
    pub line: String,
    /// On a light (held up); else on a weapon (in hand).
    pub on_light: bool,
    pub curse: bool,
    /// A weapon's: per mille its swing's noise, harm, and wind-up and recovery; added to its
    /// chance to land (out of 10,000); per mille its wear.
    pub noise: u32,
    pub damage: u32,
    pub swing: u32,
    pub hit: i32,
    pub wear: u32,
    /// What it puts down stays down (`remains.rs`): as good as burning.
    pub hallowed: bool,
    /// Tiles of light: a weapon's glows round you in hand; a light's adds to its reach.
    pub glow: i32,
    /// A light's: per mille how fast it burns down.
    pub wane: u32,
}

impl Default for Property {
    fn default() -> Property {
        Property {
            id: String::new(),
            name: String::new(),
            line: String::new(),
            on_light: false,
            curse: false,
            noise: 1000,
            damage: 1000,
            swing: 1000,
            hit: 0,
            wear: 1000,
            hallowed: false,
            glow: 0,
            wane: 1000,
        }
    }
}

/// How magic is rolled and learnt (rules.toml's `[magic]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MagicRules {
    /// Out of 10,000: of the magic items, how many are Rare; and how many are cursed.
    pub rare: u32,
    pub cursed: u32,
    /// The worth a magic item adds to its kind's, by rarity (Uncommon, Rare), in thousandths.
    pub worth: [u32; 2],
    /// Blows struck with an unknown magic weapon before you know what it is.
    pub learn: u8,
}

impl Default for MagicRules {
    fn default() -> MagicRules {
        MagicRules {
            rare: 3000,
            cursed: 2000,
            worth: [20_000, 50_000],
            learn: 15,
        }
    }
}

fn per(n: u32, by: u32) -> u32 {
    (u64::from(n) * u64::from(by) / 1000) as u32
}

impl Defs {
    /// An item's properties (its curse among them), as definitions.
    pub fn properties_of(&self, item: &Item) -> impl Iterator<Item = &Property> + '_ {
        item.magic
            .all()
            .filter_map(|p| self.properties.get(usize::from(p)))
    }

    /// What an item is worth: its kind's, and more for magic.
    pub fn worth(&self, item: &Item) -> u32 {
        let base = self.item(item.def).map_or(0, |d| d.worth);
        let more = match item.magic.rarity {
            0 => 0,
            r => self.magic.worth[usize::from(r.min(2)) - 1],
        };
        base + more
    }

    /// What an item is called, as far as you know it: "cudgel", "cudgel (magic, unknown)",
    /// "Muffled Keen cudgel", with its curse's word once it's shown.
    pub fn label(&self, item: &Item) -> String {
        let name = self.item(item.def).map_or("something", |d| d.name.as_str());
        let m = item.magic;
        if !m.is_magic() {
            return name.to_string();
        }
        if !m.known {
            let curse = if m.cursed() && m.curse_shown {
                self.properties
                    .get(usize::from(m.curse))
                    .map_or(String::new(), |p| format!("{} ", p.name))
            } else {
                String::new()
            };
            return format!("{curse}{name} (magic, unknown)");
        }
        let words: Vec<&str> = self.properties_of(item).map(|p| p.name.as_str()).collect();
        format!("{} {name}", words.join(" "))
    }

    /// An item's lines, as far as you know it: its rarity, and what each property does.
    pub fn lines(&self, item: &Item) -> Vec<String> {
        let m = item.magic;
        if !m.is_magic() {
            return Vec::new();
        }
        let rarity = if m.rarity >= 2 { "Rare" } else { "Uncommon" };
        let mut out = vec![if m.known {
            rarity.to_string()
        } else {
            "Magic: what it does, you don't know yet".to_string()
        }];
        for p in self.properties_of(item) {
            if m.known || (p.curse && m.curse_shown) {
                out.push(format!("{}: {}", p.name, p.line));
            }
        }
        out
    }

    /// A magic item for a loot table's line, by a roll's key: its rarity, its properties (that
    /// fit its kind), and maybe a curse. Unknown.
    pub(crate) fn roll_magic(&self, item: &Item, r: impl Fn(u64) -> u64) -> Magic {
        let on_light = matches!(
            self.item(item.def).map(|d| &d.kind),
            Some(ItemKind::Light { .. })
        );
        let fits = |curse: bool| -> Vec<u8> {
            (0..self.properties.len().min(255))
                .filter(|&k| {
                    let p = &self.properties[k];
                    p.on_light == on_light && p.curse == curse
                })
                .map(|k| k as u8)
                .collect()
        };
        let mut good = fits(false);
        if good.is_empty() {
            return Magic::MUNDANE;
        }
        let rare = below(r(1), 10_000) < self.magic.rare;
        let mut magic = Magic {
            rarity: if rare { 2 } else { 1 },
            known: false,
            curse_shown: false,
            ..Magic::MUNDANE
        };
        for (slot, k) in (0..usize::from(magic.rarity)).zip(2u64..) {
            if good.is_empty() {
                break;
            }
            let pick = below(r(k), good.len() as u32) as usize;
            magic.props[slot] = good.remove(pick);
        }
        let curses = fits(true);
        if !curses.is_empty() && below(r(5), 10_000) < self.magic.cursed {
            magic.curse = curses[below(r(6), curses.len() as u32) as usize];
        }
        magic
    }
}

impl World {
    /// A swing as the weapon's properties change it.
    pub(crate) fn magic_swing(&self, item: &Item, mut swing: Swing) -> Swing {
        for p in self.defs.properties_of(item) {
            swing.noise = per(swing.noise.max(0) as u32, p.noise) as i32;
            swing.damage = (
                per(swing.damage.0.max(0) as u32, p.damage) as i32,
                per(swing.damage.1.max(0) as u32, p.damage) as i32,
            );
            swing.wind_up = per(swing.wind_up, p.swing).max(1);
            swing.recovery = per(swing.recovery, p.swing);
            swing.hit = (swing.hit as i32 + p.hit).clamp(0, 10_000) as u32;
        }
        swing
    }

    /// How fast a weapon wears, per mille, by its properties.
    pub(crate) fn magic_wear(&self, item: &Item) -> u32 {
        self.defs
            .properties_of(item)
            .fold(1000, |w, p| per(w, p.wear))
    }

    /// Whether the weapon in `id`'s hand is Hallowed: what it puts down stays down.
    pub(crate) fn hallowed_hand(&self, id: ActorId) -> bool {
        let Some(held) = self.actors.get(id).and_then(|a| a.hands) else {
            return false;
        };
        self.defs.properties_of(&held).any(|p| p.hallowed)
    }

    /// Whether the weapon in `id`'s hand clings: cursed.
    pub fn hand_clings(&self, id: ActorId) -> bool {
        self.actors
            .get(id)
            .and_then(|a| a.hands)
            .is_some_and(|h| h.magic.cursed())
    }

    /// A blow struck with the weapon in hand: unknown, it's learnt a blow at a time.
    pub(crate) fn learn_by_use(&mut self, id: ActorId) {
        let learn = self.defs.magic.learn;
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let Some(held) = a.hands.as_mut() else { return };
        if !held.magic.is_magic() || held.magic.known {
            return;
        }
        held.magic.uses = held.magic.uses.saturating_add(1);
        if held.magic.uses >= learn {
            held.magic.known = true;
            held.magic.curse_shown = true;
            let item = held.def;
            self.events.push(Event::Identified { who: id, item });
        }
    }

    /// A weapon just taken in hand: a curse shows itself, and it clings.
    pub(crate) fn cling(&mut self, id: ActorId) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let Some(held) = a.hands.as_mut() else { return };
        if held.magic.cursed() && !held.magic.curse_shown {
            held.magic.curse_shown = true;
            let item = held.def;
            self.events.push(Event::CurseClings { who: id, item });
        }
    }

    /// A light just lit: you see what it is.
    pub(crate) fn know_light(&mut self, id: ActorId) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let Some(lit) = a.light.as_mut() else { return };
        if lit.magic.is_magic() && !lit.magic.known {
            lit.magic.known = true;
            lit.magic.curse_shown = true;
            let item = lit.def;
            self.events.push(Event::Identified { who: id, item });
        }
    }

    /// A key for rolling an item's magic: so two containers' rolls differ, and two lines'.
    pub(crate) fn magic_roll(&self, key: u64, line: u64) -> impl Fn(u64) -> u64 + '_ {
        let seed = self.seed;
        move |k| roll(seed, key, line * 16 + k, Purpose::Magic)
    }
}
