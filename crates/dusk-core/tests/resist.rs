//! Damage by weapon: what a kind of the dead takes of a blow, by the weapon's skill. Bone turns a
//! blade and a mace shatters it; bare hands strike as blunt; most kinds take a blow whole.

use std::sync::Arc;

use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Event, Intent, ItemDef, ItemKind, Skill, Tile, TilePos, World,
    WoundKind,
};

const SWORD: u16 = 0;
const MACE: u16 = 1;

/// A sword and a mace, each landing every time for 10 points; you hold `wield` (or nothing).
fn defs(wield: Option<u16>) -> Defs {
    let weapon = |id: &str, skill| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 1000,
        stack: 1,
        kind: ItemKind::Weapon {
            swing: Swing {
                wind_up: 3,
                recovery: 3,
                endurance: 10,
                hit: 10_000,
                damage: (100, 100),
                noise: 3,
            },
            wear: 0,
            skill: Some(skill),
        },
        worth: 0,
    };
    Defs {
        items: vec![weapon("sword", Skill::Blade), weapon("mace", Skill::Blunt)],
        wield,
        carry: 30_000,
        ..Defs::default()
    }
}

/// You on grass beside one of the dead, holding `wield`, its kind taking `resist` of a blow by the
/// weapon's skill. It never lands a blow; yours always do.
fn beside(wield: Option<u16>, resist: [u32; 4]) -> (World, ActorId, ActorId) {
    let mut w = World::new(1);
    w.defs = Arc::new(defs(wield));
    for x in 0..4 {
        w.grid.set_tile(TilePos::new(x, 0, 0), Tile::GRASS);
    }
    let c = w.creature_mut(0);
    (c.wander, c.health, c.resist) = (0, 5000, resist);
    c.attack.hit = 0;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    w.rules.combat.attack.hit = 10_000;
    w.rules.combat.attack.damage = (100, 100);
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(1, 0, 0))
        .unwrap();
    (w, me, z)
}

/// Your first blow on it: what it did, what your swing does whole, and whether its kind turned the
/// blow (false) or told on it (true).
fn first_blow(w: &mut World, me: ActorId, z: ActorId) -> (i32, i32, Option<bool>) {
    let whole = w.swing(me).damage.0;
    w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
    let mut resisted = None;
    for _ in 0..50 {
        w.step(1);
        for e in w.events() {
            match e {
                Event::Resisted { target, weak, .. } if target == z => resisted = Some(weak),
                Event::Struck { who, damage, .. } if who == me => return (damage, whole, resisted),
                _ => {}
            }
        }
    }
    panic!("no blow landed");
}

/// Bone: 40% of a blade, 150% of anything blunt, 70% of an axe, 30% of a spear.
const BONE: [u32; 4] = [400, 1500, 700, 300];

#[test]
fn bone_turns_a_blade_and_a_mace_shatters_it() {
    let (mut w, me, z) = beside(Some(SWORD), BONE);
    let (damage, whole, resisted) = first_blow(&mut w, me, z);
    assert_eq!(damage, whole * 400 / 1000, "a blade glances off");
    assert_eq!(resisted, Some(false));

    let (mut w, me, z) = beside(Some(MACE), BONE);
    let (damage, whole, resisted) = first_blow(&mut w, me, z);
    assert_eq!(damage, whole * 1500 / 1000, "a mace shatters it");
    assert_eq!(resisted, Some(true));

    // Bare hands strike as blunt.
    let (mut w, me, z) = beside(None, BONE);
    let (damage, whole, resisted) = first_blow(&mut w, me, z);
    assert_eq!((damage, resisted), (whole * 1500 / 1000, Some(true)));
}

#[test]
fn a_kind_that_minds_no_weapon_takes_a_blow_whole() {
    let (mut w, me, z) = beside(Some(SWORD), [1000; 4]);
    let (damage, whole, resisted) = first_blow(&mut w, me, z);
    assert_eq!((damage, resisted), (whole, None));
    assert_eq!(w.resistance(z, Some(Skill::Spear)), 1000);
    assert_eq!(
        w.resistance(me, Some(Skill::Blade)),
        1000,
        "nor do the living"
    );
}

#[test]
fn held_a_biting_kind_bites_but_bone_has_no_bite() {
    // One of the dead facing south at you, landing every blow and taking hold with the first;
    // held, the mouth gets close every time, for a kind that bites at all. The wounds it leaves.
    let held_wounds = |bites: u32| {
        let mut w = World::new(1);
        for y in 0..4 {
            w.grid.set_tile(TilePos::new(0, y, 0), Tile::GRASS);
        }
        let c = w.creature_mut(0);
        c.wander = 0;
        c.attack.hit = 10_000;
        c.wounds = [10_000 - bites, 0, bites];
        (w.rules.combat.grab, w.rules.combat.held_bite) = (10_000, 10_000);
        w.spawn(ActorKind::Creature(0), TilePos::new(0, 0, 0))
            .unwrap();
        let me = w
            .spawn(ActorKind::Character, TilePos::new(0, 1, 0))
            .unwrap();
        let mut kinds = Vec::new();
        for _ in 0..1000 {
            w.step(1);
            for e in w.events() {
                if let Event::Struck {
                    target,
                    wound: Some((kind, _)),
                    ..
                } = e
                    && target == me
                {
                    kinds.push(kind);
                }
            }
            if kinds.len() >= 3 || w.actor(me).unwrap().dead {
                break;
            }
        }
        kinds
    };
    let flesh = held_wounds(500);
    assert!(flesh.len() >= 2, "{flesh:?}");
    assert!(
        flesh[1..].iter().all(|&k| k == WoundKind::Bite),
        "{flesh:?}"
    );
    let bone = held_wounds(0);
    assert!(bone.len() >= 2, "{bone:?}");
    assert!(!bone.contains(&WoundKind::Bite), "{bone:?}");
}
