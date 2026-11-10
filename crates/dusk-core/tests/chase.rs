//! How a shambler gets to what it noticed: through doors, never across a door frame, and
//! pressing against the wall when there's no way at all.

use dusk_core::{ActorId, Event, TilePos, World};

fn shambler(w: &World) -> ActorId {
    w.actors()
        .find(|(_, a)| !a.kind.is_character())
        .map(|(id, _)| id)
        .expect("the map has a shambler")
}

fn pos(w: &World, id: ActorId) -> TilePos {
    w.actor(id).expect("still there").pos
}

#[test]
fn comes_round_through_the_door() {
    // It hears you through the wall, but the straight line is a wall: the way is the door.
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+
|. . . z .|
+-+-+-+d+-+
|@ . . . .|
+-+-+-+-+-+
",
    )
    .unwrap();
    w.creature_mut(0).wander = 0;
    let z = shambler(&w);
    w.make_sound(TilePos::new(0, 1, 0), 10, None);
    let (mut through_the_door, mut reached) = (false, false);
    for _ in 0..150 {
        w.step(1);
        through_the_door |= pos(&w, z) == TilePos::new(3, 1, 0);
        reached |= w.events().any(|e| matches!(e, Event::Reached { .. }));
    }
    assert!(through_the_door, "it never came through the door");
    assert!(reached, "it never reached you");
    assert_eq!(pos(&w, z), TilePos::new(1, 1, 0), "it ends up next to you");
}

#[test]
fn never_reaches_across_a_door_frame() {
    // From its room it can see you diagonally past the door frame, but it can't step there: it
    // has to come through the door first.
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+
|. z .|
+-+d+-+
|. . @|
+-+-+-+
",
    )
    .unwrap();
    w.creature_mut(0).wander = 0;
    let z = shambler(&w);
    let mut reached = false;
    for _ in 0..60 {
        w.step(1);
        if w.events().any(|e| matches!(e, Event::Reached { .. })) {
            reached = true;
            assert_eq!(
                pos(&w, z),
                TilePos::new(1, 1, 0),
                "reached from inside the doorway"
            );
        }
    }
    assert!(reached);
}

#[test]
fn presses_against_the_wall_where_there_is_no_way() {
    // Shut in its room, it hears you next door: it looks for a way once, then presses on.
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+
|z .|@|
+-+-+-+
",
    )
    .unwrap();
    w.creature_mut(0).wander = 0;
    let z = shambler(&w);
    w.make_sound(TilePos::new(2, 0, 0), 10, None);
    let mut no_way = 0;
    for _ in 0..200 {
        w.step(1);
        no_way += w
            .events()
            .filter(|e| matches!(e, Event::NoWay { .. }))
            .count();
    }
    assert_eq!(no_way, 1, "it gave up planning after the first try");
    assert_eq!(
        pos(&w, z),
        TilePos::new(1, 0, 0),
        "against the wall, as close as it gets"
    );
}
