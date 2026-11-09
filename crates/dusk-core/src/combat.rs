//! Timed actions, and the fight: one rule for all of them (`ARCHITECTURE.md`).
//!
//! An action starts only between steps, faces its way, pays its endurance, then winds up (the
//! tell everyone can see), takes effect on its strike tick, and recovers. Nobody moves while
//! acting, and a blow taken during the wind-up breaks it off. Exhausted, every wind-up and
//! recovery is half as long again, and the character's blows do half the harm.

use crate::actor::{Action, ActionKind, ActorId, ActorKind, Intent};
use crate::body::{Part, Wound, WoundKind};
use crate::geom::{Dir8, TilePos};
use crate::grid::{Edge, edge_owner};
use crate::hash::{Purpose, below, roll};
use crate::world::{Event, Sound, World};

/// The living fight the dead, and the dead go for the living.
fn enemies(a: ActorKind, b: ActorKind) -> bool {
    a != b
}

impl World {
    /// An actor's turn at its timed action, this tick: start one if its intent asks and it's
    /// between steps, carry on with the one under way, or lie there if it's down. True while
    /// it's busy, so it takes no step this tick.
    pub(crate) fn act(&mut self, id: ActorId) -> bool {
        let tick = self.tick;
        let Some(a) = self.actors.get_mut(id) else {
            return true;
        };
        if a.prone_until > 0 {
            if tick < a.prone_until {
                return true;
            }
            a.prone_until = 0;
        }
        // Asleep, nothing happens until something wakes you: any intent does.
        if a.body.asleep {
            if a.intent != Intent::Idle {
                a.intent = Intent::Idle;
                self.wake(id, crate::needs::Woke::Chose);
            }
            return true;
        }
        if a.action.is_none() {
            if a.step.is_some() || !a.intent.is_action() {
                return false;
            }
            let intent = std::mem::take(&mut a.intent);
            let Some(action) = self.begin(id, intent) else {
                return false;
            };
            if let Some(a) = self.actors.get_mut(id) {
                a.action = Some(action);
            }
        }
        let Some(a) = self.actors.get_mut(id) else {
            return true;
        };
        let Some(mut action) = a.action else {
            return false;
        };
        action.elapsed += 1;
        a.action = (action.elapsed < action.length).then_some(action);
        if action.elapsed == action.strike {
            match action.kind {
                ActionKind::Attack => self.strike(id, action.dir),
                ActionKind::Shove => self.shove(id, action.dir),
                ActionKind::Door => self.toggle_door(id, action.dir),
                ActionKind::Struggle => self.struggle(id),
                ActionKind::Take { at, slot, def } => self.take(id, at, slot, def),
                ActionKind::Drop { slot, def } => self.drop_item(id, slot, def),
                ActionKind::Use { slot, def } => self.use_item(id, slot, def),
                ActionKind::Wield { slot, def } => self.wield(id, slot, def),
                ActionKind::Unwield => self.unwield(id),
                ActionKind::Sleep => self.fall_asleep(id),
                ActionKind::DrinkWater => self.drink_water(id),
                ActionKind::Light { slot, def } => self.hold_light(id, slot, def),
                ActionKind::Douse => self.douse(id),
            }
        }
        true
    }

    /// Starts the action an intent asks for, if it can start here and now. Held, you can only
    /// struggle or shove; struggling is only for the held. Item actions take all their time
    /// before they take effect, so a blow breaks them off.
    fn begin(&mut self, id: ActorId, intent: Intent) -> Option<Action> {
        let held = self.is_held(id);
        let a = self.actors.get(id)?;
        let (kind, dir, wind_up, recovery, endurance) = match intent {
            Intent::Struggle if !held => return None,
            Intent::Struggle | Intent::Shove { .. } => {
                // the two things the held can do: below
                return self.begin_free(id, intent, held);
            }
            _ if held => return None,
            Intent::Take { at, slot, def } => {
                let ticks = self.take_ticks(at, slot, def)?;
                if !crate::items::within_reach(&self.grid, a.pos, at) {
                    return None;
                }
                (ActionKind::Take { at, slot, def }, a.facing, ticks, 0, 0)
            }
            Intent::Drop { slot, def } => {
                a.items.get(usize::from(slot)).filter(|i| i.def == def)?;
                let kind = ActionKind::Drop { slot, def };
                (kind, a.facing, self.defs.drop.max(1), 0, 0)
            }
            Intent::Use { slot, def } => {
                let ticks = self.item_ticks(id, slot, def, true)?;
                (ActionKind::Use { slot, def }, a.facing, ticks, 0, 0)
            }
            Intent::Sleep => {
                self.can_sleep(id).ok()?;
                let ticks = self.rules.needs.lie_down;
                (ActionKind::Sleep, a.facing, ticks, 0, 0)
            }
            Intent::DrinkWater => {
                self.water_beside(a.pos)?;
                let ticks = self.rules.needs.drink_water;
                (ActionKind::DrinkWater, a.facing, ticks, 0, 0)
            }
            Intent::Wield { slot, def } => {
                let ticks = self.item_ticks(id, slot, def, false)?;
                (ActionKind::Wield { slot, def }, a.facing, ticks, 0, 0)
            }
            Intent::Unwield => {
                a.hands?;
                (ActionKind::Unwield, a.facing, self.defs.unwield_time, 0, 0)
            }
            Intent::Light { slot, def } => {
                let ticks = self.light_ticks(id, slot, def)?;
                (ActionKind::Light { slot, def }, a.facing, ticks, 0, 0)
            }
            Intent::Douse => {
                a.light?;
                (
                    ActionKind::Douse,
                    a.facing,
                    self.defs.douse_time.max(1),
                    0,
                    0,
                )
            }
            _ => return self.begin_free(id, intent, held),
        };
        self.start(id, kind, dir, wind_up, recovery, endurance)
    }

    /// The fight's actions: a swing, a shove, a door, a struggle.
    fn begin_free(&mut self, id: ActorId, intent: Intent, held: bool) -> Option<Action> {
        let c = self.rules.combat;
        let a = self.actors.get(id)?;
        let (kind, dir, wind_up, recovery, endurance) = match intent {
            Intent::Attack { .. } | Intent::Door if held => return None,
            Intent::Struggle => {
                let s = c.struggle;
                (
                    ActionKind::Struggle,
                    a.facing,
                    s.wind_up,
                    s.recovery,
                    s.endurance,
                )
            }
            Intent::Attack { dir } => {
                let s = self.swing_of(id);
                let dir = self.aim(id, dir);
                (ActionKind::Attack, dir, s.wind_up, s.recovery, s.endurance)
            }
            Intent::Shove { dir } => {
                let s = c.shove;
                let dir = self.aim(id, dir);
                (ActionKind::Shove, dir, s.wind_up, s.recovery, s.endurance)
            }
            Intent::Door => {
                let dir = self.door_beside(a.pos, a.facing)?;
                (ActionKind::Door, dir, c.door_ticks, 0, 0)
            }
            _ => return None,
        };
        self.start(id, kind, dir, wind_up, recovery, endurance)
    }

    /// An action begins: it faces its way and pays its endurance. Out of breath, it takes half
    /// as long again; tired, longer still (`needs.rs`).
    fn start(
        &mut self,
        id: ActorId,
        kind: ActionKind,
        dir: Dir8,
        wind_up: u32,
        recovery: u32,
        endurance: i32,
    ) -> Option<Action> {
        let tired = self.tired_slow(id);
        let a = self.actors.get_mut(id)?;
        let slow = |t: u32| {
            let t = t * tired / 1000;
            if a.body.exhausted { t * 3 / 2 } else { t }
        };
        let (wind_up, recovery) = (slow(wind_up).max(1), slow(recovery));
        let was = a.body.exhausted;
        a.facing = dir;
        a.body.spend(endurance);
        if a.body.exhausted && !was {
            self.events.push(Event::Exhausted { who: id });
        }
        Some(Action {
            kind,
            dir,
            elapsed: 0,
            strike: wind_up,
            length: wind_up + recovery,
        })
    }

    /// Where a swing or shove goes: the way asked for, else at the nearest enemy that can be
    /// reached (the one ahead first, then clockwise from north), else straight ahead.
    fn aim(&self, id: ActorId, asked: Option<Dir8>) -> Dir8 {
        let Some(a) = self.actors.get(id) else {
            return Dir8::S;
        };
        if let Some(dir) = asked {
            return dir;
        }
        std::iter::once(a.facing)
            .chain(Dir8::ALL)
            .find(|&d| self.target_at(id, a.pos.step(d)).is_some())
            .unwrap_or(a.facing)
    }

    /// The enemy a blow from `id` would hit on `at`: alive, and reachable from where `id` stands
    /// (no wall or corner in the way).
    fn target_at(&self, id: ActorId, at: TilePos) -> Option<ActorId> {
        let a = self.actors.get(id)?;
        let &other = self.occupied.get(&at)?;
        let o = self.actors.get(other)?;
        (other != id && !o.dead && enemies(a.kind, o.kind) && self.can_reach(a.pos, at))
            .then_some(other)
    }

    /// The side of a tile with a door: the one faced (or, facing a diagonal, either of its
    /// sides), else the first of north, east, south and west.
    fn door_beside(&self, pos: TilePos, facing: Dir8) -> Option<Dir8> {
        let faced = match facing.split() {
            Some((a, b)) => [Some(a), Some(b)],
            None => [Some(facing), None],
        };
        faced
            .into_iter()
            .flatten()
            .chain([Dir8::N, Dir8::E, Dir8::S, Dir8::W])
            .find(|&d| matches!(self.grid.edge(pos, d), Edge::Door { .. }))
    }

    /// A blow during the wind-up breaks it off.
    fn interrupt(&mut self, id: ActorId) {
        if let Some(a) = self.actors.get_mut(id)
            && a.action.is_some_and(|act| act.winding_up())
        {
            a.action = None;
        }
    }

    /// A swing lands (or doesn't) on its strike tick.
    fn strike(&mut self, id: ActorId, dir: Dir8) {
        let (c, tick, seed) = (self.rules.combat, self.tick, self.seed);
        let Some(a) = self.actors.get(id) else { return };
        let (pos, kind, tired) = (a.pos, a.kind, a.body.exhausted);
        let swing = self.swing_of(id);
        self.emit(Sound {
            at: pos,
            loudness: swing.noise,
            by: Some(id),
        });
        let at = pos.step(dir);
        let Some(target) = self.target_at(id, at) else {
            if dir.split().is_none() && self.grid.edge(pos, dir) == (Edge::Door { open: false }) {
                self.thump(id, dir);
            } else {
                self.events.push(Event::Missed { who: id, at });
            }
            return;
        };
        let who = id.index() as u64;
        let down = self.actors.get(target).is_some_and(|t| t.prone_until > 0);
        let holds = self
            .actors
            .get(id)
            .is_some_and(|a| a.holding == Some(target));
        // On one that's down, or one it holds, a blow always lands.
        if !down && !holds && below(roll(seed, tick, who, Purpose::Hit), 10_000) >= swing.hit {
            self.events.push(Event::Missed { who: id, at });
            return;
        }
        let (damage, wound) = match kind {
            ActorKind::Character => {
                let (lo, hi) = swing.damage;
                let span = (hi - lo).max(0) as u32 + 1;
                let mut damage = lo + below(roll(seed, tick, who, Purpose::Damage), span) as i32;
                if tired {
                    damage /= 2;
                }
                if down {
                    damage *= 2;
                }
                (damage, None)
            }
            ActorKind::Shambler => {
                let mut left = below(roll(seed, tick, who, Purpose::Wound), 10_000);
                let mut kind = WoundKind::Scratch;
                if holds && left < c.held_bite {
                    // Held, the mouth gets close.
                    kind = WoundKind::Bite;
                } else {
                    for (k, &odds) in WoundKind::ALL.iter().zip(&c.shambler_wounds) {
                        if left < odds {
                            kind = *k;
                            break;
                        }
                        left -= odds;
                    }
                }
                let part = Part::from_roll(below(roll(seed, tick, who, Purpose::Part), 100));
                let rule = c.wounds[kind as usize];
                (rule.damage, Some((kind, part, rule.bleed)))
            }
        };
        if let Some(t) = self.actors.get_mut(target) {
            t.body.health -= damage;
            if let Some((kind, part, bleeding)) = wound {
                t.body.wounds.push(Wound {
                    part,
                    kind,
                    bleeding,
                    bandaged: false,
                    heal_left: self.rules.needs.wound_heal[kind as usize],
                });
            }
        }
        self.interrupt(target);
        self.wake(target, crate::needs::Woke::Hurt);
        self.events.push(Event::Struck {
            who: id,
            target,
            damage,
            wound: wound.map(|(k, p, _)| (k, p)),
        });
        // Every blow that lands wears the weapon that struck it.
        self.wear(id);
        if self.actors.get(target).is_some_and(|t| t.body.health <= 0) {
            self.kill(target);
            return;
        }
        // A creature's hit may take hold, the likelier the more of them are on you.
        if kind != ActorKind::Character
            && !holds
            && self.actors.get(id).is_some_and(|a| a.holding.is_none())
            && below(roll(seed, tick, who, Purpose::Grab), 10_000) < self.grab_chance(target)
        {
            self.grab(id, target);
        }
    }

    /// A blow on a shut door. Beaten past its strength, the door gives way for good.
    fn thump(&mut self, id: ActorId, dir: Dir8) {
        let (c, tick, seed) = (self.rules.combat, self.tick, self.seed);
        let Some(a) = self.actors.get(id) else { return };
        let (pos, kind, tired) = (a.pos, a.kind, a.body.exhausted);
        let (lo, hi) = match kind {
            ActorKind::Character => self.swing_of(id).damage,
            ActorKind::Shambler => c.shambler_thump,
        };
        let span = (hi - lo).max(0) as u32 + 1;
        let mut harm = lo + below(roll(seed, tick, id.index() as u64, Purpose::Thump), span) as i32;
        if tired {
            harm /= 2;
        }
        let (at, side) = edge_owner(pos, dir);
        let taken = self.door_damage.entry((at, side)).or_insert(0);
        *taken += harm;
        let broke = *taken >= c.door_strength;
        self.emit(Sound {
            at: pos,
            loudness: c.thump_noise,
            by: Some(id),
        });
        self.events.push(Event::Thumped { who: id, at, side });
        self.wear(id);
        if broke {
            self.door_damage.remove(&(at, side));
            self.grid.set_edge(pos, dir, Edge::Open);
            self.events.push(Event::DoorBroke { at, side });
        }
    }

    /// A shove pushes whoever is there back a tile (if there's room), breaks off what it was
    /// doing, and sometimes knocks it down.
    fn shove(&mut self, id: ActorId, dir: Dir8) {
        let (c, tick, seed) = (self.rules.combat, self.tick, self.seed);
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        self.emit(Sound {
            at: pos,
            loudness: c.shove.noise,
            by: Some(id),
        });
        let at = pos.step(dir);
        let Some(target) = self.target_at(id, at) else {
            self.events.push(Event::Missed { who: id, at });
            return;
        };
        let back = at.step(dir);
        let room = self.grid.can_step(at, dir) && !self.occupied.contains_key(&back);
        let down = below(
            roll(seed, tick, id.index() as u64, Purpose::KnockDown),
            10_000,
        ) < c.knock_down;
        // A shove breaks a hold.
        self.release(target);
        let Some(t) = self.actors.get_mut(target) else {
            return;
        };
        t.action = None;
        let mut freed = vec![t.pos];
        if room {
            freed.extend(t.step.map(|s| s.to));
            t.step = None;
            t.carry = 0;
            t.pos = back;
        }
        if down {
            t.prone_until = tick + u64::from(c.prone);
        }
        if room {
            for tile in freed {
                if self.occupied.get(&tile) == Some(&target) {
                    self.occupied.remove(&tile);
                }
            }
            self.occupied.insert(back, target);
        }
        self.events.push(Event::Shoved {
            who: id,
            target,
            down,
        });
    }

    /// Opens or shuts the door on that side. It won't shut on someone stepping through.
    fn toggle_door(&mut self, id: ActorId, dir: Dir8) {
        let c = self.rules.combat;
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        let Edge::Door { open } = self.grid.edge(pos, dir) else {
            return;
        };
        let other = pos.step(dir);
        let crossing = self.actors.iter().any(|(_, a)| {
            a.step.is_some_and(|s| {
                (s.from == pos && s.to == other) || (s.from == other && s.to == pos)
            })
        });
        if open && crossing {
            return;
        }
        self.grid.set_edge(pos, dir, Edge::Door { open: !open });
        self.emit(Sound {
            at: pos,
            loudness: c.door_noise,
            by: Some(id),
        });
        let (at, side) = edge_owner(pos, dir);
        self.events.push(Event::Door {
            who: id,
            at,
            side,
            open: !open,
        });
    }

    /// Wounds an actor as a creature's blow would, by the rules for that kind of wound: for
    /// tests, tools and, later, traps.
    pub fn wound(&mut self, id: ActorId, kind: WoundKind, part: Part) {
        let rule = self.rules.combat.wounds[kind as usize];
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        a.body.health -= rule.damage;
        a.body.wounds.push(Wound {
            part,
            kind,
            bleeding: rule.bleed,
            bandaged: false,
            heal_left: self.rules.needs.wound_heal[kind as usize],
        });
        if a.body.health <= 0 {
            self.kill(id);
        }
    }

    /// Death: the body stays where it fell, as a corpse that nobody has to walk round.
    pub(crate) fn kill(&mut self, id: ActorId) {
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if a.dead {
            return;
        }
        a.dead = true;
        a.action = None;
        a.prone_until = 0;
        a.intent = Intent::Idle;
        let held = a.holding.is_some();
        if held {
            self.release(id);
        }
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let mut freed = vec![a.pos];
        freed.extend(a.step.take().map(|s| s.to));
        for tile in freed {
            if self.occupied.get(&tile) == Some(&id) {
                self.occupied.remove(&tile);
            }
        }
        self.events.push(Event::Died { who: id });
    }

    /// The characters' bodies, each tick: endurance spent running and got back walking or
    /// standing still, bleeding, and death.
    pub(crate) fn bodies(&mut self, strides: &[(ActorId, Option<bool>)]) {
        let (e, c, tick) = (self.rules.endurance, self.rules.combat, self.tick);
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            // Held, you strain: no breath comes back. Hungry or thirsty, it comes back slower.
            let held = self.is_held(id);
            let recovery = self.recovery(id) as i32;
            let Some(a) = self.actors.get_mut(id) else {
                continue;
            };
            if a.dead {
                continue;
            }
            let was = a.body.exhausted;
            let stride = strides
                .iter()
                .find(|(who, _)| *who == id)
                .and_then(|&(_, s)| s);
            match (stride, a.action) {
                (Some(true), _) => a.body.spend(e.run),
                _ if held => {}
                (Some(false), _) => a.body.endurance += e.walk * recovery / 1000,
                (None, None) => a.body.endurance += e.rest * recovery / 1000,
                (None, Some(_)) => {}
            }
            a.body.endurance = a.body.endurance.min(crate::body::FULL);
            if a.body.exhausted && a.body.endurance >= e.recover_at {
                a.body.exhausted = false;
            }
            let bleeding = a.body.wounds.iter().filter(|w| w.bleeding > 0).count() as i32;
            for w in &mut a.body.wounds {
                w.bleeding = w.bleeding.saturating_sub(1);
            }
            if bleeding > 0 && tick.is_multiple_of(u64::from(c.bleed_every.max(1))) {
                a.body.health -= bleeding * c.bleed_damage;
            }
            let (now, dead) = (a.body.exhausted, a.body.health <= 0);
            if now && !was {
                self.events.push(Event::Exhausted { who: id });
            }
            if dead {
                self.kill(id);
            }
        }
    }
}
