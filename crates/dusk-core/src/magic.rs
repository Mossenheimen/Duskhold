//! Magic items (`CONCEPT.md`, loot): few, and each one an event.
//!
//! - **Rarity:** Uncommon (one property or two), Rare (three or four, and a made-up name: "Grim
//!   Bite", Path of Exile's), or Unique. The rarer tiers wait for the dungeons.
//! - **Properties are data** (`data/properties.toml`): each reads in one line and changes how
//!   you play. On a weapon, while it's in your hand: quieter swings, harder or surer blows, a
//!   quicker tell, less wear, a glow round you (a trade: the dead see it too), and Hallowed: what
//!   it puts down stays down. On a light, held up: farther, or slower to burn down.
//! - **Numbers, as Diablo's and Path of Exile's are** (the user, 2026-11-14): a property's card
//!   line is its numbers ("+80% damage", "+2 to Strength": `card.rs`) unless it's more than
//!   numbers, and says so in words (Hallowed, a unique's effect). A weapon in hand can make you
//!   stronger or fitter (`Boosts`: `fitness.rs` reads them), give levels in a skill (its own, the
//!   Wizard's Arcana, the Cleric's Devotion: `World::skill_level`), and add fire's harm to its
//!   blows, or holy harm on the dead (`World::elemental`).
//! - **Tiers** (Path of Exile's): most properties are one of a family's tiers ("Sharp", "Keen",
//!   "Cruel": harder blows, each more than the last), and a thing has at most one of a family.
//!   How far out a find was made sets the best tier it can roll (`World::find_level`): near where
//!   you come in, the first; out about the hamlet, the second; far out, the third. A rare's pack
//!   and an old tomb reach a tier further (`LootTable::level`).
//! - **Curses** are properties too, the bad kind. A cursed weapon clings: taken in hand, it
//!   can't be put away or changed for another until its curse is lifted (the abbey does it, for
//!   coin). Its curse shows itself the moment you take it in hand: the hard way.
//! - **Unknown until identified:** a magic item found works from the first, but says only how
//!   rare it is ("a sword (rare, unknown)"), as its colour does. The abbey reads it for coin; or strike enough blows with it in hand and you
//!   know it; a light, you know as soon as you light it.
//! - **Where they come from:** a loot table's line can say how often its item is magic
//!   (`LootEntry::magic`), rolled with the container's seed like everything in it. Rares carry
//!   better.
//! - **Uniques** (`data/uniques.toml`; Diablo's and Path of Exile's): weapons made by hand, each
//!   with a name of its own, its kind and how well it was made, properties from the list, and an
//!   effect of its own no rolled property has: its blows drink, what it puts down burns, a kill
//!   carries on into the next, its blows knock down or put the dead to flight, a kill eases your
//!   dread (`Gifts`). A loot table's line says how often its item is one of the uniques made on
//!   it (`LootEntry::unique`): rares carry them, and the old tombs hold them. Found unknown, as
//!   any magic is, but "(unique, unknown)" says what it is.

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::hash::{Purpose, below, mix64, roll};
use crate::items::{Defs, Item, ItemKind};
use crate::skills::{SKILLS, Skill};
use crate::world::{Event, Swing, World};

/// No property in this slot.
pub const NONE: u8 = u8::MAX;

/// A unique's rarity (`Magic::rarity`).
pub const UNIQUE: u8 = 3;

/// What makes an item magic, if anything. Mundane is rarity 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Magic {
    /// 0 mundane, 1 Uncommon, 2 Rare, 3 Unique (`UNIQUE`).
    pub rarity: u8,
    /// A unique's number in `Defs::uniques` (`NONE`: it isn't one). Its properties are the
    /// unique's, and `props` and `curse` are none.
    pub unique: u8,
    /// Its properties and its curse, by their numbers in `Defs::properties` (`NONE`: none).
    pub props: [u8; 4],
    pub curse: u8,
    /// A Rare's made-up name: its first word and its second, by their numbers in
    /// `Defs::thing_names` (`NONE`: it has none, and is called by its properties).
    pub name: [u8; 2],
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
        unique: NONE,
        props: [NONE; 4],
        curse: NONE,
        name: [NONE; 2],
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

/// A weapon's numbers as its card has them (`Defs::card`; the card itself is `card.rs`'s).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    pub swing: Swing,
    pub skill: Option<Skill>,
    /// The condition a blow takes off it, out of `items::NEW`.
    pub wear: u32,
}

/// The skill a property gives levels in (`Property::levels`): none, the one its weapon trains, or
/// one by name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Raises {
    #[default]
    Nothing,
    /// The skill of the weapon it's on: "+2 to Blade" on a sword, "+2 to Axe" on an axe.
    Own,
    Skill(Skill),
}

/// A property (or a curse): what it does, in the core's units. Per mille is 1000 for none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub id: String,
    /// The word it puts before an item's name, and its line on the card: in words, for what's
    /// more than numbers (Hallowed, a unique's effect); empty, and the card says its numbers
    /// (`card.rs`).
    pub name: String,
    pub line: String,
    /// Its family: a thing has at most one of a family (empty: its own, by its id). And its tier
    /// in it, 1 to 3: a find rolls it only as far out as that reaches (0: a property with no
    /// tiers, found anywhere).
    pub family: String,
    pub tier: u8,
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
    /// A weapon's: per mille the breath a blow costs.
    pub endurance: u32,
    /// What it puts down stays down (`remains.rs`): as good as burning.
    pub hallowed: bool,
    /// Tiles of light: a weapon's glows round you in hand; a light's adds to its reach.
    pub glow: i32,
    /// A light's: per mille how fast it burns down.
    pub wane: u32,
    /// A weapon's, what a unique's effect does (`Gifts`), in hand. A blow that puts one down
    /// carries on into another beside you, as the Cleave talent does.
    pub cleave: bool,
    /// Per mille of the harm a blow does that comes back to you as health.
    pub drain: u32,
    /// The dead it puts down catch fire where they fall (`remains.rs`): they won't get up.
    pub pyre: bool,
    /// Out of 10,000: the chance a blow that lands sends one of the dead fleeing, as turning
    /// does (`faith.rs`).
    pub terror: u32,
    /// Dread eased each time it puts one of the dead down, in thousandths of a point.
    pub hearten: i32,
    /// Out of 10,000: the chance a blow that lands knocks down what it hits, as a heavy blow may.
    pub fell: u32,
    /// A weapon's, in hand (`Boosts`): Strength and Fitness it adds, in thousandths of a level
    /// (`fitness.rs`), and levels in a skill (`World::skill_level`).
    pub strength: u32,
    pub fitness: u32,
    pub raises: Raises,
    pub levels: u8,
    /// A weapon's: the harm its blows add, least and most, in tenths of a point: fire's, on
    /// anything; holy harm, on the dead only (`World::elemental`).
    pub fire: (i32, i32),
    pub holy: (i32, i32),
}

impl Property {
    /// Its family: its own, by its id, when it names none.
    pub fn family_of(&self) -> &str {
        if self.family.is_empty() {
            &self.id
        } else {
            &self.family
        }
    }
}

/// Words for Rares' made-up names (`names.toml`'s `[things]`; Path of Exile's rares): a first
/// word, and a second by what the thing is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThingNames {
    pub first: Vec<String>,
    /// By what it is: a blade, a blunt weapon, an axe, a spear, any other weapon, a light.
    pub second: [Vec<String>; 6],
}

/// A unique (`data/uniques.toml`): a weapon made by hand, with a name of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unique {
    pub id: String,
    pub name: String,
    /// The kind of weapon it is (`Defs::items`), and how well it was made (`quality.rs`).
    pub item: u16,
    pub quality: u8,
    /// Its properties, by their numbers in `Defs::properties`: any weapon's, a curse's among them
    /// as a cost it carries (it doesn't cling: there's no curse on it).
    pub props: Vec<u8>,
    /// Its own effect, read as a property is: a name, a line, and what it does.
    pub effect: Property,
    /// A line of its story, on its card once you know it.
    pub lore: String,
}

/// What the weapon in a hand gives a blow past its swing, by its properties (a unique's effect):
/// read as a blow begins, since the blow may break it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gifts {
    pub cleave: bool,
    pub drain: u32,
    pub pyre: bool,
    pub terror: u32,
    pub hearten: i32,
    pub fell: u32,
    /// Fire's harm and holy harm, least and most, in tenths of a point.
    pub fire: (i32, i32),
    pub holy: (i32, i32),
}

/// What a weapon gives whoever holds it, all its properties together, known or not (Diablo's "+2
/// to Strength", "+1 to a skill"): Strength and Fitness, in thousandths of a level, and levels by
/// `Skill::ALL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Boosts {
    pub strength: u32,
    pub fitness: u32,
    pub levels: [u8; SKILLS],
}

impl Default for Property {
    fn default() -> Property {
        Property {
            id: String::new(),
            name: String::new(),
            line: String::new(),
            family: String::new(),
            tier: 0,
            on_light: false,
            curse: false,
            noise: 1000,
            damage: 1000,
            swing: 1000,
            hit: 0,
            wear: 1000,
            endurance: 1000,
            hallowed: false,
            glow: 0,
            wane: 1000,
            cleave: false,
            drain: 0,
            pyre: false,
            terror: 0,
            hearten: 0,
            fell: 0,
            strength: 0,
            fitness: 0,
            raises: Raises::Nothing,
            levels: 0,
            fire: (0, 0),
            holy: (0, 0),
        }
    }
}

/// How magic is rolled and learnt (rules.toml's `[magic]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MagicRules {
    /// Out of 10,000: of the magic items, how many are Rare; and how many are cursed.
    pub rare: u32,
    pub cursed: u32,
    /// How many properties an Uncommon has (the fewest and the most), and a Rare.
    pub count: [(u8, u8); 2],
    /// Tiles from where you come in at which a find reaches the second tier, and the third.
    pub far: [u32; 2],
    /// The worth a magic item adds to its kind's, by rarity (Uncommon, Rare, Unique), in
    /// thousandths.
    pub worth: [u32; 3],
    /// Blows struck with an unknown magic weapon before you know what it is.
    pub learn: u8,
}

impl Default for MagicRules {
    fn default() -> MagicRules {
        MagicRules {
            rare: 3000,
            cursed: 2000,
            count: [(1, 2), (3, 4)],
            far: [60, 140],
            worth: [20_000, 50_000, 150_000],
            learn: 15,
        }
    }
}

fn per(n: u32, by: u32) -> u32 {
    (u64::from(n) * u64::from(by) / 1000) as u32
}

impl Defs {
    /// The unique an item is, if it's one.
    pub fn unique_of(&self, item: &Item) -> Option<&Unique> {
        match item.magic.unique {
            NONE => None,
            u => self.uniques.get(usize::from(u)),
        }
    }

    /// An item's properties (its curse among them; a unique's, and its effect last), as
    /// definitions.
    pub fn properties_of(&self, item: &Item) -> impl Iterator<Item = &Property> + '_ {
        let effect = self.unique_of(item).map(|u| &u.effect);
        self.rolled_of(item).chain(effect)
    }

    /// An item's properties but a unique's effect: a unique's own, then what was rolled, then
    /// its curse.
    pub fn rolled_of(&self, item: &Item) -> impl Iterator<Item = &Property> + '_ {
        let fixed = self.unique_of(item).map_or(&[][..], |u| u.props.as_slice());
        fixed
            .iter()
            .copied()
            .chain(item.magic.all())
            .filter_map(|p| self.properties.get(usize::from(p)))
    }

    /// The properties of an item you know: all of them once it's known, else only a curse that
    /// has shown itself.
    pub fn known_of(&self, item: &Item) -> impl Iterator<Item = &Property> + '_ {
        let m = item.magic;
        self.properties_of(item)
            .filter(move |p| m.known || (p.curse && m.curse_shown))
    }

    /// What a weapon gives whoever holds it (`Boosts`): its properties' Strength, Fitness and
    /// levels, known or not, as its blows' are. Its own skill is the one it trains.
    pub fn boosts_of(&self, item: &Item) -> Boosts {
        let own = match self.item(item.def).map(|d| &d.kind) {
            Some(ItemKind::Weapon { skill, .. }) => *skill,
            _ => None,
        };
        let mut b = Boosts::default();
        for p in self.properties_of(item) {
            b.strength += p.strength;
            b.fitness += p.fitness;
            let skill = match p.raises {
                Raises::Nothing => None,
                Raises::Own => own,
                Raises::Skill(s) => Some(s),
            };
            if let Some(s) = skill {
                let n = &mut b.levels[s as usize];
                *n = n.saturating_add(p.levels);
            }
        }
        b
    }

    /// The uniques made on a kind of item, by their numbers.
    pub fn uniques_on(&self, def: u16) -> Vec<u8> {
        (0..self.uniques.len().min(usize::from(NONE)))
            .filter(|&u| self.uniques[u].item == def)
            .map(|u| u as u8)
            .collect()
    }

    /// A unique as it's found: made as the data says, a find, unknown.
    pub fn unique_item(&self, u: u8) -> Option<Item> {
        let made = self.uniques.get(usize::from(u))?;
        Some(Item {
            quality: made.quality,
            magic: Magic {
                rarity: UNIQUE,
                unique: u,
                known: false,
                curse_shown: false,
                ..Magic::MUNDANE
            },
            ..Item::found(made.item, 1)
        })
    }

    /// What an item is worth: its kind's, as well or badly as it was made, and more for magic.
    pub fn worth(&self, item: &Item) -> u32 {
        let base = per(
            self.item(item.def).map_or(0, |d| d.worth),
            self.quality.worth_of(item.quality),
        );
        let more = match item.magic.rarity {
            0 => 0,
            r => self.magic.worth[usize::from(r.min(UNIQUE)) - 1],
        };
        base + more
    }

    /// A magic item's rarity in a word: "Uncommon", "Rare" or "Unique".
    pub fn rarity_word(&self, item: &Item) -> &'static str {
        match item.magic.rarity {
            _ if self.unique_of(item).is_some() => "Unique",
            1 => "Uncommon",
            _ => "Rare",
        }
    }

    /// What a thing is, for a Rare's made-up name: a blade, a blunt weapon, an axe, a spear, any
    /// other weapon, or a light (`ThingNames::second`); None for anything else.
    fn name_class(&self, def: u16) -> Option<usize> {
        match &self.item(def)?.kind {
            ItemKind::Weapon { skill, .. } => Some(match skill {
                Some(Skill::Blade) => 0,
                Some(Skill::Blunt) => 1,
                Some(Skill::Axe) => 2,
                Some(Skill::Spear) => 3,
                _ => 4,
            }),
            ItemKind::Light { .. } => Some(5),
            _ => None,
        }
    }

    /// A Rare's made-up name for a kind of thing, by a roll's keys (14 and 15): its words'
    /// numbers, or none with no words to make one from.
    pub fn roll_name(&self, def: u16, r: impl Fn(u64) -> u64) -> [u8; 2] {
        let words = &self.thing_names;
        let Some(second) = self.name_class(def).map(|c| &words.second[c]) else {
            return [NONE; 2];
        };
        if words.first.is_empty() || second.is_empty() {
            return [NONE; 2];
        }
        let pick = |k: u64, n: usize| below(r(k), n.min(usize::from(NONE)) as u32) as u8;
        [pick(14, words.first.len()), pick(15, second.len())]
    }

    /// A Rare's made-up name, if it has one: "Grim Bite".
    pub fn thing_name(&self, item: &Item) -> Option<String> {
        let [a, b] = item.magic.name;
        if a == NONE || b == NONE {
            return None;
        }
        let words = &self.thing_names;
        let first = words.first.get(usize::from(a))?;
        let second = words.second[self.name_class(item.def)?].get(usize::from(b))?;
        Some(format!("{first} {second}"))
    }

    /// What a thing is, and how well it was made: "cudgel", "fine sword".
    pub fn made_name(&self, item: &Item) -> String {
        let kind = self.item(item.def).map_or("something", |d| d.name.as_str());
        match self.quality.name_of(item.quality) {
            "" => kind.to_string(),
            made => format!("{made} {kind}"),
        }
    }

    /// A known Rare's or unique's own name: "Grim Edge", "Redhand". None while it's unknown, and
    /// for anything else.
    pub fn own_name(&self, item: &Item) -> Option<String> {
        if !item.magic.known {
            return None;
        }
        match self.unique_of(item) {
            Some(u) => Some(u.name.clone()),
            None => self.thing_name(item),
        }
    }

    /// What an item is called, as far as you know it: "cudgel", "cudgel (uncommon, unknown)",
    /// "Muffled Keen cudgel", with its curse's word once it's shown; a Rare's, "fine sword (rare,
    /// unknown)", then "Grim Edge (fine sword)"; a unique's, "fine hand axe (unique, unknown)",
    /// then "Redhand (fine hand axe)".
    pub fn label(&self, item: &Item) -> String {
        // How well it was made goes first: "fine sword", "Keen fine sword".
        let name = self.made_name(item);
        let m = item.magic;
        if !m.is_magic() {
            return name;
        }
        if !m.known {
            let what = self.rarity_word(item).to_lowercase();
            let curse = if m.cursed() && m.curse_shown {
                self.properties
                    .get(usize::from(m.curse))
                    .map_or(String::new(), |p| format!("{} ", p.name))
            } else {
                String::new()
            };
            return format!("{curse}{name} ({what}, unknown)");
        }
        if let Some(own) = self.own_name(item) {
            return format!("{own} ({name})");
        }
        let words: Vec<&str> = self.properties_of(item).map(|p| p.name.as_str()).collect();
        format!("{} {name}", words.join(" "))
    }

    /// A weapon's swing as it was made (its quality's harm) and as its magic changes it: every
    /// property it has, or (`known_only`) those you know, as its card shows them (a curse once it
    /// has shown itself).
    pub fn made_swing(&self, item: &Item, mut swing: Swing, known_only: bool) -> Swing {
        let made = self.quality.damage_of(item.quality);
        swing.damage = (
            per(swing.damage.0.max(0) as u32, made) as i32,
            per(swing.damage.1.max(0) as u32, made) as i32,
        );
        let m = item.magic;
        for p in self.properties_of(item) {
            if known_only && !m.known && !(p.curse && m.curse_shown) {
                continue;
            }
            swing.noise = per(swing.noise.max(0) as u32, p.noise) as i32;
            swing.damage = (
                per(swing.damage.0.max(0) as u32, p.damage) as i32,
                per(swing.damage.1.max(0) as u32, p.damage) as i32,
            );
            swing.wind_up = per(swing.wind_up, p.swing).max(1);
            swing.recovery = per(swing.recovery, p.swing);
            swing.hit = (swing.hit as i32 + p.hit).clamp(0, 10_000) as u32;
            swing.endurance = per(swing.endurance.max(0) as u32, p.endurance) as i32;
        }
        swing
    }

    /// How fast a weapon wears, per mille: by its quality and its properties (`known_only`: those
    /// you know, as on its card).
    pub fn made_wear(&self, item: &Item, known_only: bool) -> u32 {
        let m = item.magic;
        self.properties_of(item)
            .filter(|p| !known_only || m.known || (p.curse && m.curse_shown))
            .fold(self.quality.wear_of(item.quality), |w, p| per(w, p.wear))
    }

    /// A weapon's numbers as its card has them (`card.rs` writes the card): its swing as made and
    /// as far as you know its magic, the skill it trains, and the condition a blow takes off it
    /// (out of `items::NEW`), before skill makes any of it better. None for what isn't a weapon.
    pub fn card(&self, item: &Item) -> Option<Card> {
        let Some(ItemKind::Weapon { swing, wear, skill }) = self.item(item.def).map(|d| &d.kind)
        else {
            return None;
        };
        Some(Card {
            swing: self.made_swing(item, *swing, true),
            skill: *skill,
            wear: per(u32::from(*wear), self.made_wear(item, true)),
        })
    }

    /// A magic item for a loot table's line, by a roll's keys, found where it reaches tier
    /// `level` (1 to 3): its rarity, its properties (that fit its kind, at most one of a family,
    /// each of a tier it reaches), maybe a curse, and a Rare's made-up name. Unknown.
    pub(crate) fn roll_magic(&self, item: &Item, level: u8, r: impl Fn(u64) -> u64) -> Magic {
        let on_light = matches!(
            self.item(item.def).map(|d| &d.kind),
            Some(ItemKind::Light { .. })
        );
        let fits = |curse: bool| -> Vec<u8> {
            (0..self.properties.len().min(usize::from(NONE)))
                .filter(|&k| {
                    let p = &self.properties[k];
                    p.on_light == on_light && p.curse == curse && p.tier <= level
                })
                .map(|k| k as u8)
                .collect()
        };
        let mut good = fits(false);
        if good.is_empty() {
            return Magic::MUNDANE;
        }
        let rare = below(r(1), 10_000) < self.magic.rare;
        let (fewest, most) = self.magic.count[usize::from(rare)];
        let count = fewest + below(r(9), u32::from(most.saturating_sub(fewest)) + 1) as u8;
        let mut magic = Magic {
            rarity: if rare { 2 } else { 1 },
            known: false,
            curse_shown: false,
            ..Magic::MUNDANE
        };
        // A family first, then one of its tiers it reaches: so a family of three tiers comes up
        // no more often than a property on its own.
        let family = |k: u8| self.properties[usize::from(k)].family_of();
        let keys = [(2, 10), (3, 11), (7, 12), (8, 13)];
        for (slot, (fk, tk)) in keys.into_iter().enumerate().take(usize::from(count)) {
            let mut families: Vec<&str> = good.iter().map(|&k| family(k)).collect();
            families.sort_unstable();
            families.dedup();
            if families.is_empty() {
                break;
            }
            let f = families[below(r(fk), families.len() as u32) as usize];
            let tiers: Vec<u8> = good.iter().copied().filter(|&k| family(k) == f).collect();
            magic.props[slot] = tiers[below(r(tk), tiers.len() as u32) as usize];
            good.retain(|&k| family(k) != f);
        }
        let curses = fits(true);
        if !curses.is_empty() && below(r(5), 10_000) < self.magic.cursed {
            magic.curse = curses[below(r(6), curses.len() as u32) as usize];
        }
        if rare {
            magic.name = self.roll_name(item.def, &r);
        }
        magic
    }
}

impl World {
    /// A swing as the weapon was made and as its properties change it, all of them.
    pub(crate) fn magic_swing(&self, item: &Item, swing: Swing) -> Swing {
        self.defs.made_swing(item, swing, false)
    }

    /// How fast a weapon wears, per mille, by its quality and its properties.
    pub(crate) fn magic_wear(&self, item: &Item) -> u32 {
        self.defs.made_wear(item, false)
    }

    /// The best tier of property a find at `at` can roll, 1 to 3: by how far it is from where you
    /// come in (`MagicRules::far`), and `more` for what holds it (`LootTable::level`).
    pub fn find_level(&self, at: TilePos, more: u8) -> u8 {
        let from = self.region.start.unwrap_or(TilePos::new(0, 0, at.z));
        let d2 = at.dist2(from);
        let far = self.defs.magic.far;
        let past = far
            .iter()
            .filter(|&&f| d2 >= i64::from(f) * i64::from(f))
            .count() as u8;
        (1 + past).saturating_add(more).min(3)
    }

    /// What the weapon in `id`'s hand gives a blow past its swing, all its properties together.
    pub(crate) fn gifts(&self, id: ActorId) -> Gifts {
        let Some(held) = self.actors.get(id).and_then(|a| a.hands) else {
            return Gifts::default();
        };
        let add = |a: (i32, i32), b: (i32, i32)| (a.0 + b.0, a.1 + b.1);
        self.defs
            .properties_of(&held)
            .fold(Gifts::default(), |g, p| Gifts {
                cleave: g.cleave || p.cleave,
                drain: g.drain + p.drain,
                pyre: g.pyre || p.pyre,
                terror: (g.terror + p.terror).min(10_000),
                hearten: g.hearten + p.hearten,
                fell: (g.fell + p.fell).min(10_000),
                fire: add(g.fire, p.fire),
                holy: add(g.holy, p.holy),
            })
    }

    /// What the weapon in `id`'s hand gives it (`Boosts`): Strength, Fitness, levels in skills.
    pub fn boosts(&self, id: ActorId) -> Boosts {
        self.actors
            .get(id)
            .and_then(|a| a.hands)
            .map_or(Boosts::default(), |held| self.defs.boosts_of(&held))
    }

    /// The fire's harm and the holy harm a blow from `id` on `target` adds, by what its weapon
    /// gives (`Gifts`): each rolled from its least to its most. Holy harm is the dead's only: a
    /// beast takes none.
    pub(crate) fn elemental(&self, id: ActorId, target: ActorId, g: Gifts) -> (i32, i32) {
        let r = roll(self.seed, self.tick, id.index() as u64, Purpose::Element);
        let pick = |(lo, hi): (i32, i32), r: u64| {
            if hi <= 0 {
                0
            } else {
                lo + below(r, (hi - lo).max(0) as u32 + 1) as i32
            }
        };
        let holy = if self.is_wild(target) {
            0
        } else {
            pick(g.holy, mix64(r))
        };
        (pick(g.fire, r), holy)
    }

    /// A blow from `id` that did `damage` to `target`, by what its weapon gives: some of the harm
    /// comes back as health, and one of the dead may break and flee.
    pub(crate) fn gifted_blow(&mut self, id: ActorId, target: ActorId, damage: i32, g: Gifts) {
        if g.drain > 0 && damage > 0 {
            let most = self.max_health(id);
            let back = per(damage as u32, g.drain) as i32;
            if let Some(a) = self.actors.get_mut(id).filter(|a| !a.dead)
                && a.body.health < most
            {
                a.body.health = (a.body.health + back).min(most);
            }
        }
        let frightens = roll(self.seed, self.tick, id.index() as u64, Purpose::Terror);
        if g.terror > 0 && below(frightens, 10_000) < g.terror {
            self.terrify(id, target);
        }
    }

    /// One of the dead struck with a weapon that frightens: it turns and flees from `by`, as from
    /// the holy symbol, for as long as its kind stays turned (no Devotion in it: `faith.rs`). Not
    /// one the blow put down, nor a beast, nor a kind that can't be turned.
    fn terrify(&mut self, by: ActorId, target: ActorId) {
        let share = self.stats(target).map_or(0, |c| c.turned);
        let standing = self.actors.get(target).is_some_and(|t| {
            !t.dead && t.body.health > 0 && matches!(t.kind, crate::actor::ActorKind::Creature(_))
        });
        if !standing || share == 0 || self.is_wild(target) {
            return;
        }
        let until = self.tick + self.rules.faith.turned * u64::from(share) / 1000;
        self.send_fleeing(target, by, until);
        self.events.push(Event::Terrified { who: by, target });
    }

    /// One of the dead `id` just put down, by what its weapon gives: it catches fire where it
    /// falls, and `id`'s dread eases. Not a beast: a pyre would burn the meat.
    pub(crate) fn gifted_kill(&mut self, id: ActorId, body: ActorId, g: Gifts) {
        if self.is_wild(body) {
            return;
        }
        if g.pyre && self.burnable(body) {
            self.set_alight(body);
            self.events.push(Event::Kindled { who: id, body });
        }
        if g.hearten > 0
            && let Some(a) = self.actors.get_mut(id)
        {
            a.body.dread = (a.body.dread - g.hearten).max(0);
        }
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
