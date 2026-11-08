//! Grabs with a way out, and doors the dead beat down.

use dusk_core::world::WoundRule;
use dusk_core::{ActorId, ActorKind, Dir8, Edge, Event, Intent, TilePos, World};

/// A corridor running south: a shambler facing you from the north, room to back away.
const CORRIDOR: &str = "\
+-+
|z|
+ +
|@|
+ +
|.|
+ +
|.|
+ +
|.|
+-+
";

fn shambler(w: &World) -> ActorId {
    w.actors()
        .find(|(_, a)| a.kind == ActorKind::Shambler)
        .map(|(id, _)| id)
        .expect("the map has a shambler")
}

/// Steps until an event turns up; returns it, or panics after `ticks`.
fn until(w: &mut World, ticks: u32, what: &str, mut want: impl FnMut(&Event) -> bool) -> Event {
    for _ in 0..ticks {
        w.step(1);
        if let Some(e) = w.events().find(|e| want(e)) {
            return e;
        }
    }
    panic!("no {what} in {ticks} ticks");
}

/// The corridor, its shambler sure to take hold with its first hit, and harmless blows.
fn held_in_corridor() -> (World, ActorId, ActorId) {
    let (mut w, _) = World::from_text(1, CORRIDOR).unwrap();
    let c = &mut w.rules.combat;
    w.rules.wander_chance = 0;
    c.grab = 10_000;
    c.shambler_attack.hit = 10_000;
    c.parting_grab = 0;
    c.wounds = [WoundRule {
        damage: 0,
        bleed: 0,
    }; 3];
    let (me, z) = (w.character().unwrap(), shambler(&w));
    until(&mut w, 100, "grab", |e| {
        *e == Event::Grabbed { who: z, target: me }
    });
    (w, me, z)
}

#[test]
fn held_you_cant_walk_until_you_struggle_free() {
    let (mut w, me, z) = held_in_corridor();
    assert_eq!(w.holders(me), vec![z]);
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::S,
            run: false,
        },
    );
    w.step(30);
    assert_eq!(w.actor(me).unwrap().pos, TilePos::new(0, 1, 0), "held fast");
    w.intent(me, Intent::Attack { dir: None });
    w.step(3);
    assert!(
        w.actor(me).unwrap().action.is_none(),
        "no swinging while held"
    );

    w.rules.combat.struggle.hit = 10_000;
    w.rules.combat.grab = 0;
    w.intent(me, Intent::Struggle);
    until(&mut w, 20, "struggling free", |e| {
        *e == Event::Freed { who: me, from: z }
    });
    assert!(w.holders(me).is_empty());
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::S,
            run: true,
        },
    );
    w.step(15);
    assert!(w.actor(me).unwrap().pos.y >= 2, "and away");
}

#[test]
fn a_shove_breaks_a_hold() {
    let (mut w, me, z) = held_in_corridor();
    w.rules.combat.grab = 0;
    w.intent(me, Intent::Shove { dir: None });
    until(&mut w, 10, "a shove", |e| {
        *e == Event::Freed { who: me, from: z }
    });
    assert!(w.holders(me).is_empty());
    assert_eq!(w.actor(z).unwrap().holding, None);
}

#[test]
fn stepping_away_from_one_on_you_risks_a_grab() {
    let (mut w, _) = World::from_text(1, CORRIDOR).unwrap();
    w.rules.wander_chance = 0;
    let c = &mut w.rules.combat;
    (c.grab, c.shambler_attack.hit, c.parting_grab) = (0, 0, 10_000);
    let (me, z) = (w.character().unwrap(), shambler(&w));
    w.step(10); // it looks, and sees you
    assert_eq!(w.engaged(me), vec![z]);
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::S,
            run: false,
        },
    );
    let e = until(&mut w, 5, "a parting grab", |e| {
        matches!(e, Event::Grabbed { .. })
    });
    assert_eq!(e, Event::Grabbed { who: z, target: me });
    assert_eq!(
        w.actor(me).unwrap().pos,
        TilePos::new(0, 1, 0),
        "caught as you went"
    );
}

#[test]
fn three_holding_you_drag_you_down() {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+
|. z .|
+     +
|z @ z|
+     +
|. . .|
+-+-+-+
",
    )
    .unwrap();
    w.rules.wander_chance = 0;
    let c = &mut w.rules.combat;
    (c.grab, c.shambler_attack.hit, c.parting_grab) = (10_000, 10_000, 0);
    c.wounds = [WoundRule {
        damage: 0,
        bleed: 0,
    }; 3];
    let me = w.character().unwrap();
    w.make_sound(TilePos::new(1, 1, 0), 5, Some(me)); // they turn to you
    let e = until(&mut w, 400, "the drag-down", |e| {
        matches!(e, Event::DraggedDown { .. })
    });
    assert_eq!(
        e,
        Event::DraggedDown {
            who: me,
            holders: 3
        }
    );
    assert!(w.actor(me).unwrap().dead);
}

#[test]
fn the_dead_beat_down_a_shut_door() {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+-+-+
|@ . .D. . . z|
+-+-+-+-+-+-+-+
",
    )
    .unwrap();
    w.rules.wander_chance = 0;
    w.rules.combat.door_strength = 100;
    let (me, z) = (w.character().unwrap(), shambler(&w));
    w.make_sound(TilePos::new(0, 0, 0), 12, Some(me));
    until(&mut w, 300, "a blow on the door", |e| {
        matches!(e, Event::Thumped { .. })
    });
    let broke = until(&mut w, 300, "the door giving way", |e| {
        matches!(e, Event::DoorBroke { .. })
    });
    assert_eq!(
        broke,
        Event::DoorBroke {
            at: TilePos::new(3, 0, 0),
            side: Dir8::W
        }
    );
    assert_eq!(
        w.grid.edge(TilePos::new(3, 0, 0), Dir8::W),
        Edge::Open,
        "gone for good"
    );
    until(&mut w, 200, "it coming through", |e| {
        *e == Event::Reached { who: z, target: me }
    });
}
