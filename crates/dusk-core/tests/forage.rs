//! Herbs and foraging: a wild plant picked gives its herb and is left bare, with nothing more to
//! pick till it grows back; it grows back in its time and not before, unless something else
//! stands there by then; Foraging grows by picking (less in one spot) and finds one more herb
//! more often; a poultice dresses a wound so it closes sooner and hurts less than under a plain
//! bandage; and a tea calms.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Defs, Event, Intent, Item, ItemDef, ItemKind, Part, Plant, Skill, Tile,
    TilePos, World, WoundKind,
};

const YARROW: u16 = 0;
const BANDAGE: u16 = 1;
const POULTICE: u16 = 2;
const TEA: u16 = 3;
const GALL: u16 = 4;
/// Fixture 1 yarrow (one or two a picking, back in a game day), fixture 2 a plant picked bare,
/// fixture 3 a stone.
const PLANT: u16 = 1;
const BARE: u16 = 2;
const STONE: u16 = 3;
/// Ticks in a game hour and a game day (25 a minute).
const HOUR: u32 = 25 * 60;
const DAY: u64 = 25 * 60 * 24;

/// Yarrow; a bandage and a poultice (half the healing left: a salve); a tea (30 points of
/// thirst, 20 of dread); and gall, which sits ill (30 points of dread added).
fn defs(kit: Vec<Item>) -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 10,
        stack: 20,
        kind,
        worth: 0,
    };
    Defs {
        items: vec![
            item("yarrow", ItemKind::Other),
            item(
                "bandage",
                ItemKind::Bandage {
                    time: 10,
                    salve: 1000,
                },
            ),
            item(
                "poultice",
                ItemKind::Bandage {
                    time: 10,
                    salve: 500,
                },
            ),
            item(
                "tea",
                ItemKind::Drink {
                    water: 30_000,
                    time: 5,
                    comfort: 20_000,
                },
            ),
            item(
                "gall",
                ItemKind::Food {
                    nutrition: 0,
                    time: 5,
                    comfort: -30_000,
                    spoils: 0,
                },
            ),
        ],
        plants: vec![
            None,
            Some(Plant {
                item: YARROW,
                count: (1, 2),
                regrow: DAY,
            }),
            None,
            None,
        ],
        bare: BARE,
        kit,
        carry: 30_000,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// Open grass by day, yarrow on each of `plants`; nobody hungry or thirsty, dread held still.
fn meadow(plants: &[TilePos], kit: Vec<Item>) -> (World, ActorId) {
    let mut w = World::new(4);
    w.defs = Arc::new(defs(kit));
    for x in -4..5 {
        for y in -4..5 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    for &p in plants {
        w.grid.set_tile(
            p,
            Tile {
                fixture: PLANT,
                ..Tile::GRASS
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
    let items = &w.actor(me).unwrap().items;
    items.iter().position(|i| i.def == def).unwrap() as u16
}

/// What one picking gave: the count in its Picked event.
fn picked(seen: &[Event]) -> Option<u16> {
    seen.iter().find_map(|e| match *e {
        Event::Picked { count, .. } => Some(count),
        _ => None,
    })
}

#[test]
fn a_plant_picked_gives_its_herb_and_is_left_bare() {
    let plant = at(1, 0);
    let (mut w, me) = meadow(&[plant], Vec::new());
    assert_eq!(w.plant_beside(me), Some(plant));
    let seen = act(&mut w, me, Intent::Pick);
    let got = picked(&seen).expect("picked");
    assert!((1..=2).contains(&got), "one or two: {got}");
    assert_eq!(count(&w, me, YARROW), got, "in the pack");
    assert!(seen.contains(&Event::Picked {
        who: me,
        at: plant,
        item: YARROW,
        count: got,
    }));
    assert_eq!(w.grid.tile(plant).fixture, BARE, "picked bare");
    assert_eq!(w.plant_beside(me), None, "nothing left to pick");
    assert_eq!(picked(&act(&mut w, me, Intent::Pick)), None);
    assert_eq!(count(&w, me, YARROW), got);
}

#[test]
fn a_plant_grows_back_in_its_time_and_not_before() {
    let plant = at(0, 1);
    let (mut w, me) = meadow(&[plant], Vec::new());
    act(&mut w, me, Intent::Pick);
    assert_eq!(w.grid.tile(plant).fixture, BARE);
    // A game day less an hour or two: still bare.
    w.step((DAY as u32) - 2 * HOUR);
    assert_eq!(w.grid.tile(plant).fixture, BARE, "not yet");
    let mut seen = Vec::new();
    for _ in 0..3 {
        w.step(HOUR);
        seen.extend(w.events());
    }
    assert_eq!(w.grid.tile(plant).fixture, PLANT, "grown back");
    assert!(seen.contains(&Event::Regrew { at: plant }));
    assert_eq!(w.plant_beside(me), Some(plant), "to pick again");
}

#[test]
fn a_plant_built_over_while_its_bare_stays_gone() {
    let plant = at(-1, 0);
    let (mut w, me) = meadow(&[plant], Vec::new());
    act(&mut w, me, Intent::Pick);
    w.grid.set_tile(
        plant,
        Tile {
            fixture: STONE,
            solid: true,
            ..Tile::GRASS
        },
    );
    w.step(DAY as u32 + 2 * HOUR);
    assert_eq!(
        w.grid.tile(plant).fixture,
        STONE,
        "nothing grows through it"
    );
}

#[test]
fn foraging_grows_by_picking_and_less_in_one_spot() {
    // Both in the one 8-tile spot (rules.skills.spot): x = -1 would be the next spot west.
    let (mut w, me) = meadow(&[at(1, 0), at(1, 1)], Vec::new());
    let xp = |w: &World| w.actor(me).unwrap().skills.xp[Skill::Foraging as usize];
    act(&mut w, me, Intent::Pick);
    let first = xp(&w);
    assert!(first > 0, "picking teaches Foraging");
    act(&mut w, me, Intent::Pick);
    let second = xp(&w) - first;
    assert!(
        second > 0 && second < first,
        "the same spot again pays less: {first} then {second}"
    );
}

#[test]
fn foraging_finds_one_more_more_often() {
    let plant = at(1, 0);
    let pick = |skilled: bool| -> u16 {
        let (mut w, me) = meadow(&[plant], Vec::new());
        // A level of Foraging, and a level makes one more certain.
        w.rules.skills.forage_more = 1000;
        if skilled {
            w.give_xp(me, Skill::Foraging, 10_000);
        }
        assert_eq!(w.skill_level(me, Skill::Foraging), u8::from(skilled));
        picked(&act(&mut w, me, Intent::Pick)).unwrap()
    };
    assert_eq!(pick(true), pick(false) + 1, "one more, every time");
}

#[test]
fn a_poultice_closes_a_wound_sooner_and_it_hurts_less() {
    let dressed = |with: u16| -> (u32, i32) {
        let (mut w, me) = meadow(&[], vec![Item::new(with, 1)]);
        w.wound(me, WoundKind::Laceration, Part::LeftArm);
        let s = slot(&w, me, with);
        act(&mut w, me, Intent::Use { slot: s, def: with });
        let wound = w.actor(me).unwrap().body.wounds[0];
        assert!(
            wound.bandaged && wound.bleeding == 0,
            "dressed, and stopped"
        );
        (wound.heal_left, w.pain(me))
    };
    let (bandage_left, bandage_pain) = dressed(BANDAGE);
    let (poultice_left, poultice_pain) = dressed(POULTICE);
    assert!(
        poultice_left.abs_diff(bandage_left / 2) <= 1,
        "half the healing left: {poultice_left} of {bandage_left}"
    );
    assert!(
        poultice_pain < bandage_pain,
        "it hurts less: {poultice_pain} to {bandage_pain}"
    );
}

#[test]
fn a_tea_calms() {
    let (mut w, me) = meadow(&[], vec![Item::new(GALL, 1), Item::new(TEA, 1)]);
    let gall = slot(&w, me, GALL);
    act(
        &mut w,
        me,
        Intent::Use {
            slot: gall,
            def: GALL,
        },
    );
    assert_eq!(w.dread(me), 30_000);
    let tea = slot(&w, me, TEA);
    let seen = act(
        &mut w,
        me,
        Intent::Use {
            slot: tea,
            def: TEA,
        },
    );
    assert!(seen.contains(&Event::Drank { who: me, item: TEA }));
    assert_eq!(w.dread(me), 10_000, "twenty points of dread eased");
}
