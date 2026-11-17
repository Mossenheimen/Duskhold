//! Potions and the belt (`potion.rs`): a potion that comes to you goes on the belt first, a loop a
//! kind; it's quickest drunk from there; each works on you a while, life and mana coming back a
//! little at a time, the lasting ones (haste, iron skin, vigour, courage) wearing off with news;
//! and a stack goes on the belt and off it, loose once it's dropped.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Class, Defs, Dir8, Draught, Effect, Event, Intent, Item, ItemDef, ItemKind,
    Moodle, Tile, TilePos, World,
};

const LIFE: u16 = 0;
const MANA: u16 = 1;
const HASTE: u16 = 2;
const IRON: u16 = 3;
const VIGOUR: u16 = 4;
const COURAGE: u16 = 5;
const BANDAGE: u16 = 6;
const HERE: TilePos = TilePos::new(0, 0, 0);

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A potion: a second to drink from the belt.
fn potion(id: &str, effect: Effect, amount: i32, lasts: u32) -> ItemDef {
    ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 300,
        stack: 5,
        kind: ItemKind::Potion(Draught {
            effect,
            amount,
            lasts,
            time: 10,
        }),
        worth: 0,
    }
}

/// The six, as the game's are but quicker (courage ten seconds), and a bandage.
fn defs() -> Defs {
    Defs {
        items: vec![
            potion("life", Effect::Life, 350, 40),
            potion("mana", Effect::Mana, 25_000, 40),
            potion("haste", Effect::Haste, 300, 300),
            potion("iron skin", Effect::IronSkin, 400, 300),
            potion("vigour", Effect::Vigour, 500, 600),
            potion("courage", Effect::Courage, 1000, 100),
            ItemDef {
                id: "bandage".into(),
                name: "bandage".into(),
                weight: 100,
                stack: 10,
                kind: ItemKind::Bandage {
                    time: 20,
                    salve: 1000,
                },
                worth: 0,
            },
        ],
        carry: 30_000,
        take: 1,
        drop: 1,
        shift: 1,
        ..Defs::default()
    }
}

/// A field of grass, the needs and healing stilled. The belt is the rules' own: four loops of
/// three, and a second's digging in the pockets.
fn field(seed: u64) -> World {
    let mut w = World::new(seed);
    w.defs = Arc::new(defs());
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    w.rules.needs.heal = 0;
    for y in -3..=3 {
        for x in -5..=20 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    w
}

fn me_in(w: &mut World) -> ActorId {
    w.spawn(ActorKind::Character, HERE).unwrap()
}

/// Takes everything on the floor where you stand, a stack at a time.
fn take_all(w: &mut World, me: ActorId) {
    for _ in 0..20 {
        let Some(first) = w.store(HERE).first().copied() else {
            return;
        };
        w.intent(
            me,
            Intent::Take {
                at: HERE,
                slot: 0,
                def: first.def,
            },
        );
        w.step(3);
    }
    panic!("the floor never emptied");
}

/// The slot of the first stack of `def` on the belt (`belted`) or off it.
fn slot_of(w: &World, me: ActorId, def: u16, belted: bool) -> Option<u16> {
    let items = &w.actor(me).unwrap().items;
    let n = items
        .iter()
        .position(|i| i.def == def && (i.belt > 0) == belted)?;
    Some(n as u16)
}

/// Drinks the potion at `slot`: the ticks it took.
fn drink(w: &mut World, me: ActorId, slot: u16, def: u16) -> u32 {
    w.intent(me, Intent::Use { slot, def });
    for t in 1..=60 {
        w.step(1);
        if w.events()
            .any(|e| e == Event::Quaffed { who: me, item: def })
        {
            return t;
        }
    }
    panic!("it was never drunk");
}

fn moodle(w: &World, me: ActorId, m: Moodle) -> bool {
    w.moodles(me).iter().any(|&(x, _)| x == m)
}

#[test]
fn a_potion_that_comes_to_you_goes_on_the_belt_first_a_loop_a_kind() {
    let mut w = field(1);
    let me = me_in(&mut w);
    for (def, n) in [(LIFE, 4), (MANA, 1), (HASTE, 1), (IRON, 1)] {
        w.put(HERE, Item::new(def, n));
    }
    take_all(&mut w, me);
    let a = w.actor(me).unwrap();
    let held = |slot: Option<u16>| {
        slot.map(|s| {
            let i = a.items[usize::from(s)];
            (i.def, i.count, i.belt)
        })
    };
    let belt: Vec<_> = w.belt(me).into_iter().map(held).collect();
    // Three of life fill the first loop, the fourth starts the next empty one.
    assert_eq!(
        belt,
        [
            Some((LIFE, 3, 1)),
            Some((LIFE, 1, 2)),
            Some((MANA, 1, 3)),
            Some((HASTE, 1, 4))
        ]
    );
    // The belt full, the last went into the pockets.
    assert!(
        a.items
            .iter()
            .any(|i| i.def == IRON && i.belt == 0 && !i.bagged)
    );
    // What's on the belt takes none of the pockets' room.
    assert_eq!(w.filled(me).0, 300, "only the iron skin in the pockets");
}

#[test]
fn drinking_from_the_belt_is_quicker_than_digging_in_the_pockets() {
    let mut w = field(1);
    let me = me_in(&mut w);
    w.put(HERE, Item::new(LIFE, 1));
    take_all(&mut w, me);
    // One more, given straight into the pockets.
    w.give(me, Item::new(LIFE, 1));
    let belted = slot_of(&w, me, LIFE, true).expect("one on the belt");
    assert_eq!(drink(&mut w, me, belted, LIFE), 10, "a second off the belt");
    let loose = slot_of(&w, me, LIFE, false).expect("one in the pockets");
    assert_eq!(
        drink(&mut w, me, loose, LIFE),
        20,
        "another, digging for it"
    );
}

#[test]
fn a_potion_of_life_gives_its_health_back_a_little_each_tick_and_all_of_it() {
    let mut w = field(1);
    w.rules.combat.character_health = 300;
    let me = me_in(&mut w);
    w.rules.combat.character_health = 1000;
    w.give(me, Item::new(LIFE, 3));
    let health = |w: &World| w.actor(me).unwrap().body.health;
    drink(&mut w, me, 0, LIFE);
    // Its first share at once, not all of it.
    assert!(health(&w) > 300 && health(&w) < 400, "{}", health(&w));
    assert!(moodle(&w, me, Moodle::Reviving));
    w.step(39);
    assert_eq!(health(&w), 650, "35 points, every one");
    assert!(!moodle(&w, me, Moodle::Reviving));
    // Two drunk together both run; and none past the most you can have.
    drink(&mut w, me, 0, LIFE);
    drink(&mut w, me, 0, LIFE);
    assert_eq!(w.doses(me).len(), 2, "both at once");
    w.step(40);
    assert_eq!(health(&w), 1000);
    assert!(w.doses(me).is_empty());
}

#[test]
fn a_mana_potion_needs_mana_to_fill_and_fills_it() {
    // No class with mana: nothing to fill, and it isn't drunk.
    let mut w = field(1);
    let me = me_in(&mut w);
    w.give(me, Item::new(MANA, 1));
    w.intent(me, Intent::Use { slot: 0, def: MANA });
    w.step(5);
    assert!(w.actor(me).unwrap().action.is_none());
    assert_eq!(w.actor(me).unwrap().items[0].count, 1, "still there");
    // A wizard holding 40 points, with room for 80: it fills 25 of them, by its end.
    let mut w = field(1);
    Arc::make_mut(&mut w.defs).classes = vec![Class {
        id: "wizard".into(),
        name: "Wizard".into(),
        mana: 40_000,
        ..Class::default()
    }];
    let me = me_in(&mut w);
    assert_eq!(w.mana(me), 40_000);
    Arc::make_mut(&mut w.defs).classes[0].mana = 80_000;
    w.give(me, Item::new(MANA, 1));
    drink(&mut w, me, 0, MANA);
    assert!(moodle(&w, me, Moodle::ManaRising));
    w.step(39);
    assert_eq!(w.mana(me), 65_000);
}

#[test]
fn haste_takes_you_further_a_tick_till_it_wears_off_with_news() {
    let mut w = field(1);
    let me = me_in(&mut w);
    w.give(me, Item::new(HASTE, 1));
    drink(&mut w, me, 0, HASTE);
    assert!(moodle(&w, me, Moodle::Quickened));
    let walk = |w: &mut World| {
        let x = w.actor(me).unwrap().position().0;
        w.intent(
            me,
            Intent::Move {
                dir: Dir8::E,
                run: false,
            },
        );
        w.step(1);
        w.intent(me, Intent::Idle);
        ((w.actor(me).unwrap().position().0 - x) * 1000.0).round() as i32
    };
    // A walk's 150 thousandths of a tile a tick, 30% more.
    assert_eq!(walk(&mut w), 195);
    let mut waned = false;
    for _ in 0..300 {
        w.step(1);
        waned |= w.events().any(|e| {
            e == Event::Waned {
                who: me,
                effect: Effect::Haste,
            }
        });
    }
    assert!(waned, "its end is news");
    assert!(!moodle(&w, me, Moodle::Quickened));
    assert_eq!(walk(&mut w), 150);
}

/// The harm and the bleeding of the first blow one of the dead lands on you, after a potion.
fn first_blow(potion: u16) -> (i32, u32) {
    let mut w = field(3);
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    w.rules.combat.character_health = 1_000_000;
    let me = me_in(&mut w);
    w.give(me, Item::new(potion, 1));
    drink(&mut w, me, 0, potion);
    // North of you, facing south (as everyone starts): it sees you.
    w.creature_mut(0).wander = 0;
    w.spawn(ActorKind::Creature(0), at(0, -1)).unwrap();
    for _ in 0..300 {
        w.step(1);
        let struck = w.events().find_map(|e| match e {
            Event::Struck { target, damage, .. } if target == me => Some(damage),
            _ => None,
        });
        if let Some(damage) = struck {
            let wounds = &w.actor(me).unwrap().body.wounds;
            return (damage, wounds.iter().map(|w| w.bleeding).sum());
        }
    }
    panic!("it never landed a blow");
}

#[test]
fn iron_skin_takes_harm_off_a_blow_and_bleeding_off_its_wound() {
    // The same world, the same blow: after courage (which does nothing for a blow), then after
    // iron skin.
    let (plain, plain_bleeding) = first_blow(COURAGE);
    let (iron, iron_bleeding) = first_blow(IRON);
    assert!(plain > 0);
    assert_eq!(iron, plain * 600 / 1000, "40% less harm");
    assert_eq!(iron_bleeding, plain_bleeding * 600 / 1000);
}

#[test]
fn vigour_gives_your_breath_back_and_running_costs_less() {
    let spent = |potion: u16| {
        let mut w = field(1);
        let me = me_in(&mut w);
        w.give(me, Item::new(potion, 1));
        // Out of breath first, running.
        w.intent(
            me,
            Intent::Move {
                dir: Dir8::E,
                run: true,
            },
        );
        w.step(30);
        w.intent(me, Intent::Idle);
        let tired = w.actor(me).unwrap().body.endurance;
        drink(&mut w, me, 0, potion);
        let now = w.actor(me).unwrap().body.endurance;
        w.intent(
            me,
            Intent::Move {
                dir: Dir8::W,
                run: true,
            },
        );
        w.step(10);
        (tired, now, now - w.actor(me).unwrap().body.endurance)
    };
    let (tired, after, vigorous) = spent(VIGOUR);
    assert!(tired < 1000);
    assert_eq!(after, 1000, "your breath back at once");
    let (_, _, plain) = spent(COURAGE);
    assert!(plain > 0);
    assert!(
        vigorous * 10 >= plain * 4 && vigorous * 10 <= plain * 6,
        "half the breath: {vigorous} against {plain}"
    );
}

#[test]
fn courage_takes_the_dread_away_and_none_rises_till_it_wears_off() {
    let mut w = field(1);
    // Everywhere's dark: dread rises.
    w.rules.dread.dark_below = u32::MAX;
    let me = me_in(&mut w);
    w.give(me, Item::new(COURAGE, 1));
    let minute = w.rules.clock.ticks_per_minute;
    w.step(minute * 5);
    assert!(w.dread(me) > 0, "the dark tells");
    drink(&mut w, me, 0, COURAGE);
    assert_eq!(w.dread(me), 0, "gone at once");
    assert!(moodle(&w, me, Moodle::Fearless));
    for _ in 0..85 {
        w.step(1);
        assert_eq!(w.dread(me), 0, "none rises while it lasts");
    }
    w.step(20);
    assert!(!moodle(&w, me, Moodle::Fearless));
    w.step(minute * 3);
    assert!(w.dread(me) > 0, "worn off, the dark tells again");
}

#[test]
fn another_of_a_lasting_one_makes_it_the_longer() {
    let mut w = field(1);
    let me = me_in(&mut w);
    w.give(me, Item::new(HASTE, 2));
    drink(&mut w, me, 0, HASTE);
    // A dose works on the tick it's drunk, too.
    let fresh = w.doses(me)[0].left;
    assert_eq!(fresh, 299);
    w.step(100);
    drink(&mut w, me, 0, HASTE);
    let doses = w.doses(me);
    assert_eq!(doses.len(), 1, "one haste");
    assert_eq!(doses[0].left, fresh, "as long as a new one");
}

#[test]
fn a_stack_goes_on_the_belt_and_off_it_and_drops_loose() {
    let mut w = field(1);
    let me = me_in(&mut w);
    w.give(me, Item::new(LIFE, 2));
    w.give(me, Item::new(BANDAGE, 1));
    let bandage = slot_of(&w, me, BANDAGE, false).unwrap();
    assert_eq!(w.can_belt(me, bandage, BANDAGE), 0, "no potion");
    let life = slot_of(&w, me, LIFE, false).unwrap();
    assert_eq!(w.can_belt(me, life, LIFE), 2);
    w.intent(
        me,
        Intent::Belt {
            slot: life,
            def: LIFE,
        },
    );
    w.step(3);
    let on = slot_of(&w, me, LIFE, true).expect("on the belt");
    assert_eq!(w.belt(me)[0], Some(on));
    assert_eq!(w.actor(me).unwrap().items[usize::from(on)].count, 2);
    // Off it again, into the pockets.
    w.intent(
        me,
        Intent::Belt {
            slot: on,
            def: LIFE,
        },
    );
    w.step(3);
    assert!(w.belt(me).iter().all(Option::is_none));
    let off = slot_of(&w, me, LIFE, false).expect("in the pockets");
    // On, then dropped: it lies loose.
    w.intent(
        me,
        Intent::Belt {
            slot: off,
            def: LIFE,
        },
    );
    w.step(3);
    let on = slot_of(&w, me, LIFE, true).unwrap();
    w.intent(
        me,
        Intent::Drop {
            slot: on,
            def: LIFE,
        },
    );
    w.step(3);
    let floor = w.store(HERE);
    assert!(
        floor
            .iter()
            .any(|i| i.def == LIFE && i.count == 2 && i.belt == 0)
    );
    assert!(w.belt(me).iter().all(Option::is_none));
}

#[test]
fn a_potions_card_and_line_say_what_it_does() {
    let w = field(1);
    assert_eq!(
        w.defs.potion_line(LIFE),
        "every wound closed, the fever cured, and 35 health back over 4 s"
    );
    assert_eq!(
        w.defs.potion_line(HASTE),
        "30% faster on your feet for 30 s"
    );
    assert_eq!(w.defs.potion_line(BANDAGE), "", "no potion");
    let card = w.card(&Item::new(IRON, 1), None);
    let says = |text: &str, value: &str| card.iter().any(|l| l.text == text && l.value == value);
    assert!(says("Potion", ""));
    assert!(says("Less harm from blows", "40%"));
    assert!(says("Lasts", "30 s"));
    assert!(says("To drink, from the belt", "1 s") && says("From the pack", "2 s"));
}

#[cfg(feature = "save")]
#[test]
fn the_belt_and_what_works_on_you_outlast_a_save() {
    let mut w = field(1);
    let me = me_in(&mut w);
    w.put(HERE, Item::new(HASTE, 2));
    take_all(&mut w, me);
    let on = slot_of(&w, me, HASTE, true).unwrap();
    drink(&mut w, me, on, HASTE);
    let bytes = w.save("test", &[]);
    let mut back = field(1);
    back.restore(&bytes).unwrap();
    assert_eq!(back.state_hash(), w.state_hash());
    assert_eq!(back.doses(me), w.doses(me));
    assert_eq!(back.belt(me), w.belt(me));
}
