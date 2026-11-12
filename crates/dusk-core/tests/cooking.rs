//! Cooking and food that goes off: a meal cooked only at a lit fire, teaching Cooking; a hot
//! meal easing dread and raw meat adding to it; food going off by the hour in a pack and on a
//! floor; and a larder nobody has opened aging with the world.

use std::sync::Arc;

use dusk_core::craft::{NoCraft, Recipe};
use dusk_core::items::{LootEntry, LootTable, NEW};
use dusk_core::{
    ActorId, ActorKind, Defs, Event, Intent, Item, ItemDef, ItemKind, Skill, Tile, TilePos, World,
};

const FIREWOOD: u16 = 0;
const CANDLE: u16 = 1;
const RAW: u16 = 2;
const ROAST: u16 = 3;
/// Fixture 1 a stove, fixture 2 a larder.
const STOVE: u16 = 1;
const LARDER: u16 = 2;
/// Ticks in a game hour (25 a minute).
const HOUR: u32 = 25 * 60;

/// Firewood, a candle, raw meat (half its condition gone an hour: gone in two; 5 points of
/// hunger, 5 of dread added) and roast meat (30 points, 10 of dread taken, keeps); a stove, a
/// larder that always holds one piece of raw meat; and the roast, cooked at a fire.
fn defs(kit: Vec<Item>) -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 100,
        stack: 10,
        kind,
        worth: 0,
    };
    let food = |nutrition, comfort, spoils| ItemKind::Food {
        nutrition,
        time: 5,
        comfort,
        spoils,
    };
    Defs {
        items: vec![
            item("firewood", ItemKind::Other),
            item("candle", ItemKind::Light { radius: 2, wane: 4 }),
            item("raw", food(5_000, -5_000, NEW / 2)),
            item("roast", food(30_000, 10_000, 0)),
        ],
        loot: vec![LootTable {
            id: "larder".into(),
            rolls: (1, 1),
            entries: vec![LootEntry {
                item: RAW,
                weight: 1,
                count: (1, 1),
                magic: 0,
            }],
        }],
        containers: vec![None, None, Some(0)],
        hearths: vec![0, 4, 0],
        fuel: vec![(FIREWOOD, 120)],
        recipes: vec![Recipe {
            id: "roast".into(),
            makes: (ROAST, 1),
            uses: vec![(RAW, 1)],
            tools: Vec::new(),
            fire: true,
            time: 20,
            skill: Some(Skill::Cooking),
            xp: 1000,
        }],
        kit,
        carry: 30_000,
        light_time: 5,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

const STOVE_AT: TilePos = TilePos::new(1, 0, 0);
const LARDER_AT: TilePos = TilePos::new(-1, 0, 0);

/// A floor by day, a stove east of you and a larder west; nobody hungry, and dread held still.
fn kitchen(kit: Vec<Item>) -> (World, ActorId) {
    let mut w = World::new(4);
    w.defs = Arc::new(defs(kit));
    for x in -3..4 {
        for y in -3..4 {
            w.grid.set_tile(at(x, y), Tile::FLOOR);
        }
    }
    for (pos, fixture) in [(STOVE_AT, STOVE), (LARDER_AT, LARDER)] {
        w.grid.set_tile(
            pos,
            Tile {
                fixture,
                solid: true,
                ..Tile::FLOOR
            },
        );
    }
    w.rules.clock.start = 8 * 60;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    (w.rules.dread.rise, w.rules.dread.fall) = (0, 0);
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
    (w, me)
}

/// Does `intent` and runs on till it's done: the events.
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

fn count(w: &World, me: ActorId, def: u16) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .filter(|i| i.def == def)
        .map(|i| i.count)
        .sum()
}

fn slot(w: &World, me: ActorId, def: u16) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == def)
        .unwrap() as u16
}

#[test]
fn a_meal_is_cooked_only_at_a_lit_fire_and_teaches_cooking() {
    let (mut w, me) = kitchen(vec![
        Item::new(RAW, 1),
        Item::new(FIREWOOD, 1),
        Item::new(CANDLE, 1),
    ]);
    assert_eq!(w.can_craft(me, 0), Err(NoCraft::NoFire), "no fire at all");
    act(&mut w, me, Intent::Tend);
    assert_eq!(w.can_craft(me, 0), Err(NoCraft::NoFire), "laid, not lit");
    let candle = slot(&w, me, CANDLE);
    act(
        &mut w,
        me,
        Intent::Light {
            slot: candle,
            def: CANDLE,
        },
    );
    act(&mut w, me, Intent::Tend);
    assert!(w.hearth(STOVE_AT).lit);
    assert_eq!(w.can_craft(me, 0), Ok(()));
    let seen = act(&mut w, me, Intent::Craft { recipe: 0 });
    assert!(seen.contains(&Event::Crafted { who: me, recipe: 0 }));
    assert_eq!((count(&w, me, RAW), count(&w, me, ROAST)), (0, 1));
    assert!(w.actor(me).unwrap().skills.xp[Skill::Cooking as usize] > 0);
    // Cooking makes it quicker, as Carpentry does carpentry.
    let skills = w.rules.skills;
    assert!(skills.cook_ticks(100, 5) < skills.cook_ticks(100, 0));
}

#[test]
fn a_hot_meal_eases_dread_and_raw_meat_adds_to_it() {
    let (mut w, me) = kitchen(vec![Item::new(RAW, 3), Item::new(ROAST, 1)]);
    for _ in 0..3 {
        let raw = slot(&w, me, RAW);
        act(
            &mut w,
            me,
            Intent::Use {
                slot: raw,
                def: RAW,
            },
        );
    }
    assert_eq!(w.dread(me), 15_000, "raw meat sits ill");
    let roast = slot(&w, me, ROAST);
    let seen = act(
        &mut w,
        me,
        Intent::Use {
            slot: roast,
            def: ROAST,
        },
    );
    assert!(seen.contains(&Event::Ate {
        who: me,
        item: ROAST
    }));
    assert_eq!(w.dread(me), 5_000, "a hot meal eases it");
}

#[test]
fn food_goes_off_by_the_hour_in_a_pack_and_on_a_floor() {
    let (mut w, me) = kitchen(vec![Item::new(RAW, 2), Item::new(ROAST, 1)]);
    w.put(at(0, 2), Item::new(RAW, 1));
    w.step(HOUR - w.tick as u32);
    w.events();
    let raw = |w: &World| {
        let a = w.actor(me).unwrap();
        a.items.iter().find(|i| i.def == RAW).map(|i| i.condition)
    };
    assert_eq!(raw(&w), Some(NEW / 2), "half gone in an hour");
    assert_eq!(w.store(at(0, 2))[0].condition, NEW / 2);
    w.step(HOUR);
    let seen: Vec<Event> = w.events().collect();
    assert_eq!(raw(&w), None, "gone off");
    assert!(seen.contains(&Event::Spoiled { who: me, item: RAW }));
    assert!(w.store(at(0, 2)).is_empty(), "on the floor too");
    assert_eq!(count(&w, me, ROAST), 1, "what keeps, keeps");
}

#[test]
fn a_larder_nobody_opened_has_aged_with_the_world() {
    let (mut w, _) = kitchen(Vec::new());
    let meat = |w: &World| w.store(LARDER_AT).first().map(|i| (i.def, i.condition));
    assert_eq!(meat(&w), Some((RAW, NEW)));
    w.step(HOUR - w.tick as u32);
    assert_eq!(meat(&w), Some((RAW, NEW / 2)));
    w.step(HOUR);
    assert_eq!(meat(&w), None, "a larder left alone holds no meat");
}
