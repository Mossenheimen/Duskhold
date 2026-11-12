//! Sneaking: a slower, quieter step, no running, seen only from nearer; a sneak attack on one of
//! the dead that doesn't know you're there; and Stealth, learnt by both.

use dusk_core::{ActorId, ActorKind, Dir8, Event, Intent, Mind, Skill, Tile, TilePos, World};

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// Open grass by day, nobody wandering, and you at (0, 0) needing nothing.
fn field() -> (World, ActorId) {
    let mut w = World::new(7);
    w.rules.clock.start = 12 * 60;
    for y in -12..12 {
        for x in -12..20 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    w.creature_mut(0).wander = 0;
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
    (w, me)
}

/// Two seconds walking east: how far you got, and the loudest footstep.
fn walk(sneaking: bool, run: bool) -> (i32, i32) {
    let (mut w, me) = field();
    w.sneak(me, sneaking);
    w.intent(me, Intent::Move { dir: Dir8::E, run });
    let mut loudest = 0;
    for _ in 0..20 {
        w.step(1);
        for e in w.events() {
            if let Event::Sound {
                by: Some(b),
                loudness,
                ..
            } = e
                && b == me
            {
                loudest = loudest.max(loudness);
            }
        }
    }
    (w.actor(me).unwrap().pos.x, loudest)
}

#[test]
fn sneaking_is_slower_quieter_and_never_a_run() {
    let (far, loud) = walk(false, false);
    let (near, soft) = walk(true, false);
    assert!(near < far, "slower: {near} against {far}");
    assert!(soft < loud && soft >= 1, "quieter: {soft} against {loud}");
    assert_eq!(walk(true, true), (near, soft), "no running");
}

/// One of the dead six tiles south of you, facing you: whether it's after you a while later.
fn noticed(sneaking: bool) -> bool {
    let (mut w, me) = field();
    w.sneak(me, sneaking);
    let z = w.spawn(ActorKind::Creature(0), at(0, 6)).unwrap();
    // Spawned, it faces away (south): eyes all round, so only how far you are counts.
    w.creature_mut(0).cone = dusk_core::senses::Cone { cos: -10_000 };
    w.step(30);
    matches!(
        w.actor(z).unwrap().mind,
        Mind::Hunt {
            target: Some(_),
            ..
        }
    )
}

#[test]
fn sneaking_the_dead_see_you_only_from_nearer() {
    assert!(noticed(false), "standing, it sees you at six tiles");
    assert!(!noticed(true), "sneaking, it doesn't");
}

/// Your first blow, bare-handed for 10 points, on one of the dead at (0, 1) that faces south:
/// you behind it, or (`in_front`) it turned round to face you first. What it did, and whether it
/// was a sneak attack.
fn blow(in_front: bool) -> (i32, bool, u32) {
    let (mut w, me) = field();
    w.rules.combat.attack.hit = 10_000;
    w.rules.combat.attack.damage = (100, 100);
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    let c = w.creature_mut(0);
    (c.health, c.attack.hit) = (5000, 0);
    if in_front {
        // It sees all round: there's no behind it.
        c.cone = dusk_core::senses::Cone { cos: -10_000 };
    }
    let z = w.spawn(ActorKind::Creature(0), at(0, 1)).unwrap();
    w.intent(me, Intent::Attack { dir: Some(Dir8::S) });
    let mut sneak = false;
    for _ in 0..60 {
        w.step(1);
        let seen: Vec<Event> = w.events().collect();
        for e in seen {
            match e {
                Event::SneakAttack { target, .. } if target == z => sneak = true,
                Event::Struck { who, damage, .. } if who == me => {
                    let stealth = w.actor(me).unwrap().skills.xp[Skill::Stealth as usize];
                    return (damage, sneak, stealth);
                }
                _ => {}
            }
        }
    }
    panic!("no blow landed");
}

#[test]
fn a_blow_from_behind_on_one_that_doesnt_know_youre_there_is_a_sneak_attack() {
    let (damage, sneak, stealth) = blow(false);
    assert_eq!((damage, sneak), (300, true), "three times the harm");
    assert!(stealth > 0, "Stealth learnt from it");
    let (damage, sneak, stealth) = blow(true);
    assert_eq!(
        (damage, sneak, stealth),
        (100, false, 0),
        "seen, a blow is a blow"
    );
}

#[test]
fn sneaking_past_the_dead_unseen_teaches_stealth() {
    let (mut w, me) = field();
    // One of the dead far down the field, facing away.
    w.spawn(ActorKind::Creature(0), at(4, 6)).unwrap();
    w.sneak(me, true);
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    w.step(60);
    assert!(w.actor(me).unwrap().skills.xp[Skill::Stealth as usize] > 0);
    // Not sneaking, the same walk teaches nothing.
    let (mut w, me) = field();
    w.spawn(ActorKind::Creature(0), at(4, 6)).unwrap();
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    w.step(60);
    assert_eq!(w.actor(me).unwrap().skills.xp[Skill::Stealth as usize], 0);
}
