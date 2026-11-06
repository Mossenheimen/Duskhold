//! The same seed and the same intents play the same game, tick for tick.

use dusk_core::{Dir8, Intent, World};

/// The demo's rooms and corridor, with a field below where two shamblers wander.
const MAP: &str = "\
+-+-+-+-+-+-+-+-+-+-+
|. . . . .|. . . z .|
+         +-+-+-+d+-+
|@ . . . .d. . . . .|
+         +-+-+-+-+-+
|. . . . .|. z . . .|
+-+-+-+-+-+-+-+-+-+-+
|, , z , , , , z , ,|
+                   +
|, , , , , , , , , ,|
+-+-+-+-+-+-+-+-+-+-+
";

fn play(seed: u64) -> Vec<u64> {
    let (mut w, _) = World::from_text(seed, MAP).unwrap();
    let me = w.character().unwrap();
    let mut hashes = Vec::with_capacity(600);
    for t in 0..600 {
        match t {
            0 => w.intent(
                me,
                Intent::Move {
                    dir: Dir8::E,
                    run: false,
                },
            ),
            80 => w.intent(
                me,
                Intent::Move {
                    dir: Dir8::W,
                    run: true,
                },
            ),
            200 => w.intent(me, Intent::Idle),
            _ => {}
        }
        w.step(1);
        hashes.push(w.state_hash());
    }
    hashes
}

#[test]
fn the_same_seed_plays_the_same_game() {
    assert_eq!(play(7), play(7));
}

#[test]
fn another_seed_plays_another() {
    assert_ne!(play(7).last(), play(8).last());
}
