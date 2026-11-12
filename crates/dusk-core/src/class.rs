//! Classes and talents (`CONCEPT.md`, character): the Fighter and the Cleric.
//!
//! - **A class is data** (`data/classes.toml`): the skills it learns faster, its endurance (a
//!   Fighter's costs less: "more of it than anyone"), its signature (a Fighter shrugs off
//!   grabs: the dead get hold of it less often; a Cleric turns the dead, with faith:
//!   `faith.rs`), its starting kit, and the talents it picks from.
//! - **Choosing one:** a character comes into the world as the first class in the data, and
//!   takes up another before it's under way (`World::choose_class`): in its first game minute,
//!   with nothing learnt or done. The new class's kit replaces the old.
//! - **Talents are data too** (`data/talents.toml`): each reads in one line, and most change
//!   numbers the rules already use: health, endurance, harm with a kind of weapon, a shove's
//!   knock-down, struggling free, how many it takes to drag you down, faith, how long and how far
//!   the dead are turned. Some add something: Cleave (a blow that puts one down carries on into
//!   another beside you), Second Wind (a verb: once between sleeps, catch your breath), Mending
//!   and Warding (verbs of faith). One works only in a doorway (the Doorwarden).
//! - **Each level from 2 offers three** of the class's talents not yet taken, chosen by the
//!   seed, and you pick one when you like: a choice, not a timed action, like a mastery. There
//!   are no trees and no one-point picks: each talent is a thing you'd notice.

use crate::actor::ActorId;
use crate::body::FULL;
use crate::geom::Dir8;
use crate::grid::Edge;
use crate::hash::{Purpose, below, roll};
use crate::items::Item;
use crate::skills::Skill;
use crate::world::{Event, World};

/// What a character starts with: a weapon in hand, and its pack.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Kit {
    pub wield: Option<u16>,
    pub pack: Vec<Item>,
}

/// A class, in the core's units.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Class {
    pub id: String,
    pub name: String,
    /// What it is, in a line, for whoever chooses.
    pub line: String,
    /// The skills it learns faster, and how much faster (per mille).
    pub skills: Vec<Skill>,
    pub learn: u32,
    /// Per mille: what its running, swings and struggles cost in endurance.
    pub endurance: u32,
    /// Per mille: the chance a creature's hit takes hold of it.
    pub grabbed: u32,
    /// Its talents, by their numbers in `Defs::talents`.
    pub talents: Vec<u8>,
    /// The faith it holds, in thousandths of a point (0: none: `faith.rs`).
    pub faith: i32,
    /// It turns the dead (`faith.rs`): the Cleric's signature.
    pub turns: bool,
    /// What it starts with; None: the rules' kit, everyone's (`Defs::wield`, `Defs::kit`).
    pub kit: Option<Kit>,
    /// The armour it starts in, by item number (`armour.rs`), whatever its kit.
    pub wear: Vec<u16>,
}

/// A talent, in the core's units. Per mille is 1000 for none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Talent {
    pub id: String,
    pub name: String,
    pub line: String,
    /// Health it adds, out of `body::FULL`.
    pub health: i32,
    /// Per mille: endurance costs.
    pub endurance: u32,
    /// Per mille: a blow's harm, with any weapon or only `skill`'s.
    pub damage: u32,
    pub skill: Option<Skill>,
    /// Added, out of 10,000: a blow landing, a shove knocking down, struggling free.
    pub hit: i32,
    pub knock_down: i32,
    pub struggle: i32,
    /// Per mille: the chance a creature's hit takes hold of you.
    pub grabbed: u32,
    /// How many more must hold you to drag you down.
    pub hold: u32,
    /// Its hit and grabbed numbers count only beside a door (on a side of your tile).
    pub doorway: bool,
    /// A blow that puts one down carries on into another beside you.
    pub cleave: bool,
    /// Gives the Second Wind: once between sleeps, catch your breath.
    pub second_wind: bool,
    /// Faith it adds to what the class holds, in thousandths of a point (`faith.rs`).
    pub faith: i32,
    /// Gives Mending: faith closes the worst bleeding wound, and gives health back.
    pub mending: bool,
    /// Gives Warding: faith wards a shut door against the dead.
    pub warding: bool,
    /// Prayer at a shrine cures ghoul fever too.
    pub purify: bool,
    /// Per mille: how fast dread rises while you hold a light up.
    pub steady: u32,
    /// Per mille: how long the dead you turn stay turned; and tiles added to the turning's reach.
    pub turn_time: u32,
    pub turn_reach: i32,
    /// The dead you turn with this much health or less (out of `body::FULL`) fall instead.
    pub sunder: i32,
    /// The dead you put down stay down, as with a Hallowed weapon.
    pub last_rites: bool,
}

impl Default for Talent {
    fn default() -> Talent {
        Talent {
            id: String::new(),
            name: String::new(),
            line: String::new(),
            health: 0,
            endurance: 1000,
            damage: 1000,
            skill: None,
            hit: 0,
            knock_down: 0,
            struggle: 0,
            grabbed: 1000,
            hold: 0,
            doorway: false,
            cleave: false,
            second_wind: false,
            faith: 0,
            mending: false,
            warding: false,
            purify: false,
            steady: 1000,
            turn_time: 1000,
            turn_reach: 0,
            sunder: 0,
            last_rites: false,
        }
    }
}

/// How many talents a level offers.
pub const OFFERED: usize = 3;
/// What the Second Wind gives back, out of `body::FULL`: a tenth of your health.
const SECOND_WIND_HEALTH: i32 = 100;

fn per(n: u32, by: u32) -> u32 {
    (u64::from(n) * u64::from(by) / 1000) as u32
}

impl World {
    /// An actor's class.
    pub fn class_of(&self, id: ActorId) -> Option<&Class> {
        let a = self.actors.get(id)?;
        self.defs.classes.get(usize::from(a.class?))
    }

    /// Whether `id` can still take up a class: a living character not yet under way, in its first
    /// game minute, with nothing learnt or done.
    pub fn can_choose_class(&self, id: ActorId) -> bool {
        let Some(a) = self.actors.get(id) else {
            return false;
        };
        a.kind.is_character()
            && !a.dead
            && a.talents.is_empty()
            && a.deeds == 0
            && a.skills.xp.iter().all(|&xp| xp == 0)
            && self.clock_minutes() == a.born
    }

    /// `id` takes up a class before it's under way: the class's kit replaces the one it came
    /// with, and its faith is full. False if it can't choose now, or there's no such class.
    pub fn choose_class(&mut self, id: ActorId, class: u8) -> bool {
        if !self.can_choose_class(id) || self.defs.classes.get(usize::from(class)).is_none() {
            return false;
        }
        if let Some(a) = self.actors.get_mut(id) {
            a.class = Some(class);
            a.hands = None;
            a.light = None;
            a.shield = None;
            a.bracing = false;
            a.items.clear();
            a.worn.clear();
        }
        self.equip(id);
        self.events.push(Event::ClassChosen { who: id, class });
        true
    }

    /// Gives a character its class's kit (or the rules', everyone's: something in hand, the rest
    /// in its pack) and its class's faith, full.
    pub(crate) fn equip(&mut self, id: ActorId) {
        let defs = self.defs.clone();
        let faith = self.max_faith(id);
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let kit = a
            .class
            .and_then(|c| defs.classes.get(usize::from(c)))
            .and_then(|c| c.kit.as_ref());
        let (wield, pack) = match kit {
            Some(k) => (k.wield, k.pack.as_slice()),
            None => (defs.wield, defs.kit.as_slice()),
        };
        a.hands = wield.map(|def| Item::new(def, 1));
        for &item in pack {
            defs.stow(&mut a.items, item);
        }
        a.faith = faith;
        // And the armour it comes in (`armour.rs`).
        let wear = a
            .class
            .and_then(|c| defs.classes.get(usize::from(c)))
            .map(|c| c.wear.clone())
            .unwrap_or_default();
        for def in wear {
            self.don(id, Item::new(def, 1));
        }
    }

    /// An actor's talents, as definitions.
    pub(crate) fn talents_of(&self, id: ActorId) -> impl Iterator<Item = &Talent> + '_ {
        let taken = self
            .actors
            .get(id)
            .map(|a| a.talents.as_slice())
            .unwrap_or(&[]);
        taken
            .iter()
            .filter_map(|&t| self.defs.talents.get(usize::from(t)))
    }

    /// Whether `id` stands beside a door: one on a side of its tile, open or shut.
    fn in_a_doorway(&self, id: ActorId) -> bool {
        let Some(a) = self.actors.get(id) else {
            return false;
        };
        [Dir8::N, Dir8::E, Dir8::S, Dir8::W]
            .iter()
            .any(|&d| matches!(self.grid.edge(a.pos, d), Edge::Door { .. }))
    }

    /// The talents that count where `id` stands now (a doorway one only in a doorway).
    fn talents_here(&self, id: ActorId) -> impl Iterator<Item = &Talent> + '_ {
        let doorway = self.in_a_doorway(id);
        self.talents_of(id).filter(move |t| !t.doorway || doorway)
    }

    /// Per mille: what `id`'s endurance costs come to, by its class and talents, its pain
    /// (`dread.rs`), the armour it wears (`armour.rs`) and its Fitness (`fitness.rs`).
    pub(crate) fn endurance_rate(&self, id: ActorId) -> i32 {
        let class = self.class_of(id).map_or(1000, |c| c.endurance);
        let rate = self.talents_of(id).fold(class, |r, t| per(r, t.endurance));
        let rate = per(rate, self.pain_cost(id));
        let rate = per(rate, self.armour_endurance(id));
        per(rate, self.fitness_cost(id)) as i32
    }

    /// The chance, out of 10,000, that a creature's hit takes hold of `id`, by its class and
    /// talents, from the rules' chance.
    pub(crate) fn grabbed_chance(&self, id: ActorId, chance: u32) -> u32 {
        let class = self.class_of(id).map_or(1000, |c| c.grabbed);
        let rate = self.talents_here(id).fold(class, |r, t| per(r, t.grabbed));
        per(chance, rate)
    }

    /// What `id`'s talents add to struggling free, and how many more must hold it to drag it
    /// down.
    pub(crate) fn struggle_bonus(&self, id: ActorId) -> i32 {
        self.talents_of(id).map(|t| t.struggle).sum()
    }

    pub(crate) fn extra_hold(&self, id: ActorId) -> u32 {
        self.talents_of(id).map(|t| t.hold).sum()
    }

    /// What `id`'s talents add to a shove's knock-down.
    pub(crate) fn knock_down_bonus(&self, id: ActorId) -> i32 {
        self.talents_of(id).map(|t| t.knock_down).sum()
    }

    /// A swing as `id`'s talents change it: its harm (with any weapon, or one of a skill) and
    /// its chance to land.
    pub(crate) fn talent_swing(
        &self,
        id: ActorId,
        skill: Option<Skill>,
        mut swing: crate::world::Swing,
    ) -> crate::world::Swing {
        for t in self.talents_here(id) {
            if t.skill.is_none() || t.skill == skill {
                swing.damage = (
                    per(swing.damage.0.max(0) as u32, t.damage) as i32,
                    per(swing.damage.1.max(0) as u32, t.damage) as i32,
                );
            }
            swing.hit = (swing.hit as i32 + t.hit).clamp(0, 10_000) as u32;
        }
        swing
    }

    /// The health `id`'s talents add.
    pub(crate) fn talent_health(&self, id: ActorId) -> i32 {
        self.talents_of(id).map(|t| t.health).sum()
    }

    /// Whether `id` has Cleave.
    pub(crate) fn cleaves(&self, id: ActorId) -> bool {
        self.talents_of(id).any(|t| t.cleave)
    }

    /// Talent picks `id` has earned and not made: one for each level past the first.
    pub fn picks_left(&self, id: ActorId) -> u8 {
        let Some(a) = self.actors.get(id) else {
            return 0;
        };
        if a.class.is_none() {
            return 0;
        }
        let earned = self.level(id).saturating_sub(1);
        earned.saturating_sub(a.talents.len() as u8)
    }

    /// The talents offered for `id`'s next pick: three of its class's not yet taken, chosen by
    /// the seed (the same each time it's asked, until a pick is made). None with no pick left.
    pub fn talent_offer(&self, id: ActorId) -> Vec<u8> {
        if self.picks_left(id) == 0 {
            return Vec::new();
        }
        let (Some(a), Some(class)) = (self.actors.get(id), self.class_of(id)) else {
            return Vec::new();
        };
        let mut pool: Vec<u8> = class
            .talents
            .iter()
            .copied()
            .filter(|t| !a.talents.contains(t))
            .collect();
        let (who, nth) = (id.index() as u64, a.talents.len() as u64);
        let mut out = Vec::new();
        for k in 0..OFFERED as u64 {
            if pool.is_empty() {
                break;
            }
            let r = roll(self.seed, nth * 8 + k, who, Purpose::Talent);
            out.push(pool.remove(below(r, pool.len() as u32) as usize));
        }
        out
    }

    /// `id` picks a talent from its offer: false if it isn't on offer. Its health comes at once.
    pub fn choose_talent(&mut self, id: ActorId, talent: u8) -> bool {
        if !self.talent_offer(id).contains(&talent) {
            return false;
        }
        let health = self
            .defs
            .talents
            .get(usize::from(talent))
            .map_or(0, |t| t.health);
        let Some(a) = self.actors.get_mut(id) else {
            return false;
        };
        a.talents.push(talent);
        a.body.health += health;
        self.events.push(Event::TalentChosen { who: id, talent });
        true
    }

    /// Whether `id` can catch its second wind now: it has the talent, and hasn't used it since
    /// it last slept.
    pub fn can_second_wind(&self, id: ActorId) -> bool {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return false;
        };
        !a.winded && self.talents_of(id).any(|t| t.second_wind)
    }

    /// The Second Wind, on its strike tick: breath full again, and a little health back.
    pub(crate) fn second_wind(&mut self, id: ActorId) {
        if !self.can_second_wind(id) {
            return;
        }
        let most = self.max_health(id);
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        a.winded = true;
        a.body.endurance = FULL;
        a.body.exhausted = false;
        a.body.health = (a.body.health + SECOND_WIND_HEALTH).min(most);
        self.events.push(Event::SecondWind { who: id });
    }
}
