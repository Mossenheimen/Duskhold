//! The Fighter: a class's faster learning, cheaper endurance and fewer grabs; three talents
//! offered at each level, and what the talents do.

use std::sync::Arc;

use dusk_core::skills::Skill;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Class, Defs, Dir8, Edge, Event, Intent, ItemDef, ItemKind, Power, Talent,
    Tile, TilePos, World,
};

const CUDGEL: u16 = 0;
// Talents.
const TOUGH: u8 = 0;
const SECOND_WIND: u8 = 1;
const CLEAVE: u8 = 2;
const SHRUG: u8 = 3;
const DOORWARDEN: u8 = 4;
const HEAVY_HAND: u8 = 5;

fn defs(fighter: bool) -> Defs {
    let club = Swing {
        wind_up: 4,
        recovery: 4,
        endurance: 100,
        hit: 5000,
        damage: (100, 100),
        noise: 2,
    };
    let talent = |id: &str| Talent {
        id: id.into(),
        name: id.into(),
        line: id.into(),
        ..Talent::default()
    };
    Defs {
        items: vec![ItemDef {
            id: "cudgel".into(),
            name: "cudgel".into(),
            weight: 100,
            stack: 1,
            kind: ItemKind::Weapon {
                swing: club,
                wear: 0,
                skill: Some(Skill::Blunt),
            },
            worth: 0,
        }],
        wield: Some(CUDGEL),
        carry: 15_000,
        classes: if fighter {
            vec![Class {
                id: "fighter".into(),
                name: "Fighter".into(),
                skills: vec![Skill::Blunt, Skill::Athletics],
                learn: 1500,
                endurance: 800,
                grabbed: 500,
                talents: (0..6).collect(),
                ..Class::default()
            }]
        } else {
            Vec::new()
        },
        talents: vec![
            Talent {
                health: 150,
                ..talent("tough")
            },
            Talent {
                second_wind: true,
                ..talent("second_wind")
            },
            Talent {
                cleave: true,
                ..talent("cleave")
            },
            Talent {
                hold: 1,
                ..talent("shrug")
            },
            Talent {
                doorway: true,
                hit: 3000,
                ..talent("doorwarden")
            },
            Talent {
                damage: 1250,
                skill: Some(Skill::Blunt),
                ..talent("heavy_hand")
            },
        ],
        ..Defs::default()
    }
}

/// A field, the needs stilled, a cudgel in hand, health enough that wounds never end a test: a
/// Fighter, or with no class at all.
fn field(fighter: bool) -> (World, ActorId) {
    let mut w = World::new(31);
    w.defs = Arc::new(defs(fighter));
    w.creature_mut(0).wander = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    w.rules.combat.character_health = 1_000_000;
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

fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut seen = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

/// Levels up, a level at a time, until `talent` is offered, and picks it (picking the first on
/// offer at each level it isn't).
fn take(w: &mut World, me: ActorId, talent: u8) {
    for _ in 0..9 {
        let need = w.rules.deeds.levels[usize::from(w.level(me)) - 1];
        let have = w.actor(me).unwrap().deeds;
        w.gain_deeds(me, need - have);
        let offer = w.talent_offer(me);
        let pick = if offer.contains(&talent) {
            talent
        } else {
            offer[0]
        };
        assert!(w.choose_talent(me, pick));
        if pick == talent {
            return;
        }
    }
    panic!("talent {talent} was never offered");
}

#[test]
fn a_fighters_swings_cost_less_breath() {
    let (mut f, fighter) = field(true);
    let (mut p, plain) = field(false);
    assert_eq!(f.class_of(fighter).unwrap().name, "Fighter");
    assert!(p.class_of(plain).is_none());
    for (w, me) in [(&mut f, fighter), (&mut p, plain)] {
        w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
        run(w, 2);
    }
    let spent = |w: &World, me: ActorId| 1000 - w.actor(me).unwrap().body.endurance;
    assert_eq!(
        (spent(&f, fighter), spent(&p, plain)),
        (80, 100),
        "four fifths"
    );
}

#[test]
fn a_fighter_is_taken_hold_of_less_often() {
    let (f, fighter) = field(true);
    let (p, plain) = field(false);
    assert_eq!(f.grab_chance(fighter), p.grab_chance(plain) / 2);
}

#[test]
fn each_level_offers_three_talents_and_you_pick_one() {
    let (mut w, me) = field(true);
    assert!(w.talent_offer(me).is_empty(), "nothing at level 1");
    w.gain_deeds(me, 30_000);
    let offer = w.talent_offer(me);
    assert_eq!(offer.len(), 3);
    assert_eq!(
        offer,
        w.talent_offer(me),
        "the same offer each time it's asked"
    );
    let not_offered = (0..6).find(|t| !offer.contains(t)).unwrap();
    assert!(!w.choose_talent(me, not_offered), "only what's offered");
    assert!(w.choose_talent(me, offer[0]));
    assert_eq!(w.picks_left(me), 0);
    assert!(w.talent_offer(me).is_empty());
    // Level 3: three more, none of them taken.
    w.gain_deeds(me, 50_000);
    let next = w.talent_offer(me);
    assert_eq!(next.len(), 3);
    assert!(!next.contains(&offer[0]));
}

#[test]
fn tough_adds_health_at_once() {
    let (mut w, me) = field(true);
    take(&mut w, me, TOUGH);
    // Each level's 5 points, and Tough's 15.
    let levels = i32::from(w.level(me)) - 1;
    assert_eq!(w.max_health(me), 1_000_000 + 50 * levels + 150);
    assert_eq!(w.actor(me).unwrap().body.health, w.max_health(me));
}

#[test]
fn the_second_wind_comes_once_between_sleeps() {
    let (mut w, me) = field(true);
    assert!(!w.can_second_wind(me), "only with the talent");
    take(&mut w, me, SECOND_WIND);
    // Run a dozen tiles (the field's 20 across from here), and stop.
    w.intent(
        me,
        Intent::Move {
            dir: Dir8::E,
            run: true,
        },
    );
    run(&mut w, 40);
    w.intent(me, Intent::Idle);
    run(&mut w, 2);
    assert!(w.actor(me).unwrap().body.endurance < 1000);
    w.intent(me, Intent::SecondWind);
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::SecondWind { who: me }));
    assert_eq!(w.actor(me).unwrap().body.endurance, 1000);
    assert!(!w.can_second_wind(me), "spent until you sleep");
}

#[test]
fn a_cleaving_blow_carries_on_into_another_beside_you() {
    let (mut w, me) = field(true);
    take(&mut w, me, CLEAVE);
    w.creature_mut(0).health = 100; // one blow of the cudgel puts one down
    w.creature_mut(0).attack.hit = 0;
    let a = w
        .spawn(ActorKind::Creature(0), TilePos::new(1, 0, 0))
        .unwrap();
    let b = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    let mut seen = Vec::new();
    while !w.actor(a).unwrap().dead {
        w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
        seen.extend(run(&mut w, 10));
    }
    // The blow that put the first down went on to the second (it lands or misses: a swing).
    let at_b = |e: &&Event| match **e {
        Event::Struck { who, target, .. } => who == me && target == b,
        Event::Missed { who, at } => who == me && at == TilePos::new(0, 1, 0),
        _ => false,
    };
    assert!(
        seen.iter().any(|e| at_b(&e)),
        "the blow carried on to the second"
    );
}

#[test]
fn shrug_takes_one_more_to_drag_you_down() {
    let dragged_down_by_three = |shrug: bool| {
        let (mut w, me) = field(true);
        if shrug {
            take(&mut w, me, SHRUG);
        }
        w.creature_mut(0).attack.hit = 10_000;
        w.rules.combat.grab = 10_000;
        // North of you: the dead face south as they come, so they see you, and go for you.
        for (dx, dy) in [(-1, -1), (0, -1), (1, -1)] {
            w.spawn(ActorKind::Creature(0), TilePos::new(dx, dy, 0))
                .unwrap();
        }
        let seen = run(&mut w, 400);
        seen.iter()
            .any(|e| matches!(*e, Event::DraggedDown { who, .. } if who == me))
    };
    assert!(dragged_down_by_three(false));
    assert!(!dragged_down_by_three(true));
}

#[test]
fn the_doorwarden_hits_surer_only_beside_a_door() {
    let (mut w, me) = field(true);
    take(&mut w, me, DOORWARDEN);
    let open = w.swing(me).hit;
    w.grid.set_west(TilePos::new(0, 0, 0), Edge::door(true));
    let doorway = w.swing(me).hit;
    assert_eq!((open, doorway), (5000, 8000));
}

#[test]
fn a_weapon_focus_adds_harm_with_its_own_weapons_only() {
    let (mut w, me) = field(true);
    take(&mut w, me, HEAVY_HAND);
    assert_eq!(w.swing(me).damage, (125, 125), "a cudgel is Blunt");
    w.intent(me, Intent::Unwield);
    run(&mut w, 10);
    assert!(w.actor(me).unwrap().hands.is_none());
    assert_eq!(
        w.swing(me).damage,
        w.rules.combat.attack.damage,
        "bare hands aren't"
    );
}

#[test]
fn a_fighter_learns_its_own_skills_half_as_fast_again() {
    // One blow on a shambler, as a Fighter and as no class at all. (It could hurt you, so the
    // blow teaches; its wind-up is too slow for it to get the chance.)
    let blunt_after_a_blow = |fighter: bool| {
        let (mut w, me) = field(fighter);
        w.creature_mut(0).health = 1_000_000;
        w.creature_mut(0).attack.wind_up = 1_000;
        w.spawn(ActorKind::Creature(0), TilePos::new(0, -1, 0))
            .unwrap();
        w.rules.skills.hit = 1000;
        let mut seen = Vec::new();
        while !seen
            .iter()
            .any(|e| matches!(*e, Event::Struck { who, .. } if who == me))
        {
            w.intent(me, Intent::Attack { dir: Some(Dir8::N) });
            seen.extend(run(&mut w, 10));
        }
        w.actor(me).unwrap().skills.xp[Skill::Blunt as usize]
    };
    let (fighter, plain) = (blunt_after_a_blow(true), blunt_after_a_blow(false));
    assert!(plain > 0);
    assert_eq!(fighter, plain * 3 / 2);
}

#[test]
fn the_second_wind_is_a_power_once_taken() {
    let (mut w, me) = field(true);
    assert!(w.powers(me).is_empty(), "only with the talent");
    take(&mut w, me, SECOND_WIND);
    assert_eq!(w.powers(me), [Power::SecondWind]);
}
