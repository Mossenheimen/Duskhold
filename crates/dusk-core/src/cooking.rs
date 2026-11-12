//! Cooking, and food that goes off (`CONCEPT.md`: hunger, and the fire as the heart of a base).
//!
//! - **Cooking** is crafting at a fire: a recipe that wants `fire` is made only beside a lit
//!   hearth, stove or forge (`fire.rs`), as any recipe is (`craft.rs`): its makings used up, a
//!   tool to hand if it names one (a pot for a stew). Cooking, the eleventh skill, learns from it
//!   and makes it quicker, as Carpentry does carpentry.
//! - **A meal's comfort:** food can ease dread as it's eaten (a hot stew most), or add to it (raw
//!   meat sits ill). It's food's `comfort`, taken off dread at once (`items.rs`).
//! - **Food goes off:** a food that `spoils` loses so much of its condition each game hour, in a
//!   pack or in a container or on a floor, and at none it's gone. What a container holds before
//!   anyone opens it has aged as if it had been there from the first hour (`World::fill`), so a
//!   larder left alone a week holds no meat. Cooked food keeps longer than raw; dried meat and
//!   cheese keep long; bread and fruit a few days.

use crate::actor::{ActorId, ActorKind};
use crate::items::{Defs, Item, ItemKind, NEW};
use crate::world::{Event, World};

impl Defs {
    /// The condition a food loses a game hour: 0 for anything that keeps.
    pub fn spoils(&self, def: u16) -> u16 {
        match self.item(def).map(|d| &d.kind) {
            Some(ItemKind::Food { spoils, .. }) => *spoils,
            _ => 0,
        }
    }

    /// Whether anything in the game goes off at all (none does in most test sets).
    fn any_spoils(&self) -> bool {
        self.items
            .iter()
            .any(|d| matches!(d.kind, ItemKind::Food { spoils, .. } if spoils > 0))
    }

    /// A food's condition after `hours` game hours, if it hasn't gone off: what a container's
    /// draws are when they're first looked at.
    pub(crate) fn aged(&self, def: u16, hours: u64) -> Option<u16> {
        let lost = u64::from(self.spoils(def)) * hours;
        let left = u64::from(NEW).saturating_sub(lost);
        (left > 0).then_some(left as u16)
    }

    /// An hour of ageing on a list: each food that spoils loses its share, and what's gone off is
    /// taken out. What went off, by definition.
    fn age(&self, list: &mut Vec<Item>) -> Vec<u16> {
        let mut gone = Vec::new();
        list.retain_mut(|item| {
            let spoils = self.spoils(item.def);
            if spoils == 0 {
                return true;
            }
            item.condition = item.condition.saturating_sub(spoils);
            if item.condition == 0 {
                gone.push(item.def);
            }
            item.condition > 0
        });
        gone
    }
}

impl World {
    /// The game hours gone since the world began.
    pub fn hours(&self) -> u64 {
        self.tick / (u64::from(self.rules.clock.ticks_per_minute.max(1)) * 60)
    }

    /// Whether `id` has a lit fire within reach, to cook on.
    pub fn by_a_fire(&self, id: ActorId) -> bool {
        self.hearth_beside(id, true).is_some()
    }

    /// Each game hour, food ages wherever it is: in packs, in containers, on floors. Food that
    /// goes off in a character's pack is news.
    pub(crate) fn spoil(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1)) * 60;
        if self.tick == 0 || !self.tick.is_multiple_of(per) || !self.defs.any_spoils() {
            return;
        }
        let defs = self.defs.clone();
        let carrying: Vec<ActorId> = self
            .actors
            .iter()
            .filter(|(_, a)| !a.items.is_empty())
            .map(|(id, _)| id)
            .collect();
        for id in carrying {
            let Some(a) = self.actors.get_mut(id) else {
                continue;
            };
            let gone = defs.age(&mut a.items);
            if a.kind == ActorKind::Character && !a.dead {
                for item in gone {
                    self.events.push(Event::Spoiled { who: id, item });
                }
            }
        }
        for list in self.stores.values_mut() {
            defs.age(list);
        }
    }
}
