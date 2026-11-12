//! Wounds by where they land: a hurt arm swings slower and less surely, a hurt leg is slower and
//! running costs it more, a hurt head aims worse and sees less far, and a wound to the body hurts
//! more. A bandage eases each, as it heals; the dead feel none of it.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Group, Intent, Item, ItemDef, ItemKind, Moodle, Part, Tile,
    TilePos, World, WoundKind,
};

/// Ticks in a game minute.
const MINUTE: u32 = 25;
const BANDAGE: u16 = 0;

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// Open grass by day, nobody wandering, and you at its west end needing nothing, with bandages.
fn field() -> (World, ActorId) {
    let mut w = World::new(8);
    w.rules.clock.start = 12 * 60;
    w.defs = Arc::new(Defs {
        items: vec![ItemDef {
            id: "bandage".into(),
            name: "bandage".into(),
            weight: 100,
            stack: 10,
            kind: ItemKind::Bandage {
                time: 10,
                salve: 1000,
            },
            worth: 0,
        }],
        kit: vec![Item::new(BANDAGE, 4)],
        carry: 30_000,
        ..Defs::default()
    });
    w.creature_mut(0).wander = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -25..25 {
        for x in -10..40 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
    (w, me)
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) {
    w.intent(me, intent);
    for _ in 0..100 {
        w.step(1);
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
}

fn bandage(w: &mut World, me: ActorId) {
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == BANDAGE)
        .unwrap() as u16;
    act(w, me, Intent::Use { slot, def: BANDAGE });
}

fn moodle(w: &World, me: ActorId, m: Moodle) -> u8 {
    w.moodles(me)
        .iter()
        .find(|&&(x, _)| x == m)
        .map_or(0, |&(_, level)| level)
}

#[test]
fn a_hurt_arm_swings_slower_and_lands_less_and_a_bandage_eases_it() {
    let (mut w, me) = field();
    let whole = w.swing(me);
    // A laceration: hurt.
    w.wound(me, WoundKind::Laceration, Part::LeftArm);
    assert_eq!(w.hurt_level(me, Group::Arms), 1);
    assert_eq!(moodle(&w, me, Moodle::Arm), 1);
    let hurt = w.swing(me);
    assert_eq!(hurt.wind_up, whole.wind_up * 1150 / 1000);
    assert_eq!(hurt.recovery, whole.recovery * 1150 / 1000);
    assert_eq!(hurt.hit, whole.hit * 900 / 1000);
    // And a bite on the hand, the same arm's part of the body: badly hurt.
    w.wound(me, WoundKind::Bite, Part::RightHand);
    assert_eq!(w.hurt_level(me, Group::Arms), 2);
    let bad = w.swing(me);
    assert_eq!(bad.wind_up, whole.wind_up * 1300 / 1000);
    assert_eq!(bad.hit, whole.hit * 800 / 1000);
    assert_eq!(w.hurt_level(me, Group::Legs), 0, "nothing on the legs");

    // A laceration alone, dressed: half the pain, and as it starts to heal, whole again.
    let (mut w, me) = field();
    w.wound(me, WoundKind::Laceration, Part::RightArm);
    bandage(&mut w, me);
    w.step(2 * MINUTE);
    assert_eq!(w.hurt_level(me, Group::Arms), 0);
    assert_eq!(w.swing(me), whole);
}

#[test]
fn a_hurt_leg_is_slower_and_running_costs_it_more() {
    // Six seconds' walk east, whole and with a laceration on the leg.
    let walked = |hurt: bool| {
        let (mut w, me) = field();
        if hurt {
            w.wound(me, WoundKind::Laceration, Part::LeftLeg);
            assert_eq!(moodle(&w, me, Moodle::Leg), 1, "limping");
        }
        w.intent(
            me,
            Intent::Move {
                dir: Dir8::E,
                run: false,
            },
        );
        w.step(60);
        w.actor(me).unwrap().pos.x
    };
    let (whole, hurt) = (walked(false), walked(true));
    assert!(
        hurt < whole && hurt * 1000 >= whole * 700,
        "{hurt} of {whole}"
    );

    // Two seconds' run: the breath it costs.
    let ran = |hurt: bool| {
        let (mut w, me) = field();
        if hurt {
            w.wound(me, WoundKind::Laceration, Part::RightFoot);
        }
        w.intent(
            me,
            Intent::Move {
                dir: Dir8::E,
                run: true,
            },
        );
        w.step(20);
        1000 - w.actor(me).unwrap().body.endurance
    };
    assert!(ran(true) > ran(false));
}

#[test]
fn a_hurt_head_aims_worse_and_sees_less_far() {
    let (mut w, me) = field();
    let whole = w.swing(me);
    // Due south, 18 tiles: by day, in sight.
    let far = at(0, 18);
    assert!(w.makes_out(me, far));
    w.wound(me, WoundKind::Laceration, Part::Head);
    assert_eq!(moodle(&w, me, Moodle::Head), 1);
    assert_eq!(w.swing(me).hit, whole.hit * 900 / 1000);
    assert_eq!(w.swing(me).wind_up, whole.wind_up, "the arms are whole");
    assert_eq!(w.sight_range(me), 17);
    assert!(!w.makes_out(me, far), "past what a hurt head sees");
    assert!(w.makes_out(me, at(0, 15)));
}

#[test]
fn a_wound_to_the_body_hurts_half_again() {
    let (mut w, me) = field();
    w.wound(me, WoundKind::Laceration, Part::Torso);
    assert_eq!(w.pain(me), 30_000, "a laceration's 20, half again");
    assert_eq!(moodle(&w, me, Moodle::Pain), 2, "in pain");
    let (mut w, me) = field();
    w.wound(me, WoundKind::Laceration, Part::LeftLeg);
    assert_eq!(w.pain(me), 20_000);
    assert_eq!(moodle(&w, me, Moodle::Pain), 1, "only sore");
}

#[test]
fn the_dead_feel_none_of_it() {
    let (mut w, _) = field();
    let z = w.spawn(ActorKind::Creature(0), at(10, 0)).unwrap();
    let whole = w.swing(z);
    w.wound(z, WoundKind::Laceration, Part::LeftArm);
    w.wound(z, WoundKind::Laceration, Part::Head);
    assert_eq!(w.swing(z), whole);
}
