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
//! - **Armour too** (2026-11-15): a piece is made well or badly as a weapon is (its quality adds
//!   to its chance to turn a blow, and changes its wear), and its properties (`Fits::Armour`)
//!   turn more blows, warm more, wear slower, and make whoever wears it stronger, fitter, or
//!   better at a skill (`Boosts`, as a weapon's in hand). Put on, it shows what it does, as a
//!   light lit does. Its curses are costs it carries (louder footsteps, quicker wear): they don't
//!   cling, as a weapon's do. A Rare piece has a made-up name too ("Grim Shroud").
//! - **Bows and shields too** (2026-11-15), each with properties of its own (`Fits::Bow`,
//!   `Fits::Shield`; Path of Exile's mods by item class). A bow's quality changes its shots' harm,
//!   and its properties make them harder, quicker drawn, surer, farther (`Property::range`), its
//!   arrows break less often (`arrows`), add fire's harm, and give levels in Archery: a bow is
//!   learnt by shots that land, as a weapon by blows, and a cursed one clings to your hand. A
//!   shield's quality adds to its block as armour's to its turning; its properties block more
//!   often (`block`), bash harder (`bash`), cost less breath a block, wear slower, and give
//!   levels in Shield. Taken up on the arm, it shows what it does, as armour put on; its curses
//!   don't cling.
//! - **Tiers** (Path of Exile's): most properties are one of a family's tiers ("Sharp", "Keen",
//!   "Cruel": harder blows, each more than the last), and a thing has at most one of a family.
//!   How far out a find was made sets the best tier it can roll (`World::find_level`): near where
//!   you come in, the first; out about the hamlet, the second; far out, the third. A rare's pack
//!   and an old tomb reach a tier further (`LootTable::level`).
//! - **Rolled within its tier** (Path of Exile's ranges, 2026-11-15): a tiered property's numbers
//!   are its tier's middle, and each one found rolls where it falls, a fifth either way
//!   (`MagicRules::spread`: Keen is +40% to +60% damage, Cruel +64% to +96%), kept with the thing
//!   (`Magic::rolls`). Its card gives the range after each number that rolls: "+56(40–60)%
//!   damage". Tiles, levels and Strength don't roll, nor a curse, nor a unique's numbers.
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
//! - **Uniques on armour, bows and shields too** (2026-11-15), each effect its kind's: a bow's
//!   work through its arrows as a weapon's through its blows (what they put down burns; one that
//!   pierces flies on past the one it put down, `archery.rs`); a shield's through the blows it
//!   blocks (harm back on whatever struck, the dead put to flight or knocked down:
//!   `World::gifted_block`); armour's through the blows that land on you (thorns:
//!   `World::thorns`) or on your mind (a piece that calms slows your dread's rise:
//!   `World::worn_calm`).

use std::borrow::Cow;

use crate::actor::ActorId;
use crate::archery::Ranged;
use crate::armour::Armour;
use crate::geom::TilePos;
use crate::hash::{Purpose, below, mix64, roll};
use crate::items::{Defs, Item, ItemKind};
use crate::shield::Guard;
use crate::skills::{SKILLS, Skill};
use crate::world::{Event, Swing, World};

/// No property in this slot.
pub const NONE: u8 = u8::MAX;

/// A unique's rarity (`Magic::rarity`).
pub const UNIQUE: u8 = 3;

/// Where a property's numbers fall in its tier's range (`Magic::rolls`): 0 its least, `MIDDLE`
/// its data's own, `MOST` its most.
pub const MIDDLE: u8 = 100;
pub const MOST: u8 = 200;

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
    /// Where each property's numbers fell in its tier's range, 0 to `MOST` (`MIDDLE`: its data's
    /// own). A property with no tiers doesn't roll, nor a curse.
    pub rolls: [u8; 4],
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
        rolls: [MIDDLE; 4],
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

/// What kind of thing a property goes on, and works on: a weapon in hand, a light held up, armour
/// worn, a bow in hand or a shield on the arm. Each has its own (Path of Exile's: an item class's
/// own mods).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fits {
    #[default]
    Weapon,
    Light,
    Armour,
    Bow,
    Shield,
}

impl Fits {
    /// What it is, in words: "a weapon", "armour".
    pub fn word(self) -> &'static str {
        match self {
            Fits::Weapon => "a weapon",
            Fits::Light => "a light",
            Fits::Armour => "armour",
            Fits::Bow => "a bow",
            Fits::Shield => "a shield",
        }
    }
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
    /// What it goes on: a weapon (in hand), a light (held up) or armour (worn).
    pub fits: Fits,
    pub curse: bool,
    /// A weapon's: per mille its swing's noise, harm, and wind-up and recovery; added to its
    /// chance to land (out of 10,000); per mille its wear (a weapon's or a piece of armour's).
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
    /// Mana it adds to the most its bearer holds, in thousandths of a point, and per mille how
    /// much faster mana comes back (`World::max_mana`, `World::mana_regen`: `Boosts`).
    pub mana: i32,
    pub mana_regen: u32,
    /// A weapon's: the harm its blows add, least and most, in tenths of a point: fire's, on
    /// anything; holy harm, on the dead only (`World::elemental`).
    pub fire: (i32, i32),
    pub holy: (i32, i32),
    /// Armour's, worn (`Defs::made_armour`): a chance added to turn every kind of blow (out of
    /// 10,000), warmth added (thousandths of a point), and tiles added to every footstep.
    pub ward: u32,
    pub warmth: i32,
    pub steps: i32,
    /// A bow's (`Defs::made_bow`): tiles added to its range, and per mille the chance an arrow
    /// breaks on a hit.
    pub range: i32,
    pub arrows: u32,
    /// A shield's (`Defs::made_guard`): a chance added to block (out of 10,000), and to a bash's
    /// knocking down.
    pub block: u32,
    pub bash: u32,
    /// What a unique's effect does past a weapon's (`Gifts`): harm back, least and most in
    /// tenths of a point, on whatever's blow lands on you (armour's, worn) or whatever's blow you
    /// block (a shield's).
    pub thorns: (i32, i32),
    /// Armour's, worn: per mille how fast your dread rises (`World::dread_rise`).
    pub calm: u32,
}

impl Property {
    /// As rolled within its tier, `by` per mille of the way from none to its data's numbers (800:
    /// a fifth less of each, 1200: a fifth more). What rolls: its multipliers, chances, harms,
    /// warmth and dread; not tiles, levels or Strength, and nothing that's on or off.
    pub fn rolled(&self, by: u32) -> Property {
        let scale = |x: i64| (x * i64::from(by) + 500).div_euclid(1000);
        let times = |m: u32| (1000 + scale(i64::from(m) - 1000)).max(0) as u32;
        let more = |n: u32| scale(i64::from(n)).max(0) as u32;
        let points = |n: i32| scale(i64::from(n)) as i32;
        let span = |(lo, hi): (i32, i32)| (points(lo), points(hi));
        Property {
            noise: times(self.noise),
            damage: times(self.damage),
            swing: times(self.swing),
            wear: times(self.wear),
            endurance: times(self.endurance),
            wane: times(self.wane),
            arrows: times(self.arrows),
            calm: times(self.calm),
            hit: points(self.hit),
            drain: more(self.drain),
            terror: more(self.terror),
            fell: more(self.fell),
            ward: more(self.ward),
            block: more(self.block),
            bash: more(self.bash),
            hearten: points(self.hearten),
            warmth: points(self.warmth),
            mana: points(self.mana),
            mana_regen: more(self.mana_regen),
            fire: span(self.fire),
            holy: span(self.holy),
            thorns: span(self.thorns),
            ..self.clone()
        }
    }

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
    /// By what it is: a blade, a blunt weapon, an axe, a spear, any other weapon, a light, a piece
    /// of armour, a bow, a shield.
    pub second: [Vec<String>; 9],
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

/// What a thing gives past its numbers, by its properties (a unique's effect): the weapon or bow
/// in a hand a blow or a shot (read as it begins, since it may break the weapon), the shield on
/// the arm a block, what's worn a blow that lands on you.
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
    /// Harm back on whoever struck, least and most, in tenths of a point.
    pub thorns: (i32, i32),
}

impl Gifts {
    /// Both together: each property's on a thing, or each piece worn.
    fn and(self, p: Gifts) -> Gifts {
        let add = |a: (i32, i32), b: (i32, i32)| (a.0 + b.0, a.1 + b.1);
        Gifts {
            cleave: self.cleave || p.cleave,
            drain: self.drain + p.drain,
            pyre: self.pyre || p.pyre,
            terror: (self.terror + p.terror).min(10_000),
            hearten: self.hearten + p.hearten,
            fell: (self.fell + p.fell).min(10_000),
            fire: add(self.fire, p.fire),
            holy: add(self.holy, p.holy),
            thorns: add(self.thorns, p.thorns),
        }
    }
}

/// What a weapon gives whoever holds it, all its properties together, known or not (Diablo's "+2
/// to Strength", "+1 to a skill"): Strength and Fitness, in thousandths of a level, and levels by
/// `Skill::ALL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Boosts {
    pub strength: u32,
    pub fitness: u32,
    pub levels: [u8; SKILLS],
    /// Mana added to the most its bearer holds (thousandths of a point), and per mille how much
    /// faster it comes back (`spell.rs`).
    pub mana: i32,
    pub mana_regen: u32,
}

impl Boosts {
    /// Both together: a weapon's and what's worn.
    fn and(mut self, other: Boosts) -> Boosts {
        self.strength += other.strength;
        self.fitness += other.fitness;
        self.mana += other.mana;
        self.mana_regen += other.mana_regen;
        for (n, more) in self.levels.iter_mut().zip(other.levels) {
            *n = n.saturating_add(more);
        }
        self
    }
}

impl Default for Property {
    fn default() -> Property {
        Property {
            id: String::new(),
            name: String::new(),
            line: String::new(),
            family: String::new(),
            tier: 0,
            fits: Fits::Weapon,
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
            mana: 0,
            mana_regen: 0,
            fire: (0, 0),
            holy: (0, 0),
            ward: 0,
            warmth: 0,
            steps: 0,
            range: 0,
            arrows: 1000,
            block: 0,
            bash: 0,
            thorns: (0, 0),
            calm: 1000,
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
    /// Per mille: how far a tiered property's numbers roll either way of its data's (200: a fifth
    /// less to a fifth more).
    pub spread: u32,
}

impl MagicRules {
    /// A roll (0 to `MOST`) as how much of its data's numbers a property has, per mille.
    pub fn by(&self, roll: u8) -> u32 {
        let off = i64::from(roll.min(MOST)) - i64::from(MIDDLE);
        (1000 + i64::from(self.spread) * off / i64::from(MIDDLE)).max(0) as u32
    }
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
            spread: 200,
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

    /// An item's properties (its curse among them; a unique's, and its effect last), each as it
    /// rolled.
    pub fn properties_of(&self, item: &Item) -> impl Iterator<Item = Cow<'_, Property>> + '_ {
        let effect = self.unique_of(item).map(|u| Cow::Borrowed(&u.effect));
        self.rolled_of(item).chain(effect)
    }

    /// An item's properties but a unique's effect: a unique's own, then what was rolled, then
    /// its curse; each as it rolled.
    pub fn rolled_of(&self, item: &Item) -> impl Iterator<Item = Cow<'_, Property>> + '_ {
        self.rolls_of(item).map(|(p, roll)| self.as_rolled(p, roll))
    }

    /// An item's properties but a unique's effect, as the data has them, and where each rolled
    /// in its tier's range: None for one that doesn't roll (a unique's own, a curse, one with no
    /// tiers).
    pub fn rolls_of(&self, item: &Item) -> impl Iterator<Item = (&Property, Option<u8>)> + '_ {
        let m = item.magic;
        let fixed = self.unique_of(item).map_or(&[][..], |u| u.props.as_slice());
        let rolled = m.props.into_iter().zip(m.rolls).map(|(p, r)| (p, Some(r)));
        fixed
            .iter()
            .map(|&p| (p, None))
            .chain(rolled)
            .chain(std::iter::once((m.curse, None)))
            .filter_map(|(p, roll)| {
                let p = self.properties.get(usize::from(p))?;
                Some((p, roll.filter(|_| p.tier > 0)))
            })
    }

    /// A property as it rolled (`roll`: None, as the data has it).
    pub fn as_rolled<'a>(&self, p: &'a Property, roll: Option<u8>) -> Cow<'a, Property> {
        match roll {
            Some(r) if r != MIDDLE => Cow::Owned(p.rolled(self.magic.by(r))),
            _ => Cow::Borrowed(p),
        }
    }

    /// The properties of an item you know, as they rolled: all of them once it's known, else
    /// only a curse that has shown itself.
    pub fn known_of(&self, item: &Item) -> impl Iterator<Item = Cow<'_, Property>> + '_ {
        self.counted(item, true)
    }

    /// The properties that count, as they rolled: all of an item's, or (`known_only`) those you
    /// know, as its card shows them (a curse once it has shown itself).
    fn counted(
        &self,
        item: &Item,
        known_only: bool,
    ) -> impl Iterator<Item = Cow<'_, Property>> + '_ {
        let m = item.magic;
        self.properties_of(item)
            .filter(move |p| !known_only || m.known || (p.curse && m.curse_shown))
    }

    /// The skill a thing trains, its own for "+2 to its skill" (`Raises::Own`): a weapon's, a
    /// bow's Archery, a shield's Shield.
    pub fn own_skill(&self, def: u16) -> Option<Skill> {
        match self.item(def)?.kind {
            ItemKind::Weapon { skill, .. } => skill,
            ItemKind::Bow(_) => Some(Skill::Archery),
            ItemKind::Shield(_) => Some(Skill::Shield),
            _ => None,
        }
    }

    /// What a thing gives whoever holds it or wears it (`Boosts`): its properties' Strength,
    /// Fitness and levels, known or not, as its blows' are. Its own skill is the one it trains.
    pub fn boosts_of(&self, item: &Item) -> Boosts {
        let own = self.own_skill(item.def);
        let mut b = Boosts::default();
        for p in self.properties_of(item) {
            b.strength += p.strength;
            b.fitness += p.fitness;
            b.mana += p.mana;
            b.mana_regen += p.mana_regen;
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

    /// What a thing gives past its numbers (`Gifts`): all its properties together, known or not.
    pub fn gifts_of(&self, item: &Item) -> Gifts {
        self.properties_of(item).fold(Gifts::default(), |g, p| {
            g.and(Gifts {
                cleave: p.cleave,
                drain: p.drain,
                pyre: p.pyre,
                terror: p.terror,
                hearten: p.hearten,
                fell: p.fell,
                fire: p.fire,
                holy: p.holy,
                thorns: p.thorns,
            })
        })
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
    /// other weapon, a light, armour, a bow or a shield (`ThingNames::second`); None for anything
    /// else.
    pub(crate) fn name_class(&self, def: u16) -> Option<usize> {
        match &self.item(def)?.kind {
            ItemKind::Weapon { skill, .. } => Some(match skill {
                Some(Skill::Blade) => 0,
                Some(Skill::Blunt) => 1,
                Some(Skill::Axe) => 2,
                Some(Skill::Spear) => 3,
                _ => 4,
            }),
            ItemKind::Light { .. } => Some(5),
            ItemKind::Armour(_) => Some(6),
            ItemKind::Bow(_) => Some(7),
            ItemKind::Shield(_) => Some(8),
            _ => None,
        }
    }

    /// What kind of thing an item is for properties: a weapon, a light, armour, a bow or a
    /// shield. None for one that takes none.
    pub fn fits(&self, def: u16) -> Option<Fits> {
        match self.item(def)?.kind {
            ItemKind::Weapon { .. } => Some(Fits::Weapon),
            ItemKind::Light { .. } => Some(Fits::Light),
            ItemKind::Armour(_) => Some(Fits::Armour),
            ItemKind::Bow(_) => Some(Fits::Bow),
            ItemKind::Shield(_) => Some(Fits::Shield),
            _ => None,
        }
    }

    /// A piece of armour as it was made (its quality's turning and wear) and as its magic changes
    /// it: every property it has, or (`known_only`) those you know, as its card shows them (a
    /// curse once it has shown itself). Nothing turns every blow: 95% at most.
    pub fn made_armour(&self, item: &Item, mut a: Armour, known_only: bool) -> Armour {
        let q = &self.quality;
        let turn = |a: &mut Armour, by: i64| {
            for t in &mut a.turn {
                *t = (i64::from(*t) + by).clamp(0, 9500) as u32;
            }
        };
        let worn = a.wear;
        turn(&mut a, i64::from(q.turn_of(item.quality)));
        let mut wear = per(u32::from(worn), q.wear_of(item.quality));
        for p in self.counted(item, known_only) {
            turn(&mut a, i64::from(p.ward));
            a.warmth += p.warmth;
            a.noise += p.steps;
            wear = per(wear, p.wear);
        }
        // Never none for a piece that wears at all.
        a.wear = if worn > 0 { wear.max(1) as u16 } else { 0 };
        a
    }

    /// A bow as it was made (its quality's harm) and as its magic changes it (`known_only`: as
    /// its card shows it): its shot as a weapon's swing is (`made_swing`), its range (a tile at
    /// least), and how often its arrows break.
    pub fn made_bow(&self, item: &Item, mut r: Ranged, known_only: bool) -> Ranged {
        r.swing = self.made_swing(item, r.swing, known_only);
        for p in self.counted(item, known_only) {
            r.range += p.range;
            r.breaks = per(r.breaks, p.arrows).min(10_000);
        }
        r.range = r.range.max(1);
        r
    }

    /// A shield as it was made (its quality adds to its block, as to armour's turning, and
    /// changes its wear) and as its magic changes it (`known_only`: as its card shows it): its
    /// block (95% at most), its bash, the breath a block costs and the wear it takes.
    pub fn made_guard(&self, item: &Item, mut g: Guard, known_only: bool) -> Guard {
        let q = &self.quality;
        let mut block = i64::from(g.block) + i64::from(q.turn_of(item.quality));
        let worn = g.wear;
        let mut wear = per(u32::from(worn), q.wear_of(item.quality));
        for p in self.counted(item, known_only) {
            block += i64::from(p.block);
            g.bash = (g.bash + p.bash).min(10_000);
            g.endurance = per(g.endurance.max(0) as u32, p.endurance) as i32;
            wear = per(wear, p.wear);
        }
        g.block = block.clamp(0, 9500) as u32;
        g.wear = if worn > 0 { wear.max(1) as u16 } else { 0 };
        g
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
        let words: Vec<String> = self.properties_of(item).map(|p| p.name.clone()).collect();
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
        for p in self.counted(item, known_only) {
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
        self.counted(item, known_only)
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
    /// each of a tier it reaches, and where in that tier's range it falls), maybe a curse, and a
    /// Rare's made-up name. Unknown.
    pub(crate) fn roll_magic(&self, item: &Item, level: u8, r: impl Fn(u64) -> u64) -> Magic {
        let Some(kind) = self.fits(item.def) else {
            return Magic::MUNDANE;
        };
        let fits = |curse: bool| -> Vec<u8> {
            (0..self.properties.len().min(usize::from(NONE)))
                .filter(|&k| {
                    let p = &self.properties[k];
                    p.fits == kind && p.curse == curse && p.tier <= level
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
            // And where in its tier's range it falls: key 4, mixed with the slot (a line's keys
            // stop at 15, `World::magic_roll`).
            let falls = mix64(r(4).wrapping_add(slot as u64));
            magic.rolls[slot] = below(falls, u32::from(MOST) + 1) as u8;
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

    /// What the weapon or bow in `id`'s hand gives a blow or a shot past its swing, all its
    /// properties together.
    pub(crate) fn gifts(&self, id: ActorId) -> Gifts {
        self.actors
            .get(id)
            .and_then(|a| a.hands)
            .map_or_else(Gifts::default, |h| self.defs.gifts_of(&h))
    }

    /// What `id` wears gives a blow that lands on it, every piece together: harm back.
    pub(crate) fn worn_gifts(&self, id: ActorId) -> Gifts {
        self.actors.get(id).map_or_else(Gifts::default, |a| {
            a.worn
                .iter()
                .fold(Gifts::default(), |g, i| g.and(self.defs.gifts_of(i)))
        })
    }

    /// Per mille: how fast `id`'s dread rises for what it wears (a calming piece: slower).
    pub(crate) fn worn_calm(&self, id: ActorId) -> u32 {
        let Some(a) = self.actors.get(id) else {
            return 1000;
        };
        a.worn
            .iter()
            .flat_map(|i| self.defs.properties_of(i))
            .fold(1000, |r, p| per(r, p.calm))
    }

    /// Harm back on `striker` from `id`'s thorns (`Gifts::thorns`, from what it wears or the
    /// shield it blocked with), rolled from least to most: whatever bone or armour it has, as
    /// fire's harm is. It may put the striker down.
    pub(crate) fn thorns(&mut self, id: ActorId, striker: ActorId, (lo, hi): (i32, i32)) {
        let standing = self
            .actors
            .get(striker)
            .is_some_and(|s| !s.dead && s.body.health > 0);
        if hi <= 0 || !standing {
            return;
        }
        let r = roll(self.seed, self.tick, id.index() as u64, Purpose::Thorns);
        let damage = lo + below(r, (hi - lo).max(0) as u32 + 1) as i32;
        if let Some(s) = self.actors.get_mut(striker) {
            s.body.health -= damage;
        }
        self.events.push(Event::Thorned {
            who: id,
            target: striker,
            damage,
        });
        if self.actors.get(striker).is_some_and(|s| s.body.health <= 0) {
            self.kill(striker);
        }
    }

    /// A blow `id` blocked with its shield (read before the block wore it), by what the shield
    /// gives: harm back on whoever struck, and it may break and flee, or be knocked down.
    pub(crate) fn gifted_block(&mut self, id: ActorId, by: ActorId, g: Gifts) {
        self.thorns(id, by, g.thorns);
        let frightens = roll(self.seed, self.tick, id.index() as u64, Purpose::Terror);
        if g.terror > 0 && below(frightens, 10_000) < g.terror {
            self.terrify(id, by);
        }
        if g.fell > 0 {
            self.fell(id, by, g.fell);
        }
    }

    /// What the weapon or bow in `id`'s hand, the shield on its arm and what it wears give it
    /// (`Boosts`): Strength, Fitness, levels in skills.
    pub fn boosts(&self, id: ActorId) -> Boosts {
        let Some(a) = self.actors.get(id) else {
            return Boosts::default();
        };
        a.hands
            .iter()
            .chain(&a.shield)
            .chain(&a.worn)
            .fold(Boosts::default(), |b, i| b.and(self.defs.boosts_of(i)))
    }

    /// A magic thing that shows itself (put on, taken up on the arm, lit): you know what it is
    /// now, a curse with it.
    fn shows(&mut self, id: ActorId, place: fn(&mut crate::actor::Actor) -> Option<&mut Item>) {
        let Some(thing) = self.actors.get_mut(id).and_then(place) else {
            return;
        };
        if thing.magic.is_magic() && !thing.magic.known {
            thing.magic.known = true;
            thing.magic.curse_shown = true;
            let item = thing.def;
            self.events.push(Event::Identified { who: id, item });
        }
    }

    /// A piece just put on (the last `id` wears): you feel what it does, as a light lit shows
    /// itself.
    pub(crate) fn know_piece(&mut self, id: ActorId) {
        self.shows(id, |a| a.worn.last_mut());
    }

    /// A shield just taken up on the arm: you feel what it does, as a piece put on.
    pub(crate) fn know_shield(&mut self, id: ActorId) {
        self.shows(id, |a| a.shield.as_mut());
    }

    /// The fire's harm and the holy harm a blow (or an arrow) from `id` on `target` adds, by what
    /// its weapon gives (`Gifts`): each rolled from its least to its most. Holy harm is the dead's
    /// only: a beast takes none.
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
        self.shows(id, |a| a.light.as_mut());
    }

    /// A key for rolling an item's magic: so two containers' rolls differ, and two lines'.
    pub(crate) fn magic_roll(&self, key: u64, line: u64) -> impl Fn(u64) -> u64 + '_ {
        let seed = self.seed;
        move |k| roll(seed, key, line * 16 + k, Purpose::Magic)
    }
}
