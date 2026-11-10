use dusk_core::senses::{Cone, hears, line_of_sight};
use dusk_core::{Alert, Dir8, Event, Intent, Rules, TilePos, World};

/// Two corridors with a wall between them.
const CORRIDORS: &str = "\
+-+-+-+-+-+
|@ . . . .|
+-+-+-+-+-+
|z . . . .|
+-+-+-+-+-+
";

#[test]
fn walls_muffle_footsteps() {
    let (w, _) = World::from_text(1, CORRIDORS).unwrap();
    let rules = Rules::default();
    let (footstep, listener) = (TilePos::new(1, 0, 0), TilePos::new(0, 1, 0));
    assert!(
        !hears(
            &w.grid,
            footstep,
            listener,
            rules.walk_noise,
            rules.muffling
        ),
        "a walk doesn't get through the wall"
    );
    assert!(
        hears(&w.grid, footstep, listener, rules.run_noise, rules.muffling),
        "a run does"
    );
}

#[test]
fn a_shambler_behind_a_wall_hears_you_run_but_not_walk() {
    for (run, heard) in [(false, false), (true, true)] {
        let (mut w, _) = World::from_text(1, CORRIDORS).unwrap();
        w.rules.wander_chance = 0;
        let me = w.character().unwrap();
        w.intent(me, Intent::Move { dir: Dir8::E, run });
        w.step(40);
        let got = w.events().any(|e| matches!(e, Event::Heard { .. }));
        assert_eq!(got, heard, "running: {run}");
    }
}

#[test]
fn sight_passes_open_doors_and_glass_but_not_walls_or_shut_doors() {
    let (w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+
|.d.W.D.|.|
+-+-+-+-+-+
",
    )
    .unwrap();
    let p = |x| TilePos::new(x, 0, 0);
    assert!(
        line_of_sight(&w.grid, p(0), p(2)),
        "through the open door and the window"
    );
    assert!(!line_of_sight(&w.grid, p(0), p(3)), "the shut door");
    assert!(!line_of_sight(&w.grid, p(3), p(4)), "the wall");
}

#[test]
fn a_cone_sees_ahead_and_to_the_sides_but_not_behind() {
    let cone = Cone { cos: 5000 }; // 60° either side
    assert!(cone.contains(Dir8::E, 5, 0), "straight ahead");
    assert!(cone.contains(Dir8::E, 2, 1), "27° off");
    assert!(cone.contains(Dir8::E, 1, 1), "45° off");
    assert!(!cone.contains(Dir8::E, 1, 2), "63° off");
    assert!(!cone.contains(Dir8::E, 0, 3), "to the side");
    assert!(!cone.contains(Dir8::E, -4, 0), "behind");
    assert!(!cone.contains(Dir8::E, 0, 0), "its own tile");
    assert!(cone.contains(Dir8::NE, 3, -3), "ahead on a diagonal");
    assert!(cone.contains(Dir8::NE, 4, 0), "45° off a diagonal");
    assert!(!cone.contains(Dir8::NE, -2, -4), "72° off a diagonal");
    let half = Cone { cos: 0 };
    assert!(half.contains(Dir8::E, 0, 3), "90° is in a half-plane");
    assert!(!half.contains(Dir8::E, -1, 3), "past 90° isn't");
    assert!(Cone { cos: -10_000 }.contains(Dir8::E, -4, 0), "all round");
}

#[test]
fn a_shambler_doesnt_see_you_behind_it_until_it_hears_you() {
    // It faces south; you stand still just north of it, then take a step.
    let (mut w, _) = World::from_text(
        1,
        "\
+-+-+-+-+-+
|. . @ . .|
+         +
|. . z . .|
+         +
|. . . . .|
+-+-+-+-+-+
",
    )
    .unwrap();
    w.rules.wander_chance = 0;
    let me = w.character().unwrap();
    let z = w.actors().map(|(id, _)| id).find(|&id| id != me).unwrap();
    let alert = |w: &World| w.actor(z).unwrap().alert();
    w.step(50);
    assert!(!w.events().any(|e| matches!(e, Event::Saw { .. })));
    assert_eq!(alert(&w), Alert::Unaware, "you were behind it, and quiet");

    w.intent(
        me,
        Intent::Move {
            dir: Dir8::W,
            run: false,
        },
    );
    let mut seen = vec![alert(&w)];
    let mut reached = false;
    for t in 0..60 {
        if t == 8 {
            w.intent(me, Intent::Idle);
        }
        w.step(1);
        reached |= w.events().any(|e| matches!(e, Event::Reached { .. }));
        if seen.last() != Some(&alert(&w)) {
            seen.push(alert(&w));
        }
    }
    assert_eq!(
        seen,
        [Alert::Unaware, Alert::Curious, Alert::Hunting],
        "it heard your step (?), turned to it, and saw you (!)"
    );
    assert!(reached, "and it got you");
}

/// One shambler behind a wall, one in the passage that sees you at once.
const BEHIND_A_WALL: &str = "\
+-+
|z|
+-+
|z|
+ +
|.|
+ +
|@|
+-+
";

#[test]
fn a_shambler_that_sees_you_groans_and_the_others_come() {
    let heard_by_the_other = |groan: i32| {
        let (mut w, placed) = World::from_text(1, BEHIND_A_WALL).unwrap();
        w.rules.wander_chance = 0;
        w.rules.shambler_groan = groan;
        let (behind, sees_you) = (placed[0], placed[1]);
        let mut heard = None;
        for _ in 0..20 {
            w.step(1);
            for e in w.events() {
                if let Event::Heard { who, at } = e
                    && who == behind
                {
                    heard = Some(at);
                }
            }
        }
        assert_eq!(w.actor(sees_you).unwrap().alert(), Alert::Hunting);
        heard
    };
    assert_eq!(heard_by_the_other(0), None, "silent, nothing reaches it");
    assert_eq!(
        heard_by_the_other(10),
        Some(TilePos::new(0, 1, 0)),
        "it goes to the one that groaned"
    );
}
