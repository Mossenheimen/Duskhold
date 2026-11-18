//! Going after one of the dead (`Intent::Pursue`, Diablo's click on a monster; the second
//! playtest): walked after wherever it goes, and one swing once it's in reach.

use dusk_core::{ActorId, Event, Intent, TilePos, World};

/// A corridor running east: you at its west end, one of the dead `gap` tiles on that doesn't
/// wander and never lands a blow (so none breaks your swing), and no grabs.
fn corridor(gap: usize) -> (World, ActorId, ActorId) {
    let mut row = String::from("|@");
    for x in 1..12 {
        row.push_str(if x == gap { " z" } else { " ." });
    }
    row.push('|');
    let rule = format!("+{}", "-+".repeat(12));
    let (mut w, ids) = World::from_text(1, &format!("{rule}\n{row}\n{rule}\n")).unwrap();
    let z = w.creature_mut(0);
    z.wander = 0;
    z.attack.hit = 0;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    let me = w.character().unwrap();
    let foe = ids.into_iter().find(|&id| id != me).unwrap();
    (w, me, foe)
}

fn pursue(w: &mut World, me: ActorId, target: ActorId) {
    let at = w.actor(target).unwrap().pos;
    w.intent(
        me,
        Intent::Pursue {
            target,
            at,
            heavy: false,
            run: false,
        },
    );
}

/// Your swings in the next `ticks`, by where each went.
fn swings(w: &mut World, me: ActorId, ticks: u32) -> Vec<TilePos> {
    let mut at = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        let events: Vec<Event> = w.events().collect();
        for e in events {
            match e {
                Event::Missed { who, at: there } if who == me => at.push(there),
                Event::Struck { who, target, .. } if who == me => {
                    at.push(w.actor(target).unwrap().pos)
                }
                _ => {}
            }
        }
    }
    at
}

#[test]
fn one_of_the_dead_further_off_is_walked_to_and_swung_at_once() {
    let (mut w, me, foe) = corridor(4);
    pursue(&mut w, me, foe);
    // It may come a step to meet you (it hears you walk): the swing goes where it is.
    let swung = swings(&mut w, me, 80);
    let (mine, its) = (w.actor(me).unwrap().pos, w.actor(foe).unwrap().pos);
    assert_eq!(swung, vec![its], "one swing, at it");
    assert_eq!(mine.steps_to(its), 1, "beside it");
    assert_eq!(w.actor(me).unwrap().intent, Intent::Idle);
}

#[test]
fn one_that_walks_off_is_followed() {
    let (mut w, me, foe) = corridor(3);
    // A noise at the far end draws it away east, its back to you.
    w.make_sound(TilePos::new(11, 0, 0), 20, None);
    w.step(5);
    pursue(&mut w, me, foe);
    let swung = swings(&mut w, me, 200);
    assert_eq!(swung.len(), 1, "one swing: {swung:?}");
    assert!(
        swung[0].x > 3,
        "it was caught up with where it went: {swung:?}"
    );
}

#[test]
fn one_thats_dead_is_let_go() {
    let (mut w, me, foe) = corridor(1);
    // Your fists put it down at a blow.
    w.rules.combat.attack.damage = (10_000, 10_000);
    w.rules.combat.attack.hit = 10_000;
    pursue(&mut w, me, foe);
    swings(&mut w, me, 20);
    assert!(w.actor(foe).unwrap().dead);
    pursue(&mut w, me, foe);
    assert!(swings(&mut w, me, 20).is_empty(), "no swing at the dead");
    assert_eq!(w.actor(me).unwrap().intent, Intent::Idle);
}
