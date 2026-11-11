//! The dead get up, unless they're burned; fire through a pile of them; and the region's
//! pressure, which the risen raise and burning lowers, and which brings the dead walking in.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Defs, Edge, Event, GetUp, Intent, Item, ItemDef, ItemKind, NoBurn, Part,
    Pressure, Region, Remains, Tile, TilePos, World, WoundKind,
};

const SHAMBLER: ActorKind = ActorKind::Creature(0);
const OIL: u16 = 0;
const CANDLE: u16 = 1;
/// Ticks in a game minute.
const MINUTE: u32 = 25;

/// A flask of oil and a candle; you start with two flasks and a candle.
fn defs() -> Defs {
    let item = |id: &str, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 100,
        stack: 5,
        kind,
        worth: 0,
    };
    Defs {
        items: vec![
            item("oil", ItemKind::Oil),
            item("candle", ItemKind::Light { radius: 2, wane: 1 }),
        ],
        kit: vec![Item::new(OIL, 2), Item::new(CANDLE, 1)],
        ..Defs::default()
    }
}

/// A field of grass with you at its middle, the needs stilled. The shambler has 1 health (a
/// bite puts it down), and gets up an hour after it falls.
fn field() -> (World, ActorId) {
    let mut w = World::new(9);
    w.defs = Arc::new(defs());
    let z = w.creature_mut(0);
    (z.wander, z.health) = (0, 1);
    z.get_up = GetUp {
        chance: 10_000,
        after: (60, 60),
    };
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -20..20 {
        for x in -20..20 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    (w, me)
}

/// A shambler put down on a tile: a body.
fn body(w: &mut World, at: TilePos) -> ActorId {
    let z = w.spawn(SHAMBLER, at).unwrap();
    w.wound(z, WoundKind::Bite, Part::Torso);
    assert!(w.actor(z).unwrap().dead);
    z
}

/// Steps the world, keeping its events.
fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut seen = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

/// The slot of an item in your pack.
fn slot(w: &World, me: ActorId, def: u16) -> u16 {
    let items = &w.actor(me).unwrap().items;
    items.iter().position(|i| i.def == def).unwrap() as u16
}

/// Lights the candle and holds it up.
fn light_up(w: &mut World, me: ActorId) {
    let slot = slot(w, me, CANDLE);
    w.intent(me, Intent::Light { slot, def: CANDLE });
    run(w, 5);
    assert!(w.actor(me).unwrap().light.is_some());
}

/// Oil left in your pack.
fn oil(w: &World, me: ActorId) -> u16 {
    let a = w.actor(me).unwrap();
    a.items
        .iter()
        .filter(|i| i.def == OIL)
        .map(|i| i.count)
        .sum()
}

#[test]
fn a_body_gets_up_on_its_back_when_its_time_comes_and_only_once() {
    let (mut w, _) = field();
    let z = body(&mut w, TilePos::new(5, 5, 0));
    let seen = run(&mut w, 59 * MINUTE);
    assert!(w.actor(z).unwrap().dead, "not yet: it lies there an hour");
    assert!(!seen.contains(&Event::GotUp { who: z }));
    let seen = run(&mut w, MINUTE);
    assert!(seen.contains(&Event::GotUp { who: z }));
    let a = w.actor(z).unwrap();
    assert!(!a.dead && a.prone_until > 0, "it gets up from its back");
    assert_eq!(a.body.health, 1, "whole again");
    // Put down a second time, it stays down.
    w.wound(z, WoundKind::Bite, Part::Torso);
    run(&mut w, 3 * 60 * MINUTE);
    assert!(w.actor(z).unwrap().dead);
    assert_eq!(w.actor(z).unwrap().remains, Remains::Lying);
}

#[test]
fn a_kind_that_doesnt_get_up_stays_down() {
    let (mut w, _) = field();
    w.creature_mut(0).get_up = GetUp::default();
    let z = body(&mut w, TilePos::new(5, 5, 0));
    run(&mut w, 3 * 60 * MINUTE);
    assert!(w.actor(z).unwrap().dead);
}

#[test]
fn a_body_waits_to_get_up_while_someone_stands_on_it() {
    let (mut w, me) = field();
    let z = body(&mut w, TilePos::new(0, 1, 0));
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(0, 1, 0),
            run: false,
        },
    );
    run(&mut w, 61 * MINUTE);
    assert!(w.actor(z).unwrap().dead, "you're standing on it");
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(0, 4, 0),
            run: false,
        },
    );
    run(&mut w, 2 * MINUTE);
    assert!(!w.actor(z).unwrap().dead, "you stepped off, and it got up");
}

#[test]
fn burning_takes_oil_and_a_flame_and_a_burnt_body_never_gets_up() {
    let (mut w, me) = field();
    // You face south, and it lies south of you.
    let z = body(&mut w, TilePos::new(0, 1, 0));
    assert_eq!(w.can_burn(me), Err(NoBurn::NoFlame));
    // Drop the oil, and there's none to pour; take it back up.
    let flasks = slot(&w, me, OIL);
    w.intent(
        me,
        Intent::Drop {
            slot: flasks,
            def: OIL,
        },
    );
    run(&mut w, 5);
    light_up(&mut w, me);
    assert_eq!(w.can_burn(me), Err(NoBurn::NoOil));
    let at = TilePos::new(0, 0, 0);
    w.intent(
        me,
        Intent::Take {
            at,
            slot: 0,
            def: OIL,
        },
    );
    run(&mut w, 5);
    assert_eq!(w.can_burn(me), Ok(z));
    let before = oil(&w, me);
    w.intent(me, Intent::Burn);
    let seen = run(&mut w, 40);
    assert!(seen.contains(&Event::Kindled { who: me, body: z }));
    assert_eq!(oil(&w, me), before - 1, "a flask a body");
    assert!(matches!(
        w.actor(z).unwrap().remains,
        Remains::Burning { .. }
    ));
    assert_eq!(w.can_burn(me), Err(NoBurn::Nothing), "it's burning already");
    // It burns down to ash, and its hour comes and goes.
    let seen = run(&mut w, 2 * 60 * MINUTE);
    assert!(seen.contains(&Event::Burnt { body: z }));
    assert!(!seen.contains(&Event::GotUp { who: z }));
    let a = w.actor(z).unwrap();
    assert!(a.dead && a.remains == Remains::Ash);
}

#[test]
fn fire_goes_through_a_pile_of_bodies_but_not_through_a_wall() {
    let (mut w, me) = field();
    // A row of three south of you, then a wall, then one more behind it.
    let pile: Vec<ActorId> = (1..=3)
        .map(|y| body(&mut w, TilePos::new(0, y, 0)))
        .collect();
    w.grid.set_north(TilePos::new(0, 4, 0), Edge::Wall);
    let behind = body(&mut w, TilePos::new(0, 4, 0));
    light_up(&mut w, me);
    w.intent(me, Intent::Burn);
    run(&mut w, 40);
    let seen = run(&mut w, 5 * MINUTE);
    for &z in &pile[1..] {
        assert!(seen.contains(&Event::Caught { body: z }), "the pile caught");
    }
    assert!(!seen.contains(&Event::Caught { body: behind }));
    assert_eq!(oil(&w, me), 1, "one flask for the pile");
    // It gets up when its hour comes; the pile doesn't.
    let seen = run(&mut w, 60 * MINUTE);
    assert!(seen.contains(&Event::GotUp { who: behind }));
    assert!(pile.iter().all(|&z| w.actor(z).unwrap().dead));
}

#[test]
fn a_burning_body_lights_the_dark_round_it() {
    let (mut w, me) = field();
    w.rules.clock.start = 0; // midnight
    let z = body(&mut w, TilePos::new(0, 1, 0));
    let near = TilePos::new(2, 2, 0);
    let dark = w.light_at(near);
    light_up(&mut w, me);
    w.intent(me, Intent::Burn);
    run(&mut w, 40);
    // Walk off with the candle, and the body's fire still lights the place.
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(-12, 0, 0),
            run: false,
        },
    );
    run(&mut w, 120);
    assert!(matches!(
        w.actor(z).unwrap().remains,
        Remains::Burning { .. }
    ));
    assert!(w.light_at(near) > dark);
}

/// A region whose patch is 8 × 8 chunks and began with 10 of the dead.
fn region(pressure: Pressure) -> Region {
    Region {
        pressure,
        kinds: vec![(SHAMBLER, 10_000)],
        chunks: (8, 8),
        dead: 10,
    }
}

fn pressure() -> Pressure {
    Pressure {
        start: 1000,
        least: 250,
        most: 2000,
        per_day: 48,
        ceiling: 1100,
        per_rise: 20,
        per_burn: 30,
        every: 0,
    }
}

#[test]
fn the_pressure_rises_with_time_to_its_ceiling() {
    let (mut w, _) = field();
    w.set_region(region(pressure()));
    assert_eq!(w.pressure(), 1000);
    run(&mut w, 1440 * MINUTE);
    assert_eq!(w.pressure(), 1048, "48 a day, exactly");
    run(&mut w, 3 * 1440 * MINUTE);
    assert_eq!(w.pressure(), 1100, "and no higher than the ceiling");
}

#[test]
fn the_risen_raise_the_pressure_and_burning_lowers_it() {
    let (mut w, me) = field();
    let mut p = pressure();
    p.per_day = 0;
    w.set_region(region(p));
    body(&mut w, TilePos::new(5, 5, 0));
    run(&mut w, 60 * MINUTE);
    assert_eq!(w.pressure(), 1020, "one got up");
    body(&mut w, TilePos::new(0, 1, 0));
    light_up(&mut w, me);
    w.intent(me, Intent::Burn);
    run(&mut w, 40);
    assert_eq!(w.pressure(), 990, "one burned");
}

#[test]
fn short_of_the_dead_they_walk_in_where_you_arent() {
    let (mut w, me) = field();
    // The whole patch is grass.
    for y in 0..256 {
        for x in 0..256 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    let mut p = pressure();
    (p.per_day, p.every) = (0, 10);
    w.set_region(region(p));
    let mut seen = 0;
    for _ in 0..(12 * 60 * MINUTE) {
        w.step(1);
        seen = w.actors().filter(|(_, a)| !a.kind.is_character()).count();
    }
    assert_eq!(seen, 10, "as many as the pressure says, and no more");
    for (_, a) in w.actors().filter(|(_, a)| !a.kind.is_character()) {
        assert!(!w.is_awake(a.pos), "none walked in where you are");
    }
    assert!(w.actor(me).is_some());
    assert_eq!(w.dead_about(), w.dead_wanted());
}
