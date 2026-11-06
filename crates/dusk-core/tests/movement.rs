use dusk_core::{ActorId, Dir8, Intent, TilePos, World};

fn pos(w: &World, id: ActorId) -> TilePos {
    w.actor(id).expect("still there").pos
}

#[test]
fn a_straight_step_takes_seven_ticks_at_a_walk() {
    let (mut w, ids) = World::from_text(
        1,
        "\
+-+-+-+
|@ . .|
+-+-+-+
",
    )
    .unwrap();
    w.intent(
        ids[0],
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    w.step(6);
    assert_eq!(pos(&w, ids[0]), TilePos::new(0, 0, 0));
    w.step(1);
    assert_eq!(pos(&w, ids[0]), TilePos::new(1, 0, 0));
}

#[test]
fn nobody_cuts_a_corner() {
    let (mut w, ids) = World::from_text(
        1,
        "\
+-+-+
|@|.|
+ + +
|. .|
+-+-+
",
    )
    .unwrap();
    assert!(!w.grid.can_step(TilePos::new(0, 0, 0), Dir8::SE));
    w.intent(
        ids[0],
        Intent::WalkTo {
            to: TilePos::new(1, 0, 0),
            run: false,
        },
    );
    w.step(100);
    assert_eq!(
        pos(&w, ids[0]),
        TilePos::new(1, 0, 0),
        "round the wall: south, east, north"
    );
}

#[test]
fn shut_doors_stop_you_and_open_ones_dont() {
    let (w, _) = World::from_text(
        1,
        "\
+-+-+-+
|.D.d.|
+-+-+-+
",
    )
    .unwrap();
    assert!(!w.grid.can_step(TilePos::new(0, 0, 0), Dir8::E));
    assert!(w.grid.can_step(TilePos::new(1, 0, 0), Dir8::E));
}

#[test]
fn two_actors_never_share_a_tile() {
    let (mut w, ids) = World::from_text(
        1,
        "\
+-+-+-+
|@ . @|
+-+-+-+
",
    )
    .unwrap();
    w.intent(
        ids[0],
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    w.intent(
        ids[1],
        Intent::Move {
            dir: Dir8::W,
            run: false,
        },
    );
    w.step(50);
    let (a, b) = (pos(&w, ids[0]), pos(&w, ids[1]));
    assert_ne!(a, b);
    assert!(
        a == TilePos::new(1, 0, 0) || b == TilePos::new(1, 0, 0),
        "one of them got the middle"
    );
}
