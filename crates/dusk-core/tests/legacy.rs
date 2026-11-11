//! Death and legacy: a named character and how long they lasted, the body that rises at
//! midnight as a revenant in its gear, the gear coming back when it's put down (or when the body
//! is burned first), and the next character at the start with the old home.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, CreatureDef, Defs, Event, Intent, Item, ItemDef, ItemKind, Part, Region,
    Remains, Tile, TilePos, World, WoundKind,
};

const REVENANT: u16 = 1;
const OIL: u16 = 0;
const CANDLE: u16 = 1;
const COINS: u16 = 2;
const START: TilePos = TilePos::new(0, 1, 0);
/// Ticks in a game minute.
const MINUTE: u32 = 25;

/// Names to roll from, oil, a candle and coins in the kit, and a revenant (the test shambler's
/// numbers, 1 health) that a body rises as at midnight.
fn defs() -> Defs {
    let item = |id: &str, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 10,
        stack: 50,
        kind,
        worth: 1000,
    };
    let mut revenant = CreatureDef::shambler();
    revenant.id = "revenant".into();
    revenant.name = "revenant".into();
    revenant.stats.health = 1;
    revenant.stats.wander = 0;
    let mut defs = Defs {
        items: vec![
            item("oil", ItemKind::Oil),
            item("candle", ItemKind::Light { radius: 2, wane: 1 }),
            item("coins", ItemKind::Other),
        ],
        kit: vec![Item::new(OIL, 1), Item::new(CANDLE, 1), Item::new(COINS, 7)],
        given: vec!["Edda".into(), "Tam".into(), "Wyn".into()],
        family: vec!["Marsh".into(), "Hale".into()],
        revenant: Some((REVENANT, 0)),
        ..Defs::default()
    };
    defs.creatures.push(revenant);
    defs
}

/// A field of grass, the clock at 20:00 on day 1, the needs stilled, the start at (0, 1), and
/// you on it.
fn field() -> (World, ActorId) {
    let mut w = World::new(6);
    w.defs = Arc::new(defs());
    w.rules.clock.start = 20 * 60;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -20..20 {
        for x in -20..20 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    w.set_region(Region {
        start: Some(START),
        ..Region::default()
    });
    let me = w.spawn(ActorKind::Character, START).unwrap();
    (w, me)
}

fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut seen = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

/// Wounds `id` until it dies.
fn die(w: &mut World, id: ActorId) {
    while !w.actor(id).unwrap().dead {
        w.wound(id, WoundKind::Bite, Part::Torso);
    }
}

#[test]
fn a_character_has_a_name_and_the_time_they_lasted() {
    let (mut w, me) = field();
    let name = w.name_of(me).expect("a name");
    assert!(name.contains(' '), "a given name and a family name: {name}");
    run(&mut w, 90 * MINUTE);
    assert_eq!(w.lasted(me), 90);
    die(&mut w, me);
    run(&mut w, 30 * MINUTE);
    assert_eq!(w.lasted(me), 90, "the dead last no longer");
}

#[test]
fn the_body_rises_at_midnight_as_a_revenant_in_its_gear_and_puts_it_down_where_it_falls() {
    let (mut w, me) = field();
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(5, 1, 0),
            run: false,
        },
    );
    run(&mut w, 60);
    die(&mut w, me);
    // 20:02 now: it rises at midnight, not before.
    let seen = run(&mut w, (4 * 60 - 3) * MINUTE);
    assert!(!seen.iter().any(|e| matches!(e, Event::Risen { .. })));
    let seen = run(&mut w, 2 * MINUTE);
    let Some(&Event::Risen { body, revenant }) =
        seen.iter().find(|e| matches!(e, Event::Risen { .. }))
    else {
        panic!("it rose");
    };
    assert_eq!(body, me);
    assert_eq!(w.actor(me).unwrap().remains, Remains::Risen);
    let r = w.actor(revenant).unwrap();
    assert_eq!(r.kind, ActorKind::Creature(REVENANT));
    assert_eq!(r.pos, TilePos::new(5, 1, 0), "where the body lay");
    assert_eq!(w.name_of(revenant), w.name_of(me), "it keeps the name");
    assert!(
        r.items.iter().any(|i| i.def == COINS && i.count == 7),
        "and the gear"
    );
    // Put it down, and the gear falls where it falls.
    let at = r.pos;
    w.wound(revenant, WoundKind::Bite, Part::Torso);
    assert!(w.actor(revenant).unwrap().dead);
    let floor = w.store(at);
    assert!(floor.iter().any(|i| i.def == COINS && i.count == 7));
    run(&mut w, 3 * 1440 * MINUTE);
    assert!(
        w.actor(revenant).unwrap().dead,
        "a revenant put down stays down"
    );
}

#[test]
fn the_next_character_comes_at_the_start_and_can_burn_the_body_before_it_rises() {
    let (mut w, me) = field();
    assert_eq!(w.next_character(), None, "not while you live");
    // You die just north of the start.
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(0, 0, 0),
            run: false,
        },
    );
    run(&mut w, 40);
    die(&mut w, me);
    let next = w.next_character().expect("the next one comes");
    assert_eq!(w.character(), Some(next));
    assert_eq!(w.actor(next).unwrap().pos, START);
    // Face the body, light the candle, and burn it.
    let candle = w
        .actor(next)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == CANDLE)
        .unwrap();
    w.intent(
        next,
        Intent::Light {
            slot: candle as u16,
            def: CANDLE,
        },
    );
    run(&mut w, 5);
    assert_eq!(w.can_burn(next), Ok(me));
    w.intent(next, Intent::Burn);
    let seen = run(&mut w, 40);
    assert!(seen.contains(&Event::Kindled {
        who: next,
        body: me
    }));
    // Its gear fell where it lay, and midnight comes and goes.
    assert!(
        w.store(TilePos::new(0, 0, 0))
            .iter()
            .any(|i| i.def == COINS)
    );
    let seen = run(&mut w, 5 * 60 * MINUTE);
    assert!(!seen.iter().any(|e| matches!(e, Event::Risen { .. })));
}

#[test]
fn the_next_character_keeps_the_old_home() {
    let (mut w, me) = field();
    // Sleep the night through at the start: home.
    w.rules.needs.fatigue.rise = 50_000;
    run(&mut w, MINUTE + 1);
    w.rules.needs.fatigue.rise = 0;
    w.intent(me, Intent::Sleep);
    run(&mut w, 16 * 60 * MINUTE);
    let home = w.actor(me).unwrap().home.expect("a home");
    die(&mut w, me);
    let next = w.next_character().unwrap();
    assert_eq!(w.actor(next).unwrap().home, Some(home));
    assert_eq!(w.actor(next).unwrap().deeds, 0, "but not the levels");
}
