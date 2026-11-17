//! On guard, Zomboid's combat stance (`guard.rs`): you face where you're turned, not where you
//! walk, and step slower, never running; a swing goes where you face.

use dusk_core::{ActorId, Dir8, Event, Intent, Moodle, TilePos, World};

/// A field four tiles across, you at its west end with one of the dead to your east, beside you,
/// that doesn't wander, and no grabs (held, nobody swings).
fn field() -> (World, ActorId) {
    let (mut w, ids) = World::from_text(
        1,
        "\
+-+-+-+-+
|. . . .|
+ + + + +
|@ z . .|
+ + + + +
|. . . .|
+-+-+-+-+
",
    )
    .unwrap();
    w.creature_mut(0).wander = 0;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    (w, ids[0])
}

fn facing(w: &World, me: ActorId) -> Dir8 {
    w.actor(me).unwrap().facing
}

#[test]
fn on_guard_you_face_where_youre_turned_and_step_slower_never_running() {
    let (mut w, me) = field();
    w.on_guard(me, true);
    assert!(w.is_guarding(me));
    assert!(w.moodles(me).iter().any(|&(m, _)| m == Moodle::Guarding));
    w.face(me, Dir8::E);
    // Stepping aside, south, it keeps its eyes east.
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::S,
            run: true,
        },
    );
    w.step(1);
    let a = w.actor(me).unwrap();
    assert_eq!(a.facing, Dir8::E, "it faces where it was turned");
    // A walk's 150 thousandths of a tile a tick, 70% of it; and asked to run, it walks.
    assert_eq!(a.offset, (0, 105));
    // Off guard, its step turns it again, and turning it does nothing.
    w.on_guard(me, false);
    w.face(me, Dir8::N);
    w.step(1);
    assert_eq!(facing(&w, me), Dir8::S);
}

#[test]
fn a_swing_on_guard_goes_where_you_face_not_at_the_nearest() {
    // Off guard, a swing finds the one beside you, though you face north.
    let (mut w, me) = field();
    w.on_guard(me, true);
    w.face(me, Dir8::N);
    w.on_guard(me, false);
    w.intent(me, Intent::Attack { dir: None });
    let mut at = None;
    for _ in 0..40 {
        w.step(1);
        at = at.or(w.events().find_map(|e| match e {
            Event::Struck { who, .. } if who == me => Some(TilePos::new(1, 1, 0)),
            Event::Missed { who, at } if who == me => Some(at),
            _ => None,
        }));
    }
    assert_eq!(at, Some(TilePos::new(1, 1, 0)), "at the one beside you");
    // On guard, facing north, the swing goes north, past it.
    let (mut w, me) = field();
    w.on_guard(me, true);
    w.face(me, Dir8::N);
    w.intent(me, Intent::Attack { dir: None });
    let mut missed = None;
    for _ in 0..40 {
        w.step(1);
        missed = missed.or(w.events().find_map(|e| match e {
            Event::Missed { who, at } if who == me => Some(at),
            _ => None,
        }));
    }
    assert_eq!(missed, Some(TilePos::new(0, 0, 0)), "north, where it faces");
}

#[test]
fn one_of_the_dead_that_sees_you_near_puts_you_in_a_fight() {
    // It stands north of you, facing south (as everyone starts): it sees you.
    let (mut w, _) = World::from_text(
        1,
        "+-+-+
|z .|
+ + +
|@ .|
+-+-+
",
    )
    .unwrap();
    w.creature_mut(0).wander = 0;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    let me = w.character().unwrap();
    // It hasn't thought yet: nothing sees you.
    assert!(!w.in_combat(me));
    for _ in 0..20 {
        w.step(1);
    }
    assert!(w.in_combat(me), "it sees you");
}
