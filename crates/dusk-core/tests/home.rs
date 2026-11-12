//! Bringing it home: home is where you slept the night through, the stash is a container near
//! it, a find pays its worth once, and levels bring a little health.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Defs, Event, Intent, Item, ItemDef, ItemKind, LootEntry, LootTable,
    NoStash, Tile, TilePos, World,
};

const COINS: u16 = 0;
const BANDAGE: u16 = 1;
/// Ticks in a game hour.
const HOUR: u32 = 25 * 60;

/// Coins worth 1 XP each, and a bandage worth 3 (one in your kit). Fixture 1 is a cupboard that
/// always holds ten coins.
fn defs() -> Defs {
    let item = |id: &str, weight, stack, kind, worth| ItemDef {
        id: id.into(),
        name: id.into(),
        weight,
        stack,
        kind,
        worth,
    };
    Defs {
        items: vec![
            item("coins", 10, 100, ItemKind::Other, 1000),
            item(
                "bandage",
                100,
                10,
                ItemKind::Bandage {
                    time: 20,
                    salve: 1000,
                },
                3000,
            ),
        ],
        loot: vec![LootTable {
            id: "purse".into(),
            rolls: (2, 2),
            entries: vec![LootEntry {
                item: COINS,
                weight: 1,
                count: (5, 5),
                magic: 0,
            }],
        }],
        containers: vec![None, Some(0)],
        kit: vec![Item::new(BANDAGE, 1)],
        carry: 15_000,
        take: 4,
        take_per_kg: 1,
        ..Defs::default()
    }
}

const CUPBOARD: Tile = Tile {
    fixture: 1,
    solid: true,
    ..Tile::FLOOR
};

fn at(x: i32) -> TilePos {
    TilePos::new(x, 0, 0)
}

/// A long room under one roof, two tiles deep: a cupboard beside you at its west end, another
/// 12 tiles off.
fn house() -> (World, ActorId) {
    let (mut w, _) = World::from_text(
        2,
        "\
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|. . . . . . . . . . . . . .|
+                           +
|. . . . . . . . . . . . . .|
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs());
    w.grid.set_tile(at(1), CUPBOARD);
    w.grid.set_tile(at(13), CUPBOARD);
    let me = w.spawn(ActorKind::Character, at(0)).unwrap();
    (w, me)
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..400 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

/// Stays up until tired, then sleeps the night through, here.
fn sleep_through(w: &mut World, me: ActorId) -> Vec<Event> {
    w.step(12 * HOUR);
    act(w, me, Intent::Sleep);
    let mut seen = Vec::new();
    for _ in 0..20 * HOUR {
        w.step(1);
        seen.extend(w.events());
        if !w.actor(me).unwrap().body.asleep {
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

#[test]
fn home_is_where_you_slept_the_night_through() {
    let (mut w, me) = house();
    assert_eq!(
        w.can_stash(me, at(1)),
        Err(NoStash::NoHome),
        "no night slept yet"
    );
    let seen = sleep_through(&mut w, me);
    assert!(seen.contains(&Event::MadeHome { who: me, at: at(0) }));
    assert_eq!(w.actor(me).unwrap().home, Some(at(0)));
    assert_eq!(w.can_stash(me, at(1)), Ok(()));
    assert_eq!(
        w.can_stash(me, at(13)),
        Err(NoStash::NotHere),
        "out of reach"
    );
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(13, 1, 0),
            run: false,
        },
    );
    w.step(200);
    assert_eq!(w.can_stash(me, at(13)), Err(NoStash::FarFromHome));
}

#[test]
fn a_stash_wants_a_roof() {
    let mut w = World::new(5);
    w.defs = Arc::new(defs());
    for x in 0..3 {
        w.grid.set_tile(at(x), Tile::GRASS);
    }
    let out_in_the_open = Tile {
        roof: false,
        ground: dusk_core::Ground::Grass,
        ..CUPBOARD
    };
    w.grid.set_tile(at(1), out_in_the_open);
    let me = w.spawn(ActorKind::Character, at(0)).unwrap();
    sleep_through(&mut w, me);
    assert_eq!(w.can_stash(me, at(1)), Err(NoStash::Outdoors));
}

#[test]
fn a_find_put_in_your_stash_pays_its_worth_once() {
    let (mut w, me) = house();
    sleep_through(&mut w, me);
    let seen = act(&mut w, me, Intent::Stash { at: at(1) });
    assert!(seen.contains(&Event::Stashed { who: me, at: at(1) }));

    // The cupboard's ten coins are finds.
    assert_eq!(w.store(at(1)), vec![Item::found(COINS, 10)]);
    act(
        &mut w,
        me,
        Intent::Take {
            at: at(1),
            slot: 0,
            def: COINS,
        },
    );
    let coins = slot_of(&w, me, COINS);
    let seen = act(
        &mut w,
        me,
        Intent::Put {
            at: at(1),
            slot: coins,
            def: COINS,
        },
    );
    assert!(seen.contains(&Event::BroughtHome {
        who: me,
        item: COINS,
        count: 10,
        xp: 10_000
    }));
    assert_eq!(w.actor(me).unwrap().deeds, 10_000);
    assert_eq!(
        w.store(at(1)),
        vec![Item::new(COINS, 10)],
        "brought home: not a find any more"
    );

    // Out and back in again: nothing more. Nor for what you started with.
    act(
        &mut w,
        me,
        Intent::Take {
            at: at(1),
            slot: 0,
            def: COINS,
        },
    );
    for def in [COINS, BANDAGE] {
        let slot = slot_of(&w, me, def);
        let seen = act(
            &mut w,
            me,
            Intent::Put {
                at: at(1),
                slot,
                def,
            },
        );
        assert!(seen.iter().any(|e| matches!(e, Event::Put { .. })));
        assert!(!seen.iter().any(|e| matches!(e, Event::BroughtHome { .. })));
    }
    assert_eq!(w.actor(me).unwrap().deeds, 10_000);
}

#[test]
fn a_find_in_any_other_container_isnt_home() {
    let (mut w, me) = house();
    // No stash: putting a find away pays nothing, and it stays a find.
    act(
        &mut w,
        me,
        Intent::Take {
            at: at(1),
            slot: 0,
            def: COINS,
        },
    );
    let coins = slot_of(&w, me, COINS);
    let seen = act(
        &mut w,
        me,
        Intent::Put {
            at: at(1),
            slot: coins,
            def: COINS,
        },
    );
    assert!(!seen.iter().any(|e| matches!(e, Event::BroughtHome { .. })));
    assert_eq!(w.store(at(1)), vec![Item::found(COINS, 10)]);
}

#[test]
fn finds_and_what_was_brought_home_dont_stack_together() {
    let defs = defs();
    let mut list = vec![Item::new(COINS, 4)];
    defs.stow(&mut list, Item::found(COINS, 3));
    assert_eq!(list, vec![Item::new(COINS, 4), Item::found(COINS, 3)]);
}

#[test]
fn levels_bring_a_little_health() {
    let (mut w, me) = house();
    let first = w.max_health(me);
    assert_eq!((w.level(me), first), (1, 1000));
    w.gain_deeds(me, 29_000);
    assert_eq!(w.level(me), 1);
    let health = w.actor(me).unwrap().body.health;
    w.gain_deeds(me, 1_000);
    assert!(
        w.events()
            .any(|e| e == Event::LevelUp { who: me, level: 2 })
    );
    assert_eq!(w.max_health(me), first + 50);
    assert_eq!(w.actor(me).unwrap().body.health, health + 50, "at once");
    w.gain_deeds(me, 10_000_000);
    assert_eq!((w.level(me), w.max_health(me)), (10, 1450));
}
