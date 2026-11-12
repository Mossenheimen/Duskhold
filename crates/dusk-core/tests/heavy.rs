//! A heavy swing: a longer wind-up and recovery for more breath, more harm, a chance to knock
//! down what it hits, more wear on the weapon, a door beaten harder; and never while held.

use std::sync::Arc;

use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Edge, Event, Intent, Item, ItemDef, ItemKind, Tile, TilePos,
    World,
};

const CLUB: u16 = 0;

/// You on grass with a club in hand (a sure 10-point blow that wears it 5%), and one of the dead
/// east of you that sees all round (so no blow is a sneak attack), can't be killed by a few
/// blows, and never lands one of its own.
fn field() -> (World, ActorId, ActorId) {
    let club = ItemDef {
        id: "club".into(),
        name: "club".into(),
        weight: 1000,
        stack: 1,
        kind: ItemKind::Weapon {
            swing: Swing {
                wind_up: 4,
                recovery: 6,
                endurance: 50,
                hit: 10_000,
                damage: (100, 100),
                noise: 3,
            },
            wear: 50,
            skill: None,
        },
        worth: 0,
    };
    let mut w = World::new(3);
    w.defs = Arc::new(Defs {
        items: vec![club],
        kit: vec![Item::new(CLUB, 1)],
        ..Defs::default()
    });
    {
        let c = w.creature_mut(0);
        c.wander = 0;
        c.attack.hit = 0;
        c.health = 100_000;
        c.cone = dusk_core::senses::Cone { cos: -10_000 };
    }
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    for y in -5..5 {
        for x in -5..5 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(1, 0, 0))
        .unwrap();
    w.intent(me, Intent::Wield { slot: 0, def: CLUB });
    w.step(10);
    assert_eq!(w.actor(me).unwrap().hands.map(|h| h.def), Some(CLUB));
    (w, me, z)
}

const EAST: Option<Dir8> = Some(Dir8::E);

/// The swing's wind-up and whole length, in ticks, and the breath it cost.
fn started(intent: Intent) -> (u32, u32, i32) {
    let (mut w, me, _) = field();
    let before = w.actor(me).unwrap().body.endurance;
    w.intent(me, intent);
    w.step(1);
    let a = w.actor(me).unwrap();
    let act = a.action.expect("it's swinging");
    (act.strike, act.length, before - a.body.endurance)
}

#[test]
fn a_heavy_swing_winds_up_longer_and_costs_more_breath() {
    let h = World::new(0).rules.heavy;
    let (strike, length, breath) = started(Intent::Attack { dir: EAST });
    assert_eq!((strike, length, breath), (4, 10, 50));
    let heavy = started(Intent::HeavyAttack { dir: EAST });
    let (wind_up, recovery) = (4 * h.wind_up / 1000, 6 * h.recovery / 1000);
    assert_eq!(
        heavy,
        (wind_up, wind_up + recovery, 50 * h.endurance as i32 / 1000)
    );
    assert!(heavy.0 > strike && heavy.2 > breath);
}

/// One blow at the dead east of you: the harm it did, whether it knocked it down, how much of
/// the club it wore, and the world after.
fn blow(intent: Intent, knock_down: u32) -> (i32, bool, u16, World, ActorId) {
    let (mut w, me, z) = field();
    w.rules.heavy.knock_down = knock_down;
    let before = w.actor(me).unwrap().hands.unwrap().condition;
    w.intent(me, intent);
    let (mut harm, mut felled) = (None, false);
    for _ in 0..30 {
        w.step(1);
        for e in w.events() {
            match e {
                Event::Struck {
                    who,
                    target,
                    damage,
                    ..
                } if who == me && target == z => harm = Some(damage),
                Event::Felled { who, target } if who == me && target == z => felled = true,
                _ => {}
            }
        }
    }
    let worn = before - w.actor(me).unwrap().hands.unwrap().condition;
    (harm.expect("it landed"), felled, worn, w, z)
}

#[test]
fn a_heavy_blow_does_more_harm_and_wears_the_weapon_more() {
    let h = World::new(0).rules.heavy;
    let (harm, felled, worn, _, _) = blow(Intent::Attack { dir: EAST }, 10_000);
    assert_eq!(harm, 100);
    assert!(!felled, "a swing never fells");
    let (heavy, _, heavy_worn, _, _) = blow(Intent::HeavyAttack { dir: EAST }, 0);
    assert_eq!(heavy, 100 * h.harm as i32 / 1000);
    assert_eq!(heavy_worn, worn * u16::from(h.wear));
}

#[test]
fn a_heavy_blow_may_knock_down_what_it_hits() {
    let (_, felled, _, w, z) = blow(Intent::HeavyAttack { dir: EAST }, 10_000);
    assert!(felled);
    assert!(w.actor(z).unwrap().prone_until > 0, "it's down");
    let (_, felled, _, w, z) = blow(Intent::HeavyAttack { dir: EAST }, 0);
    assert!(!felled);
    assert_eq!(w.actor(z).unwrap().prone_until, 0);
}

#[test]
fn a_heavy_blow_beats_a_door_harder() {
    // Blows on a door until it gives: heavy ones take fewer.
    let blows = |intent: Intent| {
        let (mut w, me, _) = field();
        let at = TilePos::new(0, 0, 0);
        w.grid.set_tile(TilePos::new(0, -1, 0), Tile::GRASS);
        w.grid.set_edge(at, Dir8::N, Edge::door(false));
        for n in 1..=40 {
            w.intent(me, intent);
            w.step(20);
            if w.events().any(|e| matches!(e, Event::DoorBroke { .. })) {
                return n;
            }
        }
        panic!("the door never gave");
    };
    let north = Some(Dir8::N);
    let (light, heavy) = (
        blows(Intent::Attack { dir: north }),
        blows(Intent::HeavyAttack { dir: north }),
    );
    assert!(heavy < light, "{heavy} heavy blows, {light} light");
}

#[test]
fn held_you_cant_swing_heavy() {
    let (mut w, me, z) = field();
    w.creature_mut(0).attack.hit = 10_000;
    w.rules.combat.grab = 10_000;
    // It goes after you, and takes hold.
    w.make_sound(TilePos::new(0, 0, 0), 5, None);
    let mut held = false;
    for _ in 0..200 {
        w.step(1);
        held |= w
            .events()
            .any(|e| e == Event::Grabbed { who: z, target: me });
        if held {
            break;
        }
    }
    assert!(held, "it never took hold");
    w.intent(me, Intent::HeavyAttack { dir: EAST });
    w.step(1);
    let act = w.actor(me).unwrap().action;
    assert!(
        act.is_none_or(|a| a.kind != dusk_core::ActionKind::Heavy),
        "{act:?}"
    );
}
