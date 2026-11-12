use dusk_core::text::{TextGrid, default_legend, render};
use dusk_core::{Dir8, Edge, Grid, TilePos};

const HOUSE: &str = "\
+-+-+-+-+
|. .|. .|
+   +D+-+
|. . . .d
+-+W+-+-+
";

fn stamped(map: &str) -> Grid {
    let parsed = TextGrid::parse(map, default_legend).expect("the map parses");
    let mut grid = Grid::new();
    parsed.stamp(&mut grid, TilePos::new(0, 0, 0));
    grid
}

#[test]
fn renders_back_what_it_parsed() {
    let grid = stamped(HOUSE);
    assert_eq!(render(&grid, TilePos::new(0, 0, 0), 4, 2, |_| None), HOUSE);
}

#[test]
fn edges_land_on_the_right_sides() {
    let g = stamped(HOUSE);
    let p = |x, y| TilePos::new(x, y, 0);
    assert_eq!(g.west(p(2, 0)), Edge::Wall, "between the two rooms");
    assert_eq!(g.north(p(2, 1)), Edge::door(false), "the shut door");
    assert_eq!(
        g.edge(p(3, 1), Dir8::E),
        Edge::door(true),
        "the open door on the east side"
    );
    assert_eq!(
        g.edge(p(1, 1), Dir8::S),
        Edge::window(false),
        "the window on the south side"
    );
    assert_eq!(g.edge(p(0, 0), Dir8::E), Edge::Open);
}

#[test]
fn says_where_a_map_is_wrong() {
    let err = TextGrid::parse("+-+\n|?|\n+-+\n", default_legend).unwrap_err();
    assert_eq!((err.line, err.column, err.found), (2, 2, '?'));
}

/// HOUSE turned a quarter clockwise: the west room is now the north one, the open door on the
/// east side is on the south side, and the window on the south side is on the west.
const HOUSE_TURNED: &str = "\
+-+-+
|. .|
+   +
W. .|
+ +-+
|.D.|
+ + +
|.|.|
+d+-+
";

fn symbol(cell: dusk_core::text::Cell) -> char {
    dusk_core::text::tile_symbol(cell.tile)
}

#[test]
fn a_quarter_turn_moves_every_edge_round() {
    let house = TextGrid::parse(HOUSE, default_legend).unwrap();
    assert_eq!(house.rotated().render(symbol), HOUSE_TURNED);
}

#[test]
fn four_turns_or_two_mirrors_change_nothing() {
    let house = TextGrid::parse(HOUSE, default_legend).unwrap();
    let turned = house.rotated().rotated().rotated().rotated();
    assert_eq!(turned, house);
    assert_eq!(house.mirrored().mirrored(), house);
    assert_eq!(
        house.render(symbol),
        HOUSE,
        "render reads back the same text"
    );
    // Mirrored, the open door moves from the east side to the west.
    assert_eq!(house.mirrored().west(0, 1), Edge::door(true));
}

#[test]
fn a_broken_window_renders_back() {
    let map = "+-+-+
|.x.|
+-+-+
";
    let grid = stamped(map);
    assert_eq!(grid.west(TilePos::new(1, 0, 0)), Edge::broken_window());
    assert_eq!(render(&grid, TilePos::new(0, 0, 0), 2, 1, |_| None), map);
}
