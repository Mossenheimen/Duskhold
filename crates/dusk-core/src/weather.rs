//! Rain (`CONCEPT.md`, the realm: weather matters). It puts torches out, masks sound and washes
//! scent away, and an overcast sky is darker, so everyone sees less far.
//!
//! - **The weather is a function of the seed and the clock:** time falls into spells (some hours
//!   long), and each spell is dry, rain or heavy rain, by a roll of its own. Nothing is saved, and
//!   a spell can be known ahead.
//! - **Outdoors only:** under a roof a light stays lit, a sound carries as it would, and blood
//!   smells as it would. The sky's darker everywhere (a window lets in what light there is).
//! - **The numbers are the region's** (`Region::weather`): how much of the time it rains, how
//!   long a spell lasts, and what light and heavy rain each do.

use crate::actor::ActorId;
use crate::geom::TilePos;
use crate::hash::{Purpose, below, roll};
use crate::items::ItemKind;
use crate::world::{Event, World};

/// A region's weather, in the core's units. `spell` 0: it never rains.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Weather {
    /// Game minutes a spell of weather lasts.
    pub spell: u32,
    /// Out of 10,000: how many spells are rain, and of those, how many are heavy.
    pub rain: u32,
    pub heavy: u32,
    /// By rain (light, heavy), per mille: how far a sound made outdoors carries, how far blood
    /// outdoors is smelled, and the sky's light.
    pub sound: [u32; 2],
    pub scent: [u32; 2],
    pub sky: [u32; 2],
    /// By rain, out of 10,000: the chance a minute that a light held up outdoors goes out.
    pub douse: [u32; 2],
}

/// No rain, light rain, heavy rain.
pub const DRY: u8 = 0;
pub const RAIN: u8 = 1;
pub const HEAVY: u8 = 2;

impl World {
    /// The weather in the spell `spell` (the clock's minutes over the spell's length).
    fn rain_in(&self, spell: u64) -> u8 {
        let w = self.region.weather;
        if w.spell == 0 {
            return DRY;
        }
        let r = roll(self.seed, spell, 0, Purpose::Weather);
        if below(r, 10_000) >= w.rain {
            DRY
        } else if below(r << 32, 10_000) < w.heavy {
            HEAVY
        } else {
            RAIN
        }
    }

    /// The rain now: `DRY`, `RAIN` or `HEAVY`.
    pub fn rain(&self) -> u8 {
        let spell = u64::from(self.region.weather.spell.max(1));
        self.rain_in(self.clock_minutes() / spell)
    }

    /// One of the weather's per-mille numbers, by the rain now (1000 when dry).
    fn by_rain(&self, of: [u32; 2]) -> u32 {
        match self.rain() {
            DRY => 1000,
            r => of[usize::from(r) - 1],
        }
    }

    /// Whether a tile is out in the weather: not under a roof.
    pub fn outdoors(&self, at: TilePos) -> bool {
        !self.grid.tile(at).roof
    }

    /// How far a sound made at `at` carries now: less out in the rain.
    pub(crate) fn carries(&self, at: TilePos, loudness: i32) -> i32 {
        if !self.outdoors(at) {
            return loudness;
        }
        (i64::from(loudness) * i64::from(self.by_rain(self.region.weather.sound)) / 1000) as i32
    }

    /// How far blood at `at` is smelled, for each wound: less out in the rain.
    pub(crate) fn scent_reach(&self, at: TilePos, per_wound: i32) -> i32 {
        if !self.outdoors(at) {
            return per_wound;
        }
        (i64::from(per_wound) * i64::from(self.by_rain(self.region.weather.scent)) / 1000) as i32
    }

    /// The sky's light under the clouds, per mille of a clear sky's.
    pub(crate) fn overcast(&self) -> u32 {
        self.by_rain(self.region.weather.sky)
    }

    /// Each game minute: word of rain beginning or ending, and lights held up outdoors may go out.
    pub(crate) fn weather(&mut self) {
        let per = u64::from(self.rules.clock.ticks_per_minute.max(1));
        if !self.tick.is_multiple_of(per) {
            return;
        }
        let w = self.region.weather;
        if w.spell == 0 {
            return;
        }
        let now = self.clock_minutes();
        let spell = u64::from(w.spell);
        let (was, rain) = (self.rain_in(now.saturating_sub(1) / spell), self.rain());
        if rain != was {
            self.events.push(match rain {
                DRY => Event::RainEnded,
                r => Event::RainBegan { heavy: r == HEAVY },
            });
        }
        if rain == DRY {
            return;
        }
        let chance = w.douse[usize::from(rain) - 1];
        let (seed, tick) = (self.seed, self.tick);
        let lit: Vec<ActorId> = self
            .characters
            .iter()
            .copied()
            .filter(|&id| {
                self.actors.get(id).is_some_and(|a| {
                    !a.dead
                        && self.outdoors(a.pos)
                        && a.light.is_some_and(|l| {
                            matches!(
                                self.defs.item(l.def).map(|d| &d.kind),
                                Some(ItemKind::Light { .. })
                            )
                        })
                })
            })
            .collect();
        for id in lit {
            if below(
                roll(seed, tick, id.index() as u64, Purpose::Weather),
                10_000,
            ) < chance
            {
                let item = self
                    .actors
                    .get(id)
                    .and_then(|a| a.light)
                    .map_or(0, |l| l.def);
                self.douse(id);
                self.events.push(Event::RainedOut { who: id, item });
            }
        }
    }
}
