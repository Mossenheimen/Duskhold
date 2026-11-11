//! The clock, needs, eating and drinking, sleep, healing and moodles.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Defs, Event, Intent, Item, ItemDef, ItemKind, Moodle, Part, Tile, TilePos,
    Woke, World, WoundKind,
};

/// Ticks in a game hour (25 a game minute).
const HOUR: u32 = 25 * 60;
const BREAD: u16 = 0;
const WATER: u16 = 1;
const BED: u16 = 1;
const WELL: u16 = 2;

fn defs() -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 500,
        stack: 4,
        kind,
        worth: 0,
    };
    Defs {
        items: vec![
            item(
                "bread",
                ItemKind::Food {
                    nutrition: 40_000,
                    time: 30,
                },
            ),
            item(
                "water",
                ItemKind::Drink {
                    water: 50_000,
                    time: 20,
                },
            ),
        ],
        beds: vec![false, true],
        wells: vec![false, false, true],
        kit: vec![Item::new(BREAD, 1), Item::new(WATER, 1)],
        ..Defs::default()
    }
}

/// A room three tiles long: a bed at its west end, you in the middle, a pond to the east.
fn room() -> (World, ActorId) {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+
|. . ~|
+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs());
    w.grid.set_tile(
        TilePos::new(0, 0, 0),
        Tile {
            fixture: BED,
            solid: true,
            ..Tile::FLOOR
        },
    );
    let me = w
        .spawn(ActorKind::Character, TilePos::new(1, 0, 0))
        .unwrap();
    (w, me)
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

#[test]
fn needs_rise_with_the_clock() {
    let (mut w, me) = room();
    assert_eq!(w.clock(), (1, 8, 0), "day 1, 08:00");
    w.step(HOUR);
    assert_eq!(w.clock(), (1, 9, 0));
    let b = &w.actor(me).unwrap().body;
    assert_eq!(
        (b.hunger, b.thirst, b.fatigue),
        (6000, 7980, 4020),
        "an hour's worth"
    );
    w.step(9 * HOUR);
    assert!(
        w.moodles(me).contains(&(Moodle::Hungry, 2)),
        "{:?}",
        w.moodles(me)
    );
    assert!(w.moodles(me).contains(&(Moodle::Thirsty, 2)));
}

#[test]
fn eating_and_drinking_put_needs_back() {
    let (mut w, me) = room();
    w.step(10 * HOUR);
    let before = w.actor(me).unwrap().body.clone();
    let seen = act(
        &mut w,
        me,
        Intent::Use {
            slot: 0,
            def: BREAD,
        },
    );
    assert!(seen.contains(&Event::Ate {
        who: me,
        item: BREAD
    }));
    let seen = act(
        &mut w,
        me,
        Intent::Use {
            slot: 0,
            def: WATER,
        },
    );
    assert!(seen.contains(&Event::Drank {
        who: me,
        item: WATER
    }));
    let b = &w.actor(me).unwrap().body;
    assert!(b.hunger < before.hunger - 39_000 && b.thirst < before.thirst - 49_000);
    assert!(w.actor(me).unwrap().items.is_empty(), "both used up");

    let seen = act(&mut w, me, Intent::DrinkWater);
    assert!(
        seen.contains(&Event::DrankWater { who: me }),
        "from the pond"
    );
    assert_eq!(w.actor(me).unwrap().body.thirst, 0);
}

#[test]
fn sleep_in_a_bed_until_rested() {
    let (mut w, me) = room();
    let seen = act(&mut w, me, Intent::Sleep);
    assert!(
        !seen.iter().any(|e| matches!(e, Event::FellAsleep { .. })),
        "not tired yet"
    );
    w.step(12 * HOUR);
    let seen = act(&mut w, me, Intent::Sleep);
    assert!(seen.contains(&Event::FellAsleep {
        who: me,
        in_bed: true
    }));
    let from = w.tick;
    let mut woke = None;
    for _ in 0..30 * HOUR {
        w.step(1);
        if let Some(e) = w.events().find(|e| matches!(e, Event::WokeUp { .. })) {
            woke = Some(e);
            break;
        }
    }
    assert_eq!(
        woke,
        Some(Event::WokeUp {
            who: me,
            why: Woke::Rested
        })
    );
    assert_eq!(w.actor(me).unwrap().body.fatigue, 0);
    let slept = (w.tick - from) / u64::from(HOUR);
    assert!((3..=5).contains(&slept), "a few hours in a bed: {slept}");
}

#[test]
fn a_noise_or_choosing_to_get_up_wakes_you() {
    let (mut w, me) = room();
    w.step(12 * HOUR);
    act(&mut w, me, Intent::Sleep);
    assert!(w.actor(me).unwrap().body.asleep);
    w.make_sound(TilePos::new(2, 0, 0), 6, None);
    w.step(2);
    assert!(w.events().any(|e| e
        == Event::WokeUp {
            who: me,
            why: Woke::Noise
        }));

    act(&mut w, me, Intent::Sleep);
    w.intent(me, Intent::Wake);
    w.step(2);
    assert!(w.events().any(|e| e
        == Event::WokeUp {
            who: me,
            why: Woke::Chose
        }));
    assert!(!w.actor(me).unwrap().body.asleep);
}

#[test]
fn no_sleep_with_the_dead_about() {
    // A corridor: you at one end, a shambler at the other, facing away.
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+
|@ . . . z|
+-+-+-+-+-+
",
    )
    .unwrap();
    let me = w.character().unwrap();
    w.creature_mut(0).wander = 0;
    w.step(12 * HOUR); // tired, and it heard nothing and saw nothing: it isn't after you
    let seen = act(&mut w, me, Intent::Sleep);
    assert!(
        seen.iter().any(|e| matches!(e, Event::FellAsleep { .. })),
        "unaware, it's no reason to stay up"
    );
    act(&mut w, me, Intent::Wake);
    w.make_sound(TilePos::new(0, 0, 0), 8, Some(me)); // now it comes
    w.step(10);
    let seen = act(&mut w, me, Intent::Sleep);
    assert!(
        !seen.iter().any(|e| matches!(e, Event::FellAsleep { .. })),
        "not with it after you"
    );
}

#[test]
fn starving_drains_you_and_the_fed_heal() {
    let (mut w, me) = room();
    w.wound(me, WoundKind::Scratch, Part::LeftArm);
    let hurt = w.actor(me).unwrap().body.health;
    w.step(3 * HOUR);
    assert!(
        w.actor(me).unwrap().body.health > hurt,
        "fed and watered, you heal"
    );
    w.step(10 * HOUR);
    assert!(
        w.actor(me).unwrap().body.wounds.is_empty(),
        "a scratch closes in half a day"
    );
    // Then nobody eats: starving and dying of thirst, health drains.
    w.step(10 * HOUR);
    let moodles = w.moodles(me);
    assert!(moodles.contains(&(Moodle::Hungry, 3)) && moodles.contains(&(Moodle::Thirsty, 3)));
    let before = w.actor(me).unwrap().body.health;
    w.step(HOUR);
    assert_eq!(
        w.actor(me).unwrap().body.health,
        before - 60,
        "2 points an hour starving and 4 dying of thirst (health counts tenths)"
    );
}

/// A well is water to drink from, as a pond is; other furniture isn't.
#[test]
fn a_well_is_water_to_drink_from() {
    let (mut w, me) = room();
    let (here, east) = (TilePos::new(1, 0, 0), TilePos::new(2, 0, 0));
    w.grid.set_tile(east, Tile::FLOOR);
    assert!(
        w.water_beside(here).is_none(),
        "a bed beside you isn't water"
    );
    w.grid.set_tile(
        east,
        Tile {
            fixture: WELL,
            solid: true,
            ..Tile::FLOOR
        },
    );
    assert_eq!(w.water_beside(here), Some(east));
    w.step(6 * HOUR);
    assert!(w.actor(me).unwrap().body.thirst > 0, "thirsty by now");
    let seen = act(&mut w, me, Intent::DrinkWater);
    assert!(
        seen.contains(&Event::DrankWater { who: me }),
        "from the well"
    );
    assert_eq!(w.actor(me).unwrap().body.thirst, 0);
}
