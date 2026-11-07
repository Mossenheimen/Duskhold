//! Fight or flee: swings with a tell, shoves, doors, endurance, wounds and death.

use dusk_core::{ActorId, ActorKind, Dir8, Edge, Event, Intent, Tile, TilePos, World};

fn shambler(w: &World) -> ActorId {
    w.actors()
        .find(|(_, a)| a.kind == ActorKind::Shambler)
        .map(|(id, _)| id)
        .expect("the map has a shambler")
}

/// One tick; the character swings again whenever it's free to.
fn swing_away(w: &mut World, me: ActorId) -> Vec<Event> {
    let a = w.actor(me).unwrap();
    if a.action.is_none() && a.intent == Intent::Idle {
        w.intent(me, Intent::Attack { dir: None });
    }
    w.step(1);
    w.events().collect()
}

#[test]
fn you_can_kill_a_shambler_and_it_stays_as_a_corpse() {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+
|@ z .|
+-+-+-+
",
    )
    .unwrap();
    w.rules.wander_chance = 0;
    let (me, z) = (w.character().unwrap(), shambler(&w));
    let mut died = false;
    for _ in 0..400 {
        died |= swing_away(&mut w, me).contains(&Event::Died { who: z });
        if died {
            break;
        }
    }
    assert!(died, "it never went down");
    let corpse = w.actor(z).unwrap();
    assert!(
        corpse.dead && corpse.pos == TilePos::new(1, 0, 0),
        "it lies where it fell"
    );
    assert!(!w.actor(me).unwrap().dead);
    // Nobody has to walk round a corpse.
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(2, 0, 0),
            run: false,
        },
    );
    w.step(40);
    assert_eq!(w.actor(me).unwrap().pos, TilePos::new(2, 0, 0));
}

#[test]
fn a_shamblers_blow_comes_a_wind_up_after_its_tell() {
    // It faces south, at you.
    let (mut w, _) = World::from_text(
        1,
        "\
+-+
|z|
+ +
|@|
+-+
",
    )
    .unwrap();
    w.rules.wander_chance = 0;
    let z = shambler(&w);
    let wind_up = w.rules.combat.shambler_attack.wind_up;
    let mut tell_from = None;
    for _ in 0..100 {
        w.step(1);
        let a = w.actor(z).unwrap();
        if tell_from.is_none() && a.action.is_some_and(|act| act.winding_up()) {
            tell_from = Some(w.tick);
        }
        let blow = w.events().any(
            |e| matches!(e, Event::Struck { who, .. } | Event::Missed { who, .. } if who == z),
        );
        if blow {
            let from = tell_from.expect("a blow with no tell");
            assert_eq!(
                w.tick - from + 1,
                u64::from(wind_up),
                "the tell lasts the wind-up"
            );
            return;
        }
    }
    panic!("it never swung");
}

#[test]
fn a_shove_breaks_off_a_wind_up_and_pushes_back() {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+
|.|
+ +
|z|
+ +
|@|
+-+
",
    )
    .unwrap();
    w.rules.wander_chance = 0;
    let (me, z) = (w.character().unwrap(), shambler(&w));
    for _ in 0..50 {
        w.step(1);
        if w.actor(z).unwrap().action.is_some_and(|a| a.winding_up()) {
            break;
        }
    }
    assert!(w.actor(z).unwrap().action.is_some(), "it began a swing");
    w.intent(me, Intent::Shove { dir: None });
    let mut shoved = false;
    for _ in 0..3 {
        w.step(1);
        shoved |= w
            .events()
            .any(|e| matches!(e, Event::Shoved { target, .. } if target == z));
    }
    assert!(shoved);
    let a = w.actor(z).unwrap();
    assert_eq!(a.pos, TilePos::new(0, 0, 0), "pushed back a tile");
    assert!(a.action.is_none(), "its swing was broken off");
}

#[test]
fn a_shut_door_keeps_the_dead_out() {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+
|. @d. . z|
+-+-+-+-+-+
",
    )
    .unwrap();
    w.rules.wander_chance = 0;
    let (me, z) = (w.character().unwrap(), shambler(&w));
    w.intent(me, Intent::Door);
    w.step(10);
    let door = w.grid.edge(TilePos::new(1, 0, 0), Dir8::E);
    assert_eq!(door, Edge::Door { open: false }, "you shut it");
    w.make_sound(TilePos::new(1, 0, 0), 10, Some(me));
    w.step(300);
    assert!(w.actor(z).unwrap().pos.x >= 2, "it stayed outside");
    assert!(!w.actor(me).unwrap().dead);
}

#[test]
fn running_tires_you_and_rest_brings_your_breath_back() {
    let mut w = World::new(1);
    for x in 0..200 {
        w.grid.set_tile(TilePos::new(x, 0, 0), Tile::GRASS);
    }
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: true,
        },
    );
    let mut tired_at = None;
    for t in 0..400 {
        w.step(1);
        if w.events().any(|e| e == Event::Exhausted { who: me }) {
            tired_at = Some(t);
            break;
        }
    }
    assert!(tired_at.is_some(), "you never ran out of breath");
    let mut strides = Vec::new();
    for _ in 0..20 {
        w.step(1);
        strides.extend(w.actor(me).unwrap().step.map(|s| s.run));
    }
    assert!(
        !strides.is_empty() && strides.iter().all(|&run| !run),
        "out of breath, you walk"
    );
    w.intent(me, Intent::Idle);
    w.step(60);
    assert!(
        !w.actor(me).unwrap().body.exhausted,
        "standing still, you got it back"
    );
}

#[test]
fn wounds_bleed_and_enough_of_them_kill() {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+
|z|
+ +
|@|
+-+
",
    )
    .unwrap();
    w.rules.wander_chance = 0;
    let me = w.character().unwrap();
    let (mut bled, mut died) = (false, false);
    let mut health = w.actor(me).unwrap().body.health;
    for _ in 0..5000 {
        w.step(1);
        let struck = w.events().fold((false, false), |(s, d), e| match e {
            Event::Struck { target, .. } if target == me => (true, d),
            Event::Died { who } if who == me => (s, true),
            _ => (s, d),
        });
        let now = w.actor(me).unwrap().body.health;
        bled |= now < health && !struck.0;
        health = now;
        if struck.1 {
            died = true;
            break;
        }
    }
    assert!(bled, "a wound bled between blows");
    assert!(died, "standing there, you died");
    let me = w.actor(me).unwrap();
    assert!(me.dead && !me.body.wounds.is_empty());
}
