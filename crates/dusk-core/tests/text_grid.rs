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
    assert_eq!(
        g.north(p(2, 1)),
        Edge::Door { open: false },
        "the shut door"
    );
    assert_eq!(
        g.edge(p(3, 1), Dir8::E),
        Edge::Door { open: true },
        "the open door on the east side"
    );
    assert_eq!(
        g.edge(p(1, 1), Dir8::S),
        Edge::Window { open: false },
        "the window on the south side"
    );
    assert_eq!(g.edge(p(0, 0), Dir8::E), Edge::Open);
}

#[test]
fn says_where_a_map_is_wrong() {
    let err = TextGrid::parse("+-+\n|?|\n+-+\n", default_legend).unwrap_err();
    assert_eq!((err.line, err.column, err.found), (2, 2, '?'));
}
