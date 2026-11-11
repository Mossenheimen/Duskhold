//! Holdouts: buying from a stock that comes back each day, selling finds for their XP and coin,
//! supplying a holdout to push the dusk back, mending, and hallowed ground, where the dead
//! never come.

use std::sync::Arc;

use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Alert, Area, Defs, Event, Holdout, Intent, Item, ItemDef, ItemKind, NoDeal,
    Pressure, Region, Sale, Tile, TilePos, World,
};

const SHAMBLER: ActorKind = ActorKind::Creature(0);
const COIN: u16 = 0;
const BANDAGE: u16 = 1;
const TINCTURE: u16 = 2;
const APPLE: u16 = 3;
const CUDGEL: u16 = 4;
const ROCK: u16 = 5;
/// The counter is fixture 1; it stands north of where you start.
const COUNTER: TilePos = TilePos::new(0, -1, 0);
/// Ticks in a game minute, and in a game day.
const MINUTE: u32 = 25;
const DAY: u32 = MINUTE * 1440;

fn defs() -> Defs {
    let item = |id: &str, worth: u32, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 10,
        stack: 100,
        kind,
        worth,
    };
    let club = Swing {
        wind_up: 4,
        recovery: 4,
        endurance: 0,
        hit: 10_000,
        damage: (100, 100),
        noise: 2,
    };
    let abbey = Holdout {
        id: "abbey".into(),
        name: "the abbey".into(),
        sells: vec![
            Sale {
                item: TINCTURE,
                price: 30,
                stock: 1,
            },
            Sale {
                item: BANDAGE,
                price: 6,
                stock: 2,
            },
        ],
        buys: 500,
        repair: 1000,
        wants: vec![APPLE],
        supply: 2500,
        time: 10,
    };
    Defs {
        items: vec![
            item("coins", 1000, ItemKind::Other),
            item("bandage", 3000, ItemKind::Bandage { time: 10 }),
            item("tincture", 15_000, ItemKind::Cure { time: 10 }),
            item(
                "apple",
                1000,
                ItemKind::Food {
                    nutrition: 1,
                    time: 10,
                },
            ),
            item(
                "cudgel",
                4000,
                ItemKind::Weapon {
                    swing: club,
                    wear: 10,
                    skill: None,
                },
            ),
            item("rock", 0, ItemKind::Other),
        ],
        // 40 coins, two bandages found in the world, four apples of your own, a rock, and a
        // cudgel worn to half.
        kit: vec![
            Item::new(COIN, 40),
            Item::found(BANDAGE, 2),
            Item::new(APPLE, 4),
            Item::new(ROCK, 1),
            Item {
                condition: 500,
                ..Item::new(CUDGEL, 1)
            },
        ],
        holdouts: vec![abbey],
        counters: vec![None, Some(0)],
        coin: Some(COIN),
        ..Defs::default()
    }
}

/// A field of grass with a counter north of you, the needs stilled.
fn abbey() -> (World, ActorId) {
    let mut w = World::new(3);
    w.defs = Arc::new(defs());
    w.creature_mut(0).wander = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -20..20 {
        for x in -20..20 {
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

fn slot(w: &World, me: ActorId, def: u16) -> u16 {
    let items = &w.actor(me).unwrap().items;
    items.iter().position(|i| i.def == def).unwrap() as u16
}

fn count(w: &World, me: ActorId, def: u16) -> u16 {
    let items = &w.actor(me).unwrap().items;
    items.iter().filter(|i| i.def == def).map(|i| i.count).sum()
}

#[test]
fn you_buy_from_a_stock_that_comes_back_the_next_day() {
    let (mut w, me) = abbey();
    assert_eq!(w.counter_in_reach(me), Some((COUNTER, 0)));
    assert_eq!(w.can_buy(me, COUNTER, 0), Ok(30));
    w.intent(
        me,
        Intent::Buy {
            at: COUNTER,
            line: 0,
        },
    );
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::Bought {
        who: me,
        item: TINCTURE,
        price: 30
    }));
    assert_eq!((w.coins(me), count(&w, me, TINCTURE)), (10, 1));
    assert_eq!(w.can_buy(me, COUNTER, 0), Err(NoDeal::SoldOut), "one a day");
    w.intent(
        me,
        Intent::Buy {
            at: COUNTER,
            line: 1,
        },
    );
    run(&mut w, 20);
    assert_eq!(w.coins(me), 4);
    assert_eq!(w.can_buy(me, COUNTER, 1), Err(NoDeal::TooPoor));
    // A day on, the tincture's back, but you can't pay for it.
    run(&mut w, DAY);
    assert_eq!(w.stock_left(0, 0), 1);
    assert_eq!(w.can_buy(me, COUNTER, 0), Err(NoDeal::TooPoor));
}

#[test]
fn a_find_sold_pays_its_worth_in_xp_once_and_half_of_it_in_coin() {
    let (mut w, me) = abbey();
    let s = slot(&w, me, BANDAGE);
    assert_eq!(w.can_sell(me, COUNTER, s, BANDAGE), Ok(3));
    w.intent(
        me,
        Intent::Sell {
            at: COUNTER,
            slot: s,
            def: BANDAGE,
        },
    );
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::BroughtHome {
        who: me,
        item: BANDAGE,
        count: 2,
        xp: 6000
    }));
    assert_eq!(w.coins(me), 43);
    assert_eq!(w.actor(me).unwrap().deeds, 6000);
}

#[test]
fn coin_isnt_for_sale_nor_is_what_has_no_worth() {
    let (w, me) = abbey();
    let coin = slot(&w, me, COIN);
    assert_eq!(w.can_sell(me, COUNTER, coin, COIN), Err(NoDeal::Worthless));
    let rock = slot(&w, me, ROCK);
    assert_eq!(w.can_sell(me, COUNTER, rock, ROCK), Err(NoDeal::Worthless));
    // Out of reach of the counter, nothing.
    let far = TilePos::new(5, 5, 0);
    assert_eq!(w.can_buy(me, far, 0), Err(NoDeal::NoCounter));
}

#[test]
fn supplying_what_its_short_of_pushes_the_dusk_back() {
    let (mut w, me) = abbey();
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
    let s = slot(&w, me, APPLE);
    w.intent(
        me,
        Intent::Sell {
            at: COUNTER,
            slot: s,
            def: APPLE,
        },
    );
    let seen = run(&mut w, 20);
    // Four apples, a point of worth each, at 2.5 thousandths of the pressure a point.
    assert!(seen.contains(&Event::Supplied { who: me, by: 10 }));
    assert_eq!(w.pressure(), 990);
    assert_eq!(w.coins(me), 42, "and half their worth in coin");
}

#[test]
fn mending_costs_by_how_worn_the_weapon_is() {
    let (mut w, me) = abbey();
    let s = slot(&w, me, CUDGEL);
    w.intent(
        me,
        Intent::Wield {
            slot: s,
            def: CUDGEL,
        },
    );
    run(&mut w, 5);
    // Worn to half: half its worth of 4.
    assert_eq!(w.repair_cost(me, COUNTER), Ok(2));
    w.intent(me, Intent::Mend { at: COUNTER });
    run(&mut w, 20);
    assert_eq!(w.actor(me).unwrap().hands.unwrap().condition, 1000);
    assert_eq!(w.coins(me), 38);
    assert_eq!(w.repair_cost(me, COUNTER), Err(NoDeal::NothingToMend));
}

/// Hallowed ground round you: from (-6, -6) up to (7, 4).
fn hallow(w: &mut World) {
    w.hallowed = vec![Area {
        x0: -6,
        y0: -6,
        x1: 7,
        y1: 4,
        z: 0,
    }];
}

#[test]
fn the_dead_heed_nothing_on_hallowed_ground_and_never_set_foot_on_it() {
    let (mut w, me) = abbey();
    hallow(&mut w);
    // It stands outside, facing you: north.
    // No grabs: this is about where it goes, not the fight.
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    let z = w.spawn(SHAMBLER, TilePos::new(0, 6, 0)).unwrap();
    w.make_sound(TilePos::new(0, 0, 0), 30, Some(me));
    run(&mut w, 50);
    assert_eq!(
        w.actor(z).unwrap().alert(),
        Alert::Unaware,
        "nothing seen, nothing heard"
    );
    // Step out to it, and it's after you; step back in, and it stops at the edge.
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(1, 4, 0),
            run: false,
        },
    );
    run(&mut w, 60);
    assert_ne!(
        w.actor(z).unwrap().alert(),
        Alert::Unaware,
        "out here, it's after you"
    );
    w.intent(
        me,
        Intent::WalkTo {
            to: TilePos::new(1, -4, 0),
            run: true,
        },
    );
    let mut stepped_in = false;
    for _ in 0..600 {
        w.step(1);
        stepped_in |= w.is_hallowed(w.actor(z).unwrap().pos);
    }
    assert!(!stepped_in, "it never set foot on hallowed ground");
    assert!(!w.actor(me).unwrap().dead);
    assert_eq!(
        w.actor(z).unwrap().alert(),
        Alert::Unaware,
        "it gave you up"
    );
}

/// Tired at once, then asleep until rested: whether that made home.
fn sleep_the_night(hallowed: bool) -> Option<TilePos> {
    let (mut w, me) = abbey();
    if hallowed {
        hallow(&mut w);
    }
    w.rules.needs.fatigue.rise = 50_000;
    run(&mut w, MINUTE + 1);
    w.rules.needs.fatigue.rise = 0;
    w.intent(me, Intent::Sleep);
    run(&mut w, 16 * 60 * MINUTE);
    assert!(!w.actor(me).unwrap().body.asleep, "rested, it woke");
    w.actor(me).unwrap().home
}

#[test]
fn a_holdouts_bed_is_a_guests_it_doesnt_make_a_home() {
    assert!(sleep_the_night(false).is_some());
    assert_eq!(sleep_the_night(true), None);
}
