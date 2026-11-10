//! Day and night, light, and what the character makes out of the map.

use std::sync::Arc;

use dusk_core::view::{REMEMBERED, SEEN, UNSEEN};
use dusk_core::{
    ActorId, ActorKind, Defs, Event, Intent, Item, ItemDef, ItemKind, Tile, TilePos, View, World,
};

/// Ticks in a game minute.
const MINUTE: u32 = 25;
const TORCH: u16 = 0;

fn defs() -> Defs {
    Defs {
        items: vec![ItemDef {
            id: "torch".into(),
            name: "torch".into(),
            weight: 500,
            stack: 5,
            // Lights 5 tiles; 17 of its 1000 a minute: about an hour.
            kind: ItemKind::Light {
                radius: 5,
                wane: 17,
            },
        }],
        kit: vec![Item::new(TORCH, 2)],
        light_time: 15,
        douse_time: 5,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// Open grass, `w × h` tiles, from `hour` o'clock, with no wandering; you (with two torches)
/// at `you`.
fn field(w: i32, h: i32, hour: u32, you: TilePos) -> (World, ActorId) {
    let mut world = World::new(1);
    world.rules.clock.start = hour * 60;
    world.defs = Arc::new(defs());
    world.creature_mut(0).wander = 0;
    for y in 0..h {
        for x in 0..w {
            world.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    let me = world.spawn(ActorKind::Character, you).unwrap();
    (world, me)
}

/// Runs the world on to `hour` o'clock (today, or tomorrow if it's past).
fn until(w: &mut World, hour: u32) {
    let minutes = (hour * 60 + 24 * 60 - w.time_of_day()) % (24 * 60);
    w.step(minutes * MINUTE);
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..100 {
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
fn the_sky_follows_the_clock() {
    let mut w = World::new(1);
    assert_eq!(w.daylight(), 100, "08:00: full day");
    until(&mut w, 20);
    assert_eq!(w.daylight(), 60, "20:00: half way from sunset to dark");
    assert!(w.events().any(|e| e == Event::Nightfall), "at sunset");
    until(&mut w, 23);
    assert_eq!(w.daylight(), 20, "night: a fifth of the day's light");
    assert!(w.is_night());
    until(&mut w, 6);
    assert_eq!(w.daylight(), 60, "06:00: half way through dawn");
    assert!(w.events().any(|e| e == Event::Daybreak), "at dawn");
    until(&mut w, 12);
    assert!(!w.is_night());
}

#[test]
fn a_roof_keeps_light_out_and_a_torch_brings_it_but_not_through_walls() {
    // A room (floors are roofed) with a wall across it after its third tile.
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+-+-+
|. . .|. . . .|
+-+-+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs());
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
    assert_eq!(w.light_at(at(1, 0)), 60, "indoors by day: 60% of the sky");
    until(&mut w, 23);
    assert_eq!(w.light_at(at(1, 0)), 12, "indoors at night");
    let seen = act(
        &mut w,
        me,
        Intent::Light {
            slot: 0,
            def: TORCH,
        },
    );
    assert!(seen.contains(&Event::Lit {
        who: me,
        item: TORCH
    }));
    assert_eq!(w.light_at(at(0, 0)), 100, "in your hand");
    assert_eq!(w.light_at(at(2, 0)), 100, "full out to half its radius");
    assert_eq!(w.light_at(at(3, 0)), 12, "the wall keeps it in");
    let a = w.actor(me).unwrap();
    assert_eq!(a.light.map(|l| l.def), Some(TORCH));
    assert_eq!(
        a.items,
        vec![Item::new(TORCH, 1)],
        "one torch is left in the pack"
    );
}

#[test]
fn the_dead_see_you_by_light() {
    // By day, a shambler looking your way sees you five tiles off.
    let (mut w, _) = field(1, 7, 12, at(0, 5));
    let z = w.spawn(ActorKind::Creature(0), at(0, 0)).unwrap(); // facing south, at you
    assert!(w.sees(z, at(0, 5)));

    // At night it doesn't: only what's beside it.
    let (mut w, me) = field(1, 7, 23, at(0, 5));
    let z = w.spawn(ActorKind::Creature(0), at(0, 0)).unwrap();
    assert!(!w.sees(z, at(0, 5)) && w.sees(z, at(0, 1)));
    w.step(20);
    assert_eq!(w.actor(z).unwrap().pos, at(0, 0), "it's after nothing");

    // Light a torch, and you're bait.
    act(
        &mut w,
        me,
        Intent::Light {
            slot: 0,
            def: TORCH,
        },
    );
    assert!(w.sees(z, at(0, 5)));
    w.step(10);
    assert!(
        w.events().any(|e| e == Event::Saw { who: z, target: me }),
        "it saw your light"
    );
}

#[test]
fn you_see_ahead_and_remember_what_you_saw() {
    let (mut w, me) = field(11, 11, 12, at(5, 5)); // facing south
    let mut view = View::new(11, 11);
    w.look(me, &mut view);
    assert_eq!(view.at(at(5, 9)), SEEN, "ahead");
    assert_eq!(view.at(at(1, 9)), SEEN, "ahead and to the side");
    assert_eq!(view.at(at(5, 1)), UNSEEN, "behind you");
    assert_eq!(view.at(at(5, 4)), SEEN, "beside you, behind or not");
    // Turn round (walk a step north) and look again.
    w.intent(
        me,
        Intent::WalkTo {
            to: at(5, 4),
            run: false,
        },
    );
    w.step(20);
    w.look(me, &mut view);
    assert_eq!(view.at(at(5, 1)), SEEN, "ahead now");
    assert_eq!(view.at(at(5, 9)), REMEMBERED, "behind you now: remembered");
}

#[test]
fn in_the_dark_you_see_as_far_as_your_light() {
    let (mut w, me) = field(15, 15, 23, at(7, 2)); // facing south
    let mut view = View::new(15, 15);
    w.look(me, &mut view);
    assert!(view.sees(at(7, 6)), "the night sky shows you 4 tiles");
    assert!(!view.sees(at(7, 7)));
    act(
        &mut w,
        me,
        Intent::Light {
            slot: 0,
            def: TORCH,
        },
    );
    w.look(me, &mut view);
    assert!(view.sees(at(7, 7)), "a torch shows you 5");
    assert!(!view.sees(at(7, 8)));
    assert_eq!(view.at(at(7, 0)), UNSEEN, "behind you, still nothing");
}

#[test]
fn a_torch_burns_down_and_out() {
    let (mut w, me) = field(3, 3, 22, at(1, 1));
    act(
        &mut w,
        me,
        Intent::Light {
            slot: 0,
            def: TORCH,
        },
    );
    // Put it out after ten minutes: back in the pack, a sixth burnt, on its own.
    w.step(10 * MINUTE);
    act(&mut w, me, Intent::Douse);
    let a = w.actor(me).unwrap();
    assert!(a.light.is_none());
    assert_eq!(
        a.items,
        vec![
            Item::new(TORCH, 1),
            Item {
                def: TORCH,
                count: 1,
                condition: 1000 - 10 * 17,
            }
        ]
    );
    // Light the new one and let it burn: about an hour.
    act(
        &mut w,
        me,
        Intent::Light {
            slot: 0,
            def: TORCH,
        },
    );
    w.step(55 * MINUTE);
    assert!(w.actor(me).unwrap().light.is_some());
    w.step(5 * MINUTE);
    assert!(w.events().any(|e| e
        == Event::BurntOut {
            who: me,
            item: TORCH
        }));
    assert!(w.actor(me).unwrap().light.is_none());
}

#[test]
fn the_dead_are_faster_at_night() {
    // A shambler goes after a noise six tiles off: by day, then at night.
    let ticks = |hour: u32| {
        let (mut w, _) = field(3, 8, hour, at(2, 0)); // you, off to its side, unseen
        let z = w.spawn(ActorKind::Creature(0), at(0, 0)).unwrap();
        w.make_sound(at(0, 6), 10, None);
        (1..2000)
            .find(|_| {
                w.step(1);
                w.actor(z).unwrap().pos == at(0, 6)
            })
            .unwrap()
    };
    let (day, night) = (ticks(12), ticks(23));
    assert!(
        night * 5 <= day * 4 + 10,
        "a quarter faster: {day} by day, {night} at night"
    );
}
