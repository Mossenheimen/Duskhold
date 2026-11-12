//! Armour (`CONCEPT.md`: "Armour protects per body part, as Zomboid's clothing does", and the
//! Fighter's heavy armour, which is loud).
//!
//! - **Worn, not held.** A piece covers one or more places on the body: the head, the torso, the
//!   arms (with the hands) and the legs (with the feet: `body::Group`). Putting one on over a
//!   place already covered takes the other off, into the pack. Both take time.
//! - **A blow on a covered place may be turned,** by the piece's chance against that kind of
//!   wound. Then there's no wound (no bleeding, no fever, no pain from it), and only a share of
//!   the harm, a bruise. Each blow turned wears the piece, and at nothing it falls apart.
//! - **What it costs:** worn, a piece weighs a share of what it would in the pack. Heavy pieces
//!   make everything that takes breath cost more, make every footstep carry farther (mail
//!   clinks), and a closed helm narrows what you see.
//! - **The dead in armour:** a wight wears its barrow mail, and a revenant rises in yours. A
//!   weapon's blow on a covered place is turned by the piece's chance against a cut, as much as
//!   the weapon's kind lets it: points go through mail, and blades skid off it
//!   (`ArmourRules::pierce`).
//! - **A blow on one that's down, or one struck unawares, finds the gaps:** armour turns nothing.

use crate::actor::ActorId;
use crate::body::{Group, Part, WoundKind};
use crate::hash::{Purpose, below, roll};
use crate::items::{Item, ItemKind};
use crate::skills::Skill;
use crate::world::{Event, World};

/// A piece of armour's numbers, from its definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Armour {
    /// The places it covers, as `Group::bit`s.
    pub covers: u8,
    /// The chance it turns a blow, out of 10,000, by `WoundKind::ALL`: a scratch, a laceration
    /// and a bite. A weapon's blow is turned as a laceration would be (`ArmourRules::pierce`).
    pub turn: [u32; 3],
    /// The condition a blow turned takes off it (out of `items::NEW`).
    pub wear: u16,
    /// Tiles it adds to every footstep's reach.
    pub noise: i32,
    /// Per mille more on every cost in breath while it's worn (0: nothing more).
    pub endurance: u32,
    /// Per mille of your sight it leaves you (1000: all of it).
    pub sight: u32,
    /// Ticks to put it on. Taking it off is the rules' share of this.
    pub time: u32,
}

/// Armour's rules, per mille: what a blow turned still does (a bruise), what a piece weighs worn
/// of its weight, how much of a piece's chance holds against a blade, a blunt weapon, an axe and
/// a point (by `creature::resist_index`), and how long taking a piece off takes, of putting it
/// on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArmourRules {
    pub bruise: u32,
    pub worn_weight: u32,
    pub pierce: [u32; 4],
    pub off: u32,
}

impl Default for ArmourRules {
    fn default() -> ArmourRules {
        ArmourRules {
            bruise: 500,
            worn_weight: 500,
            pierce: [1000, 600, 800, 400],
            off: 500,
        }
    }
}

/// `x` per mille of `n`.
fn per(n: u32, x: u32) -> u32 {
    (u64::from(n) * u64::from(x) / 1000) as u32
}

impl World {
    /// A piece's numbers, if its definition is armour.
    fn armour_of(&self, item: &Item) -> Option<Armour> {
        match self.defs.item(item.def)?.kind {
            ItemKind::Armour(a) => Some(a),
            _ => None,
        }
    }

    /// What `id` wears, piece by piece, with each one's numbers.
    fn pieces(&self, id: ActorId) -> Vec<Armour> {
        self.actors.get(id).map_or(Vec::new(), |a| {
            a.worn.iter().filter_map(|i| self.armour_of(i)).collect()
        })
    }

    /// The piece `id` wears over `part`, if any: its place in what it wears, and its numbers.
    fn cover(&self, id: ActorId, part: Part) -> Option<(usize, Armour)> {
        let bit = part.group().bit();
        let a = self.actors.get(id)?;
        a.worn.iter().enumerate().find_map(|(n, i)| {
            self.armour_of(i)
                .filter(|arm| arm.covers & bit != 0)
                .map(|arm| (n, arm))
        })
    }

    /// Whether `id`'s armour turns a creature's blow, struck by `by`, that would leave a wound of
    /// `kind` on `part`: the piece that turns it, if one does.
    pub(crate) fn turns_wound(
        &self,
        id: ActorId,
        by: ActorId,
        kind: WoundKind,
        part: Part,
    ) -> Option<usize> {
        let (piece, arm) = self.cover(id, part)?;
        let r = roll(self.seed, self.tick, by.index() as u64, Purpose::Turn);
        (below(r, 10_000) < arm.turn[kind as usize]).then_some(piece)
    }

    /// Whether `id`'s armour turns a weapon's blow (or an arrow), struck by `by` with a weapon of
    /// `skill`: where it lands, and the piece that turns it, if one does. Turned as a cut would
    /// be, as far as the weapon's kind lets it.
    pub(crate) fn turns_weapon(
        &self,
        id: ActorId,
        by: ActorId,
        skill: Option<Skill>,
    ) -> Option<(usize, Part)> {
        if self.actors.get(id)?.worn.is_empty() {
            return None;
        }
        let who = by.index() as u64;
        let part = Part::from_roll(below(roll(self.seed, self.tick, who, Purpose::Part), 100));
        let (piece, arm) = self.cover(id, part)?;
        let holds = self.rules.armour.pierce[crate::creature::resist_index(skill)];
        let chance = per(arm.turn[WoundKind::Laceration as usize], holds);
        let r = roll(self.seed, self.tick, who, Purpose::Turn);
        (below(r, 10_000) < chance).then_some((piece, part))
    }

    /// What a blow turned still does: a bruise.
    pub(crate) fn bruise(&self, damage: i32) -> i32 {
        (i64::from(damage) * i64::from(self.rules.armour.bruise) / 1000) as i32
    }

    /// A blow turned: the piece that turned it wears, and falls apart at nothing.
    pub(crate) fn turned(&mut self, id: ActorId, by: ActorId, piece: usize, part: Part) {
        let Some(mut item) = self.actors.get(id).and_then(|a| a.worn.get(piece).copied()) else {
            return;
        };
        let wear = self.armour_of(&item).map_or(0, |a| a.wear);
        item.condition = item.condition.saturating_sub(wear);
        self.events.push(Event::Glanced {
            who: id,
            by,
            item: item.def,
            part,
        });
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if item.condition == 0 {
            a.worn.remove(piece);
            self.events.push(Event::WeaponBroke {
                who: id,
                item: item.def,
            });
        } else {
            a.worn[piece] = item;
        }
    }

    /// What `id`'s armour weighs worn, in grams: a share of what it would in the pack.
    pub(crate) fn worn_weight(&self, id: ActorId) -> u32 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        let grams: u32 = a.worn.iter().map(|i| self.defs.weight(i)).sum();
        per(grams, self.rules.armour.worn_weight)
    }

    /// Per mille: what `id`'s armour makes every cost in breath come to.
    pub(crate) fn armour_endurance(&self, id: ActorId) -> u32 {
        1000 + self.pieces(id).iter().map(|a| a.endurance).sum::<u32>()
    }

    /// The tiles `id`'s armour adds to every footstep.
    pub(crate) fn armour_noise(&self, id: ActorId) -> i32 {
        self.pieces(id).iter().map(|a| a.noise).sum()
    }

    /// Per mille: what's left of `id`'s sight under its helm.
    pub(crate) fn armour_sight(&self, id: ActorId) -> u32 {
        self.pieces(id)
            .iter()
            .map(|a| a.sight)
            .min()
            .unwrap_or(1000)
    }

    /// Puts a piece on `id` at once, taking off (into the pack) whatever covered the same places:
    /// for a kit, a creature's own armour, tests and tools. False if it isn't armour.
    pub fn don(&mut self, id: ActorId, item: Item) -> bool {
        let Some(arm) = self.armour_of(&item) else {
            return false;
        };
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return false;
        };
        let mut n = 0;
        while n < a.worn.len() {
            let covers = match defs.item(a.worn[n].def).map(|d| &d.kind) {
                Some(ItemKind::Armour(other)) => other.covers,
                _ => 0,
            };
            if covers & arm.covers != 0 {
                let off = a.worn.remove(n);
                defs.stow(&mut a.items, off);
            } else {
                n += 1;
            }
        }
        a.worn.push(Item { count: 1, ..item });
        true
    }

    /// The ticks it takes `id` to put on the piece at `slot` of its pack, if it's there (as
    /// `def`) and it's armour.
    pub(crate) fn wear_ticks(&self, id: ActorId, slot: u16, def: u16) -> Option<u32> {
        let a = self.actors.get(id)?;
        let item = a.items.get(usize::from(slot)).filter(|i| i.def == def)?;
        Some(self.armour_of(item)?.time.max(1))
    }

    /// Puts on the piece at `slot` of `id`'s pack (still `def`).
    pub(crate) fn put_on(&mut self, id: ActorId, slot: u16, def: u16) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let slot = usize::from(slot);
        if a.items.get(slot).is_none_or(|i| i.def != def) {
            return;
        }
        let mut item = a.items[slot];
        if item.count > 1 {
            a.items[slot].count -= 1;
            item.count = 1;
        } else {
            a.items.remove(slot);
        }
        if self.don(id, item) {
            self.events.push(Event::Wore { who: id, item: def });
        }
    }

    /// The ticks it takes `id` to take off its `piece`, if that's still `def`.
    pub(crate) fn take_off_ticks(&self, id: ActorId, piece: u16, def: u16) -> Option<u32> {
        let a = self.actors.get(id)?;
        let item = a.worn.get(usize::from(piece)).filter(|i| i.def == def)?;
        let time = self.armour_of(item)?.time;
        Some(per(time, self.rules.armour.off).max(1))
    }

    /// Takes off `id`'s `piece` (still `def`), into the pack.
    pub(crate) fn take_off(&mut self, id: ActorId, piece: u16, def: u16) {
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let piece = usize::from(piece);
        if a.worn.get(piece).is_none_or(|i| i.def != def) {
            return;
        }
        let item = a.worn.remove(piece);
        defs.stow(&mut a.items, item);
        self.events.push(Event::TookOff { who: id, item: def });
    }
}

impl Group {
    /// Its bit, in `Armour::covers`.
    pub fn bit(self) -> u8 {
        match self {
            Group::Head => 1,
            Group::Torso => 2,
            Group::Arms => 4,
            Group::Legs => 8,
        }
    }

    pub const ALL: [Group; 4] = [Group::Head, Group::Torso, Group::Arms, Group::Legs];

    pub fn name(self) -> &'static str {
        match self {
            Group::Head => "head",
            Group::Torso => "torso",
            Group::Arms => "arms",
            Group::Legs => "legs",
        }
    }
}
