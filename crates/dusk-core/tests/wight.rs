//! The wight's ways: only after dark (by day it lies still), its nearness frightening the living
//! more than any of the dead, and the light weakening it.

use dusk_core::{ActorId, ActorKind, Dir8, Event, Intent, Mind, Tile, TilePos, World};

/// Ticks in a game minute.
const MINUTE: u32 = 25;

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// Open grass from `hour` o'clock; one of the dead to the north facing south at you, three tiles
/// off, of a kind that's `nocturnal`, adds `dread` of its own and takes `lit_harm` of a blow in
/// the light. It sees `darkvision` tiles in any light. You need nothing.
fn field(hour: u32, nocturnal: bool, dread: i32, darkvision: i32) -> (World, ActorId, ActorId) {
    let mut w = World::new(6);
    w.rules.clock.start = hour * 60;
    for y in -10..10 {
        for x in -10..10 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    let c = w.creature_mut(0);
    (c.wander, c.nocturnal, c.dread, c.darkvision) = (0, nocturnal, dread, darkvision);
    c.lit_harm = 2000;
    let z = w.spawn(ActorKind::Creature(0), at(0, 0)).unwrap();
    let me = w.spawn(ActorKind::Character, at(0, 3)).unwrap();
    (w, me, z)
}

fn after(w: &World, z: ActorId) -> Option<TilePos> {
    match w.actor(z).unwrap().mind {
        Mind::Hunt { target, .. } => target,
        Mind::None | Mind::Flee { .. } => None,
    }
}

#[test]
fn by_day_it_lies_still_and_after_dark_it_hunts() {
    // By day it would see you: it doesn't stir.
    let (mut w, _, z) = field(12, true, 0, 6);
    w.step(4 * MINUTE);
    assert_eq!((w.actor(z).unwrap().pos, after(&w, z)), (at(0, 0), None));
    // The same, of a kind that walks by day: it's after you.
    let (mut w, _, z) = field(12, false, 0, 6);
    w.step(4 * MINUTE);
    assert!(after(&w, z).is_some());
    // At night it hunts, seeing you in the dark.
    let (mut w, me, z) = field(22, true, 0, 6);
    w.step(4 * MINUTE);
    assert!(after(&w, z).is_some(), "after you in the night");
    assert!(w.actor(z).unwrap().pos.dist2(w.actor(me).unwrap().pos) < 9);
}

#[test]
fn its_nearness_frightens_more_than_any_of_the_deads() {
    // At night, unseeing (no darkvision, and you're in the dark): it only stands there.
    let (mut w, me, _) = field(22, true, 25_000, 0);
    w.step(20 * MINUTE);
    assert_eq!(
        w.dread(me),
        75_000,
        "the dark, one of the dead, and its own"
    );
    assert_eq!(w.dread_level(me), 3, "frightened");
    let (mut w, me, _) = field(22, true, 0, 0);
    w.step(20 * MINUTE);
    assert_eq!(w.dread(me), 50_000, "one of the dead, no more");
}

/// Your first blow on it, from beside it, bare-handed for 10 points that always land.
fn first_blow(hour: u32) -> i32 {
    let mut w = World::new(6);
    w.rules.clock.start = hour * 60;
    for y in -3..3 {
        w.grid.set_tile(at(0, y), Tile::GRASS);
    }
    w.rules.combat.attack.hit = 10_000;
    w.rules.combat.attack.damage = (100, 100);
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    let c = w.creature_mut(0);
    (c.wander, c.lit_harm, c.health) = (0, 2000, 5000);
    c.attack.hit = 0;
    w.spawn(ActorKind::Creature(0), at(0, 0)).unwrap();
    let me = w.spawn(ActorKind::Character, at(0, 1)).unwrap();
    w.intent(me, Intent::Attack { dir: Some(Dir8::N) });
    for _ in 0..60 {
        w.step(1);
        for e in w.events() {
            if let Event::Struck { who, damage, .. } = e
                && who == me
            {
                return damage;
            }
        }
    }
    panic!("no blow landed");
}

#[test]
fn in_the_light_it_takes_twice_the_harm() {
    assert_eq!(first_blow(12), 200, "by day");
    assert_eq!(first_blow(23), 100, "in the dark");
}
