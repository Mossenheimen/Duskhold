//! Items: containers that fill from the seed, taking and dropping, bandages, weapons that wear,
//! and the weight you can carry.

use std::sync::Arc;

use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Event, Intent, Item, ItemDef, ItemKind, LootEntry, LootTable,
    Part, Tile, TilePos, World, WoundKind,
};

const AXE: u16 = 0;
const BANDAGE: u16 = 1;
const BREAD: u16 = 2;
const ANVIL: u16 = 3;

/// A few items, a kitchen loot table, and fixture 1 a cupboard that uses it.
fn defs() -> Defs {
    let item = |id: &str, weight, stack, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight,
        stack,
        kind,
    };
    let axe = Swing {
        wind_up: 5,
        recovery: 6,
        endurance: 100,
        hit: 10_000,
        damage: (200, 200),
        noise: 5,
    };
    Defs {
        items: vec![
            item(
                "axe",
                2000,
                1,
                ItemKind::Weapon {
                    swing: axe,
                    wear: 400,
                    skill: None,
                },
            ),
            item("bandage", 100, 10, ItemKind::Bandage { time: 20 }),
            item(
                "bread",
                300,
                5,
                ItemKind::Food {
                    nutrition: 30_000,
                    time: 30,
                },
            ),
            item("anvil", 30_000, 1, ItemKind::Other),
        ],
        loot: vec![LootTable {
            id: "kitchen".into(),
            rolls: (2, 4),
            entries: vec![
                LootEntry {
                    item: BREAD,
                    weight: 3,
                    count: (1, 2),
                },
                LootEntry {
                    item: BANDAGE,
                    weight: 1,
                    count: (1, 1),
                },
            ],
        }],
        containers: vec![None, Some(0)],
        beds: Vec::new(),
        wield: None,
        kit: Vec::new(),
        carry: 15_000,
        take: 4,
        take_per_kg: 1,
        drop: 2,
        wield_time: 5,
        unwield_time: 3,
        light_time: 15,
        douse_time: 5,
        boarding: None,
    }
}

const CUPBOARD: Tile = Tile {
    fixture: 1,
    solid: true,
    ..Tile::FLOOR
};

/// A room four tiles long: you at its west end, cupboards beside you and at its east end.
fn kitchen(defs: Defs) -> (World, ActorId) {
    let (mut w, _) = World::from_text(
        3,
        "\
+-+-+-+-+
|. . . .|
+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs);
    w.grid.set_tile(TilePos::new(1, 0, 0), CUPBOARD);
    w.grid.set_tile(TilePos::new(3, 0, 0), CUPBOARD);
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    (w, me)
}

/// Asks for an action and runs until it's done.
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

#[test]
fn a_cupboard_holds_the_same_whenever_it_is_looked_into() {
    let (a, _) = kitchen(defs());
    let (mut b, _) = kitchen(defs());
    b.step(50);
    let cupboard = TilePos::new(1, 0, 0);
    let held = a.store(cupboard);
    assert!(!held.is_empty(), "a kitchen cupboard isn't bare");
    assert_eq!(held, b.store(cupboard), "the same, later or sooner");
    let before = a.state_hash();
    let _ = a.store(cupboard);
    assert_eq!(a.state_hash(), before, "looking changes nothing");
    assert!(a.store(TilePos::new(0, 0, 0)).is_empty(), "an empty floor");
}

#[test]
fn you_take_what_is_within_reach_and_no_further() {
    let (mut w, me) = kitchen(defs());
    let cupboard = TilePos::new(1, 0, 0);
    let held = w.store(cupboard);
    let first = held[0];
    let seen = act(
        &mut w,
        me,
        Intent::Take {
            at: cupboard,
            slot: 0,
            def: first.def,
        },
    );
    assert!(seen.contains(&Event::Took {
        who: me,
        item: first.def,
        count: first.count
    }));
    assert_eq!(w.actor(me).unwrap().items, vec![first]);
    assert_eq!(
        w.store(cupboard),
        held[1..].to_vec(),
        "it's gone from the cupboard"
    );

    let far = TilePos::new(3, 0, 0);
    let there = w.store(far)[0];
    let seen = act(
        &mut w,
        me,
        Intent::Take {
            at: far,
            slot: 0,
            def: there.def,
        },
    );
    assert!(
        !seen.iter().any(|e| matches!(e, Event::Took { .. })),
        "out of reach"
    );
    assert_eq!(w.store(far)[0], there);
}

#[test]
fn a_bandage_stops_the_bleeding() {
    let mut d = defs();
    d.kit = vec![Item::new(BANDAGE, 2)];
    let (mut w, me) = kitchen(d);
    w.wound(me, WoundKind::Laceration, Part::LeftArm);
    assert!(w.actor(me).unwrap().body.bleeding());
    let seen = act(
        &mut w,
        me,
        Intent::Use {
            slot: 0,
            def: BANDAGE,
        },
    );
    assert!(seen.contains(&Event::Bandaged {
        who: me,
        part: Part::LeftArm
    }));
    let a = w.actor(me).unwrap();
    assert!(!a.body.bleeding() && a.body.wounds[0].bandaged);
    assert_eq!(a.items, vec![Item::new(BANDAGE, 1)], "one used");
    let seen = act(
        &mut w,
        me,
        Intent::Use {
            slot: 0,
            def: BANDAGE,
        },
    );
    assert!(seen.is_empty(), "nothing left to bandage");
}

#[test]
fn a_weapon_hits_harder_and_wears_out() {
    let mut d = defs();
    d.wield = Some(AXE);
    let (mut w, me) = World::from_text(
        1,
        "\
+-+
|z|
+ +
|.|
+-+
",
    )
    .unwrap();
    let z = me[0];
    w.defs = Arc::new(d);
    let c = &mut w.rules.combat;
    (c.grab, c.parting_grab, c.shambler_attack.hit) = (0, 0, 0);
    w.rules.wander_chance = 0;
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 1, 0))
        .unwrap();
    assert_eq!(w.actor(me).unwrap().hands.map(|h| h.def), Some(AXE));
    let mut blows = Vec::new();
    let mut broke = false;
    for _ in 0..200 {
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            w.intent(me, Intent::Attack { dir: Some(Dir8::N) });
        }
        w.step(1);
        for e in w.events() {
            match e {
                Event::Struck { who, damage, .. } if who == me => blows.push(damage),
                Event::WeaponBroke { who, item } if who == me && item == AXE => broke = true,
                _ => {}
            }
        }
        if broke || w.actor(z).unwrap().dead {
            break;
        }
    }
    assert!(blows.iter().all(|&d| d == 200), "an axe's blows: {blows:?}");
    assert!(w.actor(z).unwrap().dead, "two axe blows fell a shambler");
    assert_eq!(
        w.actor(me).unwrap().hands.map(|h| h.condition),
        Some(200),
        "worn by two"
    );
}

#[test]
fn weighed_down_you_cant_run() {
    let (mut w, me) = kitchen(defs());
    let here = TilePos::new(0, 0, 0);
    w.put(here, Item::new(ANVIL, 1));
    act(
        &mut w,
        me,
        Intent::Take {
            at: here,
            slot: 0,
            def: ANVIL,
        },
    );
    assert!(w.weighed_down(me));
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: true,
        },
    );
    w.step(3);
    assert!(
        w.actor(me).unwrap().step.is_none_or(|s| !s.run),
        "too heavy to run"
    );

    w.intent(me, Intent::Idle);
    w.step(20);
    let seen = act(
        &mut w,
        me,
        Intent::Drop {
            slot: 0,
            def: ANVIL,
        },
    );
    assert!(seen.iter().any(|e| matches!(e, Event::Dropped { .. })));
    let at = w.actor(me).unwrap().pos;
    assert_eq!(
        w.store(at),
        vec![Item::new(ANVIL, 1)],
        "it lies where you dropped it"
    );
    assert!(!w.weighed_down(me));
}
