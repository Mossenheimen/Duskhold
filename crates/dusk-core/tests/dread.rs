//! Pain and dread: pain from wounds (less dressed, less as they heal), what it costs, and no sleep
//! in agony; dread rising in the dark and near the dead, eased by light, sleep and hallowed
//! ground, fed by hunger; shaky aim, and frightened, no step nearer the dead.

use std::sync::Arc;

use dusk_core::{
    ActorId, ActorKind, Area, Defs, Dir8, Event, Intent, Item, ItemDef, ItemKind, Moodle, NoSleep,
    Part, Skill, Tile, TilePos, World, WoundKind,
};

/// Ticks in a game minute.
const MINUTE: u32 = 25;
const BANDAGE: u16 = 0;
const TORCH: u16 = 1;

/// Bandages and a torch; you start with four bandages and the torch.
fn defs() -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 100,
        stack: 10,
        kind,
        worth: 0,
    };
    Defs {
        items: vec![
            item("bandage", ItemKind::Bandage { time: 10 }),
            item("torch", ItemKind::Light { radius: 6, wane: 4 }),
        ],
        kit: vec![Item::new(BANDAGE, 4), Item::new(TORCH, 1)],
        carry: 30_000,
        light_time: 5,
        douse_time: 5,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// Open grass from `hour` o'clock, with nobody wandering, and you in the middle needing nothing.
fn field(hour: u32) -> (World, ActorId) {
    let mut w = World::new(5);
    w.rules.clock.start = hour * 60;
    w.defs = Arc::new(defs());
    w.creature_mut(0).wander = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -15..15 {
        for x in -15..15 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
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

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..100 {
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

/// Puts a bandage on: the worst wound that's bleeding, else the worst not yet dressed.
fn bandage(w: &mut World, me: ActorId) -> Vec<Event> {
    let slot = slot(w, me, BANDAGE);
    act(w, me, Intent::Use { slot, def: BANDAGE })
}

fn moodle(w: &World, me: ActorId, m: Moodle) -> u8 {
    w.moodles(me)
        .iter()
        .find(|&&(x, _)| x == m)
        .map_or(0, |&(_, level)| level)
}

#[test]
fn pain_comes_from_wounds_by_their_kind_and_a_bandage_dresses_even_one_that_has_stopped() {
    let (mut w, me) = field(12);
    assert_eq!(w.pain(me), 0);
    w.wound(me, WoundKind::Scratch, Part::LeftArm);
    assert_eq!(w.pain(me), 10_000, "a fresh scratch");
    assert_eq!(moodle(&w, me, Moodle::Pain), 1, "sore");
    w.wound(me, WoundKind::Bite, Part::LeftLeg);
    assert_eq!(w.pain(me), 40_000);
    assert_eq!(moodle(&w, me, Moodle::Pain), 2, "in pain");
    w.wound(me, WoundKind::Laceration, Part::RightLeg);
    assert_eq!(w.pain(me), 60_000);
    assert_eq!(moodle(&w, me, Moodle::Pain), 3, "agony");

    // Bandages go on the bleeding first, worst first: the bite, then the laceration. Each
    // dressed hurts half as much.
    bandage(&mut w, me);
    assert!((44_000..=45_000).contains(&w.pain(me)), "{}", w.pain(me));
    bandage(&mut w, me);
    assert!((34_000..=35_000).contains(&w.pain(me)), "{}", w.pain(me));
    // The scratch never bled, but a bandage dresses it all the same: it eases the pain, and
    // First Aid learns nothing from it.
    let first_aid = w.actor(me).unwrap().skills.xp[Skill::FirstAid as usize];
    let seen = bandage(&mut w, me);
    assert!(seen.contains(&Event::Bandaged {
        who: me,
        part: Part::LeftArm
    }));
    assert!((29_000..=30_000).contains(&w.pain(me)), "{}", w.pain(me));
    let a = w.actor(me).unwrap();
    assert!(a.body.wounds.iter().all(|wd| wd.bandaged));
    assert_eq!(a.skills.xp[Skill::FirstAid as usize], first_aid);
    // All dressed: nothing more for a bandage to do.
    let seen = bandage(&mut w, me);
    assert!(!seen.iter().any(|e| matches!(e, Event::Bandaged { .. })));
}

#[test]
fn a_wound_hurts_less_the_nearer_it_is_to_closing() {
    let (mut w, me) = field(12);
    w.wound(me, WoundKind::Scratch, Part::LeftHand);
    // A scratch closes in 12 game hours (the test rules'): half way, half the pain.
    run(&mut w, 6 * 60 * MINUTE);
    assert!((4_900..=5_100).contains(&w.pain(me)), "{}", w.pain(me));
    run(&mut w, 6 * 60 * MINUTE + MINUTE);
    assert_eq!(w.pain(me), 0, "closed");
}

#[test]
fn in_pain_everything_costs_more_breath_and_in_agony_theres_no_sleep() {
    // A swing at the air: what it costs whole, with a bite (in pain) and with two (agony).
    let cost = |bites: usize| {
        let (mut w, me) = field(12);
        for _ in 0..bites {
            w.wound(me, WoundKind::Bite, Part::Torso);
        }
        let before = w.actor(me).unwrap().body.endurance;
        w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
        w.step(1);
        before - w.actor(me).unwrap().body.endurance
    };
    let whole = cost(0);
    assert!(whole > 0);
    assert_eq!(cost(1), whole * 1250 / 1000, "in pain: a quarter more");
    assert_eq!(cost(2), whole * 1500 / 1000, "in agony: half as much more");

    let (mut w, me) = field(12);
    w.rules.needs.fatigue.levels = [0, 90_000, 95_000];
    assert_eq!(w.can_sleep(me), Ok(false), "tired, on the ground");
    w.wound(me, WoundKind::Bite, Part::Torso);
    w.wound(me, WoundKind::Bite, Part::LeftArm);
    assert_eq!(w.can_sleep(me), Err(NoSleep::Pain), "in agony");
    // Dressed, the pain's bearable.
    bandage(&mut w, me);
    bandage(&mut w, me);
    assert_eq!(w.can_sleep(me), Ok(false));
}

#[test]
fn alone_in_the_dark_you_grow_uneasy_and_no_worse_and_a_light_steadies_you() {
    let (mut w, me) = field(22);
    let seen = run(&mut w, 10 * MINUTE);
    assert_eq!(w.dread(me), 30_000, "the dark's share, six points a minute");
    assert!(seen.contains(&Event::Dread { who: me, level: 1 }));
    assert_eq!(moodle(&w, me, Moodle::Dread), 1, "uneasy");
    run(&mut w, 60 * MINUTE);
    assert_eq!(w.dread(me), 30_000, "alone, no worse");
    // A torch held up takes the dark's share away, a point a minute.
    let slot = slot(&w, me, TORCH);
    act(&mut w, me, Intent::Light { slot, def: TORCH });
    let mut seen = run(&mut w, 10 * MINUTE);
    assert!((19_000..=21_000).contains(&w.dread(me)), "{}", w.dread(me));
    seen.extend(run(&mut w, 30 * MINUTE));
    assert_eq!(w.dread(me), 0);
    assert!(
        seen.contains(&Event::Dread { who: me, level: 0 }),
        "calm again"
    );
}

#[test]
fn dread_shakes_your_aim() {
    let (mut w, me) = field(22);
    let steady = w.swing(me).hit;
    run(&mut w, 10 * MINUTE);
    assert_eq!(w.dread_level(me), 1);
    assert_eq!(w.swing(me).hit, steady * 900 / 1000, "uneasy");
    // One of the dead close by, in the dark: afraid.
    w.spawn(ActorKind::Creature(0), at(5, 0)).unwrap();
    run(&mut w, 10 * MINUTE);
    assert_eq!(w.dread(me), 50_000);
    assert_eq!(w.swing(me).hit, steady * 800 / 1000, "afraid");
}

#[test]
fn near_the_dead_in_the_dark_you_are_frightened_and_cant_step_nearer_them() {
    let (mut w, me) = field(22);
    // Three of the dead to the east, after nothing: in the dark they don't see you, but you
    // know they're there.
    for p in [at(4, 0), at(5, 1), at(6, -1)] {
        w.spawn(ActorKind::Creature(0), p).unwrap();
    }
    let seen = run(&mut w, 20 * MINUTE);
    assert_eq!(w.dread(me), 90_000, "the dark and three of the dead");
    assert!(seen.contains(&Event::Dread { who: me, level: 3 }));
    assert_eq!(moodle(&w, me, Moodle::Dread), 3, "frightened");
    // Toward them, you balk; away, you go.
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: false,
        },
    );
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::Balked { who: me }));
    assert_eq!(w.actor(me).unwrap().pos, at(0, 0));
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::W,
            run: false,
        },
    );
    run(&mut w, 30);
    assert!(w.actor(me).unwrap().pos.x < 0, "backing away");
}

#[test]
fn asleep_or_on_hallowed_ground_it_ebbs_away_and_hunger_feeds_it() {
    // Uneasy in the dark; then hallowed ground round you: down three a minute, to nothing.
    let (mut w, me) = field(22);
    run(&mut w, 10 * MINUTE);
    assert_eq!(w.dread(me), 30_000);
    w.hallowed = vec![Area {
        x0: -3,
        y0: -3,
        x1: 3,
        y1: 3,
        z: 0,
    }];
    run(&mut w, 10 * MINUTE);
    assert_eq!(w.dread(me), 0);

    // Asleep in the dark, the same.
    let (mut w, me) = field(22);
    w.rules.needs.fatigue.levels = [0, 90_000, 95_000];
    w.rules.needs.fatigue.rise = 1000;
    run(&mut w, 10 * MINUTE);
    act(&mut w, me, Intent::Sleep);
    assert!(w.actor(me).unwrap().body.asleep);
    run(&mut w, 10 * MINUTE);
    assert_eq!(w.dread(me), 0);

    // Starving, by day: hunger's share, two levels past peckish.
    let (mut w, me) = field(12);
    w.rules.needs.hunger.levels = [1, 2, 3];
    w.rules.needs.hunger.rise = 100;
    run(&mut w, 10 * MINUTE);
    assert_eq!(w.dread(me), 20_000);
}
