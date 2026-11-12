//! Rare monsters (`CONCEPT.md`: Diablo's champions): an ordinary creature with one to three
//! affixes and a name, "Grimtooth the Lantern-Eater". They carry better loot, and a holdout puts
//! a bounty on each.
//!
//! - **An affix is data** (`data/affixes.toml`): what it does to its creature's numbers (faster,
//!   tougher, louder, a better nose, a fever in its wounds, getting up again), and, for the
//!   Lantern-Eater, one rule of its own: lights held up near it go out.
//! - **An actor's numbers** are its kind's with its affixes on top (`World::stats`). Every rule
//!   that asks what a creature can do asks that, so an affix needs no rule of its own. Three hot
//!   paths read the kind in place instead, for speed: the thinking beat, the eyes (`sees`) and,
//!   for all but a rare, the legs (`pace`). No affix changes the first two; one that did would
//!   need them to ask `stats`.
//! - **A rare's pack** is filled from a loot table as it's made, as finds: put it down, and its
//!   gear falls where it does (`legacy.rs`), better than a cupboard's.
//! - **Bounties** (`holdout.rs`): each rare has one at the holdout. Put the rare down, and the
//!   bounty can be claimed over the counter, for coin, XP and the pressure.

use crate::actor::{ActorId, ActorKind, Speeds};
use crate::creature::Creature;
use crate::geom::TilePos;
use crate::items::Defs;
use crate::world::{Event, World};

/// An affix's numbers, in the core's units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Affix {
    pub id: String,
    /// What it adds to a rare's name: "the Lantern-Eater".
    pub epithet: String,
    /// Per mille: its pace (walking and running), its health, and its wind-up and recovery.
    pub speed: u32,
    pub health: u32,
    pub swing: u32,
    /// Tiles added to its groan, and to how far it smells blood for each wound.
    pub groan: i32,
    pub blood: i32,
    /// Per mille: how long it keeps after what it heard or saw.
    pub forget: u32,
    /// Out of 10,000, by wound kind: at least this chance its wounds carry ghoul fever.
    pub fever: [u32; 3],
    /// Times more it gets up after it's put down (`remains.rs`).
    pub rises: u8,
    /// Lights held up within this many tiles of it go out (0: it doesn't).
    pub douses: i32,
}

impl Default for Affix {
    /// One that changes nothing.
    fn default() -> Affix {
        Affix {
            id: String::new(),
            epithet: String::new(),
            speed: 1000,
            health: 1000,
            swing: 1000,
            groan: 0,
            blood: 0,
            forget: 1000,
            fever: [0; 3],
            rises: 0,
            douses: 0,
        }
    }
}

impl Affix {
    /// Its creature's numbers with it on top.
    fn apply(&self, c: &mut Creature) {
        let per = |n: u32, by: u32| (u64::from(n) * u64::from(by) / 1000) as u32;
        c.speed.walk = per(c.speed.walk, self.speed);
        c.speed.run = per(c.speed.run, self.speed);
        c.health = (i64::from(c.health) * i64::from(self.health) / 1000) as i32;
        c.attack.wind_up = per(c.attack.wind_up, self.swing).max(1);
        c.attack.recovery = per(c.attack.recovery, self.swing);
        c.groan += self.groan;
        c.blood += self.blood;
        c.forget_after = c.forget_after * u64::from(self.forget) / 1000;
        for k in 0..3 {
            c.fever[k] = c.fever[k].max(self.fever[k]);
        }
        c.douses = c.douses.max(self.douses);
        if self.rises > 0 {
            c.get_up.chance = 10_000;
            if c.get_up.after == (0, 0) {
                // A kind that never got up: an hour or two, the dead's own pace.
                c.get_up.after = (60, 120);
            }
        }
    }
}

impl Defs {
    /// A rare's name: its given name, and its first affix's epithet.
    pub fn rare_name(&self, name: u16, first: u8) -> Option<String> {
        let given = self.rare_names.get(usize::from(name))?;
        let epithet = self
            .affixes
            .get(usize::from(first))
            .map_or("", |a| &a.epithet);
        Some(format!("{given} {epithet}").trim_end().to_string())
    }
}

impl World {
    /// What an actor can do: its kind's numbers with its affixes on top. None for a character.
    pub fn stats(&self, id: ActorId) -> Option<Creature> {
        let a = self.actors.get(id)?;
        let mut c = self.creature(a.kind)?;
        for &k in &a.affixes {
            if let Some(affix) = self.defs.affixes.get(usize::from(k)) {
                affix.apply(&mut c);
            }
        }
        Some(c)
    }

    /// How an actor moves: its pace, its path budget, and its pace at night (a creature's). What
    /// the legs need, read from the kind for all but a rare: the move runs for every actor
    /// every tick, so it doesn't work out all its numbers.
    pub(crate) fn pace(&self, id: ActorId) -> Option<(Speeds, usize, Option<u32>)> {
        let a = self.actors.get(id)?;
        let c = match a.kind {
            ActorKind::Character => {
                // Hurt legs slow the living (`injury.rs`).
                let (s, pace) = (self.rules.character_speed, self.leg_pace(id));
                let speeds = Speeds {
                    walk: s.walk * pace / 1000,
                    run: s.run * pace / 1000,
                };
                return Some((speeds, self.rules.path_budget, None));
            }
            ActorKind::Creature(n) if a.affixes.is_empty() => self.defs.creature(n)?.stats,
            ActorKind::Creature(_) => self.stats(id)?,
        };
        Some((c.speed, c.path_budget, Some(c.night_speed)))
    }

    /// How many more times an actor gets up than an ordinary one of its kind (its affixes').
    pub(crate) fn extra_rises(&self, id: ActorId) -> u8 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        a.affixes
            .iter()
            .filter_map(|&k| self.defs.affixes.get(usize::from(k)))
            .map(|x| x.rises)
            .sum()
    }

    /// Makes one of the dead a rare: its affixes, its name (from the data's rare names), its
    /// health to match, and its pack filled from `loot`, as finds.
    pub fn make_rare(&mut self, id: ActorId, affixes: &[u8], name: u16, loot: Option<u16>) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        a.affixes = affixes.to_vec();
        a.name = Some((name, 0));
        let pos = a.pos;
        let health = self.stats(id).map(|c| c.health);
        let pack = loot.map(|t| self.fill(pos, t)).unwrap_or_default();
        if let Some(a) = self.actors.get_mut(id) {
            if let Some(h) = health {
                a.body = crate::body::Body::new(h);
            }
            a.items = pack;
        }
    }

    /// What `target` takes of a blow from a weapon of `skill` (bare hands strike as blunt), per
    /// mille, by its kind: 1000 for the living, and for a kind that minds no weapon.
    pub fn resistance(&self, target: ActorId, skill: Option<crate::skills::Skill>) -> u32 {
        self.stats(target)
            .map_or(1000, |c| c.resist[crate::creature::resist_index(skill)])
    }

    /// Whether an actor is a rare.
    pub fn is_rare(&self, id: ActorId) -> bool {
        self.actors.get(id).is_some_and(|a| !a.affixes.is_empty())
    }

    /// The rares in the world, alive or put down, in id order.
    pub fn rares(&self) -> impl Iterator<Item = ActorId> + '_ {
        self.actors
            .iter()
            .filter(|(_, a)| !a.affixes.is_empty())
            .map(|(id, _)| id)
    }

    /// The Lantern-Eater, as it thinks: lights held up within its reach go out.
    pub(crate) fn eat_lights(&mut self, id: ActorId, reach: i32) {
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        let r2 = i64::from(reach).pow(2);
        let near: Vec<ActorId> = self
            .characters
            .iter()
            .copied()
            .filter(|&c| {
                self.actors.get(c).is_some_and(|a| {
                    !a.dead && a.light.is_some() && a.pos.z == pos.z && a.pos.dist2(pos) <= r2
                })
            })
            .collect();
        for who in near {
            self.douse(who);
            self.events.push(Event::LightEaten { by: id, who });
        }
        // And the fires it passes (`fire.rs`), walls or none.
        let fires: Vec<TilePos> = self
            .hearth_fires()
            .map(|(at, _)| at)
            .filter(|at| at.z == pos.z && at.dist2(pos) <= r2)
            .collect();
        for at in fires {
            self.put_out(at, id);
        }
    }
}
