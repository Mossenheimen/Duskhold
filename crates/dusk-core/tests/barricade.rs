//! Barricades: what boarding up takes, what a boarded door and window do, the dead beating
//! through the boards before the door, prying a board off, and Carpentry.

use std::sync::Arc;

use dusk_core::barricade::Boarding;
use dusk_core::senses::line_of_sight;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Edge, Event, Intent, Item, ItemDef, ItemKind, NoBoard, Skill,
    TilePos, World,
};

const PLANK: u16 = 0;
const NAILS: u16 = 1;
const HAMMER: u16 = 2;

/// Planks, nails and a hammer; you start with the hammer in hand and `planks` planks and `nails`
/// nails in the pack.
fn defs(planks: u16, nails: u16, hammer: bool) -> Defs {
    let item = |id: &str, weight, stack, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight,
        stack,
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
    let mut kit = Vec::new();
    if planks > 0 {
        kit.push(Item::new(PLANK, planks));
    }
    if nails > 0 {
        kit.push(Item::new(NAILS, nails));
    }
    Defs {
        items: vec![
            item("plank", 2000, 10, ItemKind::Other),
            item("nails", 10, 100, ItemKind::Other),
            item(
                "hammer",
                1000,
                1,
                ItemKind::Weapon {
                    swing,
                    wear: 8,
                    skill: Some(Skill::Blunt),
                },
            ),
        ],
        wield: hammer.then_some(HAMMER),
        kit,
        carry: 30_000,
        boarding: Some(Boarding {
            plank: PLANK,
            nails: NAILS,
            nails_each: 2,
            hammers: vec![HAMMER],
        }),
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A room with a shut door on its east side and a shambler down the passage beyond it; you
/// beside the door, inside.
fn room(d: Defs) -> (World, ActorId, ActorId) {
    let (mut w, placed) = World::from_text(
        2,
        "\
+-+-+-+-+-+-+-+
|. . .D. . . z|
+-+-+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(d);
    w.creature_mut(0).wander = 0;
    let me = w.spawn(ActorKind::Character, at(2, 0)).unwrap();
    (w, me, placed[0])
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..200 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

fn count(w: &World, me: ActorId, def: u16) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .filter(|i| i.def == def)
        .map(|i| i.count)
        .sum()
}

#[test]
fn a_board_takes_a_plank_and_two_nails_bars_the_door_and_is_heard() {
    let (mut w, me, _) = room(defs(6, 20, true));
    let seen = act(&mut w, me, Intent::Board);
    assert!(seen.contains(&Event::Boarded {
        who: me,
        at: at(3, 0),
        side: Dir8::W,
        boards: 1
    }));
    assert_eq!((count(&w, me, PLANK), count(&w, me, NAILS)), (5, 18));
    let loud = seen
        .iter()
        .filter(|e| matches!(e, Event::Sound { loudness: 12, .. }))
        .count();
    assert_eq!(loud, 2, "the hammering starting, and the last nail");
    // A boarded door won't open.
    act(&mut w, me, Intent::Door);
    assert_eq!(
        w.grid.edge(at(2, 0), Dir8::E),
        Edge::Door {
            open: false,
            boards: 1
        }
    );
}

#[test]
fn what_boarding_up_takes() {
    let check = |d: Defs| {
        let (w, me, _) = room(d);
        w.can_board(me)
    };
    assert_eq!(check(defs(1, 2, true)), Ok(Dir8::E), "the door beside you");
    assert_eq!(check(defs(1, 2, false)), Err(NoBoard::NoHammer));
    assert_eq!(check(defs(0, 2, true)), Err(NoBoard::NoPlank));
    assert_eq!(check(defs(1, 1, true)), Err(NoBoard::NoNails));

    let (mut w, me, _) = room(defs(9, 40, true));
    act(&mut w, me, Intent::Door);
    assert_eq!(w.can_board(me), Err(NoBoard::Open), "shut it first");
    act(&mut w, me, Intent::Door);
    for _ in 0..4 {
        act(&mut w, me, Intent::Board);
    }
    assert_eq!(w.grid.edge(at(2, 0), Dir8::E).boards(), 4);
    assert_eq!(w.can_board(me), Err(NoBoard::Full));

    let (mut w, me, _) = room(defs(1, 2, true));
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::W,
            run: false,
        },
    );
    w.step(20);
    assert_eq!(w.can_board(me), Err(NoBoard::Nothing), "nothing beside you");
}

#[test]
fn a_boarded_window_cant_be_seen_through() {
    let (mut w, _) = World::from_text(
        2,
        "\
+-+-+-+-+
|. .W. .|
+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs(2, 4, true));
    let me = w.spawn(ActorKind::Character, at(1, 0)).unwrap();
    assert!(
        line_of_sight(&w.grid, at(0, 0), at(3, 0)),
        "through the glass"
    );
    act(&mut w, me, Intent::Board);
    assert!(
        !line_of_sight(&w.grid, at(0, 0), at(3, 0)),
        "not through boards"
    );
}

#[test]
fn the_dead_beat_through_the_boards_before_the_door() {
    let (mut w, me, z) = room(defs(6, 20, true));
    // Quiet hammering here, so the boards are both on before the shambler comes.
    w.rules.barricade.noise = 0;
    act(&mut w, me, Intent::Board);
    act(&mut w, me, Intent::Board);
    assert_eq!(w.boards(at(2, 0), Dir8::E), vec![500, 500]);
    let mut order = Vec::new();
    let mut blows = 0;
    for t in 0..4000 {
        // Someone noisy inside: the dead give up on a door after a while of hearing nothing.
        if t % 200 == 0 {
            w.make_sound(at(2, 0), 12, Some(me));
        }
        w.step(1);
        for e in w.events() {
            match e {
                Event::Thumped { who, .. } if who == z => blows += 1,
                Event::BoardBroke { boards, .. } => order.push(format!("board, {boards} left")),
                Event::DoorBroke { .. } => order.push("door".to_string()),
                _ => {}
            }
        }
        if order.last().is_some_and(|o| o == "door") {
            break;
        }
    }
    assert_eq!(order, ["board, 1 left", "board, 0 left", "door"]);
    // Two boards of 50 and a door of 60, at 2 to 4 a blow.
    assert!(blows >= (500 + 500 + 600) / 40, "{blows} blows");
    assert_eq!(w.grid.edge(at(2, 0), Dir8::E), Edge::Open);
}

#[test]
fn prying_a_board_off_gives_the_plank_back_but_not_the_nails() {
    let (mut w, me, _) = room(defs(6, 20, true));
    act(&mut w, me, Intent::Board);
    let seen = act(&mut w, me, Intent::Unboard);
    assert!(seen.contains(&Event::Unboarded {
        who: me,
        at: at(3, 0),
        side: Dir8::W,
        boards: 0
    }));
    assert_eq!((count(&w, me, PLANK), count(&w, me, NAILS)), (6, 18));
    act(&mut w, me, Intent::Door);
    assert_eq!(
        w.grid.edge(at(2, 0), Dir8::E),
        Edge::door(true),
        "it opens again"
    );
    assert_eq!(w.can_unboard(me), Err(NoBoard::Nothing));
}

#[test]
fn carpentry_learns_from_each_board_and_nails_stronger_ones() {
    let (mut w, me, _) = room(defs(6, 20, true));
    // Quiet hammering, or the shambler outside comes to beat on the boards.
    w.rules.barricade.noise = 0;
    let ticks = |w: &mut World| {
        w.intent(me, Intent::Board);
        let mut n = 0;
        loop {
            w.step(1);
            n += 1;
            let a = w.actor(me).unwrap();
            if a.action.is_none() && a.intent == Intent::Idle {
                break n;
            }
        }
    };
    let slow = ticks(&mut w);
    let xp = |w: &World| w.actor(me).unwrap().skills.xp[Skill::Carpentry as usize];
    assert_eq!(xp(&w), 2000);
    ticks(&mut w);
    assert_eq!(xp(&w), 2000 + 1500, "the same door pays less");
    w.give_xp(me, Skill::Carpentry, 300_000); // level 5
    let quick = ticks(&mut w);
    let boards = w.boards(at(2, 0), Dir8::E);
    assert_eq!(boards, vec![500, 500, 750], "+10% a level");
    assert!(
        quick < slow,
        "level 5 nails one on in {quick} ticks, level 0 in {slow}"
    );
}
