use dusk_core::path::find_path;
use dusk_core::{Dir8, TilePos, World};

/// Two rooms, an open door between them.
const ROOMS: &str = "\
+-+-+-+-+
|. . . .|
+-+-+d+-+
|. . . .|
+-+-+-+-+
";

#[test]
fn goes_round_through_the_open_door() {
    let (w, _) = World::from_text(1, ROOMS).unwrap();
    let from = TilePos::new(0, 0, 0);
    let path =
        find_path(&w.grid, from, TilePos::new(0, 1, 0), 1000).expect("a way through the door");
    assert!(
        path.contains(&TilePos::new(2, 0, 0)) && path.contains(&TilePos::new(2, 1, 0)),
        "{path:?}"
    );
    assert_eq!(path.last(), Some(&TilePos::new(0, 1, 0)));
    let mut at = from;
    for &next in &path {
        let dir = Dir8::toward(next.x - at.x, next.y - at.y).expect("each step goes somewhere");
        assert!(
            w.grid.can_step(at, dir),
            "{at:?} -> {next:?} isn't a step a walker could take"
        );
        at = next;
    }
}

#[test]
fn no_way_through_a_shut_door() {
    let (w, _) = World::from_text(1, &ROOMS.replace('d', "D")).unwrap();
    assert_eq!(
        find_path(&w.grid, TilePos::new(0, 0, 0), TilePos::new(0, 1, 0), 1000),
        None
    );
}

/// Each tile's open steps, worked out directly from the tiles and edges round it.
fn worked_out(grid: &dusk_core::Grid, p: TilePos) -> u8 {
    if !grid.tile(p).walkable() {
        return 0;
    }
    Dir8::ALL
        .iter()
        .filter(|&&d| grid.can_step(p, d))
        .fold(0, |bits, &d| bits | d.bit())
}

#[test]
fn the_kept_steps_stay_true_to_the_map_through_every_change() {
    use dusk_core::hash::{Purpose, below, roll};
    use dusk_core::{ChunkPos, Edge, Grid, Tile};
    // A map across four chunks, a fifth of it rock, with walls, doors and windows about.
    let edges = [
        Edge::Open,
        Edge::Wall,
        Edge::door(false),
        Edge::door(true),
        Edge::window(false),
    ];
    let r = |n: u64, k: u32| below(roll(7, n, 0, Purpose::Map), k);
    let mut grid = Grid::new();
    for y in -32..32 {
        for x in -32..32 {
            let n = ((y + 32) * 64 + x + 32) as u64;
            let rock = r(n, 5) == 0;
            let at = TilePos::new(x, y, 0);
            grid.set_tile(
                at,
                Tile {
                    solid: rock,
                    opaque: rock,
                    ..Tile::GRASS
                },
            );
            if r(n + 100_000, 6) == 0 {
                grid.set_north(at, edges[r(n + 200_000, 5) as usize]);
            }
            if r(n + 300_000, 6) == 0 {
                grid.set_west(at, edges[r(n + 400_000, 5) as usize]);
            }
        }
    }
    grid.index_steps();
    let check = |grid: &Grid, when: &str| {
        for y in -33..33 {
            for x in -33..33 {
                let p = TilePos::new(x, y, 0);
                assert_eq!(grid.steps_from(p), worked_out(grid, p), "{when}: at {p:?}");
            }
        }
    };
    check(&grid, "indexed");
    // Edits one by one, of every kind, near chunk borders and corners too.
    for k in 0..400u64 {
        let at = TilePos::new(r(k, 64) as i32 - 32, r(k + 9_000, 64) as i32 - 32, 0);
        match r(k + 18_000, 3) {
            0 => {
                let rock = r(k + 27_000, 2) == 0;
                grid.set_tile(
                    at,
                    Tile {
                        solid: rock,
                        opaque: rock,
                        ..Tile::GRASS
                    },
                );
            }
            1 => grid.set_north(at, edges[r(k + 36_000, 5) as usize]),
            _ => grid.set_west(at, edges[r(k + 45_000, 5) as usize]),
        }
        if k % 50 == 49 {
            check(&grid, "edited");
        }
    }
    // A whole chunk put in place, as a save does, and one made where there was none.
    let mut copy = Grid::new();
    for y in 0..32 {
        for x in 0..32 {
            copy.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    copy.set_north(TilePos::new(5, 5, 0), Edge::Wall);
    let (_, chunk) = copy.changed_chunks().remove(0);
    grid.put_chunk(ChunkPos { x: -1, y: 0 }, chunk);
    check(&grid, "a chunk put in place");
    grid.set_tile(TilePos::new(40, 3, 0), Tile::GRASS);
    check(&grid, "a new chunk");
}
