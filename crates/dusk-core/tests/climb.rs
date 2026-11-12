//! Climbing: walking straight into an open window or a fence climbs it, in its own time and for
//! breath; a run vaults a fence; a shut or boarded window can't be climbed, and the door key opens
//! a window; weighed down, it's slower; a way to somewhere goes over a fence when that's quicker;
//! and the dead that climb come over after you, tumbling, but not while they wander.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Edge, Event, Intent, Item, ItemDef, ItemKind, TilePos, World,
};

/// You on the west of a fence (`f`) or a window (`W` shut, `w` open), with room beyond it.
fn across(edge: char) -> (World, ActorId) {
    let map = format!(
        "\
+-+-+-+-+
|@{edge}. . .|
+-+-+-+-+
"
    );
    let (w, ids) = World::from_text(1, &map).unwrap();
    (w, ids[0])
}

fn pos(w: &World, id: ActorId) -> TilePos {
    w.actor(id).unwrap().pos
}

/// Steps until `id` is on `x` (at most `most` ticks): the ticks it took, and the climbs on the
/// way.
fn until_x(w: &mut World, id: ActorId, x: i32, most: u32) -> (u32, Vec<Event>) {
    let mut climbs = Vec::new();
    for t in 1..=most {
        w.step(1);
        climbs.extend(w.events().filter(|e| matches!(e, Event::Climbed { .. })));
        if pos(w, id).x == x {
            return (t, climbs);
        }
    }
    panic!("never got to x = {x}: at {:?}", pos(w, id));
}

#[test]
fn a_fence_is_climbed_in_its_time_and_for_breath() {
    let (mut w, me) = across('f');
    let fence = w.rules.climb.fence;
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    let (ticks, climbs) = until_x(&mut w, me, 1, 100);
    assert_eq!(ticks, fence, "a climb takes the fence's ticks");
    assert_eq!(
        climbs,
        vec![Event::Climbed {
            who: me,
            to: TilePos::new(1, 0, 0),
            window: false,
            fell: false,
            cut: false,
        }]
    );
    assert!(
        w.actor(me).unwrap().body.endurance < dusk_core::body::FULL,
        "it cost breath"
    );
}

#[test]
fn running_at_a_fence_you_vault_it() {
    let (mut w, me) = across('f');
    let r = w.rules.climb;
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: true,
        },
    );
    let (ticks, _) = until_x(&mut w, me, 1, 100);
    assert_eq!(ticks, r.fence * r.vault / 1000);
}

#[test]
fn a_shut_window_is_opened_with_the_door_key_then_climbed() {
    let (mut w, me) = across('W');
    let east = Intent::Move {
        dir: Dir8::E,
        run: false,
    };
    w.intent(me, east);
    w.step(30);
    assert_eq!(
        pos(&w, me),
        TilePos::new(0, 0, 0),
        "a shut window isn't climbed"
    );
    w.intent(me, Intent::Door);
    w.step(10);
    assert_eq!(
        w.grid.edge(TilePos::new(0, 0, 0), Dir8::E),
        Edge::window(true),
        "the door key opens a window"
    );
    w.intent(me, east);
    let window = w.rules.climb.window;
    let (ticks, climbs) = until_x(&mut w, me, 1, 100);
    assert_eq!(ticks, window);
    assert!(matches!(
        climbs[..],
        [Event::Climbed {
            window: true,
            fell: false,
            ..
        }]
    ));
}

#[test]
fn a_boarded_window_isnt_climbed_even_open() {
    let (mut w, me) = across('w');
    let at = TilePos::new(0, 0, 0);
    w.grid
        .set_edge(at, Dir8::E, Edge::window(true).with_boards(1));
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    w.step(60);
    assert_eq!(pos(&w, me), at);
}

#[test]
fn weighed_down_a_climb_takes_longer() {
    let (mut w, me) = across('f');
    let r = w.rules.climb;
    let mut defs = Defs::default();
    defs.items.push(ItemDef {
        id: "anvil".into(),
        name: "anvil".into(),
        weight: 40_000,
        stack: 1,
        kind: ItemKind::Other,
        worth: 0,
    });
    defs.carry = 15_000;
    w.defs = Arc::new(defs);
    let here = TilePos::new(0, 0, 0);
    w.put(here, Item::new(0, 1));
    w.intent(
        me,
        Intent::Take {
            at: here,
            slot: 0,
            def: 0,
        },
    );
    w.step(30);
    assert!(w.weighed_down(me));
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    let (ticks, _) = until_x(&mut w, me, 1, 100);
    assert_eq!(ticks, r.fence * r.heavy / 1000);
}

#[test]
fn a_way_somewhere_goes_over_the_fence_when_thats_quicker() {
    // The fence's gate is at the far east end: round by it is 14 tiles, over it 2 and a climb.
    let (mut w, ids) = World::from_text(
        1,
        "\
+-+-+-+-+-+-+-+-+
|@ . . . . . . .|
+f+f+f+f+f+f+f+ +
|. . . . . . . .|
+-+-+-+-+-+-+-+-+
",
    )
    .unwrap();
    let me = ids[0];
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(0, 1, 0),
            run: false,
        },
    );
    let mut climbed = false;
    for _ in 0..60 {
        w.step(1);
        climbed |= w.events().any(|e| matches!(e, Event::Climbed { .. }));
        if pos(&w, me) == TilePos::new(0, 1, 0) {
            break;
        }
    }
    assert!(climbed, "it went over the fence");
    assert_eq!(pos(&w, me), TilePos::new(0, 1, 0), "and got there");
}

/// One of the dead (a shambler that climbs, and tumbles) west of a fence, and you east of it.
fn the_dead_across_a_fence() -> (World, ActorId, ActorId) {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+-+
|zf. . . . @|
+-+-+-+-+-+-+
",
    )
    .unwrap();
    {
        let c = w.creature_mut(0);
        c.wander = 0;
        c.climb = 2000;
        c.tumble = true;
    }
    let find = |w: &World, character: bool| {
        w.actors()
            .find(|(_, a)| a.kind.is_character() == character)
            .map(|(id, _)| id)
            .unwrap()
    };
    let (z, me) = (find(&w, false), find(&w, true));
    (w, z, me)
}

#[test]
fn the_dead_climb_over_after_you_and_tumble() {
    let (mut w, z, _) = the_dead_across_a_fence();
    w.make_sound(TilePos::new(5, 0, 0), 10, None);
    let mut landed = None;
    for _ in 0..200 {
        w.step(1);
        for e in w.events() {
            if let Event::Climbed { who, fell, .. } = e
                && who == z
            {
                landed = Some(fell);
            }
        }
        if landed.is_some() {
            break;
        }
    }
    assert_eq!(landed, Some(true), "it came over, and fell");
    let a = w.actor(z).unwrap();
    assert_eq!(a.pos, TilePos::new(1, 0, 0));
    assert!(a.prone_until > 0, "it lies there");
}

#[test]
fn the_dead_that_dont_climb_stay_behind_the_fence() {
    let (mut w, z, _) = the_dead_across_a_fence();
    w.creature_mut(0).climb = 0;
    w.make_sound(TilePos::new(5, 0, 0), 10, None);
    w.step(200);
    assert_eq!(pos(&w, z), TilePos::new(0, 0, 0));
}

#[test]
fn wandering_the_dead_stay_on_their_side() {
    // A pen of fences with one of the dead in it, after nothing; you, walled off, out of sight.
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+-+-+
|. . . . .|. @|
+ +f+f+f+ +-+-+
|. .fzf. .|
+ +f+f+f+ +
|. . . . .|
+-+-+-+-+-+
",
    )
    .unwrap();
    {
        let c = w.creature_mut(0);
        c.wander = 10_000;
        c.climb = 1000;
    }
    let z = w
        .actors()
        .find(|(_, a)| a.kind == ActorKind::Creature(0))
        .map(|(id, _)| id)
        .unwrap();
    w.step(600);
    assert_eq!(pos(&w, z), TilePos::new(2, 1, 0));
}

/// One of the dead (a shambler that climbs and tumbles, after you) west of a window, and you east
/// of it, seen through the glass.
fn the_dead_at_a_window(edge: Edge) -> (World, ActorId, TilePos) {
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+
|zW. . . @|
+-+-+-+-+-+
",
    )
    .unwrap();
    {
        let c = w.creature_mut(0);
        c.wander = 0;
        c.climb = 2000;
        c.tumble = true;
    }
    let at = TilePos::new(0, 0, 0);
    w.grid.set_edge(at, Dir8::E, edge);
    let z = w
        .actors()
        .find(|(_, a)| !a.kind.is_character())
        .map(|(id, _)| id)
        .unwrap();
    w.make_sound(TilePos::new(4, 0, 0), 10, None);
    (w, z, at)
}

/// What broke, in order (a board, the glass), and whether `z` climbed through, in `ticks`.
fn beaten(w: &mut World, z: ActorId, ticks: u32) -> (Vec<&'static str>, bool) {
    let (mut broke, mut climbed) = (Vec::new(), false);
    for _ in 0..ticks {
        w.step(1);
        for e in w.events() {
            match e {
                Event::BoardBroke { .. } => broke.push("board"),
                Event::WindowBroke { .. } => broke.push("glass"),
                Event::Climbed { who, window, .. } if who == z && window => climbed = true,
                _ => {}
            }
        }
    }
    (broke, climbed)
}

#[test]
fn the_dead_beat_the_glass_out_of_a_shut_window_and_climb_in() {
    let (mut w, z, at) = the_dead_at_a_window(Edge::window(false));
    let (broke, climbed) = beaten(&mut w, z, 400);
    assert_eq!(broke, vec!["glass"]);
    assert_eq!(
        w.grid.edge(at, Dir8::E),
        Edge::broken_window(),
        "broken for good"
    );
    assert!(climbed, "and it came in through it");
}

#[test]
fn boards_keep_a_window_the_dead_break_them_first() {
    let (mut w, z, at) = the_dead_at_a_window(Edge::window(false).with_boards(1));
    let (broke, _) = beaten(&mut w, z, 200);
    assert!(
        broke.is_empty(),
        "two hundred ticks in, the board holds: {broke:?}"
    );
    assert_eq!(w.grid.edge(at, Dir8::E).boards(), 1);
    let (broke, climbed) = beaten(&mut w, z, 1200);
    assert_eq!(
        broke,
        vec!["board", "glass"],
        "the board first, then the glass"
    );
    assert!(climbed);
}

#[test]
fn the_dead_that_dont_climb_leave_windows_be() {
    let (mut w, z, at) = the_dead_at_a_window(Edge::window(false));
    w.creature_mut(0).climb = 0;
    let (broke, _) = beaten(&mut w, z, 400);
    assert!(broke.is_empty());
    assert_eq!(w.grid.edge(at, Dir8::E), Edge::window(false));
}

/// You climb east through a broken window, with the glass's chance to cut set: whether it cut,
/// and the wounds you have.
fn through_broken_glass(cut: u32) -> (bool, Vec<dusk_core::Part>) {
    let (mut w, me) = across('x');
    w.rules.climb.cut = cut;
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    let (_, climbs) = until_x(&mut w, me, 1, 100);
    let Event::Climbed { window, cut, .. } = climbs[0] else {
        panic!("{climbs:?}")
    };
    assert!(window);
    let parts = w
        .actor(me)
        .unwrap()
        .body
        .wounds
        .iter()
        .map(|x| x.part)
        .collect();
    (cut, parts)
}

#[test]
fn a_broken_window_cant_be_shut_and_its_glass_may_cut() {
    let (mut w, me) = across('x');
    w.intent(me, Intent::Door);
    w.step(10);
    assert_eq!(
        w.grid.edge(TilePos::new(0, 0, 0), Dir8::E),
        Edge::broken_window(),
        "there's nothing left to shut"
    );
    use dusk_core::Part::*;
    let (cut, parts) = through_broken_glass(10_000);
    assert!(cut);
    assert_eq!(parts.len(), 1);
    assert!(
        [LeftArm, RightArm, LeftHand, RightHand, LeftLeg, RightLeg].contains(&parts[0]),
        "{parts:?}"
    );
    assert_eq!(through_broken_glass(0), (false, Vec::new()));
}

#[test]
fn your_blow_breaks_the_glass_with_a_crash_heard_far_off() {
    let (mut w, me) = across('W');
    let smash = w.rules.climb.smash;
    let (mut broke, mut crash) = (false, false);
    for _ in 0..20 {
        w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
        for _ in 0..10 {
            w.step(1);
            for e in w.events() {
                broke |= matches!(e, Event::WindowBroke { .. });
                crash |= matches!(e, Event::Sound { loudness, .. } if loudness == smash);
            }
        }
        if broke {
            break;
        }
    }
    assert!(broke && crash);
    assert_eq!(
        w.grid.edge(TilePos::new(0, 0, 0), Dir8::E),
        Edge::broken_window()
    );
}
