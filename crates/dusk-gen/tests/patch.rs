//! A patch of the Hushlands: the same from the same seed, the same in any chunk order, every
//! building reachable on foot, and the dead where they can stand.

use dusk_core::path::find_path;
use dusk_core::{ActorKind, Grid, World};
use dusk_data::{Data, default_dir, load};
use dusk_gen::{Layout, patch};

fn data() -> Data {
    load(&default_dir()).unwrap_or_else(|e| panic!("{e}"))
}

fn make(seed: u64) -> (World, Layout) {
    let data = data();
    patch(seed, &data, data.region("hushlands").unwrap())
}

#[test]
fn the_same_seed_makes_the_same_patch() {
    assert_eq!(make(3).0.state_hash(), make(3).0.state_hash());
    assert_ne!(make(3).0.state_hash(), make(4).0.state_hash());
}

#[test]
fn chunks_come_out_the_same_in_any_order() {
    let data = data();
    let layout = Layout::new(9, &data, data.region("hushlands").unwrap());
    let chunks: Vec<_> = layout.chunks().collect();
    let (mut forward, mut backward) = (Grid::new(), Grid::new());
    for &c in &chunks {
        layout.fill(c, &mut forward);
    }
    for &c in chunks.iter().rev() {
        layout.fill(c, &mut backward);
    }
    assert!(forward == backward);
}

#[test]
fn every_building_can_be_walked_into_from_the_start() {
    for seed in [1, 2, 3] {
        let (world, layout) = make(seed);
        assert!(layout.lots.len() >= 6, "seed {seed}: a hamlet, not a farm");
        for lot in &layout.lots {
            let way = find_path(&world.grid, layout.start, lot.door, 200_000);
            assert!(
                way.is_some(),
                "seed {seed}: no way into the {} at {:?}",
                lot.prefab,
                lot.door
            );
        }
    }
}

#[test]
fn the_dead_stand_where_they_can_and_not_by_the_start() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let (world, layout) = patch(5, &data, region);
    let clear = i64::from(region.dead.clear_start).pow(2);
    let mut dead = 0;
    for (_, a) in world.actors() {
        assert!(world.grid.tile(a.pos).walkable());
        if a.kind == ActorKind::Shambler {
            dead += 1;
            assert!(
                a.pos.dist2(layout.start) > clear,
                "one by the start at {:?}",
                a.pos
            );
        }
    }
    assert!(dead > 20, "{dead} dead");
}
