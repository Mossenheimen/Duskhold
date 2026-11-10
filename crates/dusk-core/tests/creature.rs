//! Creatures as data: a second kind is a second row in the definitions, with no rule of its own.

use std::sync::Arc;

use dusk_core::{ActorKind, CreatureDef, Defs, Event, Tile, TilePos, World};

const SHAMBLER: ActorKind = ActorKind::Creature(0);
const RUNNER: ActorKind = ActorKind::Creature(1);

/// The test shambler, and a runner: three times as fast, half as tough, and it sees twice as far.
fn defs() -> Defs {
    let mut runner = CreatureDef::shambler();
    runner.id = "runner".into();
    runner.name = "runner".into();
    runner.stats.speed.run *= 3;
    runner.stats.health /= 2;
    runner.stats.sight *= 2;
    let mut defs = Defs::default();
    defs.creatures.push(runner);
    defs
}

/// A strip of grass 40 tiles long, you at its south end.
fn strip() -> (World, TilePos) {
    let mut w = World::new(4);
    w.defs = Arc::new(defs());
    for n in 0..2 {
        w.creature_mut(n).wander = 0;
    }
    for y in 0..40 {
        w.grid.set_tile(TilePos::new(0, y, 0), Tile::GRASS);
    }
    let you = TilePos::new(0, 30, 0);
    w.spawn(ActorKind::Character, you).unwrap();
    (w, you)
}

#[test]
fn each_kind_has_its_own_health_and_names() {
    let (mut w, _) = strip();
    let slow = w.spawn(SHAMBLER, TilePos::new(0, 0, 0)).unwrap();
    let fast = w.spawn(RUNNER, TilePos::new(0, 1, 0)).unwrap();
    assert_eq!(w.actor(slow).unwrap().body.health, 300);
    assert_eq!(w.actor(fast).unwrap().body.health, 150);
    assert_eq!(w.defs.creature_kind("runner"), Some(RUNNER));
    assert_eq!(w.defs.creature_kind("wight"), None);
    assert!(
        w.spawn(ActorKind::Creature(7), TilePos::new(0, 2, 0))
            .is_none(),
        "no such creature in the definitions"
    );
}

#[test]
fn each_kind_senses_and_moves_by_its_own_numbers() {
    // 14 tiles off: past the shambler's 8 tiles of sight, inside the runner's 16.
    let seen_and_reached = |kind: ActorKind| {
        let (mut w, you) = strip();
        let z = w.spawn(kind, TilePos::new(0, you.y - 14, 0)).unwrap();
        let (mut saw, mut reached) = (false, None);
        for tick in 1..=200 {
            w.step(1);
            for e in w.events() {
                match e {
                    Event::Saw { who, .. } if who == z => saw = true,
                    Event::Reached { who, .. } if who == z => reached = reached.or(Some(tick)),
                    _ => {}
                }
            }
        }
        (saw, reached)
    };
    assert_eq!(
        seen_and_reached(SHAMBLER),
        (false, None),
        "too far for it to see"
    );
    let (saw, reached) = seen_and_reached(RUNNER);
    assert!(saw, "the runner sees you from 14 tiles");
    // 13 tiles at 2.4 tiles a second: under 6 seconds. (A shambler's 0.8 would take 16.)
    assert!(
        reached.is_some_and(|t| t < 70),
        "and is on you fast: {reached:?}"
    );
}
