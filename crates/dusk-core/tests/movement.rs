use dusk_core::{ActorId, Dir8, Intent, TilePos, World};

fn pos(w: &World, id: ActorId) -> TilePos {
    w.actor(id).expect("still there").pos
}

/// A corridor three tiles long, you at its west end.
fn corridor() -> (World, ActorId) {
    let (w, ids) = World::from_text(
        1,
        "\
+-+-+-+
|@ . .|
+-+-+-+
",
    )
    .unwrap();
    (w, ids[0])
}

fn walk(dir: Dir8) -> Intent {
    Intent::Move { dir, run: false }
}

fn offset(w: &World, id: ActorId) -> (i32, i32) {
    w.actor(id).unwrap().offset
}

#[test]
fn you_go_a_little_each_tick_and_stop_where_youre_let_go() {
    // At a walk, 1.5 tiles a second: 150 thousandths of a tile a tick.
    let (mut w, me) = corridor();
    w.intent(me, walk(Dir8::E));
    w.step(1);
    assert_eq!(
        (pos(&w, me), offset(&w, me)),
        (TilePos::new(0, 0, 0), (150, 0))
    );
    w.step(2);
    assert_eq!(offset(&w, me), (450, 0));
    // Past the middle of the edge between, it's in the next tile.
    w.step(1);
    assert_eq!(
        (pos(&w, me), offset(&w, me)),
        (TilePos::new(1, 0, 0), (-400, 0))
    );
    // Let go, it stops where it is: not at a tile's middle.
    w.intent(me, Intent::Idle);
    w.step(10);
    assert_eq!(
        (pos(&w, me), offset(&w, me)),
        (TilePos::new(1, 0, 0), (-400, 0))
    );
}

#[test]
fn a_diagonal_is_no_quicker_and_a_wall_slides_you_along_it() {
    let (mut w, me) = corridor();
    // South-east: 150 a tick in all, 106 each way.
    w.intent(me, walk(Dir8::SE));
    w.step(1);
    assert_eq!(offset(&w, me), (106, 106));
    // The corridor's south wall: no nearer than a quarter of a tile, and on east along it.
    w.step(3);
    let (x, y) = offset(&w, me);
    assert_eq!(y, 250, "a quarter of a tile from the wall");
    assert!(x > 318, "still going east: {x}");
    // Into the wall at the end: no further than a quarter of a tile short of it.
    w.intent(me, walk(Dir8::E));
    w.step(40);
    assert_eq!(
        (pos(&w, me), offset(&w, me).0),
        (TilePos::new(2, 0, 0), 250)
    );
}

#[test]
fn a_walk_somewhere_ends_at_the_tiles_middle_a_footstep_a_tile() {
    let (mut w, me) = corridor();
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(2, 0, 0),
            run: false,
        },
    );
    let mut footsteps = 0;
    for _ in 0..40 {
        w.step(1);
        footsteps += w
            .events()
            .filter(|e| matches!(*e, dusk_core::Event::Sound { by: Some(b), .. } if b == me))
            .count();
    }
    assert_eq!(
        (pos(&w, me), offset(&w, me)),
        (TilePos::new(2, 0, 0), (0, 0))
    );
    assert_eq!(w.actor(me).unwrap().intent, Intent::Idle, "there");
    assert_eq!(footsteps, 2, "two tiles walked");
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
