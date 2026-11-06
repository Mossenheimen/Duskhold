//! A small hand-built town, for the client and the demos until the world is generated from a
//! seed (build step 4): three houses and a shed on either side of a street, with the character in
//! the street and the dead about.

use crate::actor::ActorKind;
use crate::geom::TilePos;
use crate::grid::{Edge, Grid, Tile};
use crate::world::World;

/// The town's size in tiles.
pub const TOWN_W: i32 = 32;
pub const TOWN_H: i32 = 24;

fn p(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A building's floor (indoors) and the walls round it: `w × h` tiles from (`x`, `y`).
fn building(grid: &mut Grid, x: i32, y: i32, w: i32, h: i32) {
    for ty in y..y + h {
        for tx in x..x + w {
            grid.set_tile(p(tx, ty), Tile::FLOOR);
        }
    }
    for tx in x..x + w {
        grid.set_north(p(tx, y), Edge::Wall);
        grid.set_north(p(tx, y + h), Edge::Wall);
    }
    for ty in y..y + h {
        grid.set_west(p(x, ty), Edge::Wall);
        grid.set_west(p(x + w, ty), Edge::Wall);
    }
}

fn furniture(grid: &mut Grid, at: &[(i32, i32)]) {
    for &(x, y) in at {
        grid.set_tile(
            p(x, y),
            Tile {
                solid: true,
                ..Tile::FLOOR
            },
        );
    }
}

/// The town, with the character in the street and nine of the dead about.
pub fn town(seed: u64) -> World {
    let mut w = World::new(seed);
    let g = &mut w.grid;
    for y in 0..TOWN_H {
        for x in 0..TOWN_W {
            g.set_tile(p(x, y), Tile::EARTH);
        }
    }
    let open = Edge::Door { open: true };
    let shut = Edge::Door { open: false };
    let window = Edge::Window { open: false };

    // The farmhouse, north-west: two rooms, the front door on the street.
    building(g, 2, 2, 9, 6);
    for y in 2..8 {
        g.set_west(p(7, y), Edge::Wall);
    }
    g.set_west(p(7, 4), open);
    g.set_north(p(5, 8), open);
    g.set_north(p(3, 8), window);
    g.set_north(p(9, 8), window);
    g.set_west(p(2, 4), window);
    furniture(g, &[(3, 2), (4, 2), (10, 2), (10, 3), (8, 7)]);

    // The house north-east: a side door, and a back door that's shut.
    building(g, 15, 3, 9, 6);
    g.set_west(p(15, 6), open);
    g.set_north(p(19, 3), shut);
    g.set_north(p(17, 9), window);
    g.set_north(p(21, 9), window);
    furniture(g, &[(16, 3), (22, 3), (23, 3), (23, 8)]);

    // The house south of the street: a front room and a back room.
    building(g, 6, 14, 8, 6);
    for x in 6..14 {
        g.set_north(p(x, 17), Edge::Wall);
    }
    g.set_north(p(8, 17), open);
    g.set_north(p(10, 14), open);
    g.set_north(p(7, 14), window);
    g.set_north(p(12, 14), window);
    furniture(g, &[(6, 16), (13, 15), (12, 19), (13, 19)]);

    // A shed out east.
    building(g, 26, 15, 3, 3);
    g.set_west(p(26, 16), open);
    furniture(g, &[(28, 15)]);

    w.spawn(ActorKind::Character, p(13, 11));
    for (x, y) in [
        (9, 5),
        (20, 6),
        (22, 5),
        (11, 18),
        (27, 16),
        (28, 11),
        (30, 9),
        (2, 21),
        (4, 19),
    ] {
        w.spawn(ActorKind::Shambler, p(x, y));
    }
    w
}
