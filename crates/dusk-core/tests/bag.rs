//! Bags: the pockets hold so much, and what doesn't fit isn't taken; a bag holds more, and what's
//! in it weighs less; reaching into it is slower; stacks move between the two; a dropped bag
//! falls with what's in it and goes back on with all of it; a new bag takes over the old one's
//! load; what comes to you otherwise is never refused; and a full bag lasts a save, and a death.

use std::sync::Arc;

use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Bag, CreatureDef, Defs, Event, Intent, Item, ItemDef, ItemKind, NoTake,
    Part, Region, Tile, TilePos, World, WoundKind,
};

const CLUB: u16 = 0;
const SATCHEL: u16 = 1;
const KNAPSACK: u16 = 2;
const APPLE: u16 = 3;
const STONE: u16 = 4;
const HERE: TilePos = TilePos::new(0, 0, 0);
/// Ticks more to reach into the bag; to put one on; to move a stack across; to drop.
const FROM_BAG: u32 = 10;
const BAG_ON: u32 = 15;
const SHIFT: u32 = 5;
const TAKE: u32 = 4;
const PER_KG: u32 = 1;

/// A club in hand; a satchel (0.5 kg, holds 6 kg, a quarter lighter) and a knapsack (1 kg,
/// holds 12 kg, 40% lighter); apples (150 g, ten a stack) and stones (2 kg, five a stack); and
/// pockets that hold 2 kg. A revenant (1 health) for a body to rise as, at midnight.
fn defs() -> Defs {
    let item = |id: &str, weight: u32, stack: u16, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight,
        stack,
        kind,
        worth: 0,
    };
    let club = ItemKind::Weapon {
        swing: Swing {
            wind_up: 4,
            recovery: 6,
            endurance: 0,
            hit: 10_000,
            damage: (100, 100),
            noise: 3,
        },
        wear: 0,
        skill: None,
    };
    let bag = |capacity, reduce| ItemKind::Bag(Bag { capacity, reduce });
    let apple = ItemKind::Food {
        nutrition: 5000,
        time: 10,
        comfort: 0,
        spoils: 0,
    };
    let mut revenant = CreatureDef::shambler();
    revenant.id = "revenant".into();
    revenant.stats.health = 1;
    revenant.stats.wander = 0;
    let mut defs = Defs {
        items: vec![
            item("club", 1000, 1, club),
            item("satchel", 500, 1, bag(6000, 250)),
            item("knapsack", 1000, 1, bag(12_000, 400)),
            item("apple", 150, 10, apple),
            item("stone", 2000, 5, ItemKind::Other),
        ],
        wield: Some(CLUB),
        carry: 15_000,
        pockets: 2000,
        take: TAKE,
        take_per_kg: PER_KG,
        drop: 2,
        wield_time: 5,
        unwield_time: 3,
        from_bag: FROM_BAG,
        bag_on: BAG_ON,
        shift: SHIFT,
        revenant: Some((1, 0)),
        ..Defs::default()
    };
    defs.creatures.push(revenant);
    defs
}

/// You on grass, alone, with your needs stilled.
fn field() -> (World, ActorId) {
    let mut w = World::new(9);
    w.defs = Arc::new(defs());
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -5..5 {
        for x in -5..5 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    let me = w.spawn(ActorKind::Character, HERE).unwrap();
    (w, me)
}

/// Asks for an act and steps until it's over (or never began); its events.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..60 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

/// The ticks an act takes, from its start.
fn ticks(w: &mut World, me: ActorId, intent: Intent) -> u32 {
    w.intent(me, intent);
    w.step(1);
    let length = w.actor(me).unwrap().action.expect("it began").length;
    for _ in 0..60 {
        w.step(1);
    }
    length
}

/// Takes the first stack of `def` at your feet.
fn take(w: &mut World, me: ActorId, def: u16) -> Vec<Event> {
    let slot = w
        .store(HERE)
        .iter()
        .position(|i| i.def == def)
        .expect("it's there") as u16;
    act(
        w,
        me,
        Intent::Take {
            at: HERE,
            slot,
            def,
        },
    )
}

/// The first stack of `def` in your pack, in the bag or not: its slot.
fn slot(w: &World, me: ActorId, def: u16, bagged: bool) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == def && i.bagged == bagged)
        .expect("it's in the pack") as u16
}

/// How many of `def` are in your pack, in the bag or not.
fn count(w: &World, me: ActorId, def: u16, bagged: bool) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .filter(|i| i.def == def && i.bagged == bagged)
        .map(|i| i.count)
        .sum()
}

/// Takes up a bag from your feet and puts it on.
fn put_on(w: &mut World, me: ActorId, def: u16) {
    w.put(HERE, Item::new(def, 1));
    take(w, me, def);
    let at = slot(w, me, def, false);
    act(w, me, Intent::Wear { slot: at, def });
    assert_eq!(w.actor(me).unwrap().bag.map(|b| b.def), Some(def));
}

#[test]
fn a_bag_with_no_room_in_the_pockets_goes_straight_on_a_bare_back() {
    let (mut w, me) = field();
    w.put(HERE, Item::new(STONE, 1));
    take(&mut w, me, STONE);
    assert_eq!(w.filled(me).0, 2000, "the pockets are full");
    w.put(HERE, Item::new(SATCHEL, 1));
    let seen = take(&mut w, me, SATCHEL);
    assert!(seen.contains(&Event::Wore {
        who: me,
        item: SATCHEL
    }));
    assert_eq!(w.actor(me).unwrap().bag.map(|b| b.def), Some(SATCHEL));
    // With one on, another there's no room for stays where it is.
    w.put(HERE, Item::new(KNAPSACK, 1));
    w.put(HERE, Item::new(STONE, 3));
    take(&mut w, me, STONE);
    assert_eq!(w.filled(me), (2000, 6000));
    assert_eq!(w.can_take(me, HERE, 0, KNAPSACK), Err(NoTake::NoRoom));
}

/// The satchel on, three stones in it, dropped at your feet: a full bag on the ground.
fn full_satchel_on_the_ground(w: &mut World, me: ActorId) {
    put_on(w, me, SATCHEL);
    w.put(HERE, Item::new(STONE, 3));
    take(w, me, STONE);
    assert_eq!(count(w, me, STONE, true), 3);
    act(w, me, Intent::DropBag);
}

#[test]
fn the_pockets_hold_so_much_and_what_doesnt_fit_stays_where_it_was() {
    let (mut w, me) = field();
    w.put(HERE, Item::new(APPLE, 20));
    assert_eq!(w.can_take(me, HERE, 0, APPLE), Ok(10), "1.5 of 2 kg");
    take(&mut w, me, APPLE);
    assert_eq!(
        w.can_take(me, HERE, 0, APPLE),
        Ok(3),
        "0.5 kg left: 3 of 10"
    );
    let seen = take(&mut w, me, APPLE);
    assert!(seen.contains(&Event::Took {
        who: me,
        item: APPLE,
        count: 3
    }));
    assert_eq!(count(&w, me, APPLE, false), 13);
    assert_eq!(w.store(HERE)[0].count, 7, "the rest stay");
    assert_eq!(w.can_take(me, HERE, 0, APPLE), Err(NoTake::NoRoom));
    // With no room, a take never begins.
    w.intent(
        me,
        Intent::Take {
            at: HERE,
            slot: 0,
            def: APPLE,
        },
    );
    w.step(1);
    assert!(w.actor(me).unwrap().action.is_none());
    assert_eq!(w.store(HERE)[0].count, 7);
}

#[test]
fn a_bag_holds_more_and_what_is_in_it_weighs_less() {
    let (mut w, me) = field();
    put_on(&mut w, me, SATCHEL);
    assert!(w.actor(me).unwrap().items.is_empty(), "it's on your back");
    assert_eq!(w.load(me), 1000 + 500, "the club and the satchel");
    // Five stones: three fill the satchel, one the pockets; one stays.
    w.put(HERE, Item::new(STONE, 5));
    assert_eq!(w.can_take(me, HERE, 0, STONE), Ok(4));
    take(&mut w, me, STONE);
    assert_eq!(
        (count(&w, me, STONE, true), count(&w, me, STONE, false)),
        (3, 1),
        "the bag first, then the pockets"
    );
    assert_eq!(w.store(HERE)[0].count, 1);
    assert_eq!(w.filled(me), (2000, 6000));
    // The bag's 6 kg weigh a quarter less: 4.5.
    assert_eq!(w.load(me), 1000 + 2000 + 4500 + 500);
}

#[test]
fn reaching_into_the_bag_is_slower_than_into_the_pockets() {
    let (mut w, me) = field();
    put_on(&mut w, me, SATCHEL);
    w.put(HERE, Item::new(APPLE, 2));
    take(&mut w, me, APPLE);
    assert_eq!(count(&w, me, APPLE, true), 2, "into the bag");
    let use_apple = |w: &mut World, bagged: bool| {
        let at = slot(w, me, APPLE, bagged);
        ticks(
            w,
            me,
            Intent::Use {
                slot: at,
                def: APPLE,
            },
        )
    };
    let from_bag = use_apple(&mut w, true);
    let at = slot(&w, me, APPLE, true);
    act(
        &mut w,
        me,
        Intent::Shift {
            slot: at,
            def: APPLE,
        },
    );
    assert_eq!(count(&w, me, APPLE, false), 1, "moved to the pockets");
    let from_pockets = use_apple(&mut w, false);
    assert_eq!(from_bag, from_pockets + FROM_BAG);
}

#[test]
fn a_stack_moves_between_pockets_and_bag_as_far_as_it_fits() {
    let (mut w, me) = field();
    // A stone in the pockets with no bag: nowhere to move it.
    w.put(HERE, Item::new(STONE, 1));
    take(&mut w, me, STONE);
    assert_eq!(w.can_shift(me, slot(&w, me, STONE, false), STONE), 0);
    // The satchel on (the pockets are full: straight on), filled: three stones in it, the one
    // in the pockets.
    w.put(HERE, Item::new(SATCHEL, 1));
    take(&mut w, me, SATCHEL);
    w.put(HERE, Item::new(STONE, 3));
    take(&mut w, me, STONE);
    assert_eq!(w.filled(me), (2000, 6000));
    let bagged = slot(&w, me, STONE, true);
    assert_eq!(w.can_shift(me, bagged, STONE), 0, "the pockets are full");
    // Drop the pocket stone, and one of the bag's three fits there.
    let loose = slot(&w, me, STONE, false);
    act(
        &mut w,
        me,
        Intent::Drop {
            slot: loose,
            def: STONE,
        },
    );
    let bagged = slot(&w, me, STONE, true);
    assert_eq!(w.can_shift(me, bagged, STONE), 1);
    let took = ticks(
        &mut w,
        me,
        Intent::Shift {
            slot: bagged,
            def: STONE,
        },
    );
    assert_eq!(took, SHIFT);
    assert_eq!(
        (count(&w, me, STONE, true), count(&w, me, STONE, false)),
        (2, 1)
    );
}

#[test]
fn a_dropped_bag_falls_with_whats_in_it_and_goes_back_on_with_all_of_it() {
    let (mut w, me) = field();
    full_satchel_on_the_ground(&mut w, me);
    let a = w.actor(me).unwrap();
    assert!(a.bag.is_none());
    assert!(a.items.is_empty(), "what was in it went with it");
    assert_eq!(w.load(me), 1000, "only the club");
    let ground = w.store(HERE);
    assert_eq!(ground.len(), 1, "one thing on the ground: the bag");
    assert_eq!(ground[0].def, SATCHEL);
    assert_eq!(w.weight_of(&ground[0]), 500 + 6000);
    // Taken up, it's as heavy as it is: 6.5 kg.
    let took = ticks(
        &mut w,
        me,
        Intent::Take {
            at: HERE,
            slot: 0,
            def: SATCHEL,
        },
    );
    assert_eq!(took, TAKE + 6 * PER_KG);
    let a = w.actor(me).unwrap();
    assert_eq!(a.bag.map(|b| b.def), Some(SATCHEL), "back on");
    assert_eq!(count(&w, me, STONE, true), 3, "with all of it");
    assert!(w.store(HERE).is_empty());
}

#[test]
fn a_full_bag_taken_with_one_on_empties_into_yours() {
    let (mut w, me) = field();
    full_satchel_on_the_ground(&mut w, me);
    put_on(&mut w, me, KNAPSACK);
    let seen = take(&mut w, me, SATCHEL);
    assert!(seen.contains(&Event::Took {
        who: me,
        item: STONE,
        count: 3
    }));
    assert_eq!(count(&w, me, STONE, true), 3, "into the knapsack");
    assert_eq!(count(&w, me, SATCHEL, true), 1, "and the satchel after");
    assert!(w.store(HERE).is_empty());
}

#[test]
fn a_full_bag_taken_with_one_on_gives_what_fits_and_keeps_the_rest() {
    let (mut w, me) = field();
    full_satchel_on_the_ground(&mut w, me);
    put_on(&mut w, me, KNAPSACK);
    // Six stones fill the knapsack; the pockets have room for one more.
    w.put(HERE, Item::new(STONE, 5));
    w.put(HERE, Item::new(STONE, 1));
    take(&mut w, me, STONE);
    take(&mut w, me, STONE);
    assert_eq!(w.filled(me), (0, 12_000));
    let satchel = w.store(HERE).iter().position(|i| i.def == SATCHEL).unwrap() as u16;
    assert_eq!(w.can_take(me, HERE, satchel, SATCHEL), Ok(1));
    take(&mut w, me, SATCHEL);
    assert_eq!(count(&w, me, STONE, false), 1, "one into the pockets");
    let ground = w.store(HERE);
    assert_eq!(ground.len(), 1, "the satchel stays");
    assert_eq!(w.weight_of(&ground[0]), 500 + 4000, "with two stones in it");
    // Now there's no room for any of it.
    assert_eq!(w.can_take(me, HERE, 0, SATCHEL), Err(NoTake::NoRoom));
}

#[test]
fn a_new_bag_takes_over_what_was_in_the_old_one() {
    let (mut w, me) = field();
    put_on(&mut w, me, SATCHEL);
    w.put(HERE, Item::new(STONE, 3));
    take(&mut w, me, STONE);
    w.put(HERE, Item::new(KNAPSACK, 1));
    take(&mut w, me, KNAPSACK);
    let at = slot(&w, me, KNAPSACK, false);
    let took = ticks(
        &mut w,
        me,
        Intent::Wear {
            slot: at,
            def: KNAPSACK,
        },
    );
    assert_eq!(took, BAG_ON, "from the pockets: no reaching into the bag");
    assert_eq!(w.actor(me).unwrap().bag.map(|b| b.def), Some(KNAPSACK));
    assert_eq!(count(&w, me, STONE, true), 3, "the stones moved across");
    assert_eq!(count(&w, me, SATCHEL, true), 1, "the satchel went in too");
    // 6.5 kg in the knapsack, 40% lighter: 3.9, and the knapsack itself.
    assert_eq!(w.load(me), 1000 + 3900 + 1000);
}

#[test]
fn what_comes_to_you_otherwise_is_never_refused() {
    let (mut w, me) = field();
    w.put(HERE, Item::new(STONE, 1));
    take(&mut w, me, STONE);
    assert_eq!(w.filled(me).0, 2000, "the pockets are full");
    // Putting the club away: it goes in all the same, over full.
    act(&mut w, me, Intent::Unwield);
    assert_eq!(w.filled(me).0, 3000);
    assert_eq!(count(&w, me, CLUB, false), 1);
    w.put(HERE, Item::new(APPLE, 1));
    let apple = w.store(HERE).len() as u16 - 1;
    assert_eq!(w.can_take(me, HERE, apple, APPLE), Err(NoTake::NoRoom));
}

#[test]
fn a_full_bag_lasts_a_save() {
    let (mut a, me) = field();
    full_satchel_on_the_ground(&mut a, me);
    let bytes = a.save("bags", b"");
    let (mut b, _) = field();
    b.restore(&bytes).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(a.state_hash(), b.state_hash(), "the same world");
    take(&mut b, me, SATCHEL);
    assert_eq!(
        count(&b, me, STONE, true),
        3,
        "and the stones are still in it"
    );
}

#[test]
fn a_bodys_bag_rises_with_its_revenant_and_falls_full_when_its_put_down() {
    let (mut w, me) = field();
    w.rules.clock.start = 23 * 60;
    w.set_region(Region {
        start: Some(TilePos::new(4, 4, 0)),
        ..Region::default()
    });
    full_satchel_on_the_ground(&mut w, me);
    take(&mut w, me, SATCHEL);
    while !w.actor(me).unwrap().dead {
        w.wound(me, WoundKind::Bite, Part::Torso);
    }
    let mut revenant = None;
    for _ in 0..70 * 25 {
        w.step(1);
        for e in w.events() {
            if let Event::Risen { revenant: r, .. } = e {
                revenant = Some(r);
            }
        }
    }
    let r = revenant.expect("it rose at midnight");
    let carried = w.actor(r).unwrap().items.clone();
    let bag = carried
        .iter()
        .find(|i| i.def == SATCHEL)
        .expect("it carries the satchel");
    assert_eq!(w.weight_of(bag), 500 + 6000, "full");
    let at = w.actor(r).unwrap().pos;
    w.wound(r, WoundKind::Bite, Part::Torso);
    assert!(w.actor(r).unwrap().dead);
    let ground = w.store(at);
    let bag = ground.iter().find(|i| i.def == SATCHEL).expect("it fell");
    assert_eq!(w.weight_of(bag), 500 + 6000, "with the stones still in it");
}
