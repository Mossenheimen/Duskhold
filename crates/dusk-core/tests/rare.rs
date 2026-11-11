//! Rare monsters: affixes on top of a kind's numbers, a name, a pack of finds, the
//! Lantern-Eater and the Thrice-Risen, and the bounty paid over a holdout's counter.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Affix, Bounty, Defs, Event, GetUp, Holdout, Intent, Item, ItemDef,
    ItemKind, LootEntry, LootTable, NoDeal, Part, Pressure, Region, Tile, TilePos, World,
    WoundKind,
};

const SHAMBLER: ActorKind = ActorKind::Creature(0);
const COIN: u16 = 0;
const CANDLE: u16 = 1;
const FRENZIED: u8 = 0;
const IRONHIDE: u8 = 1;
const LANTERN_EATER: u8 = 2;
const THRICE_RISEN: u8 = 3;
const PLAGUEBEARER: u8 = 4;
/// The counter is fixture 1, north of where you start.
const COUNTER: TilePos = TilePos::new(0, -1, 0);
/// Ticks in a game minute.
const MINUTE: u32 = 25;

fn defs() -> Defs {
    let item = |id: &str, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 10,
        stack: 100,
        kind,
        worth: 1000,
    };
    let affix = |id: &str, epithet: &str| Affix {
        id: id.into(),
        epithet: epithet.into(),
        ..Affix::default()
    };
    let abbey = Holdout {
        id: "abbey".into(),
        name: "the abbey".into(),
        sells: Vec::new(),
        buys: 500,
        repair: 1000,
        wants: Vec::new(),
        supply: 0,
        time: 10,
        bounty: Bounty {
            coin: 25,
            xp: 30_000,
            pressure: 40,
        },
        identify: 0,
        uncurse: 0,
    };
    Defs {
        items: vec![
            item("coins", ItemKind::Other),
            item("candle", ItemKind::Light { radius: 2, wane: 1 }),
        ],
        kit: vec![Item::new(CANDLE, 1)],
        loot: vec![LootTable {
            id: "rare".into(),
            rolls: (1, 1),
            entries: vec![LootEntry {
                item: COIN,
                weight: 1,
                count: (12, 12),
                magic: 0,
            }],
        }],
        affixes: vec![
            Affix {
                speed: 1500,
                swing: 500,
                ..affix("frenzied", "the Frenzied")
            },
            Affix {
                health: 3000,
                ..affix("ironhide", "the Ironhide")
            },
            Affix {
                douses: 4,
                ..affix("lantern_eater", "the Lantern-Eater")
            },
            Affix {
                rises: 1,
                ..affix("thrice_risen", "the Thrice-Risen")
            },
            Affix {
                fever: [10_000; 3],
                ..affix("plaguebearer", "the Plaguebearer")
            },
        ],
        rare_names: vec!["Grimtooth".into(), "Hollowjaw".into()],
        holdouts: vec![abbey],
        counters: vec![None, Some(0)],
        coin: Some(COIN),
        ..Defs::default()
    }
}

/// A field of grass, a counter north of you, the needs stilled, the dead standing still.
fn field() -> (World, ActorId) {
    let mut w = World::new(8);
    w.defs = Arc::new(defs());
    w.creature_mut(0).wander = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    for y in -30..30 {
        for x in -30..30 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    w.grid.set_tile(
        COUNTER,
        Tile {
            fixture: 1,
            solid: true,
            ..Tile::GRASS
        },
    );
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
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

/// Wounds `id` until it's down.
fn put_down(w: &mut World, id: ActorId) {
    while !w.actor(id).unwrap().dead {
        w.wound(id, WoundKind::Bite, Part::Torso);
    }
}

#[test]
fn affixes_go_on_top_of_the_kinds_numbers() {
    let (mut w, _) = field();
    let plain = w.spawn(SHAMBLER, TilePos::new(10, 10, 0)).unwrap();
    let rare = w.spawn(SHAMBLER, TilePos::new(12, 10, 0)).unwrap();
    w.make_rare(rare, &[FRENZIED, IRONHIDE], 0, None);
    let (p, r) = (w.stats(plain).unwrap(), w.stats(rare).unwrap());
    assert_eq!(r.speed.walk, p.speed.walk * 3 / 2, "half as fast again");
    assert_eq!(
        r.attack.wind_up,
        p.attack.wind_up / 2,
        "its tell twice as quick"
    );
    assert_eq!(r.health, p.health * 3);
    assert_eq!(
        w.actor(rare).unwrap().body.health,
        r.health,
        "and that health, at once"
    );
    assert_eq!(w.name_of(rare).unwrap(), "Grimtooth the Frenzied");
    assert!(w.is_rare(rare) && !w.is_rare(plain));
}

#[test]
fn a_rare_carries_finds_and_they_fall_where_it_does() {
    let (mut w, _) = field();
    let at = TilePos::new(10, 10, 0);
    let rare = w.spawn(SHAMBLER, at).unwrap();
    w.make_rare(rare, &[IRONHIDE], 1, Some(0));
    let pack = &w.actor(rare).unwrap().items;
    assert!(
        pack.iter()
            .any(|i| i.def == COIN && i.count == 12 && i.found)
    );
    put_down(&mut w, rare);
    assert!(w.store(at).iter().any(|i| i.def == COIN && i.found));
}

#[test]
fn the_lantern_eater_puts_out_lights_held_up_near_it() {
    let (mut w, me) = field();
    let rare = w.spawn(SHAMBLER, TilePos::new(6, 0, 0)).unwrap();
    w.make_rare(rare, &[LANTERN_EATER], 0, None);
    w.creature_mut(0).sight = 0;
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == CANDLE)
        .unwrap();
    w.intent(
        me,
        Intent::Light {
            slot: slot as u16,
            def: CANDLE,
        },
    );
    run(&mut w, 5);
    assert!(
        w.actor(me).unwrap().light.is_some(),
        "6 tiles off, it burns"
    );
    // Walk up to it: within 4 tiles, the light goes out.
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(3, 0, 0),
            run: false,
        },
    );
    let seen = run(&mut w, 60);
    assert!(seen.contains(&Event::LightEaten { by: rare, who: me }));
    assert!(w.actor(me).unwrap().light.is_none());
}

#[test]
fn the_thrice_risen_gets_up_twice() {
    let (mut w, _) = field();
    w.creature_mut(0).get_up = GetUp {
        chance: 10_000,
        after: (60, 60),
    };
    let plain = w.spawn(SHAMBLER, TilePos::new(10, 10, 0)).unwrap();
    let rare = w.spawn(SHAMBLER, TilePos::new(14, 10, 0)).unwrap();
    w.make_rare(rare, &[THRICE_RISEN], 0, None);
    for round in 0..3 {
        put_down(&mut w, plain);
        put_down(&mut w, rare);
        run(&mut w, 61 * MINUTE);
        let (p, r) = (w.actor(plain).unwrap().dead, w.actor(rare).unwrap().dead);
        match round {
            0 => assert!(!p && !r, "both get up once"),
            1 => assert!(p && !r, "only the Thrice-Risen a second time"),
            _ => assert!(p && r, "and then it stays down"),
        }
    }
}

#[test]
fn a_plaguebearers_wounds_carry_the_fever_whatever_its_kind() {
    let (mut w, _) = field();
    let rare = w.spawn(SHAMBLER, TilePos::new(10, 10, 0)).unwrap();
    assert_eq!(w.stats(rare).unwrap().fever, [0; 3]);
    w.make_rare(rare, &[PLAGUEBEARER], 0, None);
    assert_eq!(w.stats(rare).unwrap().fever, [10_000; 3]);
}

#[test]
fn the_bounty_is_paid_once_the_rare_is_down_and_only_once() {
    let (mut w, me) = field();
    w.set_region(Region {
        pressure: Pressure {
            start: 1000,
            least: 250,
            most: 2000,
            ceiling: 1500,
            ..Pressure::default()
        },
        ..Region::default()
    });
    let rare = w.spawn(SHAMBLER, TilePos::new(15, 15, 0)).unwrap();
    w.make_rare(rare, &[IRONHIDE], 0, None);
    assert_eq!(w.can_claim(me, COUNTER, rare), Err(NoDeal::StillAbout));
    put_down(&mut w, rare);
    assert!(w.can_claim(me, COUNTER, rare).is_ok());
    w.intent(me, Intent::Claim { at: COUNTER, rare });
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::BountyClaimed {
        who: me,
        rare,
        coin: 25
    }));
    assert_eq!(w.coins(me), 25);
    assert_eq!(w.actor(me).unwrap().deeds, 30_000);
    assert_eq!(w.pressure(), 960, "the dusk falls back");
    assert_eq!(w.can_claim(me, COUNTER, rare), Err(NoDeal::Claimed));
}
