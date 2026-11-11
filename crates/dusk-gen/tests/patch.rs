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
        // The far bank: the road 20 tiles east of the river, reached from the start on foot.
        // (The road, because any other tile over there may be a clearing shut in by woods.)
        let far_x = (x + 20).min(layout.width - 2);
        let far = (0..layout.height)
            .map(|y| TilePos::new(far_x, y, 0))
            .find(|&p| w.grid.tile(p).ground == Ground::Road)
            .expect("the road goes on over there");
        assert!(
            find_path(&w.grid, layout.start, far, 60_000).is_some(),
            "seed {seed}"
        );
    }
}

#[test]
fn the_market_town_has_its_three_shops_at_its_heart_and_room_for_the_abbey_and_the_river() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    for seed in 1..=8 {
        let (_, layout) = patch(seed, &data, region);
        let chapel = layout.lots.iter().find(|l| l.prefab == "chapel").unwrap();
        for shop in ["smithy", "store", "tavern"] {
            let found: Vec<_> = layout.lots.iter().filter(|l| l.prefab == shop).collect();
            assert_eq!(found.len(), 1, "seed {seed}: one {shop}");
            assert!(
                (found[0].door.x - chapel.door.x).abs() <= 30,
                "seed {seed}: the {shop} is by the chapel, on the high street"
            );
        }
        assert!(
            layout.lots.iter().any(|l| l.holdout),
            "seed {seed}: the abbey fits west of the town"
        );
        assert!(
            layout.river.is_some(),
            "seed {seed}: the river fits east of it"
        );
    }
}

/// The items in the containers of one kind of fixture inside a lot's walls, a container at a
/// time.
fn stores_in(
    w: &World,
    data: &Data,
    lot: &dusk_gen::Lot,
    fixture: &str,
) -> Vec<Vec<dusk_core::Item>> {
    let inside = lot.inside();
    let mut out = Vec::new();
    for y in inside.y0..inside.y1 {
        for x in inside.x0..inside.x1 {
            let at = dusk_core::TilePos::new(x, y, 0);
            let n = w.grid.tile(at).fixture;
            if data.fixture(n).is_some_and(|f| f.id == fixture) {
                out.push(w.store(at));
            }
        }
    }
    out
}

#[test]
fn each_shop_holds_its_own_loot() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    let id = |name: &str| data.defs.items.iter().position(|i| i.id == name).unwrap() as u16;
    let forged = [id("sword"), id("spear"), id("mace")];
    let (mut weapons, mut smiths_work, mut ale) = (0, 0, 0);
    for seed in 1..=6 {
        let (w, layout) = patch(seed, &data, region);
        let lot = |prefab: &str| layout.lots.iter().find(|l| l.prefab == prefab).unwrap();
        // The smithy: each rack holds one weapon.
        let racks = stores_in(&w, &data, lot("smithy"), "rack");
        assert_eq!(racks.len(), 2, "seed {seed}: two racks");
        for rack in &racks {
            assert_eq!(rack.len(), 1, "seed {seed}: a weapon on each rack");
            let def = data.defs.item(rack[0].def).unwrap();
            assert!(
                matches!(def.kind, dusk_core::ItemKind::Weapon { .. }),
                "seed {seed}: {} on a weapon rack",
                def.id
            );
            weapons += 1;
            smiths_work += usize::from(forged.contains(&rack[0].def));
        }
        // The general store: coin in its till, and its shelves stocked.
        let till = stores_in(&w, &data, lot("store"), "till");
        assert_eq!(till.len(), 1, "seed {seed}: one till");
        assert!(
            till[0].len() == 1
                && till[0][0].def == id("coins")
                && (5..=20).contains(&till[0][0].count),
            "seed {seed}: 5 to 20 coins in the till"
        );
        let goods: usize = stores_in(&w, &data, lot("store"), "shop_shelf")
            .iter()
            .map(Vec::len)
            .sum();
        assert!(
            goods >= 4,
            "seed {seed}: {goods} lots of goods on the shelves"
        );
        // The tavern: ale at the bar or in the casks.
        for fixture in ["bar", "cask"] {
            for store in stores_in(&w, &data, lot("tavern"), fixture) {
                ale += store.iter().filter(|i| i.def == id("ale")).count();
            }
        }
    }
    assert!(
        smiths_work * 2 > weapons,
        "{smiths_work} of {weapons} the smith's own work"
    );
    assert!(ale >= 6, "{ale} lots of ale in six taverns");
}

#[test]
fn the_tavern_was_full_when_the_dusk_came() {
    let data = data();
    let region = data.region("hushlands").unwrap();
    for seed in 1..=6 {
        let (w, layout) = patch(seed, &data, region);
        let tavern = layout.lots.iter().find(|l| l.prefab == "tavern").unwrap();
        assert_eq!(tavern.dead, (2, 5), "the prefab's own, not the region's");
        let inside = tavern.inside();
        let dead = w
            .actors()
            .filter(|(_, a)| !a.kind.is_character() && inside.contains(a.pos))
            .count();
        // A house holds none to two (the region's per_house); the tavern, two to five.
        assert!((2..=5).contains(&dead), "seed {seed}: {dead} in the tavern");
    }
}

#[test]
fn farms_lie_out_in_the_fields_with_a_track_to_the_road_a_well_and_wheat() {
    use dusk_core::{Ground, TilePos};
    let data = data();
    let region = data.region("hushlands").unwrap();
    let f = region.farms.as_ref().expect("the Hushlands have farms");
    let is_farm = |l: &dusk_gen::Lot| {
        data.prefab(&l.prefab)
            .is_some_and(|p| p.tags.contains(&f.tag))
    };
    for seed in 1..=8 {
        let (w, layout) = patch(seed, &data, region);
        let farms: Vec<_> = layout.lots.iter().filter(|l| is_farm(l)).collect();
        assert!(farms.len() >= 2, "seed {seed}: {} farms", farms.len());
        for farm in &farms {
            // Out in the fields: a track at least off_road long, from its door to the road.
            assert!(
                farm.path.len() as i32 >= f.off_road.0,
                "seed {seed}: a track of {}",
                farm.path.len()
            );
            let end = *farm.path.last().unwrap();
            let past = end.offset(0, if farm.north { 1 } else { -1 });
            assert_eq!(
                w.grid.tile(past).ground,
                Ground::Road,
                "seed {seed}: the track meets the road"
            );
            // Clear of every other lot by the gap, and of the river's whole course.
            let (x0, y0, x1, y1) = farm.yard;
            for other in layout.lots.iter().filter(|l| l.origin != farm.origin) {
                let (a0, b0, a1, b1) = other.yard;
                assert!(
                    x0 - f.gap >= a1 || x1 + f.gap <= a0 || y0 - f.gap >= b1 || y1 + f.gap <= b0,
                    "seed {seed}: a farm too near the {}",
                    other.prefab
                );
            }
            if let Some(river) = &layout.river {
                let x = river.base as i32;
                assert!(x1 + f.gap <= x - 12 || x0 - f.gap >= x + 12, "seed {seed}");
            }
            // One well, and a field of wheat that's walked through but not seen through.
            let inside = farm.inside();
            let (mut wells, mut wheat) = (0, 0);
            for y in inside.y0..inside.y1 {
                for x in inside.x0..inside.x1 {
                    let t = w.grid.tile(TilePos::new(x, y, 0));
                    wells += usize::from(w.defs.is_well(t.fixture));
                    if data.fixture(t.fixture).is_some_and(|fx| fx.id == "wheat") {
                        assert!(t.walkable() && t.blocks_sight());
                        wheat += 1;
                    }
                }
            }
            assert_eq!(wells, 1, "seed {seed}: one well");
            assert!(wheat >= 60, "seed {seed}: {wheat} tiles of wheat");
            // Its dead are its own: none walk in from the open at the start.
            let dead = w
                .actors()
                .filter(|(_, a)| !a.kind.is_character() && farm.inside().contains(a.pos))
                .count();
            assert!(dead <= 3, "seed {seed}: {dead} dead on a farm");
        }
    }
}
