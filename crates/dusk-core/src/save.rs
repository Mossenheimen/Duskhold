//! Saves (`ARCHITECTURE.md`: the seed, plus only what changed).
//!
//! A save is bytes: a header (which world it is) and the state that didn't come from the seed or
//! the data. The core makes and reads the bytes and never touches a file; whoever calls it does
//! that.
//!
//! - **Saved:** the clock, every actor (bodies too: lying, getting up, burning, ash or hallowed),
//!   sounds not yet heard, what containers and floors hold once touched, the harm doors and
//!   boards have taken, the region's pressure, what the holdouts sold today, the bounties
//!   claimed, the fires laid in hearths, the doors warded, and the chunks of the map that changed
//!   since it was made (a door opened, a board nailed on).
//! - **Not saved:** the land (it comes back from the seed), the rules and the definitions (they
//!   come from `data/`, so a retuned game loads old saves). Items and creatures are saved by
//!   number, so a save also carries a fingerprint of their ids, and one made with other data is
//!   refused rather than loaded wrong.
//!
//! To load: make the same world afresh (the header says which: its seed and its label), then
//! [`World::restore`] the save onto it.

use std::collections::BTreeMap;
use std::fmt;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::actor::{Actor, ActorId};
use crate::arena::Arena;
use crate::fire::Hearth;
use crate::geom::{ChunkPos, Dir8, TilePos};
use crate::grid::Chunk;
use crate::hash::Fnv64;
use crate::items::{Defs, Item};
use crate::world::{Sound, World};

/// The save format's version. A change to what's saved, or to any saved type, raises it.
pub const VERSION: u32 = 31;

const MAGIC: [u8; 8] = *b"DUSKHOLD";

/// Which world a save is of: enough to make that world afresh before restoring onto it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    magic: [u8; 8],
    pub version: u32,
    pub seed: u64,
    pub tick: u64,
    /// The caller's name for how the world was made (the game's: its region).
    pub label: String,
    /// A fingerprint of the data's item and creature ids.
    data: u64,
}

#[derive(Serialize, Deserialize)]
struct State {
    actors: Arena<Actor>,
    characters: Vec<ActorId>,
    sounds: Vec<Sound>,
    door_damage: BTreeMap<(TilePos, Dir8), i32>,
    board_strength: BTreeMap<(TilePos, Dir8), Vec<i32>>,
    stores: BTreeMap<TilePos, Vec<Item>>,
    /// What's in the full bags lying about (`bag.rs`).
    held: BTreeMap<u32, Vec<Item>>,
    /// The region's pressure.
    pressure: u32,
    /// What the holdouts have sold, and when.
    sold: BTreeMap<(u16, u16), (u64, u16)>,
    /// The bounties claimed.
    claimed: std::collections::BTreeSet<ActorId>,
    /// The fires laid in hearths, stoves and forges.
    hearths: BTreeMap<TilePos, Hearth>,
    /// The doors warded against the dead, and when each ward fades.
    wards: BTreeMap<(TilePos, Dir8), u64>,
    /// The plants picked bare, what each grows back as, and when.
    regrowing: BTreeMap<TilePos, (u16, u64)>,
    chunks: Vec<(ChunkPos, Chunk)>,
    /// The caller's own bytes, kept with the save (the game's: the map the character remembers).
    memory: Vec<u8>,
}

/// Why a save couldn't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// Not a Duskhold save, or cut short.
    NotASave,
    /// Made by another version of the save format.
    Version(u32),
    /// Made with other data: its items or creatures aren't the ones loaded now.
    DataChanged,
    /// A save of another world than the one it's being restored onto.
    OtherWorld,
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::NotASave => write!(f, "this isn't a Duskhold save, or it's damaged"),
            SaveError::Version(v) => write!(
                f,
                "this save is in format {v}; this game reads format {VERSION}"
            ),
            SaveError::DataChanged => write!(
                f,
                "this save was made with other game data (its items or creatures have changed)"
            ),
            SaveError::OtherWorld => write!(f, "this save is of another world"),
        }
    }
}

impl std::error::Error for SaveError {}

impl Defs {
    /// A fingerprint of what saves name by number: the items' and the creatures' ids, in order.
    pub fn fingerprint(&self) -> u64 {
        let mut h = Fnv64::default();
        for id in self.items.iter().map(|i| &i.id) {
            id.hash(&mut h);
        }
        0xffu8.hash(&mut h);
        for id in self.creatures.iter().map(|c| &c.id) {
            id.hash(&mut h);
        }
        h.finish()
    }
}

/// Reads a save's header: which world to make before restoring onto it.
pub fn peek(bytes: &[u8]) -> Result<Header, SaveError> {
    header_and_rest(bytes).map(|(header, _)| header)
}

fn header_and_rest(bytes: &[u8]) -> Result<(Header, &[u8]), SaveError> {
    let (header, rest) =
        postcard::take_from_bytes::<Header>(bytes).map_err(|_| SaveError::NotASave)?;
    if header.magic != MAGIC {
        return Err(SaveError::NotASave);
    }
    if header.version != VERSION {
        return Err(SaveError::Version(header.version));
    }
    Ok((header, rest))
}

impl World {
    /// The world as a save. `label` says how it was made, for whoever loads it; `memory` is the
    /// caller's own bytes to keep with it.
    pub fn save(&self, label: &str, memory: &[u8]) -> Vec<u8> {
        let header = Header {
            magic: MAGIC,
            version: VERSION,
            seed: self.seed,
            tick: self.tick,
            label: label.to_string(),
            data: self.defs.fingerprint(),
        };
        let state = State {
            actors: self.actors.clone(),
            characters: self.characters.clone(),
            sounds: self.pending_sounds().to_vec(),
            door_damage: self.door_damage.clone(),
            board_strength: self.board_strength.clone(),
            stores: self.stores.clone(),
            held: self.held.clone(),
            pressure: self.pressure,
            sold: self.sold.clone(),
            claimed: self.claimed.clone(),
            hearths: self.hearths.clone(),
            wards: self.wards.clone(),
            regrowing: self.regrowing.clone(),
            chunks: self.grid.changed_chunks(),
            memory: memory.to_vec(),
        };
        let mut bytes = postcard::to_allocvec(&header).expect("a header always encodes");
        bytes.extend(postcard::to_allocvec(&state).expect("the state always encodes"));
        bytes
    }

    /// Puts a save's state onto this world, which must be the save's world made afresh: the same
    /// seed, made the same way, with the same data. Returns the caller's bytes kept with it. On
    /// an error the world is left as it was.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<Vec<u8>, SaveError> {
        let (header, rest) = header_and_rest(bytes)?;
        if header.seed != self.seed {
            return Err(SaveError::OtherWorld);
        }
        if header.data != self.defs.fingerprint() {
            return Err(SaveError::DataChanged);
        }
        let state: State = postcard::from_bytes(rest).map_err(|_| SaveError::NotASave)?;
        self.tick = header.tick;
        for (at, chunk) in state.chunks {
            self.grid.put_chunk(at, chunk);
        }
        self.door_damage = state.door_damage;
        self.board_strength = state.board_strength;
        self.stores = state.stores;
        self.held = state.held;
        self.pressure = state.pressure;
        self.sold = state.sold;
        self.claimed = state.claimed;
        self.hearths = state.hearths;
        self.wards = state.wards;
        self.regrowing = state.regrowing;
        self.set_actors(state.actors, state.characters, state.sounds);
        Ok(state.memory)
    }
}
