//! The ghoul and ghoul fever: a creature that follows blood, wounds that carry a fever, three
//! days of it, and the cure.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, CreatureDef, Defs, Event, Intent, Item, ItemDef, ItemKind, Moodle, Part,
    Tile, TilePos, World, WoundKind,
};

const SHAMBLER: ActorKind = ActorKind::Creature(0);
const GHOUL: ActorKind = ActorKind::Creature(1);
const TINCTURE: u16 = 0;
/// Ticks in a game minute, and in a game day.
const MINUTE: u32 = 25;
const DAY: u32 = MINUTE * 1440;

/// The test shambler, and a ghoul: it smells blood 12 tiles off for each wound bleeding, and
/// every wound it leaves brings the fever. One tincture in your kit.
fn defs() -> Defs {
    let mut ghoul = CreatureDef::shambler();
    ghoul.id = "ghoul".into();
    ghoul.name = "ghoul".into();
    ghoul.stats.blood = 12;
    ghoul.stats.fever = [10_000; 3];
    let mut defs = Defs {
        items: vec![ItemDef {
            id: "tincture".into(),
            name: "tincture".into(),
            weight: 100,
            stack: 5,
            kind: ItemKind::Cure { time: 30 },
            worth: 0,
        }],
        kit: vec![Item::new(TINCTURE, 1)],
        ..Defs::default()
    };
    defs.creatures.push(ghoul);
    defs
}

/// A field of grass, the needs stilled (so only the fever harms), and you at its middle.
fn field() -> (World, ActorId) {
    let mut w = World::new(5);
    w.defs = Arc::new(defs());
    for n in 0..2 {
        w.creature_mut(n).wander = 0;
    }
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -20..20 {
        for x in -20..20 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    (w, me)
}

/// Steps the world, keeping its events.
fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut seen = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

#[test]
fn a_ghouls_wound_brings_the_fever_and_a_shamblers_never_does() {
    // Each stands beside you, facing you, and swings until it has wounded you three times.
    let wounded_by = |kind: ActorKind| {
        let (mut w, me) = field();
        w.spawn(kind, TilePos::new(0, -1, 0)).unwrap();
        let mut wounds = 0;
        for _ in 0..3000 {
            w.step(1);
            for e in w.events() {
                if matches!(e, Event::Struck { target, wound: Some(_), .. } if target == me) {
                    wounds += 1;
                }
            }
            if wounds >= 3 {
                break;
            }
        }
        assert!(wounds >= 3, "it never got its blows in");
        (w.actor(me).unwrap().body.fever, w.fever_stage(me))
    };
    let (fever, stage) = wounded_by(GHOUL);
    assert!(fever.is_some());
    assert_eq!(stage, 1, "a fever, the first day");
    assert_eq!(
        wounded_by(SHAMBLER),
        (None, 0),
        "a shambler's bite is only a wound"
    );
}

#[test]
fn the_fever_runs_three_days_and_kills_at_the_third_days_end() {
    let (mut w, me) = field();
    w.catch_fever(me);
    w.catch_fever(me); // caught once: catching it again changes nothing
    assert_eq!(w.fever_stage(me), 1);
    assert!(w.moodles(me).contains(&(Moodle::Fever, 1)));
    let events = run(&mut w, DAY);
    assert_eq!(w.fever_stage(me), 2, "weakness, the second day");
    assert!(events.contains(&Event::FeverWorse { who: me, stage: 2 }));
    let events = run(&mut w, DAY);
    assert_eq!(w.fever_stage(me), 3, "dying, the third");
    assert!(events.contains(&Event::FeverWorse { who: me, stage: 3 }));
    assert_eq!(w.fever_left(me), Some(1440));

    // Dying, health drains each hour: but it's the day's end that kills, not the drain.
    let before = w.actor(me).unwrap().body.health;
    let events = run(&mut w, DAY - MINUTE);
    let a = w.actor(me).unwrap();
    assert!(!a.dead && a.body.health < before - 800, "{}", a.body.health);
    assert!(!events.iter().any(|e| matches!(e, Event::Died { .. })));
    let events = run(&mut w, MINUTE);
    assert!(w.actor(me).unwrap().dead);
    let took = events
        .iter()
        .position(|e| *e == Event::FeverTook { who: me });
    let died = events.iter().position(|e| *e == Event::Died { who: me });
    assert!(took.is_some() && took < died, "{events:?}");
}

#[test]
fn feverish_you_thirst_faster_and_dont_heal() {
    let hour_on = |fevered: bool| {
        let (mut w, me) = field();
        w.rules.needs.thirst.rise = 100;
        if fevered {
            w.catch_fever(me);
        }
        // A wound that doesn't bleed, so the healing has something to do.
        w.wound(me, WoundKind::Scratch, Part::LeftArm);
        let hurt = w.actor(me).unwrap().body.health;
        run(&mut w, MINUTE * 120);
        let b = &w.actor(me).unwrap().body;
        (b.thirst, b.health - hurt)
    };
    let (thirst, healed) = hour_on(false);
    let (fever_thirst, fever_healed) = hour_on(true);
    assert_eq!(fever_thirst, thirst * 3 / 2, "half as fast again");
    assert!(healed > 0);
    assert_eq!(fever_healed, 0);
}

#[test]
fn weak_with_it_your_breath_comes_back_slower() {
    // Out of breath at the start of the second day, then a few seconds standing still.
    let breath = |fevered: bool| {
        let (mut w, me) = field();
        if fevered {
            w.catch_fever(me);
        }
        run(&mut w, DAY);
        let spent = run_and_breathe(&mut w, me);
        (w.fever_stage(me), spent)
    };
    let (_, well) = breath(false);
    let (stage, weak) = breath(true);
    assert_eq!(stage, 2);
    assert_eq!(weak * 2, well, "half the breath back");
}

/// Empties the character's endurance by running, then gives it 20 ticks standing still: what
/// came back.
fn run_and_breathe(w: &mut World, me: ActorId) -> i32 {
    for _ in 0..2000 {
        if w.actor(me).unwrap().body.exhausted {
            break;
        }
        let dir = if w.actor(me).unwrap().pos.x > 10 {
            dusk_core::Dir8::W
        } else {
            dusk_core::Dir8::E
        };
        w.intent(me, Intent::Move { dir, run: true });
        w.step(1);
    }
    w.intent(me, Intent::Idle);
    // Let the last stride finish.
    while w.actor(me).unwrap().step.is_some() {
        w.step(1);
    }
    let from = w.actor(me).unwrap().body.endurance;
    w.step(20);
    w.actor(me).unwrap().body.endurance - from
}

#[test]
fn the_cure_works_at_any_stage_and_only_with_a_fever() {
    let (mut w, me) = field();
    // No fever: the tincture won't be taken.
    w.intent(
        me,
        Intent::Use {
            slot: 0,
            def: TINCTURE,
        },
    );
    run(&mut w, 60);
    assert_eq!(w.actor(me).unwrap().items, vec![Item::new(TINCTURE, 1)]);

    w.catch_fever(me);
    run(&mut w, 2 * DAY + MINUTE * 60);
    assert_eq!(w.fever_stage(me), 3, "dying");
    w.intent(
        me,
        Intent::Use {
            slot: 0,
            def: TINCTURE,
        },
    );
    let events = run(&mut w, 60);
    assert!(events.contains(&Event::Cured { who: me }));
    let a = w.actor(me).unwrap();
    assert!(a.items.is_empty(), "used up");
    assert_eq!((a.body.fever, w.fever_stage(me)), (None, 0));
    // The day it would have killed comes and goes.
    run(&mut w, DAY);
    assert!(!w.actor(me).unwrap().dead);
}

/// A wall between two rows, open at its east end: you on the north row's west end, the creature
/// just south of you, through the wall.
fn wall_between(kind: ActorKind) -> (World, ActorId, ActorId) {
    let (mut w, _) = World::from_text(
        6,
        "\
+-+-+-+-+-+-+
|. . . . . .|
+-+-+-+-+-+ +
|. . . . . .|
+-+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs());
    for n in 0..2 {
        w.creature_mut(n).wander = 0;
    }
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    let z = w.spawn(kind, TilePos::new(0, 1, 0)).unwrap();
    (w, me, z)
}

#[test]
fn a_ghoul_smells_blood_through_a_wall_and_comes_round() {
    // Not bleeding, it never knows you're there.
    let (mut w, _, z) = wall_between(GHOUL);
    let events = run(&mut w, 100);
    assert!(!events.iter().any(|e| matches!(e, Event::Smelled { .. })));
    assert_eq!(w.actor(z).unwrap().pos, TilePos::new(0, 1, 0));

    // Bleeding, it smells you, and finds the way round to you.
    let (mut w, me, z) = wall_between(GHOUL);
    w.wound(me, WoundKind::Laceration, Part::LeftArm);
    assert!(w.actor(me).unwrap().body.bleeding());
    let events = run(&mut w, 400);
    assert!(events.contains(&Event::Smelled { who: z, target: me }));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Reached { who, .. } if *who == z)),
        "it came round the wall"
    );

    // A shambler, with no nose for it, stays where it is.
    let (mut w, me, z) = wall_between(SHAMBLER);
    w.wound(me, WoundKind::Laceration, Part::LeftArm);
    let events = run(&mut w, 100);
    assert!(!events.iter().any(|e| matches!(e, Event::Smelled { .. })));
    assert_eq!(w.actor(z).unwrap().pos, TilePos::new(0, 1, 0));
}

#[test]
fn the_more_wounds_bleed_the_farther_off_its_smelled() {
    // 19 tiles off: out of reach of one wound's scent (12 tiles), inside two's (24).
    for (wounds, smelled) in [(0, false), (1, false), (2, true), (3, true)] {
        let (mut w, me) = field();
        let z = w.spawn(GHOUL, TilePos::new(0, 19, 0)).unwrap();
        for _ in 0..wounds {
            w.wound(me, WoundKind::Laceration, Part::Torso);
        }
        assert_eq!(w.smells(z, 12).is_some(), smelled, "{wounds} bleeding");
    }
}
