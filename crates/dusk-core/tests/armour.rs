//! Armour: worn a piece to a place, put on and taken off in time; a blow on a covered place may
//! glance off (no wound, a bruise) and wears the piece until it falls apart; worn, it weighs
//! less, costs breath, clinks, and a closed helm narrows your sight; points go through it and a
//! blade skids off; a blow on one that's down finds the gaps; the dead come in their armour and
//! leave it where they fall; a revenant rises in yours; the abbey mends it; a class comes in it.

use std::sync::Arc;

use dusk_core::armour::Armour;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Class, CreatureDef, Defs, Dir8, Event, Holdout, Intent, Item, ItemDef,
    ItemKind, NoDeal, Part, Region, Skill, Tile, TilePos, World, WoundKind,
};

const MAIL: u16 = 0;
const JERKIN: u16 = 1;
const CAP: u16 = 2;
const HELM: u16 = 3;
const SWORD: u16 = 4;
const SPEAR: u16 = 5;
const COINS: u16 = 6;
const WHOLE: u16 = 7;
const FRAIL: u16 = 8;
const SHAMBLER: ActorKind = ActorKind::Creature(0);
const REVENANT: u16 = 1;
const MINUTE: u32 = 25;

/// Torso and arms, and the rest: each turns every blow unless said otherwise.
const TORSO_ARMS: u8 = 2 | 4;
const ALL: u8 = 1 | 2 | 4 | 8;

fn armour(covers: u8, turn: u32, wear: u16) -> Armour {
    Armour {
        covers,
        turn: [turn; 3],
        wear,
        noise: 0,
        endurance: 0,
        sight: 1000,
        time: 5,
    }
}

/// Mail (it clinks, and costs half as much breath again), a padded jerkin that turns nothing
/// (worn to half in the kit), a cap and a closed helm, a sword in hand and a spear in the pack,
/// 40 coins, a suit that covers everything, and a frail one that falls apart in three blows.
fn defs() -> Defs {
    let item = |id: &str, weight: u32, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight,
        stack: 1,
        kind,
        worth: 4000,
    };
    let blow = |skill| ItemKind::Weapon {
        swing: Swing {
            wind_up: 4,
            recovery: 4,
            endurance: 0,
            hit: 10_000,
            damage: (100, 100),
            noise: 2,
        },
        wear: 0,
        skill: Some(skill),
    };
    let mut revenant = CreatureDef::shambler();
    revenant.id = "revenant".into();
    revenant.stats.wander = 0;
    let abbey = Holdout {
        id: "abbey".into(),
        name: "the abbey".into(),
        sells: Vec::new(),
        buys: 500,
        repair: 1000,
        wants: Vec::new(),
        supply: 0,
        time: 10,
        bounty: dusk_core::Bounty::default(),
        identify: 0,
        uncurse: 0,
    };
    let mut defs = Defs {
        items: vec![
            item(
                "mail",
                8000,
                ItemKind::Armour(Armour {
                    noise: 2,
                    endurance: 500,
                    ..armour(TORSO_ARMS, 10_000, 10)
                }),
            ),
            item("jerkin", 2000, ItemKind::Armour(armour(TORSO_ARMS, 0, 0))),
            item("cap", 400, ItemKind::Armour(armour(1, 10_000, 10))),
            item(
                "helm",
                2000,
                ItemKind::Armour(Armour {
                    sight: 850,
                    ..armour(1, 10_000, 10)
                }),
            ),
            item("sword", 1000, blow(Skill::Blade)),
            item("spear", 1000, blow(Skill::Spear)),
            ItemDef {
                stack: 100,
                worth: 1000,
                ..item("coins", 10, ItemKind::Other)
            },
            item("whole", 1000, ItemKind::Armour(armour(ALL, 10_000, 10))),
            item("frail", 1000, ItemKind::Armour(armour(ALL, 10_000, 400))),
        ],
        wield: Some(SWORD),
        kit: vec![
            Item::new(MAIL, 1),
            Item {
                condition: 500,
                ..Item::new(JERKIN, 1)
            },
            Item::new(CAP, 1),
            Item::new(HELM, 1),
            Item::new(SPEAR, 1),
            Item::new(COINS, 40),
            Item::new(WHOLE, 1),
            Item::new(FRAIL, 1),
        ],
        carry: 100_000,
        wield_time: 3,
        holdouts: vec![abbey],
        counters: vec![None, Some(0)],
        coin: Some(COINS),
        revenant: Some((REVENANT, 0)),
        ..Defs::default()
    };
    defs.creatures.push(revenant);
    defs
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A field of grass at 20:00, the needs stilled, you at (0, 0) with health to spare.
fn field() -> (World, ActorId) {
    let mut w = World::new(5);
    w.defs = Arc::new(defs());
    w.rules.clock.start = 20 * 60;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    w.rules.combat.character_health = 100_000;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    for y in -20..20 {
        for x in -20..20 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    w.creature_mut(0).wander = 0;
    w.set_region(Region {
        start: Some(at(0, 0)),
        ..Region::default()
    });
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
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

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..200 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

fn slot(w: &World, me: ActorId, def: u16) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == def)
        .unwrap() as u16
}

fn put_on(w: &mut World, me: ActorId, def: u16) -> Vec<Event> {
    let slot = slot(w, me, def);
    act(w, me, Intent::Wear { slot, def })
}

fn worn(w: &World, id: ActorId) -> Vec<u16> {
    w.actor(id).unwrap().worn.iter().map(|i| i.def).collect()
}

fn in_pack(w: &World, me: ActorId, def: u16) -> bool {
    w.actor(me).unwrap().items.iter().any(|i| i.def == def)
}

#[test]
fn armour_goes_on_in_time_and_what_covered_the_same_places_comes_off() {
    let (mut w, me) = field();
    let s = slot(&w, me, JERKIN);
    w.intent(
        me,
        Intent::Wear {
            slot: s,
            def: JERKIN,
        },
    );
    run(&mut w, 3);
    assert!(worn(&w, me).is_empty(), "it takes its time");
    let seen = run(&mut w, 5);
    assert_eq!(worn(&w, me), vec![JERKIN]);
    assert!(!in_pack(&w, me, JERKIN));
    assert!(seen.contains(&Event::Wore {
        who: me,
        item: JERKIN
    }));
    put_on(&mut w, me, CAP);
    assert_eq!(
        worn(&w, me),
        vec![JERKIN, CAP],
        "the head is somewhere else"
    );
    // Mail over the same places: the jerkin comes off, into the pack.
    put_on(&mut w, me, MAIL);
    assert_eq!(worn(&w, me), vec![CAP, MAIL]);
    assert!(in_pack(&w, me, JERKIN));
    // Taken off, it's in the pack; a stale piece number does nothing.
    let seen = act(&mut w, me, Intent::TakeOff { piece: 0, def: CAP });
    assert!(seen.contains(&Event::TookOff { who: me, item: CAP }));
    assert_eq!(worn(&w, me), vec![MAIL]);
    assert!(in_pack(&w, me, CAP));
    act(&mut w, me, Intent::TakeOff { piece: 0, def: CAP });
    assert_eq!(worn(&w, me), vec![MAIL]);
}

/// One of the dead north of you, landing every blow it swings, for `ticks`: the blows that
/// struck you (with their harm and any wound) and the blows that glanced off.
fn beaten(w: &mut World, me: ActorId, ticks: u32) -> (Vec<(i32, Option<Part>)>, Vec<Part>) {
    w.creature_mut(0).attack.hit = 10_000;
    w.spawn(SHAMBLER, at(0, -1)).unwrap();
    let (mut struck, mut glanced) = (Vec::new(), Vec::new());
    for e in run(w, ticks) {
        match e {
            Event::Struck {
                target,
                damage,
                wound,
                ..
            } if target == me => struck.push((damage, wound.map(|(_, p)| p))),
            Event::Glanced { who, part, .. } if who == me => glanced.push(part),
            _ => {}
        }
    }
    (struck, glanced)
}

#[test]
fn a_blow_on_a_covered_place_glances_off_leaving_a_bruise_and_no_wound() {
    let (mut w, me) = field();
    put_on(&mut w, me, WHOLE);
    let (struck, glanced) = beaten(&mut w, me, 120);
    assert!(struck.len() >= 3, "it struck: {struck:?}");
    assert_eq!(glanced.len(), struck.len(), "every blow glanced off");
    let bruises: Vec<i32> = w.rules.combat.wounds.iter().map(|r| r.damage / 2).collect();
    for (damage, wound) in &struck {
        assert!(wound.is_none(), "no wound");
        assert!(bruises.contains(damage), "half a wound's harm: {damage}");
    }
    let a = w.actor(me).unwrap();
    assert!(a.body.wounds.is_empty() && a.body.health < 100_000);
    assert!(a.worn[0].condition < 1000, "and it wore the suit");
}

#[test]
fn only_what_a_piece_covers_is_turned() {
    let (mut w, me) = field();
    put_on(&mut w, me, CAP);
    let (struck, glanced) = beaten(&mut w, me, 400);
    assert!(glanced.iter().all(|&p| p == Part::Head), "{glanced:?}");
    let wounded: Vec<Part> = struck.iter().filter_map(|&(_, p)| p).collect();
    assert!(!wounded.is_empty(), "the rest of you takes the blows");
    assert!(wounded.iter().all(|&p| p != Part::Head), "{wounded:?}");
    assert_eq!(glanced.len() + wounded.len(), struck.len());
}

#[test]
fn a_piece_worn_through_falls_apart() {
    let (mut w, me) = field();
    put_on(&mut w, me, FRAIL);
    w.creature_mut(0).attack.hit = 10_000;
    w.spawn(SHAMBLER, at(0, -1)).unwrap();
    let seen = run(&mut w, 200);
    let glanced = seen
        .iter()
        .filter(|e| matches!(e, Event::Glanced { .. }))
        .count();
    assert_eq!(glanced, 3, "40 of its 100 a blow: three blows");
    assert!(seen.contains(&Event::WeaponBroke {
        who: me,
        item: FRAIL
    }));
    assert!(worn(&w, me).is_empty());
    assert!(
        !w.actor(me).unwrap().body.wounds.is_empty(),
        "and then they wound"
    );
}

/// Endurance spent running east for a second and a half.
fn run_east(w: &mut World, me: ActorId) -> i32 {
    let before = w.actor(me).unwrap().body.endurance;
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: true,
        },
    );
    run(w, 15);
    w.intent(me, Intent::Idle);
    before - w.actor(me).unwrap().body.endurance
}

#[test]
fn worn_it_weighs_half_costs_breath_clinks_and_a_closed_helm_narrows_your_sight() {
    let (mut w, me) = field();
    let (load, sight) = (w.load(me), w.sight_range(me));
    let bare = run_east(&mut w, me);
    // A step's footsteps, bare.
    let step = |w: &mut World| {
        w.intent(
            me,
            Intent::Move {
                dir: Dir8::W,
                run: false,
            },
        );
        let seen = run(w, 12);
        w.intent(me, Intent::Idle);
        run(w, 2);
        seen.iter()
            .find_map(|e| match *e {
                Event::Sound {
                    loudness,
                    by: Some(by),
                    ..
                } if by == me => Some(loudness),
                _ => None,
            })
            .expect("a footstep")
    };
    let quiet = step(&mut w);
    put_on(&mut w, me, MAIL);
    assert_eq!(w.load(me), load - 4000, "half its weight, worn");
    run(&mut w, 300); // get the breath back
    let in_mail = run_east(&mut w, me);
    assert!(
        in_mail * 10 >= bare * 14,
        "half as much breath again: {in_mail} against {bare}"
    );
    assert_eq!(step(&mut w), quiet + 2, "mail clinks");
    put_on(&mut w, me, HELM);
    assert_eq!(w.sight_range(me), sight * 85 / 100);
}

/// You beside one of the dead (south of you) in the whole suit, it seeing you and landing
/// nothing; `pierce`: what of the suit's chance holds against a blade, blunt, an axe and a point.
fn armoured_foe(pierce: [u32; 4]) -> (World, ActorId, ActorId) {
    let (mut w, me) = field();
    w.rules.armour.pierce = pierce;
    let c = w.creature_mut(0);
    c.attack.hit = 0;
    c.health = 100_000;
    c.cone = dusk_core::senses::Cone { cos: -10_000 };
    let foe = w.spawn(SHAMBLER, at(0, 1)).unwrap();
    assert!(w.don(foe, Item::new(WHOLE, 1)));
    run(&mut w, 20); // it sees you
    (w, me, foe)
}

/// Swings south `n` times: the harm each blow did, and how many glanced off.
fn swing_at(w: &mut World, me: ActorId, foe: ActorId, n: usize) -> (Vec<i32>, usize) {
    let (mut harm, mut glanced) = (Vec::new(), 0);
    for _ in 0..n {
        for e in act(w, me, Intent::Attack { dir: Some(Dir8::S) }) {
            match e {
                Event::Struck { target, damage, .. } if target == foe => harm.push(damage),
                Event::Glanced { who, .. } if who == foe => glanced += 1,
                _ => {}
            }
        }
    }
    (harm, glanced)
}

#[test]
fn points_go_through_mail_and_blades_skid_off_it() {
    // A blade: the suit's whole chance holds.
    let (mut w, me, foe) = armoured_foe([1000, 1000, 1000, 0]);
    let (harm, glanced) = swing_at(&mut w, me, foe, 6);
    assert_eq!(harm.len(), 6);
    assert_eq!(glanced, 6, "every blade's blow glanced");
    assert!(harm.iter().all(|&d| d == 50), "a bruise: {harm:?}");
    // A point: none of it holds.
    let (mut w, me, foe) = armoured_foe([1000, 1000, 1000, 0]);
    let s = slot(&w, me, SPEAR);
    act(
        &mut w,
        me,
        Intent::Wield {
            slot: s,
            def: SPEAR,
        },
    );
    let (harm, glanced) = swing_at(&mut w, me, foe, 6);
    assert_eq!(glanced, 0, "a spear goes through");
    assert!(harm.iter().all(|&d| d == 100), "{harm:?}");
}

#[test]
fn a_blow_on_one_thats_down_finds_the_gaps() {
    let (mut w, me, foe) = armoured_foe([1000; 4]);
    w.rules.combat.knock_down = 10_000;
    act(&mut w, me, Intent::Shove { dir: Some(Dir8::S) });
    // Pushed back a tile and down: step after it.
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::S,
            run: false,
        },
    );
    for _ in 0..20 {
        w.step(1);
        if w.actor(me).unwrap().pos == at(0, 1) {
            break;
        }
    }
    w.intent(me, Intent::Idle);
    assert!(w.actor(foe).unwrap().prone_until > 0, "it's down");
    let (harm, glanced) = swing_at(&mut w, me, foe, 1);
    assert_eq!(glanced, 0);
    assert_eq!(
        harm,
        vec![200],
        "a blow on one that's down: double, through the gaps"
    );
}

#[test]
fn a_kind_that_wears_armour_comes_in_it_and_leaves_it_where_it_falls() {
    let (mut w, _) = field();
    w.creature_mut(0).wears = Some(MAIL);
    let foe = w.spawn(SHAMBLER, at(5, 5)).unwrap();
    let a = w.actor(foe).unwrap();
    assert_eq!(a.worn.len(), 1);
    assert!(a.worn[0].def == MAIL && a.worn[0].found, "its mail, a find");
    while !w.actor(foe).unwrap().dead {
        w.wound(foe, WoundKind::Bite, Part::Torso);
    }
    assert!(
        w.store(at(5, 5)).iter().any(|i| i.def == MAIL),
        "left where it fell"
    );
}

#[test]
fn a_revenant_rises_in_the_armour_its_body_died_in() {
    let (mut w, me) = field();
    put_on(&mut w, me, MAIL);
    while !w.actor(me).unwrap().dead {
        w.wound(me, WoundKind::Bite, Part::Torso);
    }
    let seen = run(&mut w, 4 * 60 * MINUTE + MINUTE);
    let Some(&Event::Risen { revenant, .. }) =
        seen.iter().find(|e| matches!(e, Event::Risen { .. }))
    else {
        panic!("it rose");
    };
    assert_eq!(worn(&w, revenant), vec![MAIL], "in your mail");
    assert!(worn(&w, me).is_empty());
}

#[test]
fn the_abbey_mends_the_armour_you_wear() {
    let (mut w, me) = field();
    let counter = at(0, -1);
    w.grid.set_tile(
        counter,
        Tile {
            fixture: 1,
            solid: true,
            ..Tile::GRASS
        },
    );
    assert_eq!(w.repair_cost(me, counter), Err(NoDeal::NothingToMend));
    // The jerkin, worn to half: half its worth of 4.
    put_on(&mut w, me, JERKIN);
    assert_eq!(w.repair_cost(me, counter), Ok(2));
    act(&mut w, me, Intent::Mend { at: counter });
    assert_eq!(w.actor(me).unwrap().worn[0].condition, 1000);
    assert_eq!(w.coins(me), 38);
    assert_eq!(w.repair_cost(me, counter), Err(NoDeal::NothingToMend));
}

#[test]
fn a_class_comes_in_its_armour() {
    let mut defs = defs();
    defs.classes = vec![
        Class {
            id: "fighter".into(),
            learn: 1000,
            endurance: 1000,
            grabbed: 1000,
            wear: vec![CAP, JERKIN],
            ..Class::default()
        },
        Class {
            id: "other".into(),
            learn: 1000,
            endurance: 1000,
            grabbed: 1000,
            ..Class::default()
        },
    ];
    let mut w = World::new(5);
    w.defs = Arc::new(defs);
    w.grid.set_tile(at(0, 0), Tile::GRASS);
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
    assert_eq!(worn(&w, me), vec![CAP, JERKIN]);
    assert!(w.choose_class(me, 1));
    assert!(worn(&w, me).is_empty(), "the other class's kit replaces it");
}
