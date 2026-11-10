//! Far from the character the world sleeps: creatures there wait until someone comes near.

use dusk_core::{ActorKind, Tile, TilePos, World};

#[test]
fn far_from_the_character_the_world_sleeps() {
    let mut w = World::new(1);
    for y in 0..8 {
        for x in 0..200 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    w.creature_mut(0).wander = 10_000; // every thought, a shuffle
    w.rules.active_radius = 2;
    let at = |x| TilePos::new(x, 4, 0);
    w.spawn(ActorKind::Character, at(1)).unwrap();
    let near = w.spawn(ActorKind::Creature(0), at(40)).unwrap(); // a chunk away
    let far = w.spawn(ActorKind::Creature(0), at(180)).unwrap(); // five chunks away
    assert!(w.is_awake(at(40)) && !w.is_awake(at(180)));
    w.step(100);
    assert_ne!(
        w.actor(near).unwrap().pos,
        at(40),
        "the near one shuffled about"
    );
    assert_eq!(w.actor(far).unwrap().pos, at(180), "the far one slept");
}
