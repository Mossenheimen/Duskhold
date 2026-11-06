use dusk_core::senses::{hears, line_of_sight};
use dusk_core::{Dir8, Event, Intent, Rules, TilePos, World};

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
