//! Grabs: the crowd's danger, with a way out (`CONCEPT.md`, combat).
//!
//! - **Engaged:** a creature is on you when it stands beside you, sees you, and could reach
//!   you. The client draws a line from each.
//! - **Taking hold:** a creature's hit can take hold of you. Each other one on you makes that
//!   likelier, and so does being out of breath. Step out of reach of one on you, and it may
//!   grab you as you go.
//! - **Held:** you can't walk or swing, its blows always land and are more often bites, and you
//!   get no breath back. A hold breaks when the holder is shoved, knocked down or killed, or when
//!   you're no longer within its reach.
//! - **The way out:** struggle (endurance, and harder for each other holder) or shove the holder
//!   off.
//! - **The drag-down:** three holding you at once, or a hold on you with no breath left, and
//!   they drag you down. Deaths stay readable: "I let three get round me".

use crate::actor::{ActorId, Alert};
use crate::hash::{Purpose, below, roll};
use crate::world::{Event, Sound, World, reaches};

impl World {
    /// The enemies on `id` now, in id order: standing beside it, seeing it, able to reach it.
    pub fn engaged(&self, id: ActorId) -> Vec<ActorId> {
        let Some(me) = self.actors.get(id).filter(|a| !a.dead) else {
            return Vec::new();
        };
        self.actors
            .iter()
            .filter(|(other, o)| {
                *other != id
                    && !o.dead
                    && o.prone_until == 0
                    && o.kind != me.kind
                    && o.alert() == Alert::Hunting
                    && reaches(&self.grid, o.pos, me.pos)
            })
            .map(|(other, _)| other)
            .collect()
    }

    /// Who has hold of `id`, in id order.
    pub fn holders(&self, id: ActorId) -> Vec<ActorId> {
        self.actors
            .iter()
            .filter(|(_, o)| o.holding == Some(id) && !o.dead)
            .map(|(other, _)| other)
            .collect()
    }

    /// Whether anyone has hold of `id`.
    pub fn is_held(&self, id: ActorId) -> bool {
        self.actors
            .iter()
            .any(|(_, o)| o.holding == Some(id) && !o.dead)
    }

    /// The chance, out of 10,000, that a creature's hit on `target` takes hold of it.
    pub(crate) fn grab_chance(&self, target: ActorId) -> u32 {
        let c = self.rules.combat;
        let others = self.engaged(target).len().saturating_sub(1) as u32;
        let tired = self.actors.get(target).is_some_and(|t| t.body.exhausted);
        (c.grab + c.grab_per_extra * others + if tired { c.grab_tired } else { 0 }).min(10_000)
    }

    /// `who` takes hold of `target`, which stops where it stands: a step under way is lost, and
    /// so is a swing still winding up.
    pub(crate) fn grab(&mut self, who: ActorId, target: ActorId) {
        let Some(h) = self.actors.get_mut(who) else {
            return;
        };
        if h.holding.is_some() {
            return;
        }
        h.holding = Some(target);
        if let Some(t) = self.actors.get_mut(target) {
            if let Some(step) = t.step.take()
                && self.occupied.get(&step.to) == Some(&target)
            {
                self.occupied.remove(&step.to);
            }
            t.carry = 0;
            if t.action.is_some_and(|a| a.winding_up()) {
                t.action = None;
            }
        }
        self.wake(target, crate::needs::Woke::Hurt);
        self.events.push(Event::Grabbed { who, target });
    }

    /// `holder` lets go. Tells of it, unless the one it held is dead.
    pub(crate) fn release(&mut self, holder: ActorId) {
        let Some(victim) = self.actors.get_mut(holder).and_then(|h| h.holding.take()) else {
            return;
        };
        if self.actors.get(victim).is_some_and(|v| !v.dead) {
            self.events.push(Event::Freed {
                who: victim,
                from: holder,
            });
        }
    }

    /// Each tick: the holds that can't last let go (the holder is down or dead, or the one it
    /// held is dead or out of its reach).
    pub(crate) fn grips(&mut self) {
        let holds: Vec<(ActorId, ActorId)> = self
            .actors
            .iter()
            .filter_map(|(id, a)| a.holding.map(|v| (id, v)))
            .collect();
        for (holder, victim) in holds {
            let lasts = match (self.actors.get(holder), self.actors.get(victim)) {
                (Some(h), Some(v)) => {
                    !h.dead && h.prone_until == 0 && !v.dead && reaches(&self.grid, h.pos, v.pos)
                }
                _ => false,
            };
            if !lasts {
                self.release(holder);
            }
        }
    }

    /// Struggling free, on its strike tick: the first holder lets go, or nobody does. Harder for
    /// each other holder, and half as likely out of breath.
    pub(crate) fn struggle(&mut self, id: ActorId) {
        let c = self.rules.combat;
        let holders = self.holders(id);
        let Some(&first) = holders.first() else {
            return;
        };
        let Some(a) = self.actors.get(id) else { return };
        let (pos, tired) = (a.pos, a.body.exhausted);
        self.emit(Sound {
            at: pos,
            loudness: c.struggle.noise,
            by: Some(id),
        });
        let others = holders.len() as u32 - 1;
        let mut chance = c.struggle.hit.saturating_sub(c.struggle_per_extra * others);
        if tired {
            chance /= 2;
        }
        if below(
            roll(self.seed, self.tick, id.index() as u64, Purpose::Struggle),
            10_000,
        ) < chance
        {
            self.release(first);
        } else {
            self.events.push(Event::StruggledInVain { who: id });
        }
    }

    /// The drag-down, each tick: a character held by `drag_down` at once, or held with no breath
    /// left, is pulled down and dies.
    pub(crate) fn drag_downs(&mut self) {
        let c = self.rules.combat;
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            let Some(a) = self.actors.get(id).filter(|a| !a.dead) else {
                continue;
            };
            let holders = self.holders(id).len() as u32;
            let breathless = a.body.endurance == 0;
            if holders >= c.drag_down.max(1) || (holders >= 1 && breathless) {
                self.events.push(Event::DraggedDown { who: id, holders });
                self.kill(id);
            }
        }
    }
}
