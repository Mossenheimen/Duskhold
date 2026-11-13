//! Timed actions, and the fight: one rule for all of them (`ARCHITECTURE.md`).
//!
//! An action starts only between steps, faces its way, pays its endurance, then winds up (the
//! tell everyone can see), takes effect on its strike tick, and recovers. Nobody moves while
//! acting, and a blow taken during the wind-up breaks it off. Exhausted, every wind-up and
//! recovery is half as long again, and the character's blows do half the harm.
//!
//! The verbs: a swing, a heavy swing (`Heavy`: your weight behind it, a long tell, and it may
//! knock down), a shove, a blow on one that's down (it always lands, double), a struggle.

use crate::actor::{Action, ActionKind, ActorId, ActorKind, Intent};
use crate::body::{Part, Wound, WoundKind};
use crate::geom::{Dir8, TilePos};
use crate::grid::{Edge, edge_owner};
use crate::hash::{Purpose, below, roll};
use crate::skills::Skill;
use crate::world::{Event, Sound, World};

/// Ticks to catch your second wind.
const SECOND_WIND_TICKS: u32 = 15;

/// A heavy swing (rules.toml's [heavy]), by a swing's own numbers, per mille: its wind-up (the
/// tell, and a blow breaks it off) and recovery, the breath it costs, and its harm; out of
/// 10,000, its chance to knock down what it hits; and how many blows' wear it puts on the weapon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heavy {
    pub wind_up: u32,
    pub recovery: u32,
    pub endurance: u32,
    pub harm: u32,
    pub knock_down: u32,
    pub wear: u8,
}

impl Default for Heavy {
    fn default() -> Heavy {
        Heavy {
            wind_up: 2000,
            recovery: 1500,
            endurance: 2000,
            harm: 1750,
            knock_down: 5000,
            wear: 2,
        }
    }
}

/// Why the door or window beside you won't open or shut (`World::can_use_door`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoDoor {
    /// No door or window on a side of your tile.
    Nothing,
    /// Boards across it: it stays as it is until they're pried off.
    Boarded,
    /// A broken window: open for good.
    Broken,
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
                ActionKind::Heavy => {
                    self.heavy = true;
                    self.strike(id, action.dir);
                    self.heavy = false;
                }
                ActionKind::Shove => self.shove(id, action.dir),
                ActionKind::Door => self.toggle_door(id, action.dir),
                ActionKind::Struggle => self.struggle(id),
                ActionKind::Take { at, slot, def } => self.take(id, at, slot, def),
                ActionKind::Drop { slot, def } => self.drop_item(id, slot, def),
                ActionKind::Use { slot, def } => self.use_item(id, slot, def),
                ActionKind::Wield { slot, def } => self.wield(id, slot, def),
                ActionKind::Unwield => self.unwield(id),
                ActionKind::Unshield => self.unshield(id),
                ActionKind::Shoot { at } => self.loose(id, at),
                ActionKind::Sleep => self.fall_asleep(id),
                ActionKind::DrinkWater => self.drink_water(id),
                ActionKind::Light { slot, def } => self.hold_light(id, slot, def),
                ActionKind::Douse => self.douse(id),
                ActionKind::Board => self.nail_board(id, action.dir),
                ActionKind::Unboard => self.pry_board(id, action.dir),
                ActionKind::Stash { at } => self.mark_stash(id, at),
                ActionKind::Put { at, slot, def } => self.put_away(id, at, slot, def),
                ActionKind::Burn { body } => self.kindle(id, body),
                ActionKind::Butcher { body } => self.butcher(id, body),
                ActionKind::Tend { at, tend } => self.tend(id, at, tend),
                ActionKind::Smother { at } => self.smother(id, at),
                ActionKind::Craft { recipe } => self.craft(id, recipe),
                ActionKind::BreakDown { at } => self.break_down(id, at),
                ActionKind::Buy { at, line } => self.buy(id, at, line),
                ActionKind::Sell { at, slot, def } => self.sell(id, at, slot, def),
                ActionKind::Mend { at } => self.mend(id, at),
                ActionKind::Claim { at, rare } => self.claim(id, at, rare),
                ActionKind::Identify { at, slot } => self.read_item(id, at, slot),
                ActionKind::Uncurse { at } => self.lift_curse(id, at),
                ActionKind::SecondWind => self.second_wind(id),
                ActionKind::Turn => self.turn_the_dead(id),
                ActionKind::Pray { at } => self.pray(id, at),
                ActionKind::Consecrate { body } => self.consecrate(id, body),
                ActionKind::Heal => self.heal(id),
                ActionKind::Ward => self.ward(id, action.dir),
                ActionKind::Pick { at } => self.pick(id, at),
                ActionKind::Wear { slot, def } => self.put_on(id, slot, def),
                ActionKind::TakeOff { piece, def } => self.take_off(id, piece, def),
                ActionKind::Shift { slot, def } => self.shift(id, slot, def),
                ActionKind::DropBag => self.drop_bag(id),
                ActionKind::Cast { spell, at } => self.cast(id, spell, at),
            }
        }
        true
    }

    /// Starts the action an intent asks for, if it can start here and now. Held, you can only
    /// struggle, shove, or (a cleric) turn the dead; struggling is only for the held. Item actions
    /// take all their time before they take effect, so a blow breaks them off.
    fn begin(&mut self, id: ActorId, intent: Intent) -> Option<Action> {
        let held = self.is_held(id);
        let a = self.actors.get(id)?;
        let (kind, dir, wind_up, recovery, endurance) = match intent {
            Intent::Struggle if !held => return None,
            Intent::Struggle | Intent::Shove { .. } => {
                // the two things the held can do: below
                return self.begin_free(id, intent, held);
            }
            // And a cleric's third (`faith.rs`): the dead let go of whoever they turn from. Its
            // wind-up is a tell, and its word is heard as it lands.
            Intent::Turn => {
                self.can_turn(id).ok()?;
                let f = self.rules.faith;
                (
                    ActionKind::Turn,
                    a.facing,
                    f.turn_wind_up,
                    f.turn_recovery,
                    0,
                )
            }
            _ if held => return None,
            Intent::Take { at, slot, def } => {
                let ticks = self.take_ticks(id, at, slot, def)?;
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
            // A cursed weapon clings: nothing else goes in that hand, and it won't be put away.
            Intent::Wield { .. } | Intent::Unwield if self.hand_clings(id) => return None,
            Intent::Wield { slot, def } => {
                let ticks = self.item_ticks(id, slot, def, false)?;
                (ActionKind::Wield { slot, def }, a.facing, ticks, 0, 0)
            }
            Intent::Unwield => {
                a.hands?;
                (ActionKind::Unwield, a.facing, self.defs.unwield_time, 0, 0)
            }
            Intent::Shoot { at } => self.shoot_action(id, at)?,
            Intent::Unshield => {
                a.shield?;
                (ActionKind::Unshield, a.facing, self.defs.unwield_time, 0, 0)
            }
            Intent::Wear { slot, def } => {
                let ticks = self.wear_ticks(id, slot, def)?;
                (ActionKind::Wear { slot, def }, a.facing, ticks, 0, 0)
            }
            Intent::TakeOff { piece, def } => {
                let ticks = self.take_off_ticks(id, piece, def)?;
                (ActionKind::TakeOff { piece, def }, a.facing, ticks, 0, 0)
            }
            // Bags (`bag.rs`): a stack between the pockets and the bag, and the bag dropped.
            Intent::Shift { slot, def } => {
                if self.can_shift(id, slot, def) == 0 {
                    return None;
                }
                let kind = ActionKind::Shift { slot, def };
                (kind, a.facing, self.defs.shift.max(1), 0, 0)
            }
            Intent::DropBag => {
                a.bag?;
                (ActionKind::DropBag, a.facing, self.defs.drop.max(1), 0, 0)
            }
            // A spell's words are its wind-up (`spell.rs`): a blow breaks them.
            Intent::Cast { spell, at } => {
                let (kind, dir, ticks) = self.cast_action(id, spell, at)?;
                (kind, dir, ticks, 0, 0)
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
            Intent::Board => {
                let side = self.can_board(id).ok()?;
                let level = self.skill_level(id, Skill::Carpentry);
                let ticks = self
                    .rules
                    .skills
                    .board_ticks(self.rules.barricade.time, level);
                // The hammering starts, and everything in earshot hears it.
                self.start_hammering(id);
                (ActionKind::Board, side, ticks, 0, 0)
            }
            Intent::Stash { at } => {
                self.can_stash(id, at).ok()?;
                let ticks = self.rules.deeds.stash_time.max(1);
                (ActionKind::Stash { at }, a.facing, ticks, 0, 0)
            }
            Intent::Put { at, slot, def } => {
                let ticks = self.put_ticks(id, at, slot, def)?.max(1);
                (ActionKind::Put { at, slot, def }, a.facing, ticks, 0, 0)
            }
            Intent::Buy { at, line } => {
                self.can_buy(id, at, line).ok()?;
                let kind = ActionKind::Buy { at, line };
                (kind, self.toward(id, at), self.deal_ticks(at), 0, 0)
            }
            Intent::Sell { at, slot, def } => {
                self.can_sell(id, at, slot, def).ok()?;
                let kind = ActionKind::Sell { at, slot, def };
                (kind, self.toward(id, at), self.deal_ticks(at), 0, 0)
            }
            Intent::Mend { at } => {
                self.repair_cost(id, at).ok()?;
                let kind = ActionKind::Mend { at };
                (kind, self.toward(id, at), self.deal_ticks(at), 0, 0)
            }
            Intent::SecondWind => {
                if !self.can_second_wind(id) {
                    return None;
                }
                (ActionKind::SecondWind, a.facing, SECOND_WIND_TICKS, 0, 0)
            }
            Intent::Identify { at, slot } => {
                self.can_identify(id, at, slot).ok()?;
                let kind = ActionKind::Identify { at, slot };
                (kind, self.toward(id, at), self.deal_ticks(at), 0, 0)
            }
            Intent::Uncurse { at } => {
                self.can_uncurse(id, at).ok()?;
                let kind = ActionKind::Uncurse { at };
                (kind, self.toward(id, at), self.deal_ticks(at), 0, 0)
            }
            Intent::Claim { at, rare } => {
                self.can_claim(id, at, rare).ok()?;
                let kind = ActionKind::Claim { at, rare };
                (kind, self.toward(id, at), self.deal_ticks(at), 0, 0)
            }
            Intent::Burn => {
                let (kind, dir, ticks) = self.burn_action(id)?;
                (kind, dir, ticks, 0, 0)
            }
            Intent::Butcher => {
                let (kind, dir, ticks) = self.butcher_action(id)?;
                (kind, dir, ticks, 0, 0)
            }
            Intent::Tend => {
                let (kind, dir, ticks) = self.tend_action(id)?;
                (kind, dir, ticks, 0, 0)
            }
            Intent::Craft { recipe } => {
                let (kind, ticks) = self.craft_action(id, recipe)?;
                (kind, a.facing, ticks, 0, 0)
            }
            Intent::BreakDown => {
                let (kind, dir, ticks) = self.break_action(id)?;
                (kind, dir, ticks, 0, 0)
            }
            Intent::Smother => {
                let (kind, dir, ticks) = self.smother_action(id)?;
                (kind, dir, ticks, 0, 0)
            }
            Intent::Unboard => {
                let side = self.can_unboard(id).ok()?;
                let ticks = self.rules.barricade.remove_time.max(1);
                (ActionKind::Unboard, side, ticks, 0, 0)
            }
            // Acts of faith (`faith.rs`): they take all their time before they take effect.
            Intent::Pray => {
                let (kind, dir, ticks) = self.pray_action(id)?;
                (kind, dir, ticks, 0, 0)
            }
            Intent::Consecrate => {
                let (kind, dir, ticks) = self.consecrate_action(id)?;
                (kind, dir, ticks, 0, 0)
            }
            Intent::Heal => {
                self.can_heal(id).ok()?;
                (ActionKind::Heal, a.facing, self.rules.faith.mend_time, 0, 0)
            }
            Intent::Ward => {
                let side = self.can_ward(id).ok()?;
                (ActionKind::Ward, side, self.rules.faith.ward_time, 0, 0)
            }
            Intent::Pick => {
                let (kind, dir, ticks) = self.pick_action(id)?;
                (kind, dir, ticks, 0, 0)
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
            Intent::Attack { .. } | Intent::HeavyAttack { .. } | Intent::Door if held => {
                return None;
            }
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
            Intent::HeavyAttack { dir } => {
                let (s, h) = (self.swing_of(id), self.rules.heavy);
                let dir = self.aim(id, dir);
                (
                    ActionKind::Heavy,
                    dir,
                    s.wind_up * h.wind_up / 1000,
                    s.recovery * h.recovery / 1000,
                    (i64::from(s.endurance) * i64::from(h.endurance) / 1000) as i32,
                )
            }
            // Held, a shove needs a hand free: a weapon in one and a light in the other leave
            // only the struggle.
            Intent::Shove { .. } if held && !self.can_shove(id) => return None,
            Intent::Shove { dir } => {
                let s = c.shove;
                let dir = self.aim(id, dir);
                (ActionKind::Shove, dir, s.wind_up, s.recovery, s.endurance)
            }
            Intent::Door => {
                let dir = self.can_use_door(id).ok()?;
                (ActionKind::Door, dir, c.door_ticks, 0, 0)
            }
            _ => return None,
        };
        self.start(id, kind, dir, wind_up, recovery, endurance)
    }

    /// Whether `id` could shove now. Free, anyone can. Held, it takes a free hand: with a weapon
    /// in one hand and a light held up in the other, there's only the struggle.
    pub fn can_shove(&self, id: ActorId) -> bool {
        let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
            return false;
        };
        !(self.is_held(id) && a.hands.is_some() && a.light.is_some())
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
        // A Fighter's endurance goes further (`class.rs`).
        let endurance = endurance * self.endurance_rate(id) / 1000;
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

    /// Another foe beside `id` to cleave into: the way to the first one alive, clockwise from
    /// north.
    fn next_foe(&self, id: ActorId) -> Option<Dir8> {
        let pos = self.actors.get(id)?.pos;
        Dir8::ALL
            .into_iter()
            .find(|&d| self.target_at(id, pos.step(d)).is_some())
    }

    /// The enemy a blow from `id` would hit on `at`: alive, and reachable from where `id` stands
    /// (no wall or corner in the way).
    fn target_at(&self, id: ActorId, at: TilePos) -> Option<ActorId> {
        let a = self.actors.get(id)?;
        let &other = self.occupied.get(&at)?;
        let o = self.actors.get(other)?;
        (other != id && !o.dead && a.kind.hostile_to(o.kind) && self.can_reach(a.pos, at))
            .then_some(other)
    }

    /// The side of a tile with a door or a window: the one faced (or, facing a diagonal, either
    /// of its sides), else the first of north, east, south and west.
    fn door_beside(&self, pos: TilePos, facing: Dir8) -> Option<Dir8> {
        let faced = match facing.split() {
            Some((a, b)) => [Some(a), Some(b)],
            None => [Some(facing), None],
        };
        faced
            .into_iter()
            .flatten()
            .chain([Dir8::N, Dir8::E, Dir8::S, Dir8::W])
            .find(|&d| {
                matches!(
                    self.grid.edge(pos, d),
                    Edge::Door { .. } | Edge::Window { .. }
                )
            })
    }

    /// The side of `id`'s tile with the door or window it would open or shut (`door_beside`):
    /// the one it faces, else the first of north, east, south and west.
    pub fn door_side(&self, id: ActorId) -> Option<Dir8> {
        let a = self.actors.get(id)?;
        self.door_beside(a.pos, a.facing)
    }

    /// Whether `id` can open or shut the door or window beside it now (`Intent::Door`): the
    /// side it's on, or why not.
    pub fn can_use_door(&self, id: ActorId) -> Result<Dir8, NoDoor> {
        let a = self.actors.get(id).ok_or(NoDoor::Nothing)?;
        let dir = self.door_beside(a.pos, a.facing).ok_or(NoDoor::Nothing)?;
        // A boarded door (or window) stays as it is until its boards are off, and a broken
        // window can't be shut.
        match self.grid.edge(a.pos, dir) {
            edge if edge.boards() > 0 => Err(NoDoor::Boarded),
            Edge::Window { broken: true, .. } => Err(NoDoor::Broken),
            _ => Ok(dir),
        }
    }

    /// A blow during the wind-up breaks it off.
    pub(crate) fn interrupt(&mut self, id: ActorId) {
        if let Some(a) = self.actors.get_mut(id)
            && let Some(act) = a.action.filter(|act| act.winding_up())
        {
            a.action = None;
            // A spell's words broken: it's lost, and nothing's spent (`spell.rs`).
            if let ActionKind::Cast { spell, .. } = act.kind {
                self.events.push(Event::SpellBroke { who: id, spell });
            }
        }
    }

    /// A swing lands (or doesn't) on its strike tick.
    fn strike(&mut self, id: ActorId, dir: Dir8) {
        let (c, tick, seed) = (self.rules.combat, self.tick, self.seed);
        let Some(a) = self.actors.get(id) else { return };
        let (pos, kind, tired) = (a.pos, a.kind, a.body.exhausted);
        let creature = self.stats(id);
        let swing = self.swing_of(id);
        // Read now: the blow may break the weapon.
        let skill = self.weapon_skill(id);
        self.emit(Sound {
            at: pos,
            loudness: swing.noise,
            by: Some(id),
        });
        let at = pos.step(dir);
        let Some(target) = self.target_at(id, at) else {
            if dir.split().is_none() && self.grid.edge(pos, dir).beatable() {
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
        // Struck unawares, from behind, a blow lands and does many times the harm (`stealth.rs`).
        let unaware = self.unawares(id, target);
        // On one that's down, one it holds, or one struck unawares, a blow always lands.
        if !down
            && !holds
            && !unaware
            && below(roll(seed, tick, who, Purpose::Hit), 10_000) >= swing.hit
        {
            self.events.push(Event::Missed { who: id, at });
            return;
        }
        // Braced behind a shield, the living may block a blow from ahead (`shield.rs`).
        if matches!(kind, ActorKind::Creature(_)) && self.blocks(target, id) {
            self.block(target, id);
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
                if self.heavy {
                    damage = damage * self.rules.heavy.harm as i32 / 1000;
                }
                // And as Strength makes it (`fitness.rs`).
                (self.strong_harm(id, damage), None)
            }
            ActorKind::Creature(_) => {
                let odds = creature.map_or([10_000, 0, 0], |c| c.wounds);
                let mut left = below(roll(seed, tick, who, Purpose::Wound), 10_000);
                let mut kind = WoundKind::Scratch;
                // Held, the mouth gets close: of a kind that bites at all (bone has no bite).
                let bites = odds[WoundKind::Bite as usize] > 0;
                if holds && bites && left < c.held_bite {
                    kind = WoundKind::Bite;
                } else {
                    for (k, &odds) in WoundKind::ALL.iter().zip(&odds) {
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
        // Armour over where it lands may turn it (`armour.rs`): no wound, only a bruise. Not a
        // blow on one that's down, or one struck unawares: those find the gaps.
        let turned = if down || unaware {
            None
        } else {
            match wound {
                Some((k, part, _)) => self.turns_wound(target, id, k, part).map(|p| (p, part)),
                None => self.turns_weapon(target, id, skill),
            }
        };
        let (damage, wound) = match turned {
            Some(_) => (self.bruise(damage), None),
            None => (damage, wound),
        };
        // Bone turns a blade and a mace shatters it: what the foe's kind takes of the blow
        // (`creature.rs`).
        let resist = match kind {
            ActorKind::Character => self.resistance(target, skill),
            ActorKind::Creature(_) => 1000,
        };
        // And a kind that shuns the light takes more of it standing in the light.
        let lit = match kind {
            ActorKind::Character => self.stats(target).map_or(1000, |c| c.lit_harm),
            ActorKind::Creature(_) => 1000,
        };
        let in_light = self
            .actors
            .get(target)
            .is_some_and(|t| self.light_at(t.pos) * 2 >= crate::light::FULL_LIGHT);
        let lit = if in_light { lit } else { 1000 };
        let sneak = if unaware { self.sneak_harm(id) } else { 1000 };
        let damage = (i64::from(damage) * i64::from(resist) / 1000 * i64::from(lit) / 1000
            * i64::from(sneak)
            / 1000) as i32;
        if unaware {
            self.events.push(Event::SneakAttack { who: id, target });
            self.train(
                id,
                Skill::Stealth,
                self.rules.skills.sneak_attack,
                crate::skills::On::Foe(target),
            );
        }
        if resist != 1000 {
            let weak = resist > 1000;
            self.events.push(Event::Resisted {
                who: id,
                target,
                weak,
            });
        }
        // A wound from a kind that carries ghoul fever may bring it (`fever.rs`).
        let fevered = wound.is_some_and(|(k, _, _)| {
            let odds = creature.map_or(0, |c| c.fever[k as usize]);
            below(roll(seed, tick, who, Purpose::Fever), 10_000) < odds
        });
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
        if let Some((piece, part)) = turned {
            self.turned(target, id, piece, part);
        }
        self.interrupt(target);
        self.startle_by(target, id);
        // A blow ends a turning: it rounds on whoever struck (`faith.rs`).
        self.unturn(target, id);
        self.wake(target, crate::needs::Woke::Hurt);
        self.events.push(Event::Struck {
            who: id,
            target,
            damage,
            wound: wound.map(|(k, p, _)| (k, p)),
        });
        if fevered {
            self.catch_fever(target);
        }
        // A blow on a foe teaches the weapon's skill, and wears the weapon: but not a blow on one
        // that's down. Finishing it there spares the weapon (Zomboid's stomp), which is what a
        // shove buys you.
        self.train_blow(id, skill, target, down);
        self.learn_by_use(id);
        // A character's blow that lands works toward its Strength, a heavy one twice.
        let work = self.rules.strength.blow * if self.heavy { 2 } else { 1 };
        self.strain(id, work);
        // A heavy blow may knock down what it hits, and is harder on the weapon.
        if self.heavy && !down {
            self.fell(id, target);
            for _ in 1..self.rules.heavy.wear {
                self.wear(id);
            }
        }
        if !down {
            self.wear(id);
        }
        if self.actors.get(target).is_some_and(|t| t.body.health <= 0) {
            self.kill(target);
            // Put down with a Hallowed weapon (`magic.rs`), or by one with Last Rites
            // (`faith.rs`), it stays down.
            self.put_to_rest(id, target);
            // Cleave: a blow that puts one down carries on into another beside you, once a swing.
            if !self.cleaving
                && self.cleaves(id)
                && let Some(next) = self.next_foe(id)
            {
                self.cleaving = true;
                self.strike(id, next);
                self.cleaving = false;
            }
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

    /// A blow on a shut door, or a window. Boards across it take it first (`barricade.rs`);
    /// beaten past its strength, the door gives way for good, and a window's glass breaks, with a
    /// crash heard far off (`climb.rs`). A warded door holds against the dead: their blows do
    /// nothing to it (`faith.rs`).
    fn thump(&mut self, id: ActorId, dir: Dir8) {
        let (c, tick, seed) = (self.rules.combat, self.tick, self.seed);
        let Some(a) = self.actors.get(id) else { return };
        let (pos, kind, tired) = (a.pos, a.kind, a.body.exhausted);
        if !kind.is_character() && self.is_warded(pos, dir) {
            return;
        }
        let (lo, hi) = match kind {
            ActorKind::Character if self.heavy => {
                let (lo, hi) = self.swing_of(id).damage;
                let h = self.rules.heavy.harm as i32;
                (lo * h / 1000, hi * h / 1000)
            }
            ActorKind::Character => self.swing_of(id).damage,
            ActorKind::Creature(_) => self.stats(id).map_or((0, 0), |c| c.thump),
        };
        let span = (hi - lo).max(0) as u32 + 1;
        let mut harm = lo + below(roll(seed, tick, id.index() as u64, Purpose::Thump), span) as i32;
        if tired {
            harm /= 2;
        }
        if kind.is_character() {
            harm = self.strong_harm(id, harm);
        }
        let (at, side) = edge_owner(pos, dir);
        self.emit(Sound {
            at: pos,
            loudness: c.thump_noise,
            by: Some(id),
        });
        self.events.push(Event::Thumped { who: id, at, side });
        self.wear(id);
        if self.beat_boards(pos, dir, harm) {
            return;
        }
        if let Edge::Window { .. } = self.grid.edge(pos, dir) {
            self.beat_glass(id, pos, dir, harm);
            return;
        }
        let taken = self.door_damage.entry((at, side)).or_insert(0);
        *taken += harm;
        if *taken >= c.door_strength {
            self.door_damage.remove(&(at, side));
            self.grid.set_edge(pos, dir, Edge::Open);
            self.events.push(Event::DoorBroke { at, side });
        }
    }

    /// A heavy blow that landed on `target` may knock it down, by `[heavy] knock_down`, for its
    /// kind's `down` time: if it's still standing to be knocked down.
    fn fell(&mut self, id: ActorId, target: ActorId) {
        let (tick, seed) = (self.tick, self.seed);
        let chance = self.rules.heavy.knock_down;
        if below(
            roll(seed, tick, id.index() as u64, Purpose::KnockDown),
            10_000,
        ) >= chance
        {
            return;
        }
        let stays_down = self.stats(target).map_or(0, |c| c.down);
        let Some(t) = self.actors.get_mut(target) else {
            return;
        };
        if t.dead || t.body.health <= 0 || t.prone_until > 0 || stays_down == 0 {
            return;
        }
        t.prone_until = tick + u64::from(stays_down);
        t.action = None;
        self.release(target);
        self.events.push(Event::Felled { who: id, target });
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
        ) < (c.knock_down as i32
            + self.knock_down_bonus(id)
            + self.bash_bonus(id)
            + self.strength_knock_down(id))
        .max(0) as u32;
        // A shove that finds someone works toward Strength (`fitness.rs`).
        self.strain(id, self.rules.strength.shove);
        // With a shield on the arm it's a bash, and it teaches Shield (`shield.rs`).
        if self.guard(id).is_some() {
            self.train(
                id,
                Skill::Shield,
                self.rules.skills.shield,
                crate::skills::On::Foe(target),
            );
        }
        // A shove breaks a hold.
        self.release(target);
        // How long one of its kind stays down.
        let stays_down = self.stats(target).map_or(0, |c| c.down);
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
            t.prone_until = tick + u64::from(stays_down);
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

    /// Opens or shuts the door (or the window) on that side. It won't shut on someone stepping
    /// or climbing through, and a boarded one stays as it is.
    fn toggle_door(&mut self, id: ActorId, dir: Dir8) {
        let c = self.rules.combat;
        let Some(pos) = self.actors.get(id).map(|a| a.pos) else {
            return;
        };
        let (open, turned) = match self.grid.edge(pos, dir) {
            Edge::Door { open, boards: 0 } => (open, Edge::door(!open)),
            Edge::Window {
                open,
                boards: 0,
                broken: false,
            } => (open, Edge::window(!open)),
            _ => return,
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
        self.grid.set_edge(pos, dir, turned);
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
        let now = self.clock_minutes();
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if a.dead {
            return;
        }
        a.dead = true;
        a.died = Some(now);
        a.remains = crate::actor::Remains::Lying;
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
        // A creature's gear falls where it's put down (a revenant's: yours); a character's stays
        // on its body, which may rise.
        let character = self.actors.get(id).is_some_and(|a| a.kind.is_character());
        if character {
            self.settle_body(id);
        } else {
            self.shed_gear(id);
            self.settle_remains(id);
        }
    }

    /// The characters' bodies, each tick: endurance spent running and got back walking or
    /// standing still, bleeding, and death.
    pub(crate) fn bodies(&mut self, strides: &[(ActorId, Option<bool>)]) {
        let (e, c, tick) = (self.rules.endurance, self.rules.combat, self.tick);
        let s = self.rules.skills;
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            // Held, you strain: no breath comes back. Hungry or thirsty, it comes back slower.
            let held = self.is_held(id);
            let recovery = self.recovery(id) as i32;
            let rate = self.endurance_rate(id);
            // Hurt legs make running cost more (`injury.rs`).
            let legs = self.leg_run(id) as i32;
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
                (Some(true), _) => {
                    let athletics = a.skills.level(Skill::Athletics, &s);
                    a.body
                        .spend(s.run_cost(e.run, athletics) * rate / 1000 * legs / 1000);
                }
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
