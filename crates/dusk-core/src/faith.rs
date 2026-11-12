//! The Cleric's faith (`CONCEPT.md`, character: the healer, and the dead's natural enemy; its
//! part of the system is dread and the dead).
//!
//! - **Faith** is the Cleric's resource (`Class::faith`, more with Devout), kept in points as
//!   dread is. Turning the dead, consecrating a body without holy water, mending a wound and
//!   warding a door spend it, and it comes back only by prayer at a shrine (an altar: the
//!   abbey's, or the old chapel's in the heart of town).
//! - **Prayer** (`Intent::Pray`): kneeling a while at a shrine beside you. Anyone may: it eases
//!   dread. The faithful get their faith back in full; with Purify, ghoul fever goes too.
//! - **Turning the dead** (`Intent::Turn`), the Cleric's signature: the holy symbol held up (a
//!   tell, as a swing's wind-up is) and a word said aloud, heard round about. Each of the dead
//!   within reach that the cleric can see turns and flees from it (`Mind::Turned`) for a while:
//!   its kind's share of the rules' time (`Creature::turned`: an old dead thing's is brief),
//!   longer with Devotion and Rebuke. A turned one lets go of whoever it held, strikes nobody, and
//!   can't be struck unawares; a blow ends it, and it rounds on whoever struck. The wild (wolves,
//!   deer) aren't the dead, and aren't turned. With Sunder, the weakest of them fall instead.
//! - **Consecration** (`Intent::Consecrate`): holy water poured on a body, or a cleric's prayer
//!   over it, and it never gets up (`Remains::Hallowed`). Burning's quiet cousin: no fire, no
//!   light, nothing spreads, and the pressure falls as it does for a body burned. Anyone with
//!   holy water can; a cleric can with faith alone. A character's body consecrated never rises,
//!   and its gear falls where it lies.
//! - **Mending** (a talent's verb, `Intent::Heal`): faith stops the worst bleeding wound and
//!   gives health back.
//! - **Wards** (a talent's verb, `Intent::Ward`): a shut door warded holds against the dead for
//!   some hours. They don't beat on it, and a blow of theirs does nothing to it.
//! - **Devotion**, the Cleric's skill, grows by turning the dead (a foe each, so a crowd
//!   teaches and one shambler turned again and again doesn't) and by mending a wound that
//!   bleeds. A level makes the turning last longer and faith go further.

use crate::actor::{ActionKind, ActorId, ActorKind, Intent, Mind, Remains};
use crate::geom::{Dir8, TilePos};
use crate::grid::edge_owner;
use crate::items::{ItemKind, within_reach};
use crate::senses::line_of_sight;
use crate::skills::{On, Skill};
use crate::world::{Event, Sound, World};

/// Faith's numbers, in the core's units: faith in thousandths of a point, times in ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Faith {
    /// Ticks kneeling at a shrine, and the dread a prayer takes away.
    pub pray: u32,
    pub comfort: i32,
    /// Turning the dead: the faith it costs, its wind-up (the tell) and recovery, its reach
    /// (tiles), how long the dead stay turned, and how far the word carries (tiles).
    pub turn: i32,
    pub turn_wind_up: u32,
    pub turn_recovery: u32,
    pub reach: i32,
    pub turned: u64,
    pub noise: i32,
    /// Consecrating a body: the faith it costs without holy water, and its ticks.
    pub consecrate: i32,
    pub consecrate_time: u32,
    /// Mending: its faith, the health it gives back (out of `body::FULL`), and its ticks.
    pub mend: i32,
    pub mend_health: i32,
    pub mend_time: u32,
    /// Warding a door: its faith, its ticks, and how long the ward holds.
    pub ward: i32,
    pub ward_time: u32,
    pub ward_lasts: u64,
}

impl Default for Faith {
    fn default() -> Faith {
        Faith {
            pray: 100,
            comfort: 20_000,
            turn: 30_000,
            turn_wind_up: 10,
            turn_recovery: 5,
            reach: 8,
            turned: 200,
            noise: 6,
            consecrate: 15_000,
            consecrate_time: 30,
            mend: 20_000,
            mend_health: 150,
            mend_time: 20,
            ward: 40_000,
            ward_time: 40,
            // Eight game hours, at the test clock's 25 ticks a minute.
            ward_lasts: 8 * 60 * 25,
        }
    }
}

/// Why an act of faith can't be done now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoFaith {
    /// It isn't yours: your class doesn't turn the dead, or you haven't the talent.
    NotYours,
    /// Nothing here to do it to: no shrine, no body, no wound, no shut door.
    Nothing,
    /// Not enough faith left: pray at a shrine.
    TooLittle,
    /// No holy water to consecrate with, and no faith to do without it.
    NoWater,
}

/// What a body is consecrated with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hallow {
    /// A flask of holy water from the pack: anyone's.
    Water,
    /// A cleric's prayer over it: its faith.
    Faith,
}

fn per(n: u32, by: u32) -> u32 {
    (u64::from(n) * u64::from(by) / 1000) as u32
}

impl World {
    /// The most faith `id` can hold: its class's, and what its talents add (0 for a class
    /// without any).
    pub fn max_faith(&self, id: ActorId) -> i32 {
        let class = self.class_of(id).map_or(0, |c| c.faith);
        if class <= 0 {
            return 0;
        }
        class + self.talents_of(id).map(|t| t.faith).sum::<i32>()
    }

    /// `id`'s faith now, in thousandths of a point.
    pub fn faith(&self, id: ActorId) -> i32 {
        self.actors
            .get(id)
            .filter(|a| !a.dead)
            .map_or(0, |a| a.faith)
    }

    /// What an act of faith costs `id`: the rules' cost, less with Devotion.
    pub fn faith_cost(&self, id: ActorId, base: i32) -> i32 {
        let level = self.skill_level(id, Skill::Devotion);
        self.rules.skills.faith_cost(base, level)
    }

    fn has_faith_for(&self, id: ActorId, base: i32) -> bool {
        self.max_faith(id) > 0 && self.faith(id) >= self.faith_cost(id, base)
    }

    fn spend_faith(&mut self, id: ActorId, base: i32) {
        let cost = self.faith_cost(id, base);
        if let Some(a) = self.actors.get_mut(id) {
            a.faith = (a.faith - cost).max(0);
        }
    }

    /// Whether `at` holds a shrine to pray at.
    pub fn is_shrine(&self, at: TilePos) -> bool {
        let f = usize::from(self.grid.tile(at).fixture);
        self.defs.shrines.get(f).copied().unwrap_or(false)
    }

    /// Whether `id` can pray now, and at which shrine: the one it faces, else the first round it.
    pub fn can_pray(&self, id: ActorId) -> Result<TilePos, NoFaith> {
        let a = self
            .actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoFaith::Nothing)?;
        (0..8)
            .map(|t| a.pos.step(a.facing.turned(t)))
            .find(|&at| self.is_shrine(at) && within_reach(&self.grid, a.pos, at))
            .ok_or(NoFaith::Nothing)
    }

    /// The prayer's start: which way, and how long.
    pub(crate) fn pray_action(&self, id: ActorId) -> Option<(ActionKind, Dir8, u32)> {
        let at = self.can_pray(id).ok()?;
        Some((
            ActionKind::Pray { at },
            self.toward(id, at),
            self.rules.faith.pray.max(1),
        ))
    }

    /// The prayer's end: dread eased, faith full, and with Purify, the fever gone. Only if the
    /// shrine's still within reach.
    pub(crate) fn pray(&mut self, id: ActorId, at: TilePos) {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return;
        };
        if !self.is_shrine(at) || !within_reach(&self.grid, a.pos, at) {
            return;
        }
        let most = self.max_faith(id);
        let purify = self.talents_of(id).any(|t| t.purify);
        let comfort = self.rules.faith.comfort;
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        a.body.dread = (a.body.dread - comfort).max(0);
        a.faith = most;
        let cured = purify && a.body.fever.take().is_some();
        self.events.push(Event::Prayed {
            who: id,
            faith: most > 0,
        });
        if cured {
            self.events.push(Event::Cured { who: id });
        }
    }

    /// Whether `id` can turn the dead now: a class that does, and faith enough.
    pub fn can_turn(&self, id: ActorId) -> Result<(), NoFaith> {
        self.actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoFaith::NotYours)?;
        if !self.class_of(id).is_some_and(|c| c.turns) {
            return Err(NoFaith::NotYours);
        }
        if !self.has_faith_for(id, self.rules.faith.turn) {
            return Err(NoFaith::TooLittle);
        }
        Ok(())
    }

    /// How far `id`'s turning reaches, in tiles: the rules', and what its talents add.
    pub fn turn_reach(&self, id: ActorId) -> i32 {
        self.rules.faith.reach + self.talents_of(id).map(|t| t.turn_reach).sum::<i32>()
    }

    /// How long one of a kind turned by `id` stays turned, in ticks: the rules' time, the kind's
    /// share of it (per mille), longer with Devotion and with talents.
    fn turn_ticks(&self, id: ActorId, share: u32) -> u64 {
        let level = self.skill_level(id, Skill::Devotion);
        let base = self
            .rules
            .skills
            .turned_ticks(self.rules.faith.turned, level);
        let talents = self
            .talents_of(id)
            .fold(1000, |r: u32, t| per(r, t.turn_time));
        base * u64::from(share) / 1000 * u64::from(talents) / 1000
    }

    /// The dead `id` would turn now, in id order: alive, of the dead (not the wild), of a kind
    /// that can be turned, within its reach and in its line of sight.
    pub fn turnable(&self, id: ActorId) -> Vec<ActorId> {
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return Vec::new();
        };
        let r2 = i64::from(self.turn_reach(id)).pow(2);
        self.actors
            .iter()
            .filter(|(other, o)| {
                *other != id
                    && !o.dead
                    && matches!(o.kind, ActorKind::Creature(_))
                    && o.pos.z == pos.z
                    && o.pos.dist2(pos) <= r2
            })
            .map(|(other, _)| other)
            .filter(|&other| {
                !self.is_wild(other)
                    && self.stats(other).is_some_and(|c| c.turned > 0)
                    && self
                        .actors
                        .get(other)
                        .is_some_and(|o| line_of_sight(&self.grid, pos, o.pos))
            })
            .collect()
    }

    /// The turning, on its strike tick: the word said aloud, the faith spent, and each of the
    /// dead within reach turned (or with Sunder, if it's weak enough, put down).
    pub(crate) fn turn_the_dead(&mut self, id: ActorId) {
        if self.can_turn(id).is_err() {
            return;
        }
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        let f = self.rules.faith;
        self.emit(Sound {
            at: pos,
            loudness: f.noise,
            by: Some(id),
        });
        self.spend_faith(id, f.turn);
        let sunder = self.talents_of(id).map(|t| t.sunder).max().unwrap_or(0);
        let (mut turned, mut sundered) = (0u16, 0u16);
        for other in self.turnable(id) {
            let health = self.actors.get(other).map_or(0, |o| o.body.health);
            if health <= sunder {
                self.kill(other);
                sundered = sundered.saturating_add(1);
            } else {
                let share = self.stats(other).map_or(0, |c| c.turned);
                let until = self.tick + self.turn_ticks(id, share);
                // It lets go, and what it was winding up to do comes to nothing.
                self.release(other);
                if let Some(o) = self.actors.get_mut(other) {
                    o.mind = Mind::Turned { by: id, until };
                    o.action = None;
                    o.intent = Intent::Idle;
                    o.path.clear();
                    o.path_goal = None;
                }
                turned = turned.saturating_add(1);
            }
            self.train(id, Skill::Devotion, self.rules.skills.turn, On::Foe(other));
        }
        self.events.push(Event::Turned {
            who: id,
            turned,
            sundered,
        });
    }

    /// Whether `id` is turned now.
    pub fn is_turned(&self, id: ActorId) -> bool {
        self.actors
            .get(id)
            .is_some_and(|a| !a.dead && matches!(a.mind, Mind::Turned { .. }))
    }

    /// A blow on one that's turned ends it: it rounds on whoever struck.
    pub(crate) fn unturn(&mut self, id: ActorId, by: ActorId) {
        let (tick, from) = (self.tick, self.actors.get(by).map(|b| b.pos));
        if let Some(a) = self.actors.get_mut(id)
            && matches!(a.mind, Mind::Turned { .. })
        {
            a.mind = Mind::Hunt {
                target: from,
                since: tick,
                touching: false,
                sees: false,
            };
        }
    }

    /// A turned one's turn to think: it runs from whoever turned it until its time's up (or
    /// that one's dead), then it's one of the hunting dead again. True while it's still turned.
    pub(crate) fn think_turned(&mut self, id: ActorId) -> bool {
        let tick = self.tick;
        let Some(Mind::Turned { by, until }) = self.actors.get(id).map(|a| a.mind) else {
            return false;
        };
        let from = self.actors.get(by).filter(|b| !b.dead).map(|b| b.pos);
        if let Some(threat) = from.filter(|_| tick < until) {
            let intent = match self.flight(id, threat) {
                Some(dir) => Intent::Move { dir, run: true },
                None => Intent::Idle,
            };
            if let Some(a) = self.actors.get_mut(id) {
                a.intent = intent;
            }
            return true;
        }
        if let Some(a) = self.actors.get_mut(id) {
            a.mind = Mind::Hunt {
                target: None,
                since: tick,
                touching: false,
                sees: false,
            };
            a.intent = Intent::Idle;
        }
        false
    }

    /// The first stack of holy water in `id`'s pack.
    fn holy_water_slot(&self, id: ActorId) -> Option<usize> {
        let a = self.actors.get(id)?;
        a.items.iter().position(|i| {
            self.defs
                .item(i.def)
                .is_some_and(|d| matches!(d.kind, ItemKind::Holy))
        })
    }

    /// What `id` would consecrate with now: holy water first, else its faith.
    fn hallow_with(&self, id: ActorId) -> Result<Hallow, NoFaith> {
        if self.holy_water_slot(id).is_some() {
            Ok(Hallow::Water)
        } else if self.has_faith_for(id, self.rules.faith.consecrate) {
            Ok(Hallow::Faith)
        } else {
            Err(NoFaith::NoWater)
        }
    }

    /// Whether `id` can consecrate a body now, which, and with what: one beside it (as burning
    /// finds one), and holy water in its pack, else faith enough.
    pub fn can_consecrate(&self, id: ActorId) -> Result<(ActorId, Hallow), NoFaith> {
        self.actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoFaith::Nothing)?;
        let body = self.body_beside(id).ok_or(NoFaith::Nothing)?;
        Ok((body, self.hallow_with(id)?))
    }

    /// The consecration's start: which way, and how long.
    pub(crate) fn consecrate_action(&self, id: ActorId) -> Option<(ActionKind, Dir8, u32)> {
        let (body, _) = self.can_consecrate(id).ok()?;
        let to = self.actors.get(body)?.pos;
        Some((
            ActionKind::Consecrate { body },
            self.toward(id, to),
            self.rules.faith.consecrate_time.max(1),
        ))
    }

    /// The consecration's end: the holy water or the faith spent, and the body will never get
    /// up. It must still be there, within reach, and not burnt or hallowed already.
    pub(crate) fn consecrate(&mut self, id: ActorId, body: ActorId) {
        let (Some(a), Some(b)) = (self.actors.get(id), self.actors.get(body)) else {
            return;
        };
        if a.dead || !within_reach(&self.grid, a.pos, b.pos) || !self.burnable(body) {
            return;
        }
        let Ok(with) = self.hallow_with(id) else {
            return;
        };
        match with {
            Hallow::Water => {
                let Some(slot) = self.holy_water_slot(id) else {
                    return;
                };
                if let Some(a) = self.actors.get_mut(id) {
                    a.items[slot].count -= 1;
                    if a.items[slot].count == 0 {
                        a.items.remove(slot);
                    }
                }
            }
            Hallow::Faith => self.spend_faith(id, self.rules.faith.consecrate),
        }
        // A character's body that won't rise keeps nothing: its gear falls where it lies.
        if self.actors.get(body).is_some_and(|b| b.kind.is_character()) {
            self.shed_gear(body);
        }
        if let Some(b) = self.actors.get_mut(body) {
            b.remains = Remains::Hallowed;
        }
        let by = self.region.pressure.per_burn;
        self.nudge_pressure(-i64::from(by));
        self.events.push(Event::Consecrated {
            who: id,
            body,
            water: with == Hallow::Water,
        });
    }

    /// Whether `id` can mend a wound with faith now: Mending, a wound bleeding or health to get
    /// back, and faith enough.
    pub fn can_heal(&self, id: ActorId) -> Result<(), NoFaith> {
        let a = self
            .actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoFaith::NotYours)?;
        if !self.talents_of(id).any(|t| t.mending) {
            return Err(NoFaith::NotYours);
        }
        let bleeding = a.body.wounds.iter().any(|w| w.bleeding > 0);
        if !bleeding && a.body.health >= self.max_health(id) {
            return Err(NoFaith::Nothing);
        }
        if !self.has_faith_for(id, self.rules.faith.mend) {
            return Err(NoFaith::TooLittle);
        }
        Ok(())
    }

    /// The mending's end: the faith spent, the worst bleeding wound stopped, and health back.
    /// A wound that was bleeding teaches Devotion.
    pub(crate) fn heal(&mut self, id: ActorId) {
        if self.can_heal(id).is_err() {
            return;
        }
        let f = self.rules.faith;
        self.spend_faith(id, f.mend);
        let most = self.max_health(id);
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let worst = a
            .body
            .wounds
            .iter()
            .enumerate()
            .filter(|(_, w)| w.bleeding > 0)
            .max_by_key(|(_, w)| (w.kind, w.bleeding, w.heal_left))
            .map(|(i, _)| i);
        if let Some(i) = worst {
            a.body.wounds[i].bleeding = 0;
        }
        a.body.health = (a.body.health + f.mend_health).min(most);
        self.events.push(Event::Healed { who: id });
        if worst.is_some() {
            self.train(id, Skill::Devotion, self.rules.skills.mend, On::Once);
        }
    }

    /// Whether the door on `pos`'s side `dir` is warded now.
    pub fn is_warded(&self, pos: TilePos, dir: Dir8) -> bool {
        if dir.split().is_some() {
            return false;
        }
        self.wards
            .get(&edge_owner(pos, dir))
            .is_some_and(|&until| until > self.tick)
    }

    /// Whether `id` can ward a door now, and on which side of its tile: Warding, a shut door
    /// beside it (the one it faces, else the first round it), and faith enough.
    pub fn can_ward(&self, id: ActorId) -> Result<Dir8, NoFaith> {
        let a = self
            .actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoFaith::NotYours)?;
        if !self.talents_of(id).any(|t| t.warding) {
            return Err(NoFaith::NotYours);
        }
        let side = World::sides_from(a.facing)
            .find(|&d| self.grid.edge(a.pos, d).is_shut_door())
            .ok_or(NoFaith::Nothing)?;
        if !self.has_faith_for(id, self.rules.faith.ward) {
            return Err(NoFaith::TooLittle);
        }
        Ok(side)
    }

    /// The ward goes on (the warding's end): the faith spent, and the door holds for the rules'
    /// hours. A door warded again holds afresh.
    pub(crate) fn ward(&mut self, id: ActorId, dir: Dir8) {
        if self.can_ward(id) != Ok(dir) {
            return;
        }
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        self.spend_faith(id, self.rules.faith.ward);
        let (at, side) = edge_owner(pos, dir);
        self.wards
            .insert((at, side), self.tick + self.rules.faith.ward_lasts);
        self.events.push(Event::Warded { who: id, at, side });
    }

    /// Each game minute: the wards that have run their time fade.
    pub(crate) fn lapse_wards(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if self.wards.is_empty() || !self.tick.is_multiple_of(per) {
            return;
        }
        let tick = self.tick;
        let faded: Vec<(TilePos, Dir8)> = self
            .wards
            .iter()
            .filter(|&(_, &until)| until <= tick)
            .map(|(&edge, _)| edge)
            .collect();
        for (at, side) in faded {
            self.wards.remove(&(at, side));
            self.events.push(Event::WardFaded { at, side });
        }
    }

    /// Per mille: how fast `id`'s dread rises now: slower while it holds a light up, with Steady.
    pub(crate) fn dread_rise(&self, id: ActorId) -> u32 {
        if !self.actors.get(id).is_some_and(|a| a.light.is_some()) {
            return 1000;
        }
        self.talents_of(id).fold(1000, |r, t| per(r, t.steady))
    }

    /// Whether what `id` puts down stays down: a Hallowed weapon in hand, or Last Rites.
    pub(crate) fn lays_to_rest(&self, id: ActorId) -> bool {
        self.hallowed_hand(id) || self.talents_of(id).any(|t| t.last_rites)
    }

    /// One `id` put down stays down, if `id` lays the dead to rest.
    pub(crate) fn put_to_rest(&mut self, id: ActorId, body: ActorId) {
        if !self.lays_to_rest(id) {
            return;
        }
        if let Some(b) = self.actors.get_mut(body)
            && b.dead
            && matches!(b.remains, Remains::GetsUp { .. })
        {
            b.remains = Remains::Lying;
        }
    }
}
