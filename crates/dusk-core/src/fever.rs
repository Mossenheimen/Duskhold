//! Ghoul fever: the Hushlands' affliction (`CONCEPT.md`, afflictions), and the first clock on a
//! life.
//!
//! - **Caught** from a ghoul's wound, by the chance its kind gives each kind of wound
//!   (`Creature::fever`): a bite most of all. A shambler's bite is only a bad wound.
//! - **Three stages, a game day each:**
//!   1. **a fever:** thirst rises faster, and nothing heals;
//!   2. **weakness:** endurance comes back slower too;
//!   3. **dying:** health drains each hour, and at the end of the day it kills you, whatever's
//!      left. (Rising as a revenant comes with death and the next character: step 6, slice 7.)
//! - **The cure** is an item (`ItemKind::Cure`) for now; the abbey sells it later (slice 5). It
//!   works at any stage, so the question is never whether to look, but how far and past what.
//!
//! The fever is the game minute it was caught, not a countdown: the stage is worked out from the
//! clock, so nothing has to tick it down.

use crate::actor::ActorId;
use crate::world::{Event, World};

/// The last stage: dying.
pub const DYING: u8 = 3;

/// Ghoul fever's numbers, in the core's units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fever {
    /// Game minutes each of the three stages lasts.
    pub stage: u32,
    /// Feverish (from the first stage), thirst rises this much as fast, per mille.
    pub thirst: u32,
    /// Weak (from the second), endurance comes back this much as fast, per mille.
    pub recovery: u32,
    /// Dying (the third), the health it costs a game hour, out of `body::FULL`.
    pub harm: i32,
}

impl Default for Fever {
    fn default() -> Fever {
        Fever {
            stage: 1440,
            thirst: 1500,
            recovery: 500,
            harm: 40,
        }
    }
}

impl World {
    /// How far along a character's fever is: 0 with none, then 1 (a fever), 2 (weakness) and 3
    /// (dying).
    pub fn fever_stage(&self, id: ActorId) -> u8 {
        let Some(since) = self.actors.get(id).and_then(|a| a.body.fever) else {
            return 0;
        };
        let stage = u64::from(self.rules.fever.stage.max(1));
        let gone = self.clock_minutes().saturating_sub(since);
        (1 + gone / stage).min(u64::from(DYING)) as u8
    }

    /// Game minutes until the fever kills, if there is one.
    pub fn fever_left(&self, id: ActorId) -> Option<u64> {
        let since = self.actors.get(id)?.body.fever?;
        let end = since + 3 * u64::from(self.rules.fever.stage);
        Some(end.saturating_sub(self.clock_minutes()))
    }

    /// Gives `id` ghoul fever, if it hasn't got it: from a ghoul's wound, or a test.
    pub fn catch_fever(&mut self, id: ActorId) {
        let now = self.clock_minutes();
        if let Some(a) = self.actors.get_mut(id)
            && !a.dead
            && a.body.fever.is_none()
        {
            a.body.fever = Some(now);
            self.events.push(Event::Fevered { who: id });
        }
    }

    /// Each game minute, after the needs: a fever that moves on to its next stage says so, and
    /// one that's run its course kills.
    pub(crate) fn fevers(&mut self) {
        let now = self.clock_minutes();
        let stage = u64::from(self.rules.fever.stage.max(1));
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            let Some(since) = self
                .actors
                .get(id)
                .filter(|a| !a.dead)
                .and_then(|a| a.body.fever)
            else {
                continue;
            };
            let gone = now.saturating_sub(since);
            if gone >= u64::from(DYING) * stage {
                self.events.push(Event::FeverTook { who: id });
                self.kill(id);
            } else if gone > 0 && gone.is_multiple_of(stage) {
                let stage = (1 + gone / stage) as u8;
                self.events.push(Event::FeverWorse { who: id, stage });
            }
        }
    }
}
