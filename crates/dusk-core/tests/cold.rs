//! Wet and cold: the rain soaks you out of doors, and wading does; a roof dries you, a fire
//! faster. The cold drifts toward what the night, the wet and the rain are worth, less a roof
//! and what you wear (less of it wet); a fire warms you through. Shivering shakes your aim and
//! costs breath; freezing drains health and stops the healing, there's no sleep, and it wakes you.

use std::sync::Arc;

use dusk_core::armour::Armour;
use dusk_core::weather::Weather;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Event, Ground, Intent, Item, ItemDef, ItemKind, NoSleep,
    Region, Tile, TilePos, Woke, World,
};

/// Ticks in a game minute.
const MINUTE: u32 = 25;
const JERKIN: u16 = 0;
const FIREWOOD: u16 = 1;
const CANDLE: u16 = 2;
/// Fixture 1: a stove.
const STOVE: u16 = 1;
const STOVE_AT: TilePos = TilePos::new(4, 0, 0);
const OUT: TilePos = TilePos::new(-3, 0, 0);
const IN: TilePos = TilePos::new(3, 0, 0);

/// A jerkin that takes 15 points off the cold, logs and a candle (in the kit), and a stove.
fn defs() -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 500,
        stack: 10,
        kind,
        worth: 0,
    };
    let jerkin = Armour {
        covers: 2 | 4,
        turn: [0; 3],
        wear: 0,
        noise: 0,
        endurance: 0,
        sight: 1000,
        time: 5,
        warmth: 15_000,
    };
    Defs {
        items: vec![
            item("jerkin", ItemKind::Armour(jerkin)),
            item("firewood", ItemKind::Other),
            item("candle", ItemKind::Light { radius: 2, wane: 1 }),
        ],
        hearths: vec![0, 4],
        fuel: vec![(FIREWOOD, 120)],
        kit: vec![Item::new(FIREWOOD, 2), Item::new(CANDLE, 1)],
        carry: 30_000,
        light_time: 5,
        douse_time: 5,
        ..Defs::default()
    }
}

/// Midnight with no moon; out of 10,000, how often it rains and how much of it is heavy (all or
/// none, so it's always one or the other). Grass west of x = 0; from it east, a roofed floor
/// with a stove at (4, 0). You stand at `at`, needing nothing.
fn field(rain: u32, heavy: u32, at: TilePos) -> (World, ActorId) {
    let mut w = World::new(4);
    w.defs = Arc::new(defs());
    w.rules.clock.start = 0;
    w.rules.day.night = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -4..4 {
        for x in -6..8 {
            let tile = if x < 0 { Tile::GRASS } else { Tile::FLOOR };
            w.grid.set_tile(TilePos::new(x, y, 0), tile);
        }
    }
    w.grid.set_tile(
        STOVE_AT,
        Tile {
            fixture: STOVE,
            solid: true,
            ..Tile::FLOOR
        },
    );
    w.set_region(Region {
        weather: Weather {
            spell: 60,
            rain,
            heavy,
            sound: [1000; 2],
            scent: [1000; 2],
            sky: [1000; 2],
            douse: [0; 2],
        },
        ..Region::default()
    });
    let me = w.spawn(ActorKind::Character, at).unwrap();
    (w, me)
}

/// Runs `n` game minutes, and what happened.
fn minutes(w: &mut World, n: u32) -> Vec<Event> {
    let mut seen = Vec::new();
    for _ in 0..n * MINUTE {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..200 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

/// Puts a roof over `at` (a floor), or takes it off (grass).
fn roof(w: &mut World, at: TilePos, on: bool) {
    w.grid
        .set_tile(at, if on { Tile::FLOOR } else { Tile::GRASS });
}

#[test]
fn out_in_the_rain_you_get_wet_and_under_a_roof_you_dry() {
    let (mut w, me) = field(10_000, 0, OUT);
    minutes(&mut w, 30);
    assert_eq!(w.wet(me), 30 * 2000, "light rain: two points a minute");
    assert_eq!(w.wet_level(me), 2);
    let (mut heavy, other) = field(10_000, 10_000, OUT);
    let seen = minutes(&mut heavy, 30);
    assert_eq!(heavy.wet(other), 100_000, "heavy rain soaks you through");
    assert!(seen.contains(&Event::Wet {
        who: other,
        level: 3
    }));
    // Under a roof, out of the rain: a point a minute.
    roof(&mut w, OUT, true);
    let before = w.wet(me);
    minutes(&mut w, 10);
    assert_eq!(w.wet(me), before - 10 * 1000);
}

#[test]
fn wading_soaks_you_even_when_its_dry() {
    let (mut w, me) = field(0, 0, OUT);
    w.grid.set_tile(
        OUT,
        Tile {
            ground: Ground::Shallows,
            ..Tile::GRASS
        },
    );
    minutes(&mut w, 5);
    assert_eq!(w.wet(me), 5 * 10_000);
}

#[test]
fn the_cold_is_worth_the_night_the_wet_and_the_rain_less_a_roof_and_clothes() {
    let (mut w, me) = field(0, 0, OUT);
    assert_eq!(w.cold_due(me), 30_000, "the night out of doors");
    w.don(me, Item::new(JERKIN, 1));
    assert_eq!(w.cold_due(me), 15_000, "a jerkin takes 15 off");
    w.set_weathered(me, 100_000, 0);
    assert_eq!(
        w.cold_due(me),
        30_000 + 40_000 - 7_500,
        "soaked through: all the wet's worth, and half the jerkin's warmth"
    );
    w.set_weathered(me, 50_000, 0);
    assert_eq!(w.cold_due(me), 30_000 + 20_000 - 11_250, "half wet");
    roof(&mut w, OUT, true);
    assert_eq!(w.cold_due(me), 30_000 - 20_000 + 20_000 - 11_250, "a roof");
    // The rain on you, out of doors.
    let (light, me) = field(10_000, 0, OUT);
    assert_eq!(light.cold_due(me), 30_000 + 10_000);
    let (heavy, me) = field(10_000, 10_000, OUT);
    assert_eq!(heavy.cold_due(me), 30_000 + 20_000);
}

#[test]
fn the_cold_drifts_toward_what_its_worth_and_a_new_level_is_news() {
    let (mut w, me) = field(0, 0, OUT);
    minutes(&mut w, 40);
    assert_eq!(
        w.chill(me),
        40 * 500,
        "half a point a minute, getting colder"
    );
    let seen = minutes(&mut w, 40);
    assert_eq!(w.chill(me), 30_000, "no colder than it's worth");
    assert!(seen.contains(&Event::Cold { who: me, level: 1 }));
    assert_eq!(w.cold_level(me), 1, "chilly");
    roof(&mut w, OUT, true);
    minutes(&mut w, 10);
    assert_eq!(w.chill(me), 30_000 - 10 * 1000, "a point a minute, warming");
}

#[test]
fn a_fire_warms_you_through_and_dries_you() {
    let (mut w, me) = field(0, 0, IN);
    // Two logs in the stove, lit from the candle, the candle put out.
    act(&mut w, me, Intent::Tend);
    act(&mut w, me, Intent::Tend);
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == CANDLE)
        .unwrap() as u16;
    act(&mut w, me, Intent::Light { slot, def: CANDLE });
    act(&mut w, me, Intent::Tend);
    act(&mut w, me, Intent::Douse);
    assert!(w.by_fire(IN));
    assert!(!w.by_fire(OUT), "too far from it");
    w.set_weathered(me, 100_000, 80_000);
    assert_eq!(w.cold_due(me), 0, "by a fire, no cold at all");
    minutes(&mut w, 10);
    assert_eq!(w.chill(me), 80_000 - 10 * 3000, "three points a minute");
    assert_eq!(w.wet(me), 100_000 - 10 * 5000, "and five points of wet");
}

#[test]
fn shivering_shakes_your_aim_and_costs_breath() {
    let east = Some(Dir8::E);
    // What a swing at the air costs as it starts, and its chance to land, at a level of cold.
    let swing = |chill: i32| {
        let (mut w, me) = field(0, 0, OUT);
        w.set_weathered(me, 0, chill);
        let hit = w.swing(me).hit;
        let before = w.actor(me).unwrap().body.endurance;
        w.intent(me, Intent::Attack { dir: east });
        w.step(1);
        (
            hit,
            before - w.actor(me).unwrap().body.endurance,
            w.rules.cold,
        )
    };
    let (hit, cost, c) = swing(0);
    assert_eq!(swing(30_000).0, hit, "chilly is only a warning");
    let (cold_hit, cold_cost, _) = swing(60_000);
    assert_eq!(cold_hit, hit * c.aim[0] / 1000);
    assert_eq!(cold_cost, cost * c.breath[0] as i32 / 1000);
    let (freezing_hit, freezing_cost, _) = swing(90_000);
    assert_eq!(freezing_hit, hit * c.aim[1] / 1000);
    assert_eq!(freezing_cost, cost * c.breath[1] as i32 / 1000);
}

#[test]
fn freezing_drains_health_stops_the_healing_and_any_sleep() {
    // Heavy rain at midnight, soaked through: it's worth 90, freezing.
    let (mut w, me) = field(10_000, 10_000, OUT);
    w.rules.needs.fatigue.rise = 100_000;
    w.set_weathered(me, 100_000, 90_000);
    let full = w.actor(me).unwrap().body.health;
    minutes(&mut w, 60);
    assert_eq!(w.cold_level(me), 3, "freezing");
    assert_eq!(
        w.actor(me).unwrap().body.health,
        full - w.rules.cold.harm,
        "an hour's harm, and nothing healed"
    );
    assert_eq!(
        w.can_sleep(me),
        Err(NoSleep::Cold),
        "tired out, but too cold"
    );
}

#[test]
fn freezing_wakes_you() {
    // Asleep under a roof in heavy rain; then the roof's gone and you're soaked, a breath off
    // freezing.
    let (mut w, me) = field(10_000, 10_000, OUT);
    roof(&mut w, OUT, true);
    w.rules.needs.fatigue.rise = 100_000;
    minutes(&mut w, 1);
    w.rules.needs.fatigue.rise = 0;
    act(&mut w, me, Intent::Sleep);
    assert!(w.actor(me).unwrap().body.asleep);
    roof(&mut w, OUT, false);
    w.set_weathered(me, 100_000, 74_900);
    let seen = minutes(&mut w, 2);
    assert!(seen.contains(&Event::WokeUp {
        who: me,
        why: Woke::Cold
    }));
    assert!(!w.actor(me).unwrap().body.asleep);
}
