//! Bows and arrows: the first weapon that strikes from afar.
//!
//! - **A bow** is taken in hand as a weapon is, and shooting wants the other arm free too: no
//!   light held up, no shield.
//! - **A shot** (`Intent::Shoot`) draws (its wind-up, the tell), then looses an arrow from the
//!   pack along the straight line toward a tile within the bow's range. The first one on the
//!   line takes it: it lands by the bow's chance, less the farther it flies (half at the bow's
//!   range), more with Archery, and always on one unawares. Whatever stops sight stops it: a
//!   wall, a shut door, a boarded window.
//! - **A hit** does the bow's harm, taken as a spear's thrust is (bone turns it), three times on
//!   one unawares (`stealth.rs`). The arrow may break; if not, it lies where it fell, to be picked
//!   up again. A loose is quiet: the bow's noise, at the shooter.
//! - **Archery,** the tenth skill, learns from shots that land, and makes a bow's shots surer and
//!   harder as a weapon skill does a blow's.

use crate::actor::{ActionKind, ActorId};
use crate::barricade::spend;
use crate::geom::{Dir8, TilePos};
use crate::hash::{Purpose, below, roll};
use crate::items::{Item, ItemKind};
use crate::senses::walk_line;
use crate::skills::{On, Skill};
use crate::world::{Event, Sound, Swing, World};

/// A bow: its draw and loose (a swing: wind-up, recovery, breath, chance, harm, noise), how far
/// it shoots (tiles), the item it shoots, and the chance an arrow breaks on a hit (out of 10,000).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ranged {
    pub swing: Swing,
    pub range: i32,
    pub ammo: u16,
    pub breaks: u32,
}

/// Why you can't shoot there now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoShot {
    /// No bow in hand.
    NoBow,
    /// None of its arrows in your pack.
    NoArrows,
    /// A light held up, or a shield on your arm: it takes both hands.
    HandsFull,
    /// Past the bow's range, or your own tile.
    TooFar,
}

impl World {
    /// The bow in `id`'s hand, as Archery makes it.
    fn bow(&self, id: ActorId) -> Option<Ranged> {
        let held = self.actors.get(id)?.hands?;
        let ItemKind::Bow(mut r) = self.defs.item(held.def)?.kind else {
            return None;
        };
        let level = self.skill_level(id, Skill::Archery);
        r.swing = self.rules.skills.swing(r.swing, level);
        Some(r)
    }

    /// Whether `id` can shoot at `at` now.
    pub fn can_shoot(&self, id: ActorId, at: TilePos) -> Result<(), NoShot> {
        let a = self
            .actors
            .get(id)
            .filter(|a| !a.dead)
            .ok_or(NoShot::NoBow)?;
        let r = self.bow(id).ok_or(NoShot::NoBow)?;
        if a.light.is_some() || a.shield.is_some() {
            return Err(NoShot::HandsFull);
        }
        if !a.items.iter().any(|i| i.def == r.ammo) {
            return Err(NoShot::NoArrows);
        }
        if at.z != a.pos.z || at == a.pos || a.pos.steps_to(at) > r.range {
            return Err(NoShot::TooFar);
        }
        Ok(())
    }

    /// The shot's start: the draw, its way, its breath.
    pub(crate) fn shoot_action(
        &self,
        id: ActorId,
        at: TilePos,
    ) -> Option<(ActionKind, Dir8, u32, u32, i32)> {
        self.can_shoot(id, at).ok()?;
        let r = self.bow(id)?;
        let pos = self.actors.get(id)?.pos;
        let dir = Dir8::nearest(at.x - pos.x, at.y - pos.y)?;
        let s = r.swing;
        Some((
            ActionKind::Shoot { at },
            dir,
            s.wind_up,
            s.recovery,
            s.endurance,
        ))
    }

    /// The loose: an arrow from the pack flies the line toward `at`, and the first one on it
    /// takes the shot.
    pub(crate) fn loose(&mut self, id: ActorId, at: TilePos) {
        if self.can_shoot(id, at).is_err() {
            return;
        }
        let (Some(r), Some(from)) = (self.bow(id), self.actors.get(id).map(|a| a.pos)) else {
            return;
        };
        if let Some(a) = self.actors.get_mut(id) {
            spend(&mut a.items, r.ammo, 1);
        }
        self.emit(Sound {
            at: from,
            loudness: r.swing.noise,
            by: Some(id),
        });
        // Its flight: tile to tile until something stops it or it gets there.
        let (mut last, mut struck) = (from, None);
        walk_line(from, at, |p, dir| {
            let next = p.step(dir);
            let clear = match dir.split() {
                None => !self.grid.edge(p, dir).blocks_sight(),
                Some((a, b)) => {
                    let side = |x: Dir8, y: Dir8| {
                        !self.grid.edge(p, x).blocks_sight()
                            && !self.grid.tile(p.step(x)).blocks_sight()
                            && !self.grid.edge(p.step(x), y).blocks_sight()
                    };
                    side(a, b) || side(b, a)
                }
            };
            if !clear || self.grid.tile(next).blocks_sight() {
                return false;
            }
            last = next;
            if let Some(&who) = self.occupied.get(&next)
                && who != id
                && self
                    .actors
                    .get(who)
                    .is_some_and(|a| !a.dead && a.pos == next)
            {
                struck = Some(who);
                return false;
            }
            true
        });
        self.events.push(Event::Shot {
            who: id,
            from,
            to: last,
        });
        let (tick, who) = (self.tick, id.index() as u64);
        let arrow = Item::new(r.ammo, 1);
        let Some(target) = struck else {
            self.events.push(Event::Missed { who: id, at: last });
            self.put(last, arrow);
            return;
        };
        // Surer near: half its chance at the bow's range.
        let far = i64::from(from.steps_to(last));
        let reach = i64::from(r.range.max(1)) * 2;
        let chance = (i64::from(r.swing.hit) * (reach - far) / reach) as u32;
        let unaware = self.unawares(id, target);
        if !unaware && below(roll(self.seed, tick, who, Purpose::Hit), 10_000) >= chance {
            self.events.push(Event::Missed { who: id, at: last });
            self.put(last, arrow);
            return;
        }
        let (lo, hi) = r.swing.damage;
        let span = (hi - lo).max(0) as u32 + 1;
        let rolled = lo + below(roll(self.seed, tick, who, Purpose::Damage), span) as i32;
        // An arrow pierces as a spear's thrust does: bone turns it.
        let resist = self.resistance(target, Some(Skill::Spear));
        let sneak = if unaware { self.sneak_harm(id) } else { 1000 };
        let damage =
            (i64::from(rolled) * i64::from(resist) / 1000 * i64::from(sneak) / 1000) as i32;
        if let Some(t) = self.actors.get_mut(target) {
            t.body.health -= damage;
        }
        self.interrupt(target);
        self.startle_by(target, id);
        self.events.push(Event::Struck {
            who: id,
            target,
            damage,
            wound: None,
        });
        if unaware {
            self.events.push(Event::SneakAttack { who: id, target });
        }
        self.train(id, Skill::Archery, self.rules.skills.hit, On::Foe(target));
        if below(roll(self.seed, tick, who, Purpose::Arrow), 10_000) >= r.breaks {
            self.put(last, arrow);
        }
        if self.actors.get(target).is_some_and(|t| t.body.health <= 0) {
            self.kill(target);
        }
    }
}
