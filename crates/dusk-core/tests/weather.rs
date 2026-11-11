//! Rain: spells of weather from the seed and the clock, lights put out outdoors, sounds and
//! scent that carry less far in it, and a darker sky.

use std::sync::Arc;

use dusk_core::weather::{DRY, HEAVY, RAIN};
use dusk_core::{
    ActorId, ActorKind, Alert, Defs, Event, Intent, Item, ItemDef, ItemKind, Part, Region, Tile,
    TilePos, Weather, World, WoundKind,
};

const TORCH: u16 = 0;
/// Ticks in a game minute.
const MINUTE: u32 = 25;

/// Rain every spell (heavy, if `heavy`), spells of an hour: sound and scent carry half as far,
/// the sky's at 60%, and a light held up outdoors goes out one minute in five in heavy rain.
fn weather(rain: u32, heavy: u32) -> Weather {
    Weather {
        spell: 60,
        rain,
        heavy,
        sound: [500, 500],
        scent: [500, 500],
        sky: [600, 600],
        douse: [0, 2000],
    }
}

/// A field with a roofed hut at (10..14, -2..2), noon, the needs stilled, a torch in the kit.
fn field(w8: Weather) -> (World, ActorId) {
    let mut w = World::new(21);
    w.defs = Arc::new(Defs {
        items: vec![ItemDef {
            id: "torch".into(),
            name: "torch".into(),
            weight: 100,
            stack: 5,
            kind: ItemKind::Light { radius: 5, wane: 1 },
            worth: 0,
        }],
        kit: vec![Item::new(TORCH, 1)],
        ..Defs::default()
    });
    w.creature_mut(0).wander = 0;
    w.rules.clock.start = 12 * 60;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -20..20 {
        for x in -20..20 {
            let roof = (10..14).contains(&x) && (-2..2).contains(&y);
            w.grid.set_tile(
                TilePos::new(x, y, 0),
                Tile {
                    roof,
                    ..Tile::GRASS
                },
            );
        }
    }
    w.set_region(Region {
        weather: w8,
        ..Region::default()
    });
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    (w, me)
}

fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut seen = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

#[test]
fn the_weather_comes_in_spells_from_the_seed_and_the_clock() {
    let (mut w, _) = field(weather(5000, 3000));
    // Over two days of hour-long spells, all three kinds come, each lasting its spell.
    let mut kinds = Vec::new();
    let mut seen = Vec::new();
    for _ in 0..48 {
        let at_start = w.rain();
        seen.extend(run(&mut w, 59 * MINUTE));
        assert_eq!(w.rain(), at_start, "a spell holds");
        kinds.push(at_start);
        seen.extend(run(&mut w, MINUTE));
    }
    for k in [DRY, RAIN, HEAVY] {
        assert!(kinds.contains(&k), "{k} never came");
    }
    assert!(seen.iter().any(|e| matches!(e, Event::RainBegan { .. })));
    assert!(seen.contains(&Event::RainEnded));
    // The same world the same way: the same weather.
    let (mut again, _) = field(weather(5000, 3000));
    let mut kinds_again = Vec::new();
    for _ in 0..48 {
        kinds_again.push(again.rain());
        run(&mut again, 60 * MINUTE);
    }
    assert_eq!(kinds, kinds_again);
}

#[test]
fn heavy_rain_puts_out_a_light_held_up_outdoors_but_not_under_a_roof() {
    let lit_after_an_hour = |indoors: bool| {
        let (mut w, me) = field(weather(10_000, 10_000));
        if indoors {
            w.intent(
                me,
                Intent::WalkTo {
                    to: TilePos::new(11, 0, 0),
                    run: false,
                },
            );
            run(&mut w, 200);
        }
        w.intent(
            me,
            Intent::Light {
                slot: 0,
                def: TORCH,
            },
        );
        let seen = run(&mut w, 60 * MINUTE);
        (
            w.actor(me).unwrap().light.is_some(),
            seen.iter().any(|e| matches!(e, Event::RainedOut { .. })),
        )
    };
    assert_eq!(
        lit_after_an_hour(true),
        (true, false),
        "under the roof it burns on"
    );
    assert_eq!(
        lit_after_an_hour(false),
        (false, true),
        "out in it, it goes out"
    );
}

#[test]
fn a_noise_carries_less_far_in_the_rain() {
    let heard = |rain: u32| {
        let (mut w, _) = field(weather(rain, 0));
        let z = w
            .spawn(ActorKind::Creature(0), TilePos::new(0, 8, 0))
            .unwrap();
        w.creature_mut(0).sight = 0;
        w.make_sound(TilePos::new(0, 0, 0), 10, None);
        run(&mut w, 2);
        w.actor(z).unwrap().alert() != Alert::Unaware
    };
    assert!(heard(0), "dry: 8 tiles off, a 10-tile noise is heard");
    assert!(!heard(10_000), "in the rain it carries 5");
}

#[test]
fn rain_washes_blood_away() {
    let smelled = |rain: u32| {
        let (mut w, me) = field(weather(rain, 0));
        w.wound(me, WoundKind::Laceration, Part::LeftArm);
        let z = w
            .spawn(ActorKind::Creature(0), TilePos::new(0, 10, 0))
            .unwrap();
        w.smells(z, 12).is_some()
    };
    assert!(smelled(0), "dry: 12 tiles for a bleeding wound reaches 10");
    assert!(!smelled(10_000), "in the rain, 6");
}

#[test]
fn the_sky_is_darker_under_rain() {
    let (dry, _) = field(weather(0, 0));
    let (wet, _) = field(weather(10_000, 0));
    assert_eq!(wet.daylight(), dry.daylight() * 600 / 1000);
    assert_eq!(
        wet.is_night(),
        dry.is_night(),
        "but it's no more night for the dead"
    );
}
