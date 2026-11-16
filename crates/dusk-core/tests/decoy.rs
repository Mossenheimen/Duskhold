//! Decoys (`decoy.rs`): wound and set down, one rattles on its beat from when it was set to start
//! till it runs down, and lies there after; the dead that hear it go to it; taken up, it stops; and
//! it lasts a save.

use std::sync::Arc;

use dusk_core::senses::Cone;
use dusk_core::{
    ActorId, ActorKind, Decoy, Defs, Event, Intent, Item, ItemDef, ItemKind, Tile, TilePos, World,
};

const CLATTERBOX: u16 = 0;
const HERE: TilePos = TilePos::new(0, 0, 0);

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A clatterbox: a second to wind, then a second before it starts, rattling every half second
/// for three, heard 12 tiles off.
fn defs() -> Defs {
    Defs {
        items: vec![ItemDef {
            id: "clatterbox".into(),
            name: "clatterbox".into(),
            weight: 600,
            stack: 3,
            kind: ItemKind::Decoy(Decoy {
                wind: 10,
                delay: 10,
                every: 5,
                lasts: 30,
                noise: 12,
            }),
            worth: 0,
        }],
        carry: 30_000,
        ..Defs::default()
    }
}

/// A strip of grass, you at its middle with a clatterbox, and one of the dead 10 tiles east (in
/// earshot of it) that can't see (it goes by its ears alone) and doesn't wander. The needs stilled.
fn field() -> (World, ActorId, ActorId) {
    let mut w = World::new(5);
    w.defs = Arc::new(defs());
    {
        let c = w.creature_mut(0);
        (c.wander, c.sight) = (0, 0);
        c.cone = Cone { cos: -10_000 };
    }
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -3..=3 {
        for x in -12..=20 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    let me = w.spawn(ActorKind::Character, HERE).unwrap();
    assert!(w.give(me, Item::new(CLATTERBOX, 1)));
    let z = w.spawn(ActorKind::Creature(0), at(10, 0)).unwrap();
    (w, me, z)
}

/// Winds the clatterbox and sets it down: the tick it was set, and what happened.
fn set(w: &mut World, me: ActorId) -> (u64, Vec<Event>) {
    w.intent(
        me,
        Intent::Use {
            slot: 0,
            def: CLATTERBOX,
        },
    );
    let mut seen = Vec::new();
    for _ in 0..40 {
        w.step(1);
        let now: Vec<Event> = w.events().collect();
        let done = now.iter().any(|e| matches!(e, Event::DecoySet { .. }));
        seen.extend(now);
        if done {
            return (w.tick, seen);
        }
    }
    panic!("it was never set: {seen:?}");
}

/// The ticks the decoy on `HERE` rattled on, over the next `ticks`.
fn rattles(w: &mut World, ticks: u32) -> Vec<u64> {
    let mut out = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        let rattled = w
            .events()
            .any(|e| matches!(e, Event::Sound { at: p, by: None, loudness: 12 } if p == HERE));
        if rattled {
            out.push(w.tick);
        }
    }
    out
}

#[test]
fn wound_and_set_down_it_rattles_on_its_beat_till_it_runs_down() {
    let (mut w, me, _) = field();
    let (set_on, seen) = set(&mut w, me);
    assert!(seen.contains(&Event::DecoySet {
        who: me,
        at: HERE,
        delay: 10
    }));
    assert!(w.actor(me).unwrap().items.is_empty(), "out of your pack");
    assert_eq!(w.store(HERE).len(), 1, "on the floor where you stood");
    assert_eq!(w.decoys().count(), 1);
    // A second on, every half second for three: six rattles.
    let beat: Vec<u64> = (0..6).map(|k| set_on + 10 + 5 * k).collect();
    assert_eq!(rattles(&mut w, 60), beat);
    assert_eq!(w.decoys().count(), 0, "run down");
    assert_eq!(
        w.store(HERE).len(),
        1,
        "and it lies there, to be wound again"
    );
}

#[test]
fn the_dead_that_hear_it_go_to_it() {
    let (mut w, me, z) = field();
    set(&mut w, me);
    // You go off west, quietly; it rattles where you were.
    w.intent(
        me,
        Intent::WalkTo {
            to: at(-10, 0),
            run: false,
        },
    );
    let mut heard = false;
    for _ in 0..600 {
        w.step(1);
        heard |= w.events().any(|e| e == Event::Heard { who: z, at: HERE });
    }
    assert!(heard, "it heard the decoy");
    let pos = w.actor(z).unwrap().pos;
    assert!(pos.dist2(HERE) <= 2, "and went to it: {pos:?}");
}

#[test]
fn taken_up_before_it_starts_it_never_rattles() {
    let (mut w, me, _) = field();
    set(&mut w, me);
    w.intent(
        me,
        Intent::Take {
            at: HERE,
            slot: 0,
            def: CLATTERBOX,
        },
    );
    assert!(rattles(&mut w, 60).is_empty());
    assert_eq!(w.decoys().count(), 0);
    assert_eq!(w.actor(me).unwrap().items.len(), 1, "back in your pack");
}

#[test]
fn a_decoy_set_down_lasts_a_save() {
    let (mut a, me, _) = field();
    set(&mut a, me);
    let bytes = a.save("decoys", b"");
    let (mut b, _, _) = field();
    b.restore(&bytes).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(b.state_hash(), a.state_hash());
    assert_eq!(
        rattles(&mut b, 60),
        rattles(&mut a, 60),
        "it rattles on alike"
    );
    assert_eq!(b.state_hash(), a.state_hash());
}
