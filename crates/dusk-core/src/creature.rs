//! Creatures as data (`ARCHITECTURE.md`: a region is mostly data).
//!
//! A kind of creature is a row in `Defs::creatures`, and an actor of that kind is
//! `ActorKind::Creature(n)`. Everything the rules need to know about it is in its [`Creature`]:
//! how it moves, what it senses, how it thinks, and what its blows do. A new kind is a new row in
//! `data/creatures/`, not a new field in the rules.

use crate::actor::{ActorKind, Speeds};
use crate::items::Defs;
use crate::senses::Cone;
use crate::world::{Swing, World};

/// A kind of creature's numbers, in the core's units. Plain numbers, so the tick can copy them
/// out while it holds the actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Creature {
    /// Thousandths of a tile a second: about its business, and after something.
    pub speed: Speeds,
    /// At night it moves this much faster, per mille.
    pub night_speed: u32,
    /// How far it sees in full light, in tiles, and how wide: you can get behind one. In less
    /// light it sees less far (`light.rs`), but always `darkvision` tiles.
    pub sight: i32,
    pub cone: Cone,
    pub darkvision: i32,
    /// How far its groan carries as it catches sight of prey, in tiles (0: it makes none). The
    /// others in earshot come to it, and so to you: how one becomes a crowd.
    pub groan: i32,
    /// It thinks every this many ticks, staggered by id.
    pub think_every: u64,
    /// Ticks it keeps after something it heard or saw before giving up on it.
    pub forget_after: u64,
    /// Out of 10,000: how often, after nothing, it shuffles a step when it thinks.
    pub wander: u32,
    /// The most tiles one search for a way explores: a way it can't find nearby, it doesn't find.
    pub path_budget: usize,
    /// Out of `body::FULL`.
    pub health: i32,
    /// Ticks it stays down when knocked down. A blow on one that's down always lands, for double
    /// harm: Zomboid's stomp.
    pub down: u32,
    /// Its swing: it wounds rather than doing `damage`.
    pub attack: Swing,
    /// Out of 10,000, by `WoundKind::ALL`: what its hit leaves.
    pub wounds: [u32; 3],
    /// What its blow does to a shut door in its way (out of `body::FULL`, a range).
    pub thump: (i32, i32),
    /// How far it smells blood, in tiles for each wound that's bleeding (up to three): walls
    /// don't stop it, nor does the dark. 0: it doesn't.
    pub blood: i32,
    /// Out of 10,000, by `WoundKind::ALL`: the chance a wound it leaves carries ghoul fever
    /// (`fever.rs`).
    pub fever: [u32; 3],
}

impl Default for Creature {
    /// The fixed test set's shambler: it shuffles when it wanders and lurches when it's after
    /// something, sees 8 tiles in a 120° cone, and makes no groan.
    fn default() -> Creature {
        Creature {
            speed: Speeds {
                walk: 500,
                run: 800,
            },
            night_speed: 1250,
            sight: 8,
            // 60° either side of its facing: a 120° cone.
            cone: Cone { cos: 5000 },
            darkvision: 1,
            groan: 0,
            think_every: 5,
            forget_after: 300,
            wander: 1000,
            path_budget: 400,
            health: 300,
            down: 40,
            attack: Swing {
                wind_up: 8,
                recovery: 8,
                endurance: 0,
                hit: 5000,
                damage: (0, 0),
                noise: 3,
            },
            wounds: [7000, 2500, 500],
            thump: (20, 40),
            blood: 0,
            fever: [0; 3],
        }
    }
}

/// A kind of creature: its name in the data, its name in the game, and its numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatureDef {
    pub id: String,
    pub name: String,
    pub stats: Creature,
}

impl CreatureDef {
    /// The fixed test set's one creature (`Creature::default`). Test maps' `z` is the first
    /// creature in the definitions: this one, unless the data says otherwise.
    pub fn shambler() -> CreatureDef {
        CreatureDef {
            id: "shambler".into(),
            name: "shambler".into(),
            stats: Creature::default(),
        }
    }
}

impl Defs {
    /// A kind of creature, by its number.
    pub fn creature(&self, n: u16) -> Option<&CreatureDef> {
        self.creatures.get(usize::from(n))
    }

    /// The kind of actor a creature's id in the data stands for.
    pub fn creature_kind(&self, id: &str) -> Option<ActorKind> {
        let n = self.creatures.iter().position(|c| c.id == id)?;
        Some(ActorKind::Creature(n as u16))
    }
}

impl World {
    /// A kind of actor's creature numbers: None for a character, or a kind the data hasn't got.
    pub fn creature(&self, kind: ActorKind) -> Option<Creature> {
        match kind {
            ActorKind::Creature(n) => self.defs.creature(n).map(|c| c.stats),
            ActorKind::Character => None,
        }
    }

    /// A kind of creature's numbers, to change in this world alone: for tests and tools. (The
    /// definitions are shared; this world gets its own copy the first time.)
    ///
    /// # Panics
    /// If there is no creature `n`.
    pub fn creature_mut(&mut self, n: u16) -> &mut Creature {
        &mut std::sync::Arc::make_mut(&mut self.defs).creatures[usize::from(n)].stats
    }
}
