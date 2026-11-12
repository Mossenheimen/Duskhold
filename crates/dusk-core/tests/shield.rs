//! Shields: carried on the off arm instead of a light; bracing blocks blows from ahead, for
//! breath and wear, and not from behind or held; a shove with one on the arm is a bash; bracing
//! is slow and never a run.

use std::sync::Arc;

use dusk_core::shield::Guard;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Event, Intent, Item, ItemDef, ItemKind, Tile, TilePos, World,
};

const SHIELD: u16 = 0;
const TORCH: u16 = 1;

/// A shield that blocks every blow from ahead and makes every shove a knock-down, losing 40 of
/// its 100 a block; and a torch. You carry both.
fn defs() -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 1000,
        stack: 1,
        kind,
        worth: 0,
    };
    Defs {
        items: vec![
            item(
                "shield",
                ItemKind::Shield(Guard {
                    block: 10_000,
                    endurance: 50,
                    wear: 400,
                    bash: 10_000,
                }),
            ),
            item(
                "torch",
                ItemKind::Light {
                    radius: 5,
                    wane: 17,
                },
            ),
        ],
        kit: vec![Item::new(SHIELD, 1), Item::new(TORCH, 1)],
        carry: 30_000,
        wield_time: 3,
        light_time: 3,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A strip of grass running south, you at (0, 2) needing nothing.
fn strip() -> (World, ActorId) {
    let mut w = World::new(3);
    w.defs = Arc::new(defs());
    for y in -2..12 {
        for x in -3..4 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    w.creature_mut(0).wander = 0;
    let me = w.spawn(ActorKind::Character, at(0, 2)).unwrap();
    (w, me)
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) {
    w.intent(me, intent);
    for _ in 0..60 {
        w.step(1);
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
}

fn slot(w: &World, me: ActorId, def: u16) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == def)
        .unwrap() as u16
}

fn take_up_shield(w: &mut World, me: ActorId) {
    let slot = slot(w, me, SHIELD);
    act(w, me, Intent::Wield { slot, def: SHIELD });
}

#[test]
fn a_shield_and_a_light_share_the_off_arm() {
    let (mut w, me) = strip();
    let slot = slot(&w, me, TORCH);
    act(&mut w, me, Intent::Light { slot, def: TORCH });
    take_up_shield(&mut w, me);
    let a = w.actor(me).unwrap();
    assert_eq!(a.shield.map(|s| s.def), Some(SHIELD));
    assert!(a.light.is_none(), "the torch went out, into the pack");
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == TORCH)
        .unwrap() as u16;
    act(&mut w, me, Intent::Light { slot, def: TORCH });
    let a = w.actor(me).unwrap();
    assert!(
        a.light.is_some() && a.shield.is_none(),
        "the shield went in the pack"
    );
    // And put away, it's in the pack.
    take_up_shield(&mut w, me);
    act(&mut w, me, Intent::Unshield);
    assert!(w.actor(me).unwrap().shield.is_none());
}

/// One of the dead at `from`, landing every blow it swings, after you: the wounds you take in
/// three seconds, whether any was blocked, the least breath you had, and what's left of the
/// shield.
fn blows(brace: bool, from: TilePos, facing: Dir8) -> (usize, usize, i32, Option<u16>) {
    let (mut w, me) = strip();
    take_up_shield(&mut w, me);
    let c = w.creature_mut(0);
    c.attack.hit = 10_000;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    w.spawn(ActorKind::Creature(0), from).unwrap();
    // Face that way, then brace (or not).
    w.intent(
        me,
        Intent::Move {
            dir: facing,
            run: false,
        },
    );
    w.step(1);
    w.intent(me, Intent::Idle);
    w.step(1);
    w.brace(me, brace);
    let (mut wounds, mut blocked, mut least) = (0, 0, i32::MAX);
    for _ in 0..30 {
        w.step(1);
        least = least.min(w.actor(me).unwrap().body.endurance);
        for e in w.events() {
            match e {
                Event::Struck { target, .. } if target == me => wounds += 1,
                Event::Blocked { who, .. } if who == me => blocked += 1,
                _ => {}
            }
        }
    }
    let a = w.actor(me).unwrap();
    (wounds, blocked, least, a.shield.map(|s| s.condition))
}

#[test]
fn braced_a_blow_from_ahead_is_blocked_for_breath_and_wear() {
    // It stands north of you; you turn north to face it.
    let (wounds, blocked, breath, left) = blows(true, at(0, 1), Dir8::N);
    assert_eq!(wounds, 0, "nothing gets through");
    assert!(blocked >= 1);
    assert!(breath < 1000, "a block costs breath");
    assert!(left.is_none_or(|c| c < 1000), "and wears the shield");
    // Not braced, the same blows land.
    let (wounds, blocked, ..) = blows(false, at(0, 1), Dir8::N);
    assert!(wounds >= 1 && blocked == 0);
    // Braced, but facing away: from behind, nothing's blocked.
    let (wounds, blocked, ..) = blows(true, at(0, 1), Dir8::S);
    assert!(wounds >= 1 && blocked == 0);
}

#[test]
fn a_shield_breaks_when_its_last_block_is_spent() {
    let (mut w, me) = strip();
    take_up_shield(&mut w, me);
    let c = w.creature_mut(0);
    (c.attack.hit, c.attack.wind_up, c.attack.recovery) = (10_000, 2, 1);
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    w.spawn(ActorKind::Creature(0), at(0, 1)).unwrap();
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::N,
            run: false,
        },
    );
    w.step(1);
    w.intent(me, Intent::Idle);
    w.brace(me, true);
    let mut broke = false;
    for _ in 0..200 {
        w.step(1);
        broke |= w
            .events()
            .any(|e| matches!(e, Event::WeaponBroke { item: SHIELD, .. }));
    }
    assert!(broke, "three blocks of 40, and it's gone");
    assert!(w.actor(me).unwrap().shield.is_none());
}

#[test]
fn a_shove_with_a_shield_on_your_arm_is_a_bash() {
    // A shove that never knocks down, with and without the shield's bash.
    let down = |shield: bool| {
        let (mut w, me) = strip();
        w.rules.combat.knock_down = 0;
        if shield {
            take_up_shield(&mut w, me);
        }
        let z = w.spawn(ActorKind::Creature(0), at(0, 3)).unwrap();
        act(&mut w, me, Intent::Shove { dir: Some(Dir8::S) });
        w.actor(z).unwrap().prone_until > 0
    };
    assert!(!down(false));
    assert!(down(true), "the bash knocks it down");
}

#[test]
fn bracing_is_slow_and_never_a_run() {
    let walked = |brace: bool, run: bool| {
        let (mut w, me) = strip();
        take_up_shield(&mut w, me);
        w.brace(me, brace);
        w.intent(me, Intent::Move { dir: Dir8::S, run });
        w.step(30);
        w.actor(me).unwrap().pos.y
    };
    let (free, braced) = (walked(false, false), walked(true, false));
    assert!(braced < free, "{braced} against {free}");
    assert_eq!(walked(true, true), braced, "no running");
}
