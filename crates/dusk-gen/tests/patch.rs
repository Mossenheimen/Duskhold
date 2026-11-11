//! A patch of the Hushlands: the same from the same seed, the same in any chunk order, every
//! building reachable on foot, and the dead where they can stand.

use dusk_core::path::find_path;
use dusk_core::{Grid, World};
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
        if !a.kind.is_character() {
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

#[test]
fn the_dead_are_mostly_shamblers_with_some_ghouls_among_them() {
    let data = data();
    let ghoul = data
        .defs
        .creature_kind("ghoul")
        .expect("a ghoul in the data");
    let (mut all, mut ghouls) = (0, 0);
    for seed in 1..=4 {
        let (world, _) = make(seed);
        for (_, a) in world.actors() {
            if !a.kind.is_character() {
                all += 1;
                ghouls += usize::from(a.kind == ghoul);
            }
        }
    }
    // A tenth of them, give or take what four patches' rolls fall to.
    assert!(
        ghouls * 20 > all && ghouls * 6 < all,
        "{ghouls} ghouls of {all}"
    );
}

#[test]
fn you_start_inside_the_abbey_with_none_of_the_dead_within_its_walls() {
    for seed in 1..=8 {
        let (w, layout) = make(seed);
        let me = w.character().unwrap();
        let at = w.actor(me).unwrap().pos;
        assert!(
            w.is_hallowed(at),
            "seed {seed}: you start on hallowed ground"
        );
        assert!(layout.lots.iter().any(|l| l.holdout && l.prefab == "abbey"));
        assert!(
            w.actors()
                .all(|(_, a)| a.kind.is_character() || !w.is_hallowed(a.pos)),
            "seed {seed}: none of the dead inside"
        );
        // Its counter can be walked to and reached from inside.
        let counter = (0..layout.width)
            .flat_map(|x| (0..layout.height).map(move |y| dusk_core::TilePos::new(x, y, 0)))
            .find(|&p| w.counter_at(p).is_some())
            .expect("the abbey has its counter");
        assert!(w.is_hallowed(counter));
        let beside = dusk_core::Dir8::ALL
            .iter()
            .map(|&d| counter.step(d))
            .find(|&p| w.grid.tile(p).walkable())
            .expect("a place to stand at the counter");
        assert!(
            find_path(&w.grid, at, beside, 4000).is_some(),
            "seed {seed}: the counter is reachable"
        );
    }
}

#[test]
fn the_bell_hangs_in_the_chapel_where_the_dead_can_walk_to_it() {
    for seed in 1..=4 {
        let (w, layout) = make(seed);
        let bell = w.region.bell.expect("the Hushlands have a Tolling");
        let chapel = layout.lots.iter().find(|l| l.prefab == "chapel").unwrap();
        assert_eq!(
            bell, chapel.door,
            "seed {seed}: just inside the chapel's door"
        );
        assert!(w.grid.tile(bell).walkable());
        // Its field reaches most of the patch's dead within its reach.
        let t = w.region.tolling;
        let field = dusk_core::flow::Flow::build(&w.grid, bell, t.reach, t.door);
        let r2 = i64::from(t.reach).pow(2);
        let (mut near, mut way) = (0, 0);
        for (_, a) in w.actors().filter(|(_, a)| !a.kind.is_character()) {
            if a.pos.dist2(bell) <= r2 {
                near += 1;
                way += usize::from(field.cost_at(a.pos) != dusk_core::flow::NOWHERE);
            }
        }
        assert!(
            near > 0 && way * 10 >= near * 9,
            "seed {seed}: {way} of {near} have a way"
        );
    }
}

#[test]
fn every_floor_of_every_building_can_be_walked_to_from_its_door() {
    for seed in 1..=6 {
        let (w, layout) = make(seed);
        for lot in &layout.lots {
            let (x0, y0) = (lot.origin.x, lot.origin.y);
            for y in y0..y0 + lot.grid.height {
                for x in x0..x0 + lot.grid.width {
                    let at = dusk_core::TilePos::new(x, y, 0);
                    let tile = w.grid.tile(at);
                    if tile.ground != dusk_core::Ground::Floor || !tile.walkable() {
                        continue;
                    }
                    assert!(
                        find_path(&w.grid, lot.door, at, 2000).is_some(),
                        "seed {seed}: the {} at {:?} has a floor at {at:?} its door can't reach",
                        lot.prefab,
                        lot.origin
                    );
                }
            }
        }
    }
}

#[test]
fn each_patch_has_its_rares_far_from_the_start_each_named_once() {
    for seed in 1..=6 {
        let (w, layout) = make(seed);
        let rares: Vec<_> = w.rares().collect();
        assert_eq!(rares.len(), 3, "seed {seed}");
        let mut names: Vec<String> = rares.iter().filter_map(|&r| w.name_of(r)).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), 3, "seed {seed}: three names");
        for &r in &rares {
            let a = w.actor(r).unwrap();
            assert!(
                a.pos.dist2(layout.start) >= 40 * 40,
                "seed {seed}: far from the start"
            );
            assert!((1..=2).contains(&a.affixes.len()));
            assert!(
                a.items.iter().all(|i| i.found) && !a.items.is_empty(),
                "a pack of finds"
            );
        }
    }
}

#[test]
fn the_river_has_its_bridge_and_fords_clear_of_the_hamlet_and_the_far_bank_is_reached() {
    use dusk_core::{Ground, TilePos};
    for seed in 1..=6 {
        let (w, layout) = make(seed);
        let river = layout.river.as_ref().expect("a river");
        assert_eq!(river.fords.len(), 2, "seed {seed}: two fords");
        let x = river.base as i32;
        // Its rows: deep water across most of them, a bridge on the road.
        let deep = (0..layout.height)
            .filter(|&y| {
                (x - 12..=x + 12)
                    .any(|x| w.grid.tile(TilePos::new(x, y, 0)).ground == Ground::Water)
            })
            .count();
        assert!(
            deep > layout.height as usize * 3 / 4,
            "seed {seed}: deep most of the way"
        );
        let bridge = (0..layout.height).any(|y| {
            (x - 12..=x + 12).any(|x| w.grid.tile(TilePos::new(x, y, 0)).ground == Ground::Bridge)
        });
        assert!(bridge, "seed {seed}: the road crosses on a bridge");
        // No lot within its sway of it.
        for lot in &layout.lots {
            let (x0, _, x1, _) = lot.yard;
            assert!(
                x1 + 6 <= x - 9 - 3 || x0 - 6 >= x + 9 + 3,
                "seed {seed}: {} too near",
                lot.prefab
            );
        }
        // The far bank: a tile 20 east of the river, reached from the start on foot.
        let far = TilePos::new((x + 20).min(layout.width - 2), layout.start.y, 0);
        let far = (0..40)
            .map(|k| far.offset(0, k - 20))
            .find(|&p| w.grid.tile(p).walkable())
            .expect("somewhere to stand over there");
        assert!(
            find_path(&w.grid, layout.start, far, 60_000).is_some(),
            "seed {seed}"
        );
    }
}
