//! Day and night, and light (`CONCEPT.md`: night is the enemy, and light is bait).
//!
//! - **The sky's light** follows the clock: full by day, the night's own (a moon, the stars) after
//!   dark, rising at dawn and falling at dusk. Under a roof it's dimmer.
//! - **Carried lights** (a torch, a candle) are held up in the other hand, lit. Each lights the
//!   tiles round it that it can reach in a straight line, fully near and less toward its radius,
//!   and burns down a game minute at a time until it goes out.
//! - **A tile's light** is the brightest of these, out of 100.
//! - **Sight scales with light:** a tile lit at 100 is seen out to an eye's full range, one lit at
//!   50 half as far. Everyone sees what's right beside them in any light. So in the dark the dead
//!   find you only up close, and your torch lets them see you from as far as they can see.

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::items::{Item, ItemKind};
use crate::senses::{Cone, line_of_sight};
use crate::world::{Event, World};

/// Full light.
pub const FULL_LIGHT: u32 = 100;

/// The day's hours, in game minutes after midnight, and the sky's light at night and under a
/// roof. The sky starts to lighten at `dawn`, is full from `sunrise` to `sunset`, and is dark
/// from `dark` to the next dawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Day {
    pub dawn: u32,
    pub sunrise: u32,
    pub sunset: u32,
    pub dark: u32,
    /// The sky's light at night, out of 100.
    pub night: u32,
    /// Under a roof the sky's light is this much of itself, per mille.
    pub indoors: u32,
}

/// The character's eyes: how far it sees in full light (tiles), how wide, and how many tiles
/// all round it sees in any light.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sight {
    pub range: i32,
    pub cone: Cone,
    pub beside: i32,
}

/// Whether a tile `d2` away (squared), lit at `light`, is bright enough for an eye that sees
/// `range` tiles in full light: it sees as far as the light allows.
pub fn bright_enough(d2: i64, light: u32, range: i32) -> bool {
    let reach = i64::from(range) * i64::from(light);
    d2 * i64::from(FULL_LIGHT).pow(2) <= reach * reach
}

/// How bright a carried light of `radius` tiles is `d2` away (squared): full out to half its
/// radius, then less, and nothing past it.
pub fn glow(d2: i64, radius: i32) -> u32 {
    let r = i64::from(radius.max(0));
    if d2 > r * r {
        return 0;
    }
    let (d, full) = (d2.isqrt(), r / 2);
    if d <= full {
        return FULL_LIGHT;
    }
    (i64::from(FULL_LIGHT) * (r + 1 - d) / (r + 1 - full)) as u32
}

impl World {
    /// Game minutes past midnight, today.
    pub fn time_of_day(&self) -> u32 {
        (self.clock_minutes() % (24 * 60)) as u32
    }

    /// The sky's light now, out of 100.
    pub fn daylight(&self) -> u32 {
        // Under rain clouds, darker (`weather.rs`).
        self.clear_daylight() * self.overcast() / 1000
    }

    /// The sky's light as if it were clear: what the dead count night by.
    fn clear_daylight(&self) -> u32 {
        let d = self.rules.day;
        let m = self.time_of_day();
        // From one light to another over a span of minutes, `t` minutes in.
        let ramp = |from: u32, to: u32, t: u32, span: u32| {
            let (from, to) = (i64::from(from), i64::from(to));
            (from + (to - from) * i64::from(t) / i64::from(span.max(1))) as u32
        };
        if (d.sunrise..d.sunset).contains(&m) {
            FULL_LIGHT
        } else if (d.dawn..d.sunrise).contains(&m) {
            ramp(d.night, FULL_LIGHT, m - d.dawn, d.sunrise - d.dawn)
        } else if (d.sunset..d.dark).contains(&m) {
            ramp(FULL_LIGHT, d.night, m - d.sunset, d.dark - d.sunset)
        } else {
            d.night
        }
    }

    /// Night, for the dead: the sky's light under half.
    pub fn is_night(&self) -> bool {
        self.clear_daylight() * 2 < FULL_LIGHT
    }

    /// The sky's light on a tile: dimmer under a roof.
    pub fn sky_light(&self, at: TilePos) -> u32 {
        let sky = self.daylight();
        if self.grid.tile(at).roof {
            sky * self.rules.day.indoors / 1000
        } else {
            sky
        }
    }

    /// The lights held up, lit: where each is, and its radius (more for a Bright one). And a
    /// glowing weapon in hand (Blazing, `magic.rs`) lights round whoever holds it.
    pub fn carried_lights(&self) -> impl Iterator<Item = (TilePos, i32)> + '_ {
        let held = self.characters.iter().filter_map(|&id| {
            let a = self.actors.get(id).filter(|a| !a.dead)?;
            let lit = a.light?;
            match &self.defs.item(lit.def)?.kind {
                ItemKind::Light { radius, .. } => {
                    let more: i32 = self.defs.properties_of(&lit).map(|p| p.glow).sum();
                    Some((a.pos, radius + more))
                }
                _ => None,
            }
        });
        let glowing = self.characters.iter().filter_map(|&id| {
            let a = self.actors.get(id).filter(|a| !a.dead)?;
            let glow: i32 = self.defs.properties_of(&a.hands?).map(|p| p.glow).sum();
            (glow > 0).then_some((a.pos, glow))
        });
        held.chain(glowing)
    }

    /// A tile's light, out of 100: the sky's, or that of a carried light, a burning body
    /// (`remains.rs`) or a fire in a hearth (`fire.rs`) that reaches it in a straight line,
    /// whichever is brighter.
    pub fn light_at(&self, at: TilePos) -> u32 {
        let mut best = self.sky_light(at);
        let lights = self.carried_lights().chain(self.fires());
        for (from, radius) in lights.chain(self.hearth_fires()) {
            if from.z != at.z {
                continue;
            }
            let lit = glow(from.dist2(at), radius);
            if lit > best && line_of_sight(&self.grid, from, at) {
                best = lit;
            }
        }
        best
    }

    /// The ticks it takes to light the item at `slot` and hold it up, if it's a light (still
    /// `def`).
    pub(crate) fn light_ticks(&self, id: ActorId, slot: u16, def: u16) -> Option<u32> {
        let a = self.actors.get(id)?;
        a.items.get(usize::from(slot)).filter(|i| i.def == def)?;
        matches!(self.defs.item(def)?.kind, ItemKind::Light { .. })
            .then_some(self.defs.light_time.max(1))
    }

    /// Lights the item at `slot` and holds it up. A light already held goes out, into the pack.
    pub(crate) fn hold_light(&mut self, id: ActorId, slot: u16, def: u16) {
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        let slot = usize::from(slot);
        if a.items.get(slot).is_none_or(|i| i.def != def) {
            return;
        }
        let mut item = a.items.remove(slot);
        if item.count > 1 {
            // One held up, the rest stay in the pack.
            let rest = Item {
                count: item.count - 1,
                ..item
            };
            a.items.insert(slot, rest);
            item.count = 1;
        }
        // The off arm holds a light or a shield: the shield goes in the pack (`shield.rs`).
        if let Some(s) = a.shield.take() {
            defs.stow(&mut a.items, s);
            a.bracing = false;
        }
        if let Some(was) = a.light.replace(item) {
            defs.stow(&mut a.items, was);
        }
        self.events.push(Event::Lit { who: id, item: def });
        // Lit, you see what it is.
        self.know_light(id);
    }

    /// Puts out the light held up, into the pack (burnt down as far as it got).
    pub(crate) fn douse(&mut self, id: ActorId) {
        let defs = self.defs.clone();
        let Some(a) = self.actors.get_mut(id) else {
            return;
        };
        if let Some(was) = a.light.take() {
            defs.stow(&mut a.items, was);
            self.events.push(Event::Doused {
                who: id,
                item: was.def,
            });
        }
    }

    /// Each game minute: the sky turns at sunset and at dawn, and the lights held up burn down,
    /// going out at nothing.
    pub(crate) fn burn_lights(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        let (d, m) = (self.rules.day, self.time_of_day());
        if m == d.sunset {
            self.events.push(Event::Nightfall);
        }
        if m == d.dawn {
            self.events.push(Event::Daybreak);
        }
        let defs = self.defs.clone();
        for i in 0..self.characters.len() {
            let id = self.characters[i];
            let Some(a) = self.actors.get_mut(id).filter(|a| !a.dead) else {
                continue;
            };
            let Some(mut lit) = a.light else { continue };
            let Some(ItemKind::Light { wane, .. }) = defs.item(lit.def).map(|d| &d.kind) else {
                continue;
            };
            // A Lasting light burns down slower (`magic.rs`).
            let pace = defs
                .properties_of(&lit)
                .fold(1000u32, |w, p| w * p.wane / 1000);
            let wane = (u32::from(*wane) * pace / 1000).max(1) as u16;
            lit.condition = lit.condition.saturating_sub(wane);
            if lit.condition == 0 {
                a.light = None;
                self.events.push(Event::BurntOut {
                    who: id,
                    item: lit.def,
                });
            } else {
                a.light = Some(lit);
            }
        }
    }
}
