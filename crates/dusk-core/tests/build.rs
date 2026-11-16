//! Doors and walls of your own (`build.rs`): a wall of planks put up on any open side you face, a
//! door only in a gap framed at both ends, both with a hammer; the dead beat a wall of planks down
//! as they do a door, and it's mended as a door is; a door hung is a door like any; and both last a
//! save. (You face south as you come into the world: the side these maps build on.)

use std::sync::Arc;

use dusk_core::barricade::Boarding;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Edge, Event, Intent, Item, ItemDef, ItemKind, NoPutUp, Part,
    Piece, Repair, Skill, Tile, TilePos, World, WoundKind,
};

const PLANK: u16 = 0;
const NAILS: u16 = 1;
const HAMMER: u16 = 2;
const DOOR: u16 = 3;
const WALL: u16 = 4;

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// Planks, nails, a hammer, a door and a wall of planks; you start with the hammer in hand (if
/// `hammer`), a door, a wall, two planks and ten nails.
fn defs(hammer: bool) -> Defs {
    let item = |id: &str, weight, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight,
        stack: 10,
        kind,
        worth: 0,
    };
    let swing = Swing {
        wind_up: 3,
        recovery: 5,
        endurance: 80,
        hit: 8500,
        damage: (70, 110),
        noise: 4,
    };
    Defs {
        items: vec![
            item("plank", 2000, ItemKind::Other),
            item("nails", 10, ItemKind::Other),
            item(
                "hammer",
                1000,
                ItemKind::Weapon {
                    swing,
                    wear: 8,
                    skill: Some(Skill::Blunt),
                },
            ),
            item("door", 12_000, ItemKind::Piece(Piece::Door)),
            item("plank_wall", 10_000, ItemKind::Piece(Piece::Wall)),
        ],
        wield: hammer.then_some(HAMMER),
        kit: vec![
            Item::new(DOOR, 1),
            Item::new(WALL, 1),
            Item::new(PLANK, 2),
            Item::new(NAILS, 10),
        ],
        carry: 60_000,
        boarding: Some(Boarding {
            plank: PLANK,
            nails: NAILS,
            nails_each: 2,
            hammers: vec![HAMMER],
        }),
        ..Defs::default()
    }
}

/// Two rooms, a wall between them with a gap one tile wide above (2, 1); you on `you`.
fn rooms(you: TilePos, hammer: bool) -> (World, ActorId) {
    let (mut w, _) = World::from_text(
        3,
        "\
+-+-+-+-+-+
|. . . . .|
+-+-+ +-+-+
|. . . . .|
+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs(hammer));
    let me = w.spawn(ActorKind::Character, you).unwrap();
    (w, me)
}

/// Open grass, you at its middle.
fn field() -> (World, ActorId) {
    let mut w = World::new(3);
    w.defs = Arc::new(defs(true));
    for y in -3..=3 {
        for x in -3..=3 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
    (w, me)
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..300 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

/// Puts up what's at `slot` of the pack (`def`).
fn put_up(w: &mut World, me: ActorId, def: u16) -> Vec<Event> {
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == def)
        .unwrap() as u16;
    act(w, me, Intent::Use { slot, def })
}

fn has(w: &World, me: ActorId, def: u16) -> bool {
    w.actor(me).unwrap().items.iter().any(|i| i.def == def)
}

#[test]
fn a_wall_of_planks_goes_up_on_any_open_side_you_face() {
    let (mut w, me) = field();
    assert_eq!(w.can_put_up(me, Piece::Wall), Ok(Dir8::S));
    let seen = put_up(&mut w, me, WALL);
    assert!(seen.contains(&Event::PutUp {
        who: me,
        at: at(0, 1),
        side: Dir8::N,
        door: false
    }));
    assert_eq!(w.grid.edge(at(0, 0), Dir8::S), Edge::PlankWall);
    assert!(!has(&w, me, WALL), "out of your pack");
    assert!(!w.grid.can_step(at(0, 0), Dir8::S), "it stops you");
    assert!(w.actor(me).unwrap().skills.xp[Skill::Carpentry as usize] > 0);
    // A wall there now: nothing open to put another on.
    assert_eq!(w.can_put_up(me, Piece::Wall), Err(NoPutUp::Nothing));
}

#[test]
fn a_door_hangs_only_in_a_gap_framed_at_both_ends() {
    // Out in the open, no gap: a wall would go up, but not a door.
    let (w, me) = field();
    assert_eq!(w.can_put_up(me, Piece::Door), Err(NoPutUp::NoGap));
    // Facing the gap in the wall between the rooms: hung, shut.
    let (mut w, me) = rooms(at(2, 0), true);
    assert_eq!(w.can_put_up(me, Piece::Door), Ok(Dir8::S));
    put_up(&mut w, me, DOOR);
    assert_eq!(w.grid.edge(at(2, 0), Dir8::S), Edge::door(false));
    // Facing the wall itself: nothing open.
    let (w, me) = rooms(at(0, 0), true);
    assert_eq!(w.can_put_up(me, Piece::Door), Err(NoPutUp::Nothing));
    // And no hammer, none.
    let (w, me) = rooms(at(2, 0), false);
    assert_eq!(w.can_put_up(me, Piece::Door), Err(NoPutUp::NoHammer));
}

#[test]
fn a_door_hung_is_a_door_like_any() {
    let (mut w, me) = rooms(at(2, 0), true);
    put_up(&mut w, me, DOOR);
    act(&mut w, me, Intent::Door);
    assert_eq!(w.grid.edge(at(2, 0), Dir8::S), Edge::door(true), "opened");
    act(&mut w, me, Intent::Door);
    act(&mut w, me, Intent::Board);
    assert_eq!(w.grid.edge(at(2, 0), Dir8::S).boards(), 1, "and boarded up");
}

/// A passage with a wall of planks across it east of (3, 0), and a shambler beyond it, drawn by
/// your noise; you at (2, 0).
fn passage() -> (World, ActorId, ActorId) {
    let (mut w, placed) = World::from_text(
        2,
        "\
+-+-+-+-+-+-+-+
|. . . .P. . z|
+-+-+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs(true));
    w.creature_mut(0).wander = 0;
    let me = w.spawn(ActorKind::Character, at(2, 0)).unwrap();
    (w, me, placed[0])
}

#[test]
fn the_dead_beat_a_wall_of_planks_down_as_they_do_a_door() {
    let (mut w, me, z) = passage();
    assert_eq!(w.grid.edge(at(3, 0), Dir8::E), Edge::PlankWall);
    let mut blows = 0;
    let mut broke = false;
    for t in 0..6000 {
        if t % 200 == 0 {
            w.make_sound(at(2, 0), 12, Some(me));
        }
        w.step(1);
        for e in w.events() {
            match e {
                Event::Thumped { who, .. } if who == z => blows += 1,
                Event::WallBroke { .. } => broke = true,
                _ => {}
            }
        }
        if broke {
            break;
        }
    }
    assert!(broke, "it gave way");
    // 80 points, at 2 to 4 a blow.
    assert!(blows >= 800 / 40, "{blows} blows");
    assert_eq!(w.grid.edge(at(3, 0), Dir8::E), Edge::Open);
}

#[test]
fn a_beaten_wall_of_planks_is_mended_as_a_door_is() {
    let (mut w, me, z) = passage();
    // Walk up to the wall (facing east), and let the shambler beat on it a while.
    act(
        &mut w,
        me,
        Intent::WalkTo {
            to: at(3, 0),
            run: false,
        },
    );
    for t in 0..4000 {
        if t % 200 == 0 {
            w.make_sound(at(3, 0), 12, Some(me));
        }
        w.step(1);
        if w.door_beaten(at(3, 0), Dir8::E) > 0 {
            break;
        }
    }
    while !w.actor(z).unwrap().dead {
        w.wound(z, WoundKind::Bite, Part::Torso);
    }
    assert!(w.door_beaten(at(3, 0), Dir8::E) > 0, "beaten");
    assert_eq!(w.can_repair(me), Ok((Dir8::E, Repair::Door)));
    act(&mut w, me, Intent::Repair);
    assert_eq!(w.door_beaten(at(3, 0), Dir8::E), 0, "whole again");
    assert_eq!(w.grid.edge(at(3, 0), Dir8::E), Edge::PlankWall);
}

#[test]
fn a_wall_and_a_door_of_your_own_last_a_save() {
    let (mut a, me) = rooms(at(2, 0), true);
    put_up(&mut a, me, DOOR);
    let bytes = a.save("build", b"");
    let (mut b, _) = rooms(at(2, 0), true);
    b.restore(&bytes).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(b.grid.edge(at(2, 0), Dir8::S), Edge::door(false));
    assert_eq!(b.state_hash(), a.state_hash());
    let (mut c, me) = field();
    put_up(&mut c, me, WALL);
    let bytes = c.save("build", b"");
    let (mut d, _) = field();
    d.restore(&bytes).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(d.grid.edge(at(0, 0), Dir8::S), Edge::PlankWall);
}
