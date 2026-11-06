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
