//! Death and legacy (`CONCEPT.md`): permadeath in a persistent world.
//!
//! - **A character has a name** (rolled from the data's lists as it comes into the world) and a
//!   time it came, so a death can be told: who, how long they lasted, what they carried.
//! - **The body keeps its gear** where it fell. Unburned, it rises at the next midnight
//!   (`Defs::revenant`'s hour) as a **revenant**: a creature of its own kind, with the body's name,
//!   carrying everything the body had. It haunts the place: kill it to get your things back,
//!   for whatever a creature carries falls where it's put down. Burn the body first, and it
//!   never rises; its gear falls where it lay (items don't burn).
//! - **The next character** comes into the same world at the region's start (the abbey's gate),
//!   with the last one's home and stash: the base carries over, the levels and skills don't.

use crate::actor::{ActorId, ActorKind, Remains};
use crate::geom::{Dir8, TilePos};
use crate::hash::{Purpose, below, roll};
use crate::items::{Defs, Item};
use crate::world::{Event, World};

/// Game minutes in a day.
const DAY: u64 = 1440;

impl Defs {
    /// A character's name, by its numbers in the data's lists.
    pub fn name(&self, name: (u16, u16)) -> Option<String> {
        let given = self.given.get(usize::from(name.0))?;
        Some(match self.family.get(usize::from(name.1)) {
            Some(family) => format!("{given} {family}"),
            None => given.clone(),
        })
    }
}

impl World {
    /// A name for a character coming into the world now, from the data's lists.
    pub(crate) fn roll_name(&self, who: u64) -> Option<(u16, u16)> {
        let (g, f) = (self.defs.given.len(), self.defs.family.len());
        if g == 0 {
            return None;
        }
        let r = roll(self.seed, self.tick, who, Purpose::Name);
        let given = below(r, g as u32) as u16;
        let family = below(r << 32, f.max(1) as u32) as u16;
        Some((given, family))
    }

    /// An actor's name: a character's own, a revenant's (the one it was), else its kind's.
    pub fn name_of(&self, id: ActorId) -> Option<String> {
        let a = self.actors.get(id)?;
        if let Some(name) = a.name.and_then(|n| self.defs.name(n)) {
            return Some(name);
        }
        match a.kind {
            ActorKind::Creature(n) => self.defs.creature(n).map(|c| c.name.clone()),
            ActorKind::Character => None,
        }
    }

    /// How long a character lasted, or has so far, in game minutes.
    pub fn lasted(&self, id: ActorId) -> u64 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        a.died
            .unwrap_or_else(|| self.clock_minutes())
            .saturating_sub(a.born)
    }

    /// When a character falls: unless the data has no revenant, its body rises at the next
    /// midnight (the revenant's hour), as one.
    pub(crate) fn settle_body(&mut self, id: ActorId) {
        let Some((_, hour)) = self.defs.revenant else {
            return;
        };
        let now = self.clock_minutes();
        let today = now / DAY * DAY + u64::from(hour);
        let rises = if today > now { today } else { today + DAY };
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        let at = self.tick + (rises - now) * per;
        if let Some(a) = self.actors.get_mut(id) {
            a.remains = Remains::GetsUp { at };
        }
    }

    /// Everything an actor carries, out of its hands, its pack and its other hand.
    fn take_gear(&mut self, id: ActorId) -> Vec<Item> {
        let Some(a) = self.actors.get_mut(id) else {
            return Vec::new();
        };
        let mut gear: Vec<Item> = a.hands.take().into_iter().collect();
        gear.extend(a.light.take());
        gear.append(&mut a.items);
        gear
    }

    /// Lets an actor's gear fall where it is: a body burned, a creature put down.
    pub(crate) fn shed_gear(&mut self, id: ActorId) {
        let Some(at) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        for item in self.take_gear(id) {
            self.put(at, item);
        }
    }

    /// A character's body rises: a revenant stands where it lay, with its name and its gear, and
    /// the body is gone. Not while someone stands on it: then it waits.
    pub(crate) fn rise_as_revenant(&mut self, body: ActorId) {
        let Some((kind, _)) = self.defs.revenant else {
            return;
        };
        let Some(b) = self.actors.get(body) else {
            return;
        };
        let (pos, facing, name) = (b.pos, b.facing, b.name);
        let Some(revenant) = self.spawn(ActorKind::Creature(kind), pos) else {
            return; // someone's on it
        };
        let gear = self.take_gear(body);
        let defs = self.defs.clone();
        if let Some(r) = self.actors.get_mut(revenant) {
            r.name = name;
            r.facing = facing;
            r.risen = 1;
            for item in gear {
                defs.stow(&mut r.items, item);
            }
        }
        if let Some(b) = self.actors.get_mut(body) {
            b.remains = Remains::Risen;
        }
        self.events.push(Event::Risen { body, revenant });
    }

    /// The next character, after the last one died: at the region's start (or the nearest free
    /// tile to it), with the last one's home and stash. None while the last one lives.
    pub fn next_character(&mut self) -> Option<ActorId> {
        let last = self.character();
        let (home, stash) = match last.and_then(|id| self.actors.get(id)) {
            Some(a) if !a.dead => return None,
            Some(a) => (a.home, a.stash),
            None => (None, None),
        };
        let start = self.region.start?;
        let id = std::iter::once(start)
            .chain((1..=4).flat_map(|r| {
                Dir8::ALL.map(move |d| {
                    let (dx, dy) = d.delta();
                    start.offset(dx * r, dy * r)
                })
            }))
            .find_map(|at: TilePos| self.spawn(ActorKind::Character, at))?;
        if let Some(a) = self.actors.get_mut(id) {
            a.home = home;
            a.stash = stash;
        }
        self.events.push(Event::Came { who: id });
        Some(id)
    }
}
