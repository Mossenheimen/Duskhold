//! Shallows: wading is slow going and loud, for everyone, and you can drink where you stand.

use dusk_core::{ActorKind, Event, Ground, Intent, Tile, TilePos, World};

/// A strip of grass ten tiles long, or of shallows, with you at its west end.
fn strip(ground: Ground) -> (World, dusk_core::ActorId) {
    let mut w = World::new(2);
    for x in -1..12 {
        for y in -1..2 {
            w.grid.set_tile(
                TilePos::new(x, y, 0),
                Tile {
                    ground,
                    ..Tile::GRASS
                },
            );
        }
    }
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    (w, me)
}

/// Ticks to walk ten tiles east, and how far the first step's noise carried.
fn walk(ground: Ground) -> (u32, i32) {
    let (mut w, me) = strip(ground);
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(10, 0, 0),
            run: false,
        },
    );
    let (mut ticks, mut noise) = (0, 0);
    while w.actor(me).unwrap().pos.x < 10 {
        w.step(1);
        ticks += 1;
        for e in w.events() {
            if let Event::Sound { loudness, .. } = e
                && noise == 0
            {
                noise = loudness;
            }
        }
        assert!(ticks < 1000, "it never got there");
    }
    (ticks, noise)
}

#[test]
fn wading_takes_twice_as_long_and_splashes_louder() {
    let (dry, step) = walk(Ground::Grass);
    let (wet, splash) = walk(Ground::Shallows);
    assert!(
        wet >= dry * 2 - 1 && wet <= dry * 2 + 1,
        "{dry} ticks dry, {wet} wading"
    );
    assert_eq!(splash, step * 3 / 2);
}

#[test]
fn you_can_drink_standing_in_the_shallows() {
    let (w, me) = strip(Ground::Shallows);
    let at = w.actor(me).unwrap().pos;
    assert_eq!(w.water_beside(at), Some(at));
    let (dry, me) = strip(Ground::Grass);
    let at = dry.actor(me).unwrap().pos;
    assert_eq!(dry.water_beside(at), None);
}
