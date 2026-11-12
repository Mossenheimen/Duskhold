//! Skills that grow by use (`CONCEPT.md`, levelling): only real use under real risk pays, and
//! repeats pay less.
//!
//! - **What pays:** a blow that lands on a foe trains the skill of the weapon that struck it (half
//!   on one that's down); running with one of the dead after you, or under a heavy load, trains
//!   Athletics; a bandage on a wound that's bleeding trains First Aid; a board nailed across a
//!   door or window trains Carpentry; each of the dead turned, and a bleeding wound mended with
//!   faith, trains Devotion; a wild plant picked trains Foraging. A swing at the air or at a
//!   door, or a run with nothing after you, teaches nothing. Bare fists train no skill.
//! - **Repeats pay less:** the same act on the same foe, or in the same spot, pays `repeat` of
//!   what it paid the time before, until it's been left alone for `forget` ticks. One of the dead
//!   stuck in a fence is worth a few blows, not a level; running round in circles, a few laps.
//!   (Oblivion's players trained Acrobatics by jumping under a well: here there's nothing to farm.)
//! - **Levels** 0 to 10 come from XP. Every skill rises to `free_cap` (7) freely; `masteries` (3)
//!   of them can go on to 10, each chosen once it's reached the cap. Until it's chosen, a skill at
//!   the cap stops learning, so characters stay distinct (Ultima Online's skill cap).
//! - **What a level does,** a little each: a weapon skill's blows land more often and harder, for
//!   less breath and less wear on the weapon; Athletics runs faster for less breath; First Aid
//!   bandages faster; Carpentry boards up faster, with stronger boards; Stealth sneaks faster,
//!   and its sneak attacks do more harm (`stealth.rs`); Cooking cooks faster (`cooking.rs`);
//!   Devotion turns the dead for longer, and makes faith go further (`faith.rs`); Foraging
//!   finds one more of a herb more often (`forage.rs`).

use crate::actor::{ActorId, ActorKind};
use crate::geom::{Dir8, TilePos};
use crate::items::ItemKind;
use crate::world::{Event, Swing, World};

/// The skills there is a use for so far. (The rest come with what trains them.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Skill {
    Blade,
    Blunt,
    Axe,
    Spear,
    Athletics,
    FirstAid,
    Carpentry,
    /// Sneaking, and the sneak attack (`stealth.rs`).
    Stealth,
    /// Blocking and bashing with a shield (`shield.rs`).
    Shield,
    /// Shooting a bow (`archery.rs`).
    Archery,
    /// Cooking at a fire (`cooking.rs`).
    Cooking,
    /// The Cleric's: turning the dead and mending wounds with faith (`faith.rs`).
    Devotion,
    /// Picking wild plants (`forage.rs`).
    Foraging,
}

/// How many skills there are.
pub const SKILLS: usize = 13;

impl Skill {
    pub const ALL: [Skill; SKILLS] = [
        Skill::Blade,
        Skill::Blunt,
        Skill::Axe,
        Skill::Spear,
        Skill::Athletics,
        Skill::FirstAid,
        Skill::Carpentry,
        Skill::Stealth,
        Skill::Shield,
        Skill::Archery,
        Skill::Cooking,
        Skill::Devotion,
        Skill::Foraging,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Skill::Blade => "Blade",
            Skill::Blunt => "Blunt",
            Skill::Axe => "Axe",
            Skill::Spear => "Spear",
            Skill::Athletics => "Athletics",
            Skill::FirstAid => "First Aid",
            Skill::Carpentry => "Carpentry",
            Skill::Stealth => "Stealth",
            Skill::Shield => "Shield",
            Skill::Archery => "Archery",
            Skill::Cooking => "Cooking",
            Skill::Devotion => "Devotion",
            Skill::Foraging => "Foraging",
        }
    }

    /// Its name in the data files.
    pub fn id(self) -> &'static str {
        match self {
            Skill::Blade => "blade",
            Skill::Blunt => "blunt",
            Skill::Axe => "axe",
            Skill::Spear => "spear",
            Skill::Athletics => "athletics",
            Skill::FirstAid => "first_aid",
            Skill::Carpentry => "carpentry",
            Skill::Stealth => "stealth",
            Skill::Shield => "shield",
            Skill::Archery => "archery",
            Skill::Cooking => "cooking",
            Skill::Devotion => "devotion",
            Skill::Foraging => "foraging",
        }
    }

    pub fn from_id(id: &str) -> Option<Skill> {
        Skill::ALL.into_iter().find(|s| s.id() == id)
    }

    /// A skill a weapon trains and uses.
    pub fn is_weapon(self) -> bool {
        matches!(
            self,
            Skill::Blade | Skill::Blunt | Skill::Axe | Skill::Spear
        )
    }
}

/// The highest level.
pub const MAX_LEVEL: u8 = 10;

/// The skills' numbers, in the core's units: XP in thousandths of a point, shares per mille.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillRules {
    /// The XP it takes to reach each level from 1 to 10, rising.
    pub levels: [u32; MAX_LEVEL as usize],
    /// Every skill rises this far freely; `masteries` of them go on to 10.
    pub free_cap: u8,
    pub masteries: u8,
    /// Each repeat of an act pays this much of what it paid last time (per mille), until it's
    /// been left alone `forget` ticks.
    pub repeat: u32,
    pub forget: u64,
    /// Running in a square this many tiles across is running in the same spot.
    pub spot: i32,
    /// XP for a blow that lands (on one that's down, `down` per mille of it), a tile run under
    /// risk, a bandage on a wound that's bleeding, and a board nailed on.
    pub hit: u32,
    pub down: u32,
    pub run: u32,
    pub bandage: u32,
    pub board: u32,
    /// A run trains Athletics with one of the dead after you within `chased` tiles, or carrying
    /// more than `loaded` per mille of what you can.
    pub chased: i32,
    pub loaded: u32,
    /// What a level does, a level at a time: a weapon skill adds `weapon_hit` to a blow's chance
    /// to land (out of 10,000), and `weapon_damage` per mille to its harm, and takes
    /// `weapon_endurance` and `weapon_wear` per mille off its breath and wear. Athletics adds
    /// `run_speed` per mille to running speed and takes `run_endurance` off its breath. First Aid
    /// takes `bandage_time` per mille off a bandage's time. Carpentry takes `board_time` off the
    /// time to nail a board on, and adds `board_strength` to the board.
    pub weapon_hit: u32,
    pub weapon_damage: u32,
    pub weapon_endurance: u32,
    pub weapon_wear: u32,
    pub run_speed: u32,
    pub run_endurance: u32,
    pub bandage_time: u32,
    pub board_time: u32,
    pub board_strength: u32,
    /// Stealth (`stealth.rs`): XP for a tile sneaked with the dead near and none after you,
    /// and for a sneak attack that lands; and a level adds `sneak_pace` per mille to sneaking
    /// pace and `sneak_harm` to a sneak attack's harm.
    pub sneak: u32,
    pub sneak_attack: u32,
    pub sneak_pace: u32,
    pub sneak_harm: u32,
    /// Shield (`shield.rs`): XP for a block or a bash, and a level adds `shield_block` to a
    /// shield's chance to block (out of 10,000).
    pub shield: u32,
    pub shield_block: u32,
    /// Cooking (`cooking.rs`): a level takes `cook_time` per mille off a meal's time.
    pub cook_time: u32,
    /// Devotion (`faith.rs`): XP for each of the dead turned, and for a bleeding wound mended
    /// with faith; and a level adds `devotion_turn` per mille to how long the dead stay turned,
    /// and takes `devotion_cost` per mille off what faith costs.
    pub turn: u32,
    pub mend: u32,
    pub devotion_turn: u32,
    pub devotion_cost: u32,
    /// Foraging (`forage.rs`): XP for a plant picked; and a level adds `forage_more` per mille
    /// to the chance of one more of its herb.
    pub pick: u32,
    pub forage_more: u32,
}

impl Default for SkillRules {
    /// The fixed test set: a few blows for the first level, the rest of the curve steepening.
    fn default() -> SkillRules {
        SkillRules {
            levels: [
                10_000, 30_000, 70_000, 150_000, 300_000, 550_000, 900_000, 1_400_000, 2_100_000,
                3_000_000,
            ],
            free_cap: 7,
            masteries: 3,
            repeat: 750,
            // Six game hours.
            forget: 6 * 60 * 25,
            spot: 8,
            hit: 1000,
            down: 500,
            run: 200,
            bandage: 3000,
            board: 2000,
            chased: 12,
            loaded: 500,
            weapon_hit: 100,
            weapon_damage: 50,
            weapon_endurance: 30,
            weapon_wear: 50,
            run_speed: 20,
            run_endurance: 40,
            bandage_time: 50,
            board_time: 50,
            board_strength: 100,
            sneak: 300,
            sneak_attack: 3000,
            sneak_pace: 30,
            sneak_harm: 100,
            shield: 1500,
            shield_block: 300,
            cook_time: 50,
            turn: 2000,
            mend: 3000,
            devotion_turn: 100,
            devotion_cost: 40,
            pick: 1000,
            forage_more: 100,
        }
    }
}

/// What an act was done to, so the same act on the same thing pays less each time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum On {
    /// A foe.
    Foe(ActorId),
    /// A spot on the map: a square `SkillRules::spot` tiles across, and its floor.
    Spot(i32, i32, i32),
    /// A door or window, named by the tile that owns its edge and the side.
    Edge(TilePos, Dir8),
    /// Nothing that repeats (a wound is never the same wound twice).
    Once,
}

/// An act that paid lately: how many times, and when last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub(crate) struct Repeat {
    skill: Skill,
    on: On,
    times: u32,
    last: u64,
}

/// The most recent acts remembered. Past this many, the stalest is forgotten.
const REMEMBERED: usize = 32;

/// An actor's skills.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Skills {
    /// XP in each skill, by `Skill::ALL`, in thousandths.
    pub xp: [u32; SKILLS],
    /// The skills chosen to go on past the free cap, in the order chosen.
    pub mastered: Vec<Skill>,
    /// Acts that paid lately, so a repeat pays less.
    pub(crate) repeats: Vec<Repeat>,
}

/// How a skill stands: its level, how far it is toward the next (per mille), and whether it's
/// waiting at the free cap for a mastery, mastered, or as high as it goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Standing {
    pub level: u8,
    pub progress: u32,
    pub state: Rise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rise {
    /// Learning.
    Rising,
    /// At the free cap: it learns no more unless it's mastered.
    AtCap,
    /// Chosen as a mastery, it goes on to 10.
    Mastered,
    /// At 10.
    Top,
}

/// The level some XP reaches, with no cap.
pub fn level_of(xp: u32, levels: &[u32; MAX_LEVEL as usize]) -> u8 {
    levels.iter().filter(|&&l| xp >= l).count() as u8
}

/// `base` changed by `per` per mille a level: more (`up`) or less, never below nothing.
fn by_level(base: i64, per: u32, level: u8, up: bool) -> i64 {
    let change = i64::from(per) * i64::from(level);
    let rate = if up {
        1000 + change
    } else {
        (1000 - change).max(0)
    };
    base * rate / 1000
}

impl Skills {
    fn index(skill: Skill) -> usize {
        skill as usize
    }

    pub fn is_mastered(&self, skill: Skill) -> bool {
        self.mastered.contains(&skill)
    }

    /// The level a skill has: its XP's, held at the free cap unless it's mastered.
    pub fn level(&self, skill: Skill, r: &SkillRules) -> u8 {
        let level = level_of(self.xp[Skills::index(skill)], &r.levels);
        if self.is_mastered(skill) {
            level
        } else {
            level.min(r.free_cap)
        }
    }

    /// How a skill stands, for the client and the bot.
    pub fn standing(&self, skill: Skill, r: &SkillRules) -> Standing {
        let xp = self.xp[Skills::index(skill)];
        let level = self.level(skill, r);
        let gate = r.levels[usize::from(r.free_cap)];
        let state = if level >= MAX_LEVEL {
            Rise::Top
        } else if self.is_mastered(skill) {
            Rise::Mastered
        } else if xp >= gate {
            Rise::AtCap
        } else {
            Rise::Rising
        };
        let progress = match state {
            Rise::Top | Rise::AtCap => 1000,
            _ => {
                let from = if level == 0 {
                    0
                } else {
                    r.levels[usize::from(level) - 1]
                };
                let to = r.levels[usize::from(level)];
                (u64::from(xp - from) * 1000 / u64::from((to - from).max(1))) as u32
            }
        };
        Standing {
            level,
            progress,
            state,
        }
    }
}

impl World {
    /// A skill's level for an actor (0 for one that has no such thing).
    pub fn skill_level(&self, id: ActorId, skill: Skill) -> u8 {
        self.actors
            .get(id)
            .map_or(0, |a| a.skills.level(skill, &self.rules.skills))
    }

    /// How each of an actor's skills stands, by `Skill::ALL`.
    pub fn skills(&self, id: ActorId) -> Vec<(Skill, Standing)> {
        let Some(a) = self.actors.get(id) else {
            return Vec::new();
        };
        Skill::ALL
            .iter()
            .map(|&s| (s, a.skills.standing(s, &self.rules.skills)))
            .collect()
    }

    /// Masteries an actor has still to choose.
    pub fn masteries_left(&self, id: ActorId) -> u8 {
        let r = self.rules.skills;
        self.actors.get(id).map_or(0, |a| {
            r.masteries.saturating_sub(a.skills.mastered.len() as u8)
        })
    }

    /// Chooses a skill waiting at the free cap as a mastery, so it goes on to 10. False if it
    /// isn't at the cap, is mastered already, or every mastery is taken.
    pub fn master(&mut self, id: ActorId, skill: Skill) -> bool {
        let r = self.rules.skills;
        let left = self.masteries_left(id);
        let Some(a) = self.actors.get_mut(id) else {
            return false;
        };
        if left == 0 || a.skills.standing(skill, &r).state != Rise::AtCap {
            return false;
        }
        a.skills.mastered.push(skill);
        let level = a.skills.level(skill, &r);
        self.events.push(Event::Mastered {
            who: id,
            skill,
            level,
        });
        true
    }

    /// Gives an actor XP in a skill as it is, past every gate but the cap: for tests and tools.
    pub fn give_xp(&mut self, id: ActorId, skill: Skill, xp: u32) {
        self.add_xp(id, skill, xp);
    }

    /// An act that teaches: `base` XP, less for each time lately the same act was done `on` the
    /// same thing.
    pub(crate) fn train(&mut self, id: ActorId, skill: Skill, base: u32, on: On) {
        let (r, tick) = (self.rules.skills, self.tick);
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if a.kind != ActorKind::Character {
            return; // creatures don't learn (yet)
        }
        let mut pay = base;
        if on != On::Once {
            let repeats = &mut a.skills.repeats;
            repeats.retain(|p| tick.saturating_sub(p.last) <= r.forget);
            let times = match repeats.iter_mut().find(|p| p.skill == skill && p.on == on) {
                Some(p) => {
                    p.times += 1;
                    p.last = tick;
                    p.times - 1
                }
                None => {
                    if repeats.len() >= REMEMBERED {
                        // Forget the stalest (the first of them, if two are as stale).
                        let stalest = (0..repeats.len())
                            .min_by_key(|&i| repeats[i].last)
                            .unwrap_or(0);
                        repeats.remove(stalest);
                    }
                    repeats.push(Repeat {
                        skill,
                        on,
                        times: 1,
                        last: tick,
                    });
                    0
                }
            };
            for _ in 0..times.min(64) {
                pay = (u64::from(pay) * u64::from(r.repeat) / 1000) as u32;
            }
        }
        // A class learns its own skills faster (`class.rs`).
        if let Some(c) = self.class_of(id).filter(|c| c.skills.contains(&skill)) {
            pay = (u64::from(pay) * u64::from(c.learn) / 1000) as u32;
        }
        self.add_xp(id, skill, pay);
    }

    /// Adds XP, up to the free cap's gate for a skill that isn't mastered, and tells of a level
    /// gained, or of the cap reached.
    fn add_xp(&mut self, id: ActorId, skill: Skill, xp: u32) {
        let r = self.rules.skills;
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let s = &mut a.skills;
        let i = Skills::index(skill);
        let before = s.level(skill, &r);
        let gate = r.levels[usize::from(r.free_cap)];
        let was = s.xp[i];
        let mut now = was.saturating_add(xp);
        if !s.is_mastered(skill) {
            now = now.min(gate);
        }
        s.xp[i] = now;
        let after = s.level(skill, &r);
        if after > before {
            self.events.push(Event::SkillUp {
                who: id,
                skill,
                level: after,
            });
        }
        if was < gate && now >= gate && !s.is_mastered(skill) {
            self.events.push(Event::SkillAtCap { who: id, skill });
        }
    }

    /// The skill the weapon in an actor's hand trains and uses (None for bare hands).
    pub(crate) fn weapon_skill(&self, id: ActorId) -> Option<Skill> {
        let a = self.actors.get(id)?;
        match &self.defs.item(a.hands?.def)?.kind {
            ItemKind::Weapon { skill, .. } => *skill,
            _ => None,
        }
    }

    /// A blow that landed: it trains the weapon's skill, less on a foe that's down.
    pub(crate) fn train_blow(
        &mut self,
        id: ActorId,
        skill: Option<Skill>,
        foe: ActorId,
        down: bool,
    ) {
        let Some(skill) = skill else { return };
        let r = self.rules.skills;
        let base = if down { r.hit * r.down / 1000 } else { r.hit };
        self.train(id, skill, base, On::Foe(foe));
    }

    /// A tile run: it trains Athletics with one of the dead after you, or under a heavy load.
    pub(crate) fn train_run(&mut self, id: ActorId) {
        let r = self.rules.skills;
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        let loaded =
            u64::from(self.load(id)) * 1000 > u64::from(self.defs.carry) * u64::from(r.loaded);
        if !loaded && !self.dead_after(id, r.chased) {
            return;
        }
        let spot = r.spot.max(1);
        let on = On::Spot(
            pos.x.div_euclid(spot),
            pos.y.div_euclid(spot),
            i32::from(pos.z),
        );
        self.train(id, Skill::Athletics, r.run, on);
    }
}

/// What a level does: pure functions of the rules and a level, so the tick can use them while it
/// holds an actor.
impl SkillRules {
    /// A weapon's swing at a level of its skill: it lands more often and harder, for less breath.
    /// The wind-up, the tell, stays as it is.
    pub fn swing(&self, swing: Swing, level: u8) -> Swing {
        let harm = |d: i32| by_level(i64::from(d), self.weapon_damage, level, true) as i32;
        Swing {
            hit: (swing.hit + self.weapon_hit * u32::from(level)).min(10_000),
            damage: (harm(swing.damage.0), harm(swing.damage.1)),
            endurance: by_level(
                i64::from(swing.endurance),
                self.weapon_endurance,
                level,
                false,
            ) as i32,
            ..swing
        }
    }

    /// The wear a blow costs a weapon: less with skill, but never none for one that wears.
    pub fn wear(&self, wear: u16, level: u8) -> u16 {
        let less = by_level(i64::from(wear), self.weapon_wear, level, false) as u16;
        if wear > 0 { less.max(1) } else { 0 }
    }

    /// Running speed (thousandths of a tile a second) at a level of Athletics.
    pub fn run_speed(&self, speed: u32, level: u8) -> u32 {
        by_level(i64::from(speed), self.run_speed, level, true) as u32
    }

    /// The endurance a tick of running costs at a level of Athletics.
    pub fn run_cost(&self, cost: i32, level: u8) -> i32 {
        by_level(i64::from(cost), self.run_endurance, level, false) as i32
    }

    /// The ticks to cook a meal at a level of Cooking.
    pub fn cook_ticks(&self, ticks: u32, level: u8) -> u32 {
        (by_level(i64::from(ticks), self.cook_time, level, false) as u32).max(1)
    }

    /// The ticks to nail a board on at a level of Carpentry.
    pub fn board_ticks(&self, ticks: u32, level: u8) -> u32 {
        (by_level(i64::from(ticks), self.board_time, level, false) as u32).max(1)
    }

    /// What a board nailed at a level of Carpentry takes before it breaks.
    pub fn board_strength(&self, strength: i32, level: u8) -> i32 {
        by_level(i64::from(strength), self.board_strength, level, true) as i32
    }

    /// A bandage's ticks at a level of First Aid.
    pub fn bandage(&self, ticks: u32, level: u8) -> u32 {
        (by_level(i64::from(ticks), self.bandage_time, level, false) as u32).max(1)
    }

    /// How long the dead stay turned at a level of Devotion, from the ticks the rules give.
    pub fn turned_ticks(&self, ticks: u64, level: u8) -> u64 {
        by_level(ticks as i64, self.devotion_turn, level, true) as u64
    }

    /// What an act of faith costs at a level of Devotion (thousandths of a point).
    pub fn faith_cost(&self, cost: i32, level: u8) -> i32 {
        by_level(i64::from(cost), self.devotion_cost, level, false) as i32
    }

    /// The chance, per mille, of one more of a herb picked at a level of Foraging.
    pub fn forage_more(&self, level: u8) -> u32 {
        (u32::from(level) * self.forage_more).min(1000)
    }
}
