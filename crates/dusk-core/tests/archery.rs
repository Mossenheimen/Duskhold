//! Bows: an arrow flies the line to the first one on it, a wall stops it, bone turns it, and it
//! lies where it fell to be picked up; a bow wants arrows, the other hand free, and its range.

use std::sync::Arc;

use dusk_core::archery::{NoShot, Ranged};
use dusk_core::senses::Cone;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Edge, Event, Intent, Item, ItemDef, ItemKind, Skill, Tile,
    TilePos, World,
};

const BOW: u16 = 0;
const ARROW: u16 = 1;
const TORCH: u16 = 2;

/// A bow that always lands for 10 points up close, out to 10 tiles, its arrows never breaking;
/// arrows; a torch. You hold the bow, with `arrows` arrows and the torch in your pack.
fn defs(arrows: u16) -> Defs {
    let item = |id: &str, stack, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 100,
        stack,
        kind,
        worth: 0,
    };
    let bow = Ranged {
        swing: Swing {
            wind_up: 3,
            recovery: 2,
            endurance: 10,
            hit: 10_000,
            damage: (100, 100),
            noise: 2,
        },
        range: 10,
        ammo: ARROW,
        breaks: 0,
    };
    let mut kit = vec![Item::new(TORCH, 1)];
    if arrows > 0 {
        kit.push(Item::new(ARROW, arrows));
    }
    Defs {
        items: vec![
            item("bow", 1, ItemKind::Bow(bow)),
            item("arrow", 30, ItemKind::Other),
            item(
                "torch",
                1,
                ItemKind::Light {
                    radius: 5,
                    wane: 17,
                },
            ),
        ],
        wield: Some(BOW),
        kit,
        carry: 30_000,
        light_time: 3,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A field running east by day, you at (0, 0); the dead see all round (so no shot is a sneak
/// attack), stand still and never land a blow.
fn range(arrows: u16) -> (World, ActorId) {
    let mut w = World::new(2);
    w.defs = Arc::new(defs(arrows));
    for x in -2..16 {
        for y in -2..3 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    let c = w.creature_mut(0);
    (c.wander, c.health, c.attack.hit) = (0, 5000, 0);
    c.cone = Cone { cos: -10_000 };
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
    (w, me)
}

/// Shoots at `target` and runs until it's done: the events.
fn shoot(w: &mut World, me: ActorId, target: TilePos) -> Vec<Event> {
    w.intent(me, Intent::Shoot { at: target });
    let mut seen = Vec::new();
    for _ in 0..30 {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

fn arrows(w: &World, me: ActorId) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .filter(|i| i.def == ARROW)
        .map(|i| i.count)
        .sum()
}

fn struck(seen: &[Event], me: ActorId) -> Option<(ActorId, i32)> {
    seen.iter().find_map(|e| match *e {
        Event::Struck {
            who,
            target,
            damage,
            ..
        } if who == me => Some((target, damage)),
        _ => None,
    })
}

#[test]
fn an_arrow_flies_to_the_first_on_the_line_and_lies_where_it_fell() {
    let (mut w, me) = range(3);
    let near = w.spawn(ActorKind::Creature(0), at(5, 0)).unwrap();
    let far = w.spawn(ActorKind::Creature(0), at(8, 0)).unwrap();
    let seen = shoot(&mut w, me, at(8, 0));
    let (hit, damage) = struck(&seen, me).expect("a hit");
    assert_eq!(hit, near, "the nearer one was in the way");
    assert_ne!(hit, far);
    assert_eq!(damage, 100);
    assert!(seen.contains(&Event::Shot {
        who: me,
        from: at(0, 0),
        to: at(5, 0)
    }));
    assert_eq!(arrows(&w, me), 2, "one arrow loosed");
    assert_eq!(
        w.store(at(5, 0)).first().map(|i| i.def),
        Some(ARROW),
        "it lies there"
    );
    assert!(
        w.actor(me).unwrap().skills.xp[Skill::Archery as usize] > 0,
        "Archery learnt"
    );
}

#[test]
fn a_wall_stops_an_arrow_and_bone_turns_it() {
    let (mut w, me) = range(3);
    w.spawn(ActorKind::Creature(0), at(6, 0)).unwrap();
    w.grid.set_edge(at(4, 0), Dir8::W, Edge::Wall);
    let seen = shoot(&mut w, me, at(6, 0));
    assert_eq!(struck(&seen, me), None);
    assert!(seen.contains(&Event::Shot {
        who: me,
        from: at(0, 0),
        to: at(3, 0)
    }));
    assert_eq!(w.store(at(3, 0)).len(), 1, "it fell at the wall");

    let (mut w, me) = range(3);
    w.creature_mut(0).resist = [400, 1500, 700, 300];
    w.spawn(ActorKind::Creature(0), at(4, 0)).unwrap();
    let seen = shoot(&mut w, me, at(4, 0));
    assert_eq!(
        struck(&seen, me).map(|(_, d)| d),
        Some(30),
        "taken as a spear's thrust"
    );
}

#[test]
fn a_bow_wants_arrows_the_other_hand_free_and_its_range() {
    let (w, me) = range(0);
    assert_eq!(w.can_shoot(me, at(4, 0)), Err(NoShot::NoArrows));
    let (mut w, me) = range(3);
    assert_eq!(w.can_shoot(me, at(4, 0)), Ok(()));
    assert_eq!(w.can_shoot(me, at(11, 0)), Err(NoShot::TooFar));
    assert_eq!(
        w.can_shoot(me, at(0, 0)),
        Err(NoShot::TooFar),
        "not at yourself"
    );
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == TORCH)
        .unwrap() as u16;
    w.intent(me, Intent::Light { slot, def: TORCH });
    w.step(10);
    assert_eq!(
        w.can_shoot(me, at(4, 0)),
        Err(NoShot::HandsFull),
        "a torch in the other hand"
    );
    w.intent(me, Intent::Unwield);
    w.step(20);
    assert_eq!(w.can_shoot(me, at(4, 0)), Err(NoShot::NoBow));
}
