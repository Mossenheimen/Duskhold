//! Saves (`ARCHITECTURE.md`: the seed, plus only what changed), made to outlast the game's
//! updates.
//!
//! A save is bytes: a header (which world it is) and the state that didn't come from the seed or
//! the data. The core makes and reads the bytes and never touches a file; whoever calls it does
//! that.
//!
//! - **Saved:** the clock, every actor (bodies too: lying, getting up, burning, ash or hallowed),
//!   sounds not yet heard, what containers and floors hold once touched, the harm doors and
//!   boards have taken, the region's pressure, what the holdouts sold today, the bounties
//!   claimed, the fires laid in hearths, the doors warded, the plants picked bare, the decoys set
//!   down, and the chunks of the map that changed since it was made (a door opened, a board
//!   nailed on).
//! - **Not saved:** the land (it comes back from the seed), the rules and the definitions (they
//!   come from `data/`, so a retuned game loads old saves).
//! - **Outlasting the game's updates** (since format 37; the postcard saves before it couldn't):
//!   - **the code's:** the state is CBOR, which names every field and every enum's variant. A
//!     field added to a saved type loads from an older save as its default (give it
//!     `#[serde(default)]`: the golden save's test, in `dusk-gen`, fails until it has one); a
//!     field taken away is let go; a renamed one is found by its old name (`#[serde(alias)]`).
//!     A fixed array by an enum is saved by name (a skill's XP), so a new skill loads as none.
//!   - **the data's:** what a save names by number (an item, a creature, a property, a fixture,
//!     a class...) it also names by id: it carries each table's ids (`Ids`), and loading puts
//!     its numbers onto the data's own (`State::remap`), so a table grown or reordered loads
//!     right. Only something the data no longer has refuses it, by name; a word of a name the
//!     data no longer has becomes another, as it only changes what something's called. Put onto
//!     other data, what each actor was about to do, or doing, is let go (it can name an item, a
//!     recipe or a spell), and what the holdouts sold today is forgotten if their stock changed.
//!   - **the format's:** `VERSION` moves only for a change no default or alias can carry (a
//!     field's meaning). Then an older save is read through that change's step, a version at a
//!     time (none yet), and a newer one, from a newer build, is refused.
//!
//! To load: make the same world afresh (the header says which: its seed and its label), then
//! [`World::restore`] the save onto it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::actor::{Actor, ActorId, ActorKind, Intent};
use crate::arena::Arena;
use crate::fire::Hearth;
use crate::geom::{ChunkPos, Dir8, TilePos};
use crate::grid::Chunk;
use crate::items::{Defs, Item};
use crate::magic::Magic;
use crate::world::{Sound, World};

/// The save format's version: moved only by a change no default or alias can carry.
pub const VERSION: u32 = 37;

/// The oldest version this build reads: the first that names its fields.
pub const FIRST: u32 = 37;

const MAGIC: [u8; 8] = *b"DUSKHOLD";

/// Which world a save is of: enough to make that world afresh before restoring onto it.
///
/// On disk: `MAGIC`, a 0 (the postcard saves before format 37 had their version there, never 0),
/// the version (4 bytes, little-endian), the header's length (the same), the header (CBOR), then
/// the state (CBOR).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    /// The format it was saved in (outside the CBOR: read before anything else).
    #[serde(skip)]
    pub version: u32,
    pub seed: u64,
    pub tick: u64,
    /// The caller's name for how the world was made (the game's: its region).
    pub label: String,
}

/// What a save names by number in the data, table by table: each one's ids, in order. Loading
/// puts the save's numbers onto the data's by them (`State::remap`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
struct Ids {
    items: Vec<String>,
    creatures: Vec<String>,
    properties: Vec<String>,
    uniques: Vec<String>,
    affixes: Vec<String>,
    classes: Vec<String>,
    talents: Vec<String>,
    /// By fixture number: 0 is nothing ("").
    fixtures: Vec<String>,
    /// Only to see they're the same: what an actor's about to do, or doing, can name a recipe or
    /// a spell; and what the holdouts sold today is by their stock's lines ("abbey: bread ...").
    recipes: Vec<String>,
    spells: Vec<String>,
    holdouts: Vec<String>,
    /// The words of names: characters' (given, family), rares', and Rares' made-up ones (a first
    /// word, and a second by what the thing is: `ThingNames`).
    given: Vec<String>,
    family: Vec<String>,
    rare_names: Vec<String>,
    first_words: Vec<String>,
    second_words: Vec<Vec<String>>,
}

impl Ids {
    fn of(d: &Defs) -> Ids {
        fn ids<'a, T: 'a>(table: &'a [T], id: impl Fn(&'a T) -> &'a String) -> Vec<String> {
            table.iter().map(|t| id(t).clone()).collect()
        }
        let item = |n: u16| d.item(n).map_or("?", |i| i.id.as_str());
        Ids {
            items: ids(&d.items, |i| &i.id),
            creatures: ids(&d.creatures, |c| &c.id),
            properties: ids(&d.properties, |p| &p.id),
            uniques: ids(&d.uniques, |u| &u.id),
            affixes: ids(&d.affixes, |a| &a.id),
            classes: ids(&d.classes, |c| &c.id),
            talents: ids(&d.talents, |t| &t.id),
            fixtures: d.fixtures.clone(),
            recipes: ids(&d.recipes, |r| &r.id),
            spells: ids(&d.spells, |s| &s.id),
            holdouts: d
                .holdouts
                .iter()
                .map(|h| {
                    let lines: Vec<&str> = h.sells.iter().map(|s| item(s.item)).collect();
                    format!("{}: {}", h.id, lines.join(" "))
                })
                .collect(),
            given: d.given.clone(),
            family: d.family.clone(),
            rare_names: d.rare_names.clone(),
            first_words: d.thing_names.first.clone(),
            second_words: d.thing_names.second.to_vec(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct State {
    /// What the numbers below name in the data (`Ids`).
    ids: Ids,
    actors: Arena<Actor>,
    characters: Vec<ActorId>,
    sounds: Vec<Sound>,
    door_damage: BTreeMap<(TilePos, Dir8), i32>,
    board_strength: BTreeMap<(TilePos, Dir8), Vec<i32>>,
    stores: BTreeMap<TilePos, Vec<Item>>,
    /// What's in the full bags lying about (`bag.rs`).
    held: BTreeMap<u32, Vec<Item>>,
    /// The spells lasting on their spots (`spell.rs`).
    zones: Vec<crate::spell::Zone>,
    /// The region's pressure.
    pressure: u32,
    /// What the holdouts have sold, and when.
    sold: BTreeMap<(u16, u16), (u64, u16)>,
    /// The bounties claimed.
    claimed: BTreeSet<ActorId>,
    /// The fires laid in hearths, stoves and forges.
    hearths: BTreeMap<TilePos, Hearth>,
    /// The doors warded against the dead, and when each ward fades.
    wards: BTreeMap<(TilePos, Dir8), u64>,
    /// The plants picked bare, what each grows back as, and when.
    regrowing: BTreeMap<TilePos, (u16, u64)>,
    /// The decoys set down and wound (`decoy.rs`; since 2026-11-16, none in an older save).
    decoys: BTreeMap<TilePos, crate::decoy::Ticking>,
    chunks: Vec<(ChunkPos, Chunk)>,
    /// The caller's own bytes, kept with the save (the game's: the map the character remembers).
    #[serde(with = "bytes")]
    memory: Vec<u8>,
}

/// Why a save couldn't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// Not a Duskhold save, or cut short before its state.
    NotASave,
    /// Its state couldn't be read: damaged, or a saved type changed in a way it can't carry (what
    /// the reader said).
    Unreadable(String),
    /// Made in a format this build doesn't read: an older one (before saves outlasted updates)
    /// or a newer one (by a newer build).
    Version(u32),
    /// It names something the data loaded now hasn't got (what, and its id).
    DataChanged(String),
    /// A save of another world than the one it's being restored onto.
    OtherWorld,
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::NotASave => write!(f, "this isn't a Duskhold save, or it's damaged"),
            SaveError::Unreadable(why) => write!(f, "this save couldn't be read ({why})"),
            SaveError::Version(v) if *v < FIRST => write!(
                f,
                "this save is in format {v}, from before saves outlasted the game's updates; \
                 this game reads format {FIRST} and later"
            ),
            SaveError::Version(v) => write!(
                f,
                "this save is in format {v}, from a newer build of the game; this one reads \
                 formats {FIRST} to {VERSION}"
            ),
            SaveError::DataChanged(what) => write!(
                f,
                "this save has {what}, which the game's data no longer has"
            ),
            SaveError::OtherWorld => write!(f, "this save is of another world"),
        }
    }
}

impl std::error::Error for SaveError {}

/// Reads a save's header: which world to make before restoring onto it.
pub fn peek(bytes: &[u8]) -> Result<Header, SaveError> {
    header_and_rest(bytes).map(|(header, _)| header)
}

fn header_and_rest(bytes: &[u8]) -> Result<(Header, &[u8]), SaveError> {
    let rest = bytes.strip_prefix(&MAGIC).ok_or(SaveError::NotASave)?;
    let Some((&0, rest)) = rest.split_first() else {
        // A postcard save, from before format 37: its version, as a varint, where the 0 is.
        return Err(old_version(rest).map_or(SaveError::NotASave, SaveError::Version));
    };
    let (version, rest) = take_u32(rest)?;
    if !(FIRST..=VERSION).contains(&version) {
        return Err(SaveError::Version(version));
    }
    let (length, rest) = take_u32(rest)?;
    let (head, rest) = rest
        .split_at_checked(length as usize)
        .ok_or(SaveError::NotASave)?;
    let mut header: Header = ciborium::from_reader(head).map_err(|_| SaveError::NotASave)?;
    header.version = version;
    Ok((header, rest))
}

fn take_u32(bytes: &[u8]) -> Result<(u32, &[u8]), SaveError> {
    let (n, rest) = bytes.split_first_chunk::<4>().ok_or(SaveError::NotASave)?;
    Ok((u32::from_le_bytes(*n), rest))
}

/// The version a postcard save (before format 37) gives after its magic: a varint.
fn old_version(bytes: &[u8]) -> Option<u32> {
    let mut version = 0u32;
    for (k, &b) in bytes.iter().take(5).enumerate() {
        version |= u32::from(b & 0x7f) << (7 * k);
        if b & 0x80 == 0 {
            return Some(version);
        }
    }
    None
}

impl World {
    /// The world as a save. `label` says how it was made, for whoever loads it; `memory` is the
    /// caller's own bytes to keep with it.
    pub fn save(&self, label: &str, memory: &[u8]) -> Vec<u8> {
        let header = Header {
            version: VERSION,
            seed: self.seed,
            tick: self.tick,
            label: label.to_string(),
        };
        let state = State {
            ids: Ids::of(&self.defs),
            actors: self.actors.clone(),
            characters: self.characters.clone(),
            sounds: self.pending_sounds().to_vec(),
            door_damage: self.door_damage.clone(),
            board_strength: self.board_strength.clone(),
            stores: self.stores.clone(),
            held: self.held.clone(),
            zones: self.zones.clone(),
            pressure: self.pressure,
            sold: self.sold.clone(),
            claimed: self.claimed.clone(),
            hearths: self.hearths.clone(),
            wards: self.wards.clone(),
            regrowing: self.regrowing.clone(),
            decoys: self.decoys.clone(),
            chunks: self.grid.changed_chunks(),
            memory: memory.to_vec(),
        };
        let mut head = Vec::new();
        ciborium::into_writer(&header, &mut head).expect("a header always encodes");
        let mut bytes = MAGIC.to_vec();
        bytes.push(0);
        bytes.extend(VERSION.to_le_bytes());
        bytes.extend((head.len() as u32).to_le_bytes());
        bytes.extend(head);
        ciborium::into_writer(&state, &mut bytes).expect("the state always encodes");
        bytes
    }

    /// Puts a save's state onto this world, which must be the save's world made afresh: the same
    /// seed, made the same way, with data that has everything the save names (its numbers are put
    /// onto the data's own). Returns the caller's bytes kept with it. On an error the world is
    /// left as it was.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<Vec<u8>, SaveError> {
        let (header, rest) = header_and_rest(bytes)?;
        if header.seed != self.seed {
            return Err(SaveError::OtherWorld);
        }
        // An older format would be read through each change's step here, a version at a time:
        // none yet, as format 37 is the first.
        // Cut short or garbled, it's damaged; well made but not what a type here can take (a
        // field it needs and hasn't got), it's unreadable, and says why.
        let mut state: State = ciborium::from_reader(rest).map_err(|e| match e {
            ciborium::de::Error::Semantic(_, why) => SaveError::Unreadable(why),
            _ => SaveError::NotASave,
        })?;
        state.remap(&self.defs)?;
        self.tick = header.tick;
        for (at, chunk) in state.chunks {
            self.grid.put_chunk(at, chunk);
        }
        self.door_damage = state.door_damage;
        self.board_strength = state.board_strength;
        self.stores = state.stores;
        self.held = state.held;
        self.zones = state.zones;
        self.pressure = state.pressure;
        self.sold = state.sold;
        self.claimed = state.claimed;
        self.hearths = state.hearths;
        self.wards = state.wards;
        self.regrowing = state.regrowing;
        self.decoys = state.decoys;
        self.set_actors(state.actors, state.characters, state.sounds);
        // An older save's pack and containers, with no cells, laid out on their grids (`pack.rs`).
        self.settle();
        Ok(state.memory)
    }
}

impl State {
    /// Puts the save's numbers onto the data's, by the ids it was saved with (`Ids`): nothing to
    /// do when the data is what it was saved with. Every field is named, so a field added to the
    /// state stops here until it's said whether it names the data by number.
    fn remap(&mut self, defs: &Defs) -> Result<(), SaveError> {
        let now = Ids::of(defs);
        if self.ids == now {
            return Ok(());
        }
        let m = Remap::new(&self.ids, &now);
        let State {
            ids,
            actors,
            stores,
            held,
            sold,
            regrowing,
            chunks,
            characters: _,
            sounds: _,
            door_damage: _,
            board_strength: _,
            zones: _,
            pressure: _,
            claimed: _,
            hearths: _,
            wards: _,
            decoys: _,
            memory: _,
        } = self;
        for id in actors.ids() {
            if let Some(actor) = actors.get_mut(id) {
                m.actor(actor, defs)?;
            }
        }
        for item in stores.values_mut().chain(held.values_mut()).flatten() {
            m.item(item, defs)?;
        }
        for (_, chunk) in chunks.iter_mut() {
            chunk.map_fixtures(|f| m.fixtures.get(f))?;
        }
        for (fixture, _) in regrowing.values_mut() {
            *fixture = m.fixtures.get(*fixture)?;
        }
        // What the holdouts sold today is by their stock's lines: forgotten if those changed.
        if ids.holdouts != now.holdouts {
            sold.clear();
        }
        *ids = now;
        Ok(())
    }
}

/// One of the data's tables, as a save names it by number: where each saved number is in the
/// data loaded now, found by its id (None: the data no longer has it).
struct Table {
    /// What it is, for an error: "an item".
    what: &'static str,
    saved: Vec<String>,
    now: Vec<Option<u16>>,
}

impl Table {
    fn new(what: &'static str, saved: &[String], now: &[String]) -> Table {
        let mut at = BTreeMap::new();
        for (n, id) in now.iter().enumerate() {
            at.entry(id.as_str()).or_insert(n as u16);
        }
        Table {
            what,
            saved: saved.to_vec(),
            now: saved
                .iter()
                .map(|id| at.get(id.as_str()).copied())
                .collect(),
        }
    }

    /// A saved number as the data's. One past the saved table (`NONE`, or a number only a test
    /// gave) is kept as it is: the save never named it.
    fn get(&self, n: u16) -> Result<u16, SaveError> {
        match self.now.get(usize::from(n)) {
            None => Ok(n),
            Some(Some(now)) => Ok(*now),
            Some(None) => Err(SaveError::DataChanged(format!(
                "{} {:?}",
                self.what,
                self.saved[usize::from(n)]
            ))),
        }
    }

    /// The same, for a table numbered in bytes.
    fn get8(&self, n: u8) -> Result<u8, SaveError> {
        let now = self.get(u16::from(n))?;
        u8::try_from(now).map_err(|_| SaveError::DataChanged(format!("{} past 255", self.what)))
    }

    /// A word of a name, which only changes what something's called: one the data no longer has
    /// becomes its first.
    fn word(&self, n: u16) -> u16 {
        self.get(n).unwrap_or(0)
    }

    fn word8(&self, n: u8) -> u8 {
        self.get8(n).unwrap_or(0)
    }
}

/// Every table a save names by number, from the ids it was saved with to the data's.
struct Remap {
    items: Table,
    creatures: Table,
    properties: Table,
    uniques: Table,
    affixes: Table,
    classes: Table,
    talents: Table,
    fixtures: Table,
    given: Table,
    family: Table,
    rare_names: Table,
    first_words: Table,
    second_words: Vec<Table>,
}

impl Remap {
    fn new(saved: &Ids, now: &Ids) -> Remap {
        Remap {
            items: Table::new("an item", &saved.items, &now.items),
            creatures: Table::new("a creature", &saved.creatures, &now.creatures),
            properties: Table::new("a property", &saved.properties, &now.properties),
            uniques: Table::new("a unique", &saved.uniques, &now.uniques),
            affixes: Table::new("an affix", &saved.affixes, &now.affixes),
            classes: Table::new("a class", &saved.classes, &now.classes),
            talents: Table::new("a talent", &saved.talents, &now.talents),
            fixtures: Table::new("a fixture", &saved.fixtures, &now.fixtures),
            given: Table::new("a name", &saved.given, &now.given),
            family: Table::new("a name", &saved.family, &now.family),
            rare_names: Table::new("a name", &saved.rare_names, &now.rare_names),
            first_words: Table::new("a word", &saved.first_words, &now.first_words),
            second_words: saved
                .second_words
                .iter()
                .enumerate()
                .map(|(k, words)| {
                    let none = Vec::new();
                    Table::new("a word", words, now.second_words.get(k).unwrap_or(&none))
                })
                .collect(),
        }
    }

    /// An actor's numbers onto the data's. Every field is named, so a field added to `Actor`
    /// stops here until it's said whether it names the data by number.
    fn actor(&self, a: &mut Actor, defs: &Defs) -> Result<(), SaveError> {
        let Actor {
            kind,
            intent,
            action,
            hands,
            items,
            light,
            shield,
            worn,
            bag,
            name,
            affixes,
            class,
            talents,
            pos: _,
            offset: _,
            facing: _,
            step: _,
            path: _,
            path_goal: _,
            no_way: _,
            blocked: _,
            carry: _,
            mind: _,
            body: _,
            prone_until: _,
            dead: _,
            remains: _,
            risen: _,
            holding: _,
            skills: _,
            home: _,
            stash: _,
            deeds: _,
            born: _,
            died: _,
            winded: _,
            sneaking: _,
            bracing: _,
            faith: _,
            mana: _,
            strength: _,
            fitness: _,
            strain: _,
            doses: _,
        } = a;
        if let ActorKind::Creature(n) = kind {
            *n = self.creatures.get(*n)?;
        }
        let carried = (hands.iter_mut())
            .chain(light.iter_mut())
            .chain(shield.iter_mut())
            .chain(bag.iter_mut())
            .chain(items.iter_mut())
            .chain(worn.iter_mut());
        for item in carried {
            self.item(item, defs)?;
        }
        for affix in affixes.iter_mut() {
            *affix = self.affixes.get8(*affix)?;
        }
        if let Some(class) = class {
            *class = self.classes.get8(*class)?;
        }
        for talent in talents.iter_mut() {
            *talent = self.talents.get8(*talent)?;
        }
        // Its name: a rare's from the rares' names, anyone else's from the characters'
        // (`World::name_of`).
        if let Some((given, family)) = name {
            if affixes.is_empty() {
                *given = self.given.word(*given);
                *family = self.family.word(*family);
            } else {
                *given = self.rare_names.word(*given);
            }
        }
        // What it was about to do, or doing, can name the data by number (an item, a recipe, a
        // spell): it's let go, as a blow makes it let go.
        *intent = Intent::Idle;
        *action = None;
        Ok(())
    }

    /// An item's numbers onto the data's: what it is, and its magic. Every field is named, as
    /// an actor's are.
    fn item(&self, item: &mut Item, defs: &Defs) -> Result<(), SaveError> {
        let Item {
            def,
            magic,
            count: _,
            condition: _,
            found: _,
            quality: _,
            bagged: _,
            holds: _,
            belt: _,
            cell: _,
        } = item;
        *def = self.items.get(*def)?;
        let Magic {
            unique,
            props,
            curse,
            name,
            rarity: _,
            rolls: _,
            known: _,
            curse_shown: _,
            uses: _,
        } = magic;
        *unique = self.uniques.get8(*unique)?;
        for prop in props.iter_mut() {
            *prop = self.properties.get8(*prop)?;
        }
        *curse = self.properties.get8(*curse)?;
        // A Rare's made-up name: its first word, and its second from the words for what it is.
        name[0] = self.first_words.word8(name[0]);
        if let Some(words) = defs
            .name_class(*def)
            .and_then(|class| self.second_words.get(class))
        {
            name[1] = words.word8(name[1]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! What the format promises, on a save's own CBOR changed by hand: a field it doesn't know
    //! is let go, one the state hasn't got is its default, and one an actor needs and hasn't got
    //! is named. (A patch's saves, and the data's tables by id: `dusk-gen`'s tests.)

    use super::*;
    use crate::grid::Tile;
    use ciborium::Value;

    /// Grass, you, and one of the dead.
    fn world() -> World {
        let mut w = World::new(3);
        for x in 0..4 {
            w.grid.set_tile(TilePos::new(x, 0, 0), Tile::GRASS);
        }
        w.spawn(ActorKind::Character, TilePos::new(0, 0, 0))
            .unwrap();
        w.spawn(ActorKind::Creature(0), TilePos::new(3, 0, 0))
            .unwrap();
        w
    }

    fn map(v: &mut Value) -> &mut Vec<(Value, Value)> {
        match v {
            Value::Map(fields) => fields,
            _ => panic!("not a map: {v:?}"),
        }
    }

    fn field<'a>(fields: &'a mut [(Value, Value)], name: &str) -> &'a mut Value {
        let at = fields.iter().position(|(k, _)| k.as_text() == Some(name));
        &mut fields[at.unwrap_or_else(|| panic!("no {name}"))].1
    }

    /// The first actor's fields, in a state's.
    fn first_actor(state: &mut [(Value, Value)]) -> &mut Vec<(Value, Value)> {
        let slots = field(map(field(state, "actors")), "slots");
        let Value::Array(slots) = slots else {
            panic!("slots are a list")
        };
        let full = map(&mut slots[0]);
        map(field(map(&mut full[0].1), "value"))
    }

    /// `bytes` with its state as `edit` changes it.
    fn edited(bytes: &[u8], edit: impl FnOnce(&mut Vec<(Value, Value)>)) -> Vec<u8> {
        let (_, rest) = header_and_rest(bytes).unwrap();
        let mut out = bytes[..bytes.len() - rest.len()].to_vec();
        let mut state: Value = ciborium::from_reader(rest).unwrap();
        edit(map(&mut state));
        ciborium::into_writer(&state, &mut out).unwrap();
        out
    }

    #[test]
    fn a_field_this_build_doesnt_know_is_let_go() {
        let w = world();
        let bytes = edited(&w.save("test", b""), |state| {
            let newer = (Value::Text("from_a_newer_build".into()), Value::Bool(true));
            state.push(newer.clone());
            first_actor(state).push(newer);
        });
        let mut b = world();
        b.restore(&bytes).unwrap();
        assert_eq!(b.state_hash(), w.state_hash());
    }

    #[test]
    fn a_field_the_state_hasnt_got_is_its_default() {
        let mut w = world();
        w.pressure = 1500;
        let bytes = edited(&w.save("test", b""), |state| {
            state.retain(|(k, _)| k.as_text() != Some("pressure"));
        });
        let mut b = world();
        b.restore(&bytes).unwrap();
        assert_eq!(b.pressure, 0, "the default, not refused");
    }

    #[test]
    fn a_field_an_actor_needs_and_hasnt_got_is_named() {
        // So a field added to a saved type without `#[serde(default)]` shows at once: the golden
        // save's test (`dusk-gen`) fails with this.
        let w = world();
        let bytes = edited(&w.save("test", b""), |state| {
            first_actor(state).retain(|(k, _)| k.as_text() != Some("strain"));
        });
        let mut b = world();
        let before = b.state_hash();
        let err = b.restore(&bytes).unwrap_err();
        assert!(
            matches!(&err, SaveError::Unreadable(why) if why.contains("strain")),
            "{err:?}"
        );
        assert_eq!(b.state_hash(), before, "and the world is left as it was");
    }

    #[test]
    fn a_chunk_is_saved_as_its_kinds_of_tile_and_edge_and_runs_of_them() {
        let mut w = world();
        w.grid
            .set_edge(TilePos::new(1, 0, 0), Dir8::N, crate::grid::Edge::Wall);
        let chunk = w.grid.changed_chunks().remove(0).1;
        let mut bytes = Vec::new();
        ciborium::into_writer(&chunk, &mut bytes).unwrap();
        assert!(bytes.len() < 200, "{} bytes", bytes.len());
        let back: Chunk = ciborium::from_reader(bytes.as_slice()).unwrap();
        assert_eq!(back, chunk);
        // Runs that don't make a chunk are refused.
        let mut v: Value = ciborium::from_reader(bytes.as_slice()).unwrap();
        *field(map(&mut v), "tile_runs") = Value::Array(vec![1.into(), 0.into()]);
        let mut short = Vec::new();
        ciborium::into_writer(&v, &mut short).unwrap();
        assert!(ciborium::from_reader::<Chunk, _>(short.as_slice()).is_err());
    }
}

/// The caller's bytes, saved as bytes (not a list of numbers).
mod bytes {
    use std::fmt;

    pub fn serialize<S: serde::Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        struct Bytes;
        impl serde::de::Visitor<'_> for Bytes {
            type Value = Vec<u8>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bytes")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Vec<u8>, E> {
                Ok(v.to_vec())
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Vec<u8>, E> {
                Ok(v)
            }
        }
        d.deserialize_byte_buf(Bytes)
    }
}
