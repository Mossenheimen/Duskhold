//! The looks (`looks.toml`): the game's own, the kit's shapes and turns as the client makes them,
//! and what a mistake in a look says.

use std::path::Path;

use dusk_data::{
    FIXTURE_TRIANGLES, Part, Shape, Surface, default_dir, load, parse_looks, unit_points,
};

fn part(shape: Shape, sides: u8) -> Part {
    Part {
        shape,
        sides,
        taper: 1.0,
        size: [1.0, 1.0, 1.0],
        at: [0.0; 3],
        turn: [0.0; 3],
        colour: [0.5; 3],
        surface: Surface::Plain,
        cut: true,
    }
}

#[test]
fn a_turn_is_godots() {
    // A quarter turn about y takes +z to +x, as Godot's does; about x, +y to +z.
    let mut p = part(Shape::Box, 4);
    p.turn = [0.0, 90.0, 0.0];
    let q = p.place([0.0, 0.0, 0.5]);
    assert!((q[0] - 0.5).abs() < 1e-5 && q[2].abs() < 1e-5, "{q:?}");
    p.turn = [90.0, 0.0, 0.0];
    let q = p.place([0.0, 0.5, 0.0]);
    assert!((q[2] - 0.5).abs() < 1e-5 && q[1].abs() < 1e-5, "{q:?}");
    // Y first, then x: a tilt forward, then a turn, as a node's rotation_degrees (0, 90, 0)
    // after (90, 0, 0) would be: Ry·Rx takes +y to +z, then to +x.
    p.turn = [90.0, 90.0, 0.0];
    let q = p.place([0.0, 0.5, 0.0]);
    assert!((q[0] - 0.5).abs() < 1e-5, "{q:?}");
}

#[test]
fn shapes_have_their_corners() {
    assert_eq!(unit_points(Shape::Box, 4, 1.0).len(), 8);
    assert_eq!(unit_points(Shape::Prism, 4, 1.0).len(), 6);
    assert_eq!(unit_points(Shape::Cylinder, 8, 1.0).len(), 16);
    assert_eq!(unit_points(Shape::Cylinder, 8, 0.0).len(), 9);
    assert_eq!(unit_points(Shape::Ball, 8, 1.0).len(), 2 + 3 * 8);
    // An even-sided cylinder has a flat side to +x: its first and last corners mirror in z.
    let c = unit_points(Shape::Cylinder, 8, 1.0);
    assert!((c[0][2] + c[7][2]).abs() < 1e-5 && (c[0][0] - c[7][0]).abs() < 1e-5);
    // Every shape stays within its size.
    for shape in [Shape::Box, Shape::Prism, Shape::Cylinder, Shape::Ball] {
        for p in unit_points(shape, 8, 1.0) {
            assert!(p.iter().all(|x| x.abs() <= 0.5 + 1e-5), "{shape:?}: {p:?}");
        }
    }
    assert_eq!(part(Shape::Ball, 8).triangles(), 48);
    assert_eq!(part(Shape::Cylinder, 8).triangles(), 28);
    let mut cone = part(Shape::Cylinder, 8);
    cone.taper = 0.0;
    assert_eq!(cone.triangles(), 14);
}

#[test]
fn every_fixture_has_a_look_and_keeps_to_its_size() {
    let data = load(&default_dir()).expect("the game's data loads");
    for f in &data.fixtures {
        let look = data.looks.fixtures.get(&f.id).expect("a look");
        assert!(look.triangles() <= FIXTURE_TRIANGLES, "{}", f.id);
        // Under the walls (2.2 m), all but the trees (VISUALS.md §5, size bands).
        if f.id != "tree" {
            assert!(look.height() < 2.2, "{} stands {} m", f.id, look.height());
        }
        // A fire's embers sit within its tile.
        if let Some(at) = look.fire {
            assert!(at[0].abs() < 0.5 && at[2].abs() < 0.5, "{}", f.id);
        }
    }
}

#[test]
fn mirrors_and_repeats_make_their_copies() {
    let data = load(&default_dir()).expect("the game's data loads");
    let looks = parse_looks(
        Path::new("looks.toml"),
        r##"
[material.wood]
oak = { colour = "#735233", surface = "wood" }

[[fixture]]
id = "table"
parts = [
  { shape = "box", size = [0.1, 0.7, 0.1], at = [0.4, 0.35, 0.3], turn = [0, 10, 5], material = "wood.oak", mirror = "xz" },
  { shape = "box", size = [0.8, 0.04, 0.3], at = [0, 0.1, 0], material = "wood.oak", repeat = { count = 3, step = [0, 0.5, 0] } },
]
"##,
        &data.fixtures,
    )
    .expect("it reads");
    let parts = &looks.fixtures["table"].variants[0];
    assert_eq!(parts.len(), 4 + 3);
    let legs: Vec<([f32; 3], [f32; 3])> = parts[..4].iter().map(|p| (p.at, p.turn)).collect();
    assert!(legs.contains(&([0.4, 0.35, 0.3], [0.0, 10.0, 5.0])));
    assert!(legs.contains(&([-0.4, 0.35, 0.3], [0.0, -10.0, -5.0])));
    assert!(legs.contains(&([0.4, 0.35, -0.3], [0.0, -10.0, 5.0])));
    assert!(legs.contains(&([-0.4, 0.35, -0.3], [0.0, 10.0, -5.0])));
    let ys: Vec<f32> = parts[4..].iter().map(|p| p.at[1]).collect();
    assert_eq!(ys, [0.1, 0.6, 1.1]);
    assert_eq!(parts[0].surface, Surface::Wood);
    // A mirror image is the same shape seen in a mirror: its corners are the first's, x negated.
    let (a, b) = (parts[0], parts[1]);
    for p in a.points() {
        let mirrored = [-p[0], p[1], p[2]];
        let near = b
            .points()
            .iter()
            .any(|q| (0..3).all(|i| (q[i] - mirrored[i]).abs() < 1e-4));
        assert!(near, "{p:?}");
    }
}

#[test]
fn a_mistake_in_a_look_says_what_and_where() {
    let data = load(&default_dir()).expect("the game's data loads");
    let says = |text: &str| {
        let e = parse_looks(Path::new("looks.toml"), text, &data.fixtures)
            .expect_err("it shouldn't read");
        e.to_string()
    };
    let wood = "[material.wood]\noak = { colour = \"#735233\" }\n";
    let table = |part: &str| format!("{wood}[[fixture]]\nid = \"table\"\nparts = [ {part} ]\n");
    assert!(
        says(&table(
            r#"{ shape = "box", size = [1, 1, 1], material = "wood.ash" }"#
        ))
        .contains("material \"wood.ash\"")
    );
    assert!(
        says(&table(
            r#"{ shape = "box", size = [1, 1, 1], material = "wood.oak", sides = 6 }"#
        ))
        .contains("only a cylinder or a ball has sides")
    );
    assert!(
        says(&table(
            r#"{ shape = "box", size = [0, 1, 1], material = "wood.oak" }"#
        ))
        .contains("size")
    );
    assert!(says("[material.wood]\noak = { colour = \"735233\" }\n").contains("\"#RRGGBB\""));
    let unknown = format!(
        "{wood}[[fixture]]\nid = \"throne\"\nparts = [ {{ shape = \"box\", size = [1, 1, 1], \
         material = \"wood.oak\" }} ]\n"
    );
    assert!(says(&unknown).contains("no such fixture"));
    // A hearth takes a fire, so its look must say where it burns.
    let hearth = format!(
        "{wood}[[fixture]]\nid = \"hearth\"\nparts = [ {{ shape = \"box\", size = [1, 1, 1], \
         material = \"wood.oak\" }} ]\n"
    );
    assert!(says(&hearth).contains("where it burns"));
    // Too many triangles for a fixture.
    let many = format!(
        "{wood}[[fixture]]\nid = \"table\"\nparts = [ {{ shape = \"ball\", size = [1, 1, 1], \
         sides = 16, material = \"wood.oak\", repeat = {{ count = 5, step = [0, 0.1, 0] }} }} ]\n"
    );
    assert!(says(&many).contains("triangles"));
}
