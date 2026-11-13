//! Spells (`CONCEPT.md`: the Wizard, "Spells are timed actions, stated by their limits", and
//! "Spells work on the sense fields"): the Wizard's first four.
//!
//! - **A spell is data** (`data/spells.toml`): what it does (`SpellKind`), the slots it costs,
//!   its cast time, its range, how far it reaches round the spot, the light it gives and for how
//!   long, the noise of its words, and the component it uses up.
//! - **Casting is a timed action** (`Intent::Cast`) at a tile in range and in sight. The words
//!   are its wind-up and its tell, heard as far as they carry; a blow during them breaks the spell
//!   (D&D's concentration), and nothing is spent. On its strike the slots and the component go,
//!   and it takes effect.
//! - **Slots** (`Actor::slots`) are the Wizard's resource: its class's, and more with a talent,
//!   all back once it has slept the night through. Light costs none: a cantrip.
//! - **Each spell works on a sense,** on its spot (`World::zones`) for as long as it lasts:
//!   - Light: a pale light, to see by; and at night the dead that see it come to it, as to a
//!     fire: a lure.
//!   - Silence: inside it no sound is made at all: not a footstep, a blow, or a hammer.
//!   - Fog: the dead can't see into it or out of it.
//!   - Fireball: a burst on the spot, harming everything within its reach (you too). What it puts
//!     down burns, and the bodies there catch fire, so none of them gets up. It's heard across
//!     the map, and lights the street for a moment.
//! - **Arcana** is the Wizard's skill: each spell cast teaches it, and each level casts faster.

use crate::actor::{ActionKind, ActorId};
use crate::geom::{Dir8, TilePos};
use crate::hash::{Purpose, below, roll};
use crate::senses::line_of_sight;
use crate::skills::{On, Skill};
use crate::world::{Event, Sound, World};

/// What a spell does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum SpellKind {
    Light,
    Silence,
    Fog,
    Fireball,
}

/// A spell, in the core's units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellDef {
    pub id: String,
    pub name: String,
    /// What it does, in a line.
    pub line: String,
    pub kind: SpellKind,
    /// Slots it costs (0: a cantrip).
    pub slots: u8,
    /// Ticks of words: its wind-up.
    pub cast: u32,
    /// Tiles to the spot it's cast on.
    pub range: i32,
    /// Tiles it reaches round the spot: Silence's and Fog's reach, a fireball's burst.
    pub radius: i32,
    /// Tiles it lights round the spot (0: none): Light's light, a fireball's flash.
    pub light: i32,
    /// Ticks it lasts on the spot (its light too); 0: at once.
    pub lasts: u32,
    /// Tiles its words carry from the caster, and its burst from the spot (a fireball's).
    pub noise: i32,
    pub boom: i32,
    /// The harm its burst does, least and most, out of `body::FULL`.
    pub damage: (i32, i32),
    /// The item it uses up, by definition number.
    pub component: Option<u16>,
}

/// A spell lasting on a spot: its kind, where, how far it reaches (or lights), and the tick it
/// ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Zone {
    pub kind: SpellKind,
    pub at: TilePos,
    pub radius: i32,
    pub until: u64,
}

impl Zone {
    fn holds(&self, at: TilePos) -> bool {
        at.z == self.at.z && at.dist2(self.at) <= i64::from(self.radius).pow(2)
    }
}

/// Why a spell can't be cast now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoCast {
    /// It isn't a spell this one knows (or there's no such spell).
    Unknown,
    /// Something has hold of you.
    Held,
    /// Not slots enough left: they come back with a night's sleep.
    NoSlots,
    /// Its component isn't in your pack.
    NoComponent,
    /// The spot is out of its range.
    TooFar,
    /// You can't see the spot.
    NoSight,
}

/// `n` per mille of `by`.
fn per(n: u32, by: u32) -> u32 {
    (u64::from(n) * u64::from(by) / 1000) as u32
}

impl World {
    /// A spell's definition.
    pub fn spell(&self, spell: u16) -> Option<&crate::spell::SpellDef> {
        self.defs.spells.get(usize::from(spell))
    }

    /// The spells `id` knows: its class's.
    pub fn known_spells(&self, id: ActorId) -> Vec<u16> {
        self.class_of(id).map_or(Vec::new(), |c| c.spells.clone())
    }

    /// The most slots `id` holds: its class's, and its talents' (0 for a class without).
    pub fn max_slots(&self, id: ActorId) -> u8 {
        let class = self.class_of(id).map_or(0, |c| c.slots);
        if class == 0 {
            return 0;
        }
        self.talents_of(id)
            .fold(class, |n, t| n.saturating_add(t.slots))
    }

    /// The slots `id` has left.
    pub fn slots(&self, id: ActorId) -> u8 {
        self.actors
            .get(id)
            .filter(|a| !a.dead)
            .map_or(0, |a| a.slots)
    }

    /// How far `id` casts a spell, in tiles: the spell's range, and its talents'.
    pub fn spell_range(&self, id: ActorId, spell: u16) -> i32 {
        let more: i32 = self.talents_of(id).map(|t| t.range).sum();
        self.spell(spell).map_or(0, |s| s.range + more)
    }

    /// The ticks `id`'s words take: the spell's, quicker with Arcana and its talents.
    pub fn cast_ticks(&self, id: ActorId, spell: u16) -> u32 {
        let Some(s) = self.spell(spell) else {
            return 0;
        };
        let level = self.skill_level(id, Skill::Arcana);
        let ticks = self.rules.skills.cast_ticks(s.cast, level);
        self.talents_of(id)
            .fold(ticks, |t, talent| per(t, talent.cast))
            .max(1)
    }

    /// Whether `id` can cast `spell` on `at` now.
    pub fn can_cast(&self, id: ActorId, spell: u16, at: TilePos) -> Result<(), NoCast> {
        let a = self
            .actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoCast::Unknown)?;
        let s = self.spell(spell).ok_or(NoCast::Unknown)?;
        if !self.known_spells(id).contains(&spell) {
            return Err(NoCast::Unknown);
        }
        if self.is_held(id) {
            return Err(NoCast::Held);
        }
        if a.slots < s.slots {
            return Err(NoCast::NoSlots);
        }
        if let Some(c) = s.component
            && !a.items.iter().any(|i| i.def == c)
        {
            return Err(NoCast::NoComponent);
        }
        let range = i64::from(self.spell_range(id, spell));
        if at.z != a.pos.z || a.pos.dist2(at) > range * range {
            return Err(NoCast::TooFar);
        }
        if !line_of_sight(&self.grid, a.pos, at) {
            return Err(NoCast::NoSight);
        }
        Ok(())
    }

    /// The action casting `spell` on `at` is, if it can start: facing the spot, the words its
    /// wind-up.
    pub(crate) fn cast_action(
        &self,
        id: ActorId,
        spell: u16,
        at: TilePos,
    ) -> Option<(ActionKind, Dir8, u32)> {
        self.can_cast(id, spell, at).ok()?;
        Some((
            ActionKind::Cast { spell, at },
            self.toward(id, at),
            self.cast_ticks(id, spell),
        ))
    }

    /// The words are said (the action's strike): the slots and the component go, Arcana learns
    /// from it, and the spell takes effect. The words are heard as far as they carry.
    pub(crate) fn cast(&mut self, id: ActorId, spell: u16, at: TilePos) {
        if self.can_cast(id, spell, at).is_err() {
            return;
        }
        let Some(s) = self.spell(spell).cloned() else {
            return;
        };
        let hush = self.talents_of(id).fold(1000, |h, t| per(h, t.hush));
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        a.slots -= s.slots;
        if let Some(c) = s.component
            && let Some(slot) = a.items.iter().position(|i| i.def == c)
        {
            a.items[slot].count -= 1;
            if a.items[slot].count == 0 {
                a.items.remove(slot);
            }
        }
        let pos = a.pos;
        let words = (i64::from(s.noise) * i64::from(hush) / 1000) as i32;
        if words > 0 {
            self.emit(Sound {
                at: pos,
                loudness: words,
                by: Some(id),
            });
        }
        self.events.push(Event::Cast { who: id, spell, at });
        self.train(id, Skill::Arcana, self.rules.skills.cast, On::Once);
        let until = self.tick + u64::from(s.lasts);
        match s.kind {
            SpellKind::Silence | SpellKind::Fog => self.zones.push(Zone {
                kind: s.kind,
                at,
                radius: s.radius,
                until,
            }),
            SpellKind::Light => {}
            SpellKind::Fireball => self.burst(id, &s, at),
        }
        // Its light, if it gives any: Light's own, a fireball's flash.
        if s.light > 0 && s.lasts > 0 {
            self.zones.push(Zone {
                kind: SpellKind::Light,
                at,
                radius: s.light,
                until,
            });
        }
    }

    /// A fireball bursts on `at`: everything within its reach is harmed (its caster too); what it
    /// puts down burns, as do the bodies lying there. Heard across the map.
    fn burst(&mut self, id: ActorId, s: &SpellDef, at: TilePos) {
        let reach = Zone {
            kind: s.kind,
            at,
            radius: s.radius,
            until: 0,
        };
        let caught: Vec<ActorId> = self
            .actors
            .iter()
            .filter(|(_, a)| reach.holds(a.pos))
            .map(|(n, _)| n)
            .collect();
        let (lo, hi) = s.damage;
        let span = (hi - lo).max(0) as u32 + 1;
        for target in caught {
            if self.actors.get(target).is_some_and(|a| a.dead) {
                if self.burnable(target) {
                    self.set_alight(target);
                }
                continue;
            }
            let r = roll(self.seed, self.tick, target.index() as u64, Purpose::Burst);
            let damage = lo + below(r, span) as i32;
            if let Some(t) = self.actors.get_mut(target) {
                t.body.health -= damage;
            }
            self.interrupt(target);
            self.startle_by(target, id);
            self.unturn(target, id);
            self.wake(target, crate::needs::Woke::Hurt);
            self.events.push(Event::Scorched {
                who: id,
                target,
                damage,
            });
            if self.actors.get(target).is_some_and(|t| t.body.health <= 0) {
                self.kill(target);
                self.put_to_rest(id, target);
                if self.burnable(target) {
                    self.set_alight(target);
                }
            }
        }
        self.emit(Sound {
            at,
            loudness: s.boom,
            by: Some(id),
        });
    }

    /// The spells lasting now, on their spots.
    pub fn zones(&self) -> &[Zone] {
        &self.zones
    }

    /// Whether a Silence holds `at`: no sound is made there.
    pub(crate) fn hushed(&self, at: TilePos) -> bool {
        self.zones
            .iter()
            .any(|z| z.kind == SpellKind::Silence && z.holds(at))
    }

    /// Whether a Fog holds `at`: the dead can't see into it, nor out.
    pub(crate) fn fogged(&self, at: TilePos) -> bool {
        self.zones
            .iter()
            .any(|z| z.kind == SpellKind::Fog && z.holds(at))
    }

    /// The spells' lights: where each is, and how far it lights.
    pub fn spell_lights(&self) -> impl Iterator<Item = (TilePos, i32)> + '_ {
        self.zones
            .iter()
            .filter(|z| z.kind == SpellKind::Light)
            .map(|z| (z.at, z.radius))
    }

    /// Each tick: the spells that have run their time end.
    pub(crate) fn fade_spells(&mut self) {
        if self.zones.is_empty() {
            return;
        }
        let tick = self.tick;
        self.zones.retain(|z| z.until > tick);
    }

    /// A night's sleep through: `id`'s slots are all back. True if any came back.
    pub(crate) fn rest_slots(&mut self, id: ActorId) -> bool {
        let most = self.max_slots(id);
        match self.actors.get_mut(id) {
            Some(a) if a.slots < most => {
                a.slots = most;
                true
            }
            _ => false,
        }
    }
}
