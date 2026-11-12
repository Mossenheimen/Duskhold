//! Wildlife: a beast runs from one of the living it sees near, and from a noise, and is after
//! nobody, so nobody dreads it and the pressure doesn't count it; its carcass is butchered with a
//! blade for its meat, and doesn't burn.

use std::sync::Arc;

use dusk_core::creature::GetUp;
use dusk_core::hunting::{Butcher, NoButcher};
use dusk_core::senses::Cone;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Alert, CreatureDef, Defs, Dir8, Event, Intent, ItemDef, ItemKind, NoBurn,
    Remains, Skill, Tile, TilePos, World,
};

const KNIFE: u16 = 0;
const MEAT: u16 = 1;
const DEER: ActorKind = ActorKind::Creature(1);

/// A knife that always lands for 50, and meat; the test shambler and a deer: it sees 10 tiles all
/// round, bolts from anyone within `flees`, thinks every tick, and gives 3 meat cut up.
fn defs(armed: bool, flees: i32) -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 100,
        stack: 10,
        kind,
        worth: 0,
    };
    let mut deer = CreatureDef::shambler();
    deer.id = "deer".into();
    deer.name = "deer".into();
    let s = &mut deer.stats;
    s.flees = flees;
    s.butcher = Some(Butcher {
        item: MEAT,
        count: 3,
        time: 20,
    });
    (s.sight, s.cone, s.think_every, s.wander) = (10, Cone { cos: -10_000 }, 1, 0);
    s.speed.run = 1500;
    s.attack.hit = 0;
    s.health = 40;
    s.get_up = GetUp {
        chance: 0,
        after: (0, 0),
    };
    let knife = ItemKind::Weapon {
        swing: Swing {
            wind_up: 3,
            recovery: 3,
            endurance: 0,
            hit: 10_000,
            damage: (50, 50),
            noise: 1,
        },
        wear: 0,
        skill: Some(Skill::Blade),
    };
    Defs {
        creatures: vec![CreatureDef::shambler(), deer],
        items: vec![
            item("knife", knife),
            item(
                "meat",
                ItemKind::Food {
                    nutrition: 1,
                    time: 5,
                    comfort: 0,
                    spoils: 0,
                },
            ),
        ],
        wield: armed.then_some(KNIFE),
        carry: 30_000,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A field by day, you at (0, 0).
fn field(armed: bool, flees: i32) -> (World, ActorId) {
    let mut w = World::new(5);
    w.defs = Arc::new(defs(armed, flees));
    for x in -4..40 {
        for y in -6..7 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    w.rules.clock.start = 10 * 60;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
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

#[test]
fn a_deer_bolts_from_one_it_sees_near_and_is_after_nobody() {
    let (mut w, me) = field(true, 6);
    let deer = w.spawn(DEER, at(4, 0)).unwrap();
    let seen = run(&mut w, 60);
    assert!(seen.contains(&Event::Startled { who: deer }));
    let a = w.actor(deer).unwrap();
    assert!(a.pos.x > 8, "it ran off east: at {:?}", a.pos);
    assert_eq!(a.alert(), Alert::Unaware, "it's after nobody");
    assert_eq!(w.dread(me), 0, "nobody dreads a deer");
    assert_eq!(w.dead_about(), 0, "it isn't one of the dead");
    assert!(!seen.iter().any(|e| matches!(e, Event::Struck { .. })));

    // Seen from beyond its flees, it grazes on.
    let (mut w, _) = field(true, 6);
    let deer = w.spawn(DEER, at(9, 0)).unwrap();
    let seen = run(&mut w, 60);
    assert!(!seen.contains(&Event::Startled { who: deer }));
    assert_eq!(w.actor(deer).unwrap().pos, at(9, 0));
}

#[test]
fn a_noise_startles_a_deer_that_saw_nothing() {
    let (mut w, me) = field(true, 2);
    w.creature_mut(1).cone = Cone { cos: 10_000 };
    w.creature_mut(1).sight = 0;
    let deer = w.spawn(DEER, at(7, 0)).unwrap();
    run(&mut w, 20);
    assert_eq!(
        w.actor(deer).unwrap().pos,
        at(7, 0),
        "blind, and nothing heard"
    );
    // Running is loud: 8 tiles.
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::S,
            run: true,
        },
    );
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::Startled { who: deer }));
    assert!(w.actor(deer).unwrap().pos.dist2(at(0, 0)) > at(7, 0).dist2(at(0, 0)));
}

#[test]
fn a_carcass_is_butchered_with_a_blade_and_doesnt_burn() {
    let (mut w, me) = field(true, 6);
    // It doesn't think in time to run: a deer in the hand.
    w.creature_mut(1).think_every = 100_000;
    let deer = w.spawn(DEER, at(1, 0)).unwrap();
    assert_eq!(w.can_butcher(me), Err(NoButcher::Nothing), "it's alive");
    w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
    run(&mut w, 30);
    assert!(w.actor(deer).unwrap().dead);
    assert_eq!(w.can_butcher(me), Ok(deer));
    assert_eq!(
        w.can_burn(me),
        Err(NoBurn::Nothing),
        "a beast's body doesn't burn"
    );
    w.intent(me, Intent::Butcher);
    let seen = run(&mut w, 40);
    assert!(seen.contains(&Event::Butchered {
        who: me,
        body: deer,
        item: MEAT,
        count: 3
    }));
    let meat: u16 = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .filter(|i| i.def == MEAT)
        .map(|i| i.count)
        .sum();
    assert_eq!(meat, 3);
    assert_eq!(w.actor(deer).unwrap().remains, Remains::Butchered);
    assert_eq!(w.can_butcher(me), Err(NoButcher::Nothing), "nothing left");

    // No blade, no butchering.
    let (mut w, me) = field(false, 6);
    w.creature_mut(1).think_every = 100_000;
    w.creature_mut(1).health = 1;
    let deer = w.spawn(DEER, at(1, 0)).unwrap();
    w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
    run(&mut w, 30);
    assert!(
        w.actor(deer).unwrap().dead,
        "fists will do, on one this frail"
    );
    assert_eq!(w.can_butcher(me), Err(NoButcher::NoBlade));
}
