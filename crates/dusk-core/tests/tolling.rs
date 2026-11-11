//! The Tolling: a calendar of nights, a bell the dead walk to by one shared field, doors beaten
//! down on the way, and a night they don't give up on you.

use dusk_core::flow::{Flow, NOWHERE};
use dusk_core::{ActorId, ActorKind, Alert, Edge, Event, Region, Tile, TilePos, Tolling, World};

const SHAMBLER: ActorKind = ActorKind::Creature(0);
const BELL: TilePos = TilePos::new(0, 0, 0);

/// The first night day 2 at 22:00, then every 3 days; it rings an hour, a toll every 2 minutes,
/// and draws the dead within 30 tiles.
fn tolling() -> Tolling {
    Tolling {
        first: 2,
        every: 3,
        start: 22 * 60,
        rings: 60,
        toll_every: 2,
        loudness: 40,
        reach: 30,
        door: 5000,
    }
}

/// A field of grass 80 tiles across round the bell, the clock at midnight on day 1, the needs
/// stilled, and you far off at (35, 0), out of the bell's reach.
fn field() -> (World, ActorId) {
    let mut w = World::new(4);
    w.creature_mut(0).wander = 0;
    w.rules.clock.start = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -40..40 {
        for x in -40..40 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    w.set_region(Region {
        tolling: tolling(),
        bell: Some(BELL),
        ..Region::default()
    });
    let me = w
        .spawn(ActorKind::Character, TilePos::new(35, 0, 0))
        .unwrap();
    (w, me)
}

/// Steps the world to a game minute on the clock (day 1 starts at 0), keeping its events.
fn run_to(w: &mut World, minute: u64) -> Vec<Event> {
    let mut seen = Vec::new();
    while w.clock_minutes() < minute {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

fn at(day: u64, hour: u64) -> u64 {
    (day - 1) * 1440 + hour * 60
}

#[test]
fn it_comes_on_its_nights_from_the_hour_until_dawn() {
    let (mut w, _) = field();
    assert_eq!(w.next_tolling(), Some(at(2, 22)));
    let seen = run_to(&mut w, at(2, 6));
    assert!(seen.contains(&Event::TollingTonight), "word of it at dawn");
    run_to(&mut w, at(2, 21));
    assert!(!w.is_tolling());
    let seen = run_to(&mut w, at(2, 23));
    assert!(w.is_tolling());
    assert!(seen.contains(&Event::TollingBegan { at: BELL }));
    // A toll every 2 minutes for its hour: 30.
    let tolls = seen
        .iter()
        .filter(|e| **e == Event::Tolled { at: BELL })
        .count();
    assert_eq!(tolls, 30);
    run_to(&mut w, at(3, 4));
    assert!(w.is_tolling(), "it lasts the night");
    let seen = run_to(&mut w, at(3, 6));
    assert!(!w.is_tolling());
    assert!(seen.contains(&Event::TollingEnded));
    assert_eq!(w.next_tolling(), Some(at(5, 22)), "three days on");
}

#[test]
fn the_dead_in_reach_walk_to_the_bell_round_a_wall_and_those_beyond_stay() {
    let (mut w, _) = field();
    // A wall from (-10, -8) to (10, -8), between the bell and one of the dead.
    for x in -10..=10 {
        w.grid.set_north(TilePos::new(x, -8, 0), Edge::Wall);
    }
    let near = w.spawn(SHAMBLER, TilePos::new(0, -20, 0)).unwrap();
    let far = w.spawn(SHAMBLER, TilePos::new(0, 36, 0)).unwrap();
    run_to(&mut w, at(2, 22) + 1);
    run_to(&mut w, at(2, 23) + 30);
    let pos = w.actor(near).unwrap().pos;
    assert!(
        pos.steps_to(BELL) <= 2,
        "it came round the wall to the bell: {pos:?}"
    );
    assert_eq!(
        w.actor(far).unwrap().pos,
        TilePos::new(0, 36, 0),
        "out of reach"
    );
}

#[test]
fn a_shut_door_on_the_way_gets_beaten_down() {
    let (mut w, _) = field();
    w.rules.combat.door_strength = 60;
    // A box round the bell, walls on every side, a shut door on its south side.
    for k in -3..=3 {
        w.grid.set_north(TilePos::new(k, -3, 0), Edge::Wall);
        w.grid.set_north(TilePos::new(k, 4, 0), Edge::Wall);
        w.grid.set_west(TilePos::new(-3, k, 0), Edge::Wall);
        w.grid.set_west(TilePos::new(4, k, 0), Edge::Wall);
    }
    w.grid.set_north(TilePos::new(0, 4, 0), Edge::door(false));
    let z = w.spawn(SHAMBLER, TilePos::new(0, 12, 0)).unwrap();
    run_to(&mut w, at(2, 22) + 1);
    let seen = run_to(&mut w, at(3, 1));
    assert!(seen.iter().any(|e| matches!(e, Event::DoorBroke { .. })));
    assert!(
        w.actor(z).unwrap().pos.steps_to(BELL) <= 3,
        "and it went in"
    );
}

#[test]
fn on_a_tolling_night_the_dead_dont_give_up_on_you() {
    // A noise behind a long wall with a shut door in it; one of the dead hears it and comes.
    // On an ordinary night it gives up after its 30 seconds; on a Tolling night it's still at
    // the door an hour on.
    let after_an_hour = |night: bool| {
        let (mut w, _) = field();
        w.rules.combat.door_strength = 100_000;
        for y in -40..40 {
            let edge = if y == 0 {
                Edge::door(false)
            } else {
                Edge::Wall
            };
            w.grid.set_west(TilePos::new(30, y, 0), edge);
        }
        // Within the bell's reach, and blind, so only the noise tells it anything.
        w.creature_mut(0).sight = 0;
        let z = w.spawn(SHAMBLER, TilePos::new(25, 0, 0)).unwrap();
        let when = if night { at(2, 22) + 5 } else { at(2, 12) };
        run_to(&mut w, when);
        w.make_sound(TilePos::new(31, 0, 0), 40, None);
        run_to(&mut w, when + 60);
        w.actor(z).unwrap().alert()
    };
    assert_eq!(after_an_hour(false), Alert::Unaware);
    assert_eq!(after_an_hour(true), Alert::Curious, "still at the door");
}

#[test]
fn the_field_runs_downhill_to_the_goal_and_nowhere_from_a_walled_off_place() {
    let (mut w, _) = field();
    // A closed room of walls at (20..23, 20..23).
    for k in 20..23 {
        w.grid.set_north(TilePos::new(k, 20, 0), Edge::Wall);
        w.grid.set_north(TilePos::new(k, 23, 0), Edge::Wall);
        w.grid.set_west(TilePos::new(20, k, 0), Edge::Wall);
        w.grid.set_west(TilePos::new(23, k, 0), Edge::Wall);
    }
    let flow = Flow::build(&w.grid, BELL, 30, 5000);
    assert_eq!(flow.cost_at(BELL), 0);
    assert_eq!(flow.cost_at(TilePos::new(3, 0, 0)), 3000);
    assert_eq!(flow.cost_at(TilePos::new(2, 2, 0)), 2828);
    assert_eq!(flow.cost_at(TilePos::new(21, 21, 0)), NOWHERE, "walled in");
    assert_eq!(
        flow.cost_at(TilePos::new(31, 0, 0)),
        NOWHERE,
        "out of the field"
    );
}
