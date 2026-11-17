//! Containers as Path of Exile's stash (`pack.rs`): what's in one is laid out in its grid; what's
//! put away goes where you ask if it fits there, else where it first fits, and never past its
//! room; it's taken onto the cell you ask for; it's arranged at once within reach, and not
//! beyond it; and a trial run's gift that doesn't fit is carried all the same, with no place.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Defs, Event, Intent, Item, ItemDef, ItemKind, LootEntry, LootTable, Tile,
    TilePos, World,
};

const SWORD: u16 = 0;
const APPLE: u16 = 1;

/// A sword (one cell across, three down) and apples (ten a stack, a cell each). Fixture 1 is a
/// chest two cells across and three down, that always holds four apples.
fn defs() -> Defs {
    let item = |id: &str, stack, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 100,
        stack,
        kind,
        worth: 0,
    };
    Defs {
        items: vec![
            item("sword", 1, ItemKind::Other),
            item(
                "apple",
                10,
                ItemKind::Food {
                    nutrition: 1000,
                    time: 5,
                    comfort: 0,
                    spoils: 0,
                },
            ),
        ],
        sizes: vec![(1, 3), (1, 1)],
        loot: vec![LootTable {
            id: "chest".into(),
            rolls: (1, 1),
            level: 0,
            entries: vec![LootEntry {
                item: APPLE,
                weight: 1,
                count: (4, 4),
                magic: 0,
                unique: 0,
            }],
        }],
        containers: vec![None, Some(0)],
        grids: vec![(0, 0), (2, 3)],
        pockets: (2, 3),
        carry: 50_000,
        take: 4,
        take_per_kg: 1,
        ..Defs::default()
    }
}

const CHEST: Tile = Tile {
    fixture: 1,
    solid: true,
    ..Tile::FLOOR
};

fn at(x: i32) -> TilePos {
    TilePos::new(x, 0, 0)
}

/// A long room, a chest beside you at its west end and another 12 tiles off.
fn room() -> (World, ActorId) {
    let (mut w, _) = World::from_text(
        2,
        "\
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|. . . . . . . . . . . . . .|
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs());
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    w.grid.set_tile(at(1), CHEST);
    w.grid.set_tile(at(13), CHEST);
    let me = w.spawn(ActorKind::Character, at(0)).unwrap();
    (w, me)
}

/// Asks for an act and runs until it's over (or never began); its events.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..100 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

fn slot_of(w: &World, me: ActorId, def: u16) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == def)
        .expect("in the pack") as u16
}

/// A stack as the chest holds it: its definition, how many, and its cell.
type Held = (u16, u16, Option<(u8, u8)>);

/// What's in the chest beside you.
fn chest(w: &World) -> Vec<Held> {
    w.store(at(1))
        .iter()
        .map(|i| (i.def, i.count, i.cell))
        .collect()
}

#[test]
fn a_containers_things_are_laid_out_in_its_grid_and_a_floor_has_none() {
    let (w, _) = room();
    assert_eq!(w.store_grid(at(1)), Some((2, 3)));
    assert_eq!(w.store_grid(at(2)), None, "a floor");
    assert_eq!(chest(&w), vec![(APPLE, 4, Some((0, 0)))]);
}

#[test]
fn what_is_put_away_goes_where_asked_if_it_fits_and_never_past_its_room() {
    let (mut w, me) = room();
    w.give(me, Item::new(SWORD, 1));
    w.give(me, Item::new(SWORD, 1));
    let sword = slot_of(&w, me, SWORD);
    // Not onto the apples; down the free column, yes.
    assert_eq!(w.can_put(me, at(1), sword, SWORD, Some((0, 0))), 0);
    assert_eq!(
        w.can_put(me, at(1), sword, SWORD, Some((0, 1))),
        0,
        "off the grid"
    );
    assert_eq!(w.can_put(me, at(1), sword, SWORD, Some((1, 0))), 1);
    act(
        &mut w,
        me,
        Intent::PutTo {
            at: at(1),
            slot: sword,
            def: SWORD,
            cell: (1, 0),
        },
    );
    assert_eq!(
        chest(&w),
        vec![(APPLE, 4, Some((0, 0))), (SWORD, 1, Some((1, 0)))]
    );
    // No room for the other anywhere: putting it away never begins.
    let sword = slot_of(&w, me, SWORD);
    assert_eq!(w.can_put(me, at(1), sword, SWORD, None), 0);
    w.intent(
        me,
        Intent::Put {
            at: at(1),
            slot: sword,
            def: SWORD,
        },
    );
    w.step(1);
    assert!(w.actor(me).unwrap().action.is_none());
    // Eight apples, finds as the chest's are: six onto the four, two on the next free cell.
    w.give(me, Item::found(APPLE, 8));
    let apples = slot_of(&w, me, APPLE);
    assert_eq!(w.can_put(me, at(1), apples, APPLE, None), 8);
    act(
        &mut w,
        me,
        Intent::Put {
            at: at(1),
            slot: apples,
            def: APPLE,
        },
    );
    assert_eq!(
        chest(&w),
        vec![
            (APPLE, 10, Some((0, 0))),
            (SWORD, 1, Some((1, 0))),
            (APPLE, 2, Some((0, 1)))
        ]
    );
}

#[test]
fn what_is_taken_goes_onto_the_cell_asked_for() {
    let (mut w, me) = room();
    assert_eq!(w.can_take_to(me, at(1), 0, APPLE, false, (1, 2)), 4);
    assert_eq!(
        w.can_take_to(me, at(1), 0, APPLE, true, (0, 0)),
        0,
        "no bag on"
    );
    assert_eq!(
        w.can_take_to(me, at(1), 0, APPLE, false, (2, 0)),
        0,
        "off the grid"
    );
    act(
        &mut w,
        me,
        Intent::TakeTo {
            at: at(1),
            slot: 0,
            def: APPLE,
            bagged: false,
            cell: (1, 2),
        },
    );
    let a = w.actor(me).unwrap();
    assert_eq!(a.items.len(), 1);
    assert_eq!((a.items[0].count, a.items[0].cell), (4, Some((1, 2))));
    assert!(w.store(at(1)).is_empty());
}

#[test]
fn a_container_is_arranged_at_once_within_reach_and_not_beyond_it() {
    let (mut w, me) = room();
    assert_eq!(w.can_arrange_store(me, at(1), 0, APPLE, (1, 2)), 4);
    assert!(w.arrange_store(me, at(1), 0, APPLE, (1, 2)));
    assert!(w.actor(me).unwrap().action.is_none(), "no act: a choice");
    assert_eq!(chest(&w), vec![(APPLE, 4, Some((1, 2)))]);
    assert_eq!(
        w.can_arrange_store(me, at(1), 0, APPLE, (2, 2)),
        0,
        "off the grid"
    );
    // The other chest, out of reach: no.
    assert_eq!(w.can_arrange_store(me, at(13), 0, APPLE, (1, 0)), 0);
    assert!(!w.arrange_store(me, at(13), 0, APPLE, (1, 0)));
    assert_eq!(w.store(at(13))[0].cell, Some((0, 0)));
}

#[test]
fn a_gift_past_your_room_is_carried_all_the_same_with_no_place() {
    let (mut w, me) = room();
    // Pockets two by three: two swords fill them; a third has no place, but it's yours.
    for _ in 0..3 {
        assert!(w.give(me, Item::new(SWORD, 1)));
    }
    let cells: Vec<Option<(u8, u8)>> = w.actor(me).unwrap().items.iter().map(|i| i.cell).collect();
    assert_eq!(cells, vec![Some((0, 0)), Some((1, 0)), None]);
    assert_eq!(w.pack_cells(me, false, None).unwrap().used(), 6);
}
