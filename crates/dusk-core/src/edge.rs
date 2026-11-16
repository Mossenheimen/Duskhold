//! Each class of weapon's own edge (`CONCEPT.md`, the fight; Zomboid's weapon classes, and Path
//! of Exile's implicits, which every weapon of a base has): what every weapon of a skill gives a
//! blow past its numbers, shown on its card. The game's are rules.toml's `[edges]`:
//!
//! - **a blade finishes:** more harm to one that's down, on top of the double any blow does there;
//! - **blunt knocks down:** a chance a blow that lands puts it on the ground (and a mace shatters
//!   bone, as before: that's each kind's `Creature::resist`);
//! - **an axe bleeds:** a blow that isn't turned leaves a wound that bleeds for a while, each
//!   blow another; bone has no blood (`Creature::bleeds`);
//! - **a spear holds off:** a chance a blow drives it back a tile, if there's room, breaking its
//!   hold.
//!
//! Only a character's blow with a weapon of the skill has its edge: not bare hands, nor a bow
//! swung, nor the dead's blows.

use crate::actor::ActorId;
use crate::body::{Part, Wound, WoundKind};
use crate::geom::{Dir8, TilePos};
use crate::hash::{Purpose, below, roll};
use crate::skills::Skill;
use crate::world::{Event, World};

/// What a class of weapon gives a blow past its numbers. Any class may have any of them; the
/// game gives each its own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WeaponEdge {
    /// More harm to one that's down, per mille, on top of the double any blow does (0: none).
    pub down: u32,
    /// Out of 10,000: the chance a blow that lands knocks it down.
    pub fell: u32,
    /// Ticks a blow that isn't turned leaves it bleeding (0: none).
    pub bleed: u32,
    /// Out of 10,000: the chance a blow that lands drives it back a tile.
    pub push: u32,
}

impl WeaponEdge {
    /// Whether it gives anything.
    pub fn is_none(&self) -> bool {
        *self == WeaponEdge::default()
    }

    /// A blow's harm on one that's down: the double any blow does, and this edge's more.
    pub fn on_the_ground(&self, damage: i32) -> i32 {
        (i64::from(damage) * 2 * (1000 + i64::from(self.down)) / 1000) as i32
    }
}

/// Each weapon skill's edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    pub blade: WeaponEdge,
    pub blunt: WeaponEdge,
    pub axe: WeaponEdge,
    pub spear: WeaponEdge,
}

impl Edges {
    /// A weapon skill's edge: none for any other skill, or none (bare hands).
    pub fn of(&self, skill: Option<Skill>) -> WeaponEdge {
        match skill {
            Some(Skill::Blade) => self.blade,
            Some(Skill::Blunt) => self.blunt,
            Some(Skill::Axe) => self.axe,
            Some(Skill::Spear) => self.spear,
            _ => WeaponEdge::default(),
        }
    }
}

impl World {
    /// The edge of what `id` strikes with: its weapon's skill's, a character's only.
    pub fn edge(&self, id: ActorId) -> WeaponEdge {
        match self.actors.get(id) {
            Some(a) if a.kind.is_character() => self.rules.edges.of(self.weapon_skill(id)),
            _ => WeaponEdge::default(),
        }
    }

    /// An axe's blow that wasn't turned leaves `target` bleeding for `ticks`: a wound of its own,
    /// on the torso, bleeding as any wound does. Not a kind with no blood.
    pub(crate) fn bleed(&mut self, target: ActorId, ticks: u32) {
        if ticks == 0 || !self.stats(target).is_some_and(|c| c.bleeds) {
            return;
        }
        let heal_left = self.rules.needs.wound_heal[WoundKind::Laceration as usize];
        if let Some(t) = self.actors.get_mut(target).filter(|t| !t.dead) {
            t.body.wounds.push(Wound {
                part: Part::Torso,
                kind: WoundKind::Laceration,
                bleeding: ticks,
                bandaged: false,
                heal_left,
            });
        }
    }

    /// Each tick: the dead and the wild bleed from the wounds an axe left them, as a character
    /// bleeds (`combat.rs`'s `bodies`: each bleeding wound's harm every `bleed_every` ticks), and
    /// one that bleeds out dies. A wound that's stopped is gone: nothing else reads a creature's.
    /// `ids`: the living where the world is awake.
    pub(crate) fn bleed_out(&mut self, ids: &[ActorId]) {
        let (c, tick) = (self.rules.combat, self.tick);
        let beat = tick.is_multiple_of(u64::from(c.bleed_every.max(1)));
        for &id in ids {
            let Some(a) = self.actors.get_mut(id) else {
                continue;
            };
            if a.kind.is_character() || a.body.wounds.is_empty() {
                continue;
            }
            let bleeding = a.body.wounds.iter().filter(|w| w.bleeding > 0).count() as i32;
            for w in &mut a.body.wounds {
                w.bleeding = w.bleeding.saturating_sub(1);
            }
            a.body.wounds.retain(|w| w.bleeding > 0);
            if bleeding > 0 && beat {
                a.body.health -= bleeding * c.bleed_damage;
                if a.body.health <= 0 {
                    self.kill(id);
                }
            }
        }
    }

    /// A spear's blow that landed may drive `target` (struck on `at`) back a tile along `dir`, if
    /// it's on its feet and there's room: breaking its hold, as a shove does.
    pub(crate) fn drive_back(
        &mut self,
        id: ActorId,
        target: ActorId,
        at: TilePos,
        dir: Dir8,
        chance: u32,
    ) {
        if chance == 0 {
            return;
        }
        let r = roll(self.seed, self.tick, id.index() as u64, Purpose::Push);
        let standing = self
            .actors
            .get(target)
            .is_some_and(|t| !t.dead && t.prone_until == 0);
        if below(r, 10_000) >= chance || !standing || !self.knock_back(target, at, dir) {
            return;
        }
        self.release(target);
        self.events.push(Event::DrivenBack { who: id, target });
    }
}
