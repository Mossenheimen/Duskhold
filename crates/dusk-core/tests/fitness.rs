//! Strength and Fitness: the day's breath spent and strain turn into growth at midnight, the
//! more the less each; blows, shoves and a heavy load are strain; Strength carries more, hits
//! harder and shoves down more often; Fitness makes breath cheaper and brings it back faster.

use std::sync::Arc;

use dusk_core::fitness::{Growth, Stat, TOP};
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Event, Intent, Item, ItemDef, ItemKind, Tile, TilePos, World,
};

const CLUB: u16 = 0;
const STONE: u16 = 1;
const EAST: Option<Dir8> = Some(Dir8::E);

/// You on grass with a club in hand (a sure 10-point blow for 5 points of breath), a 10 kg
/// stone at your feet (you carry 15 kg), and one of the dead east of you that sees all round,
/// takes many blows and never lands one.
fn field() -> (World, ActorId, ActorId) {
    let item = |id: &str, weight: u32, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight,
        stack: 1,
        kind,
        worth: 0,
    };
    let club = ItemKind::Weapon {
        swing: Swing {
            wind_up: 4,
            recovery: 6,
            endurance: 50,
            hit: 10_000,
            damage: (100, 100),
            noise: 3,
        },
        wear: 1,
        skill: None,
    };
    let mut w = World::new(5);
    w.defs = Arc::new(Defs {
        items: vec![
            item("club", 1000, club),
            item("stone", 10_000, ItemKind::Other),
        ],
        kit: vec![Item::new(CLUB, 1)],
        carry: 15_000,
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
    for y in -3..3 {
        for x in -10..10 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(1, 0, 0))
        .unwrap();
    w.put(TilePos::new(0, 0, 0), Item::new(STONE, 1));
    w.intent(me, Intent::Wield { slot: 0, def: CLUB });
    w.step(10);
    (w, me, z)
}

/// Steps until the act under way is over, and its events.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..40 {
        w.step(1);
        seen.extend(w.events());
        if w.actor(me).unwrap().action.is_none() && w.actor(me).unwrap().intent == Intent::Idle {
            break;
        }
    }
    seen
}

#[test]
fn a_day_gives_less_for_each_more_of_work_and_a_week_beats_a_day() {
    let g = Growth {
        most: 600,
        half: 1000,
    };
    assert_eq!(g.gain(0, 0), 0);
    assert_eq!(g.gain(1000, 0), 300, "half the most at `half`");
    assert!(
        g.gain(2000, 0) < 2 * g.gain(1000, 0),
        "the more, the less each"
    );
    assert!(g.gain(1_000_000, 0) <= 600, "never more than the most");
    assert!(
        g.gain(1000, 5000) < g.gain(1000, 0),
        "less, the higher it is"
    );
    assert_eq!(g.gain(1000, TOP), 0);
    // Seven days of a day's work against all of it in one day.
    let mut week = 0;
    for _ in 0..7 {
        week += g.gain(1000, week);
    }
    assert!(week > 2 * g.gain(7000, 0), "{week}");
}

#[test]
fn the_days_breath_and_strain_turn_into_growth_at_midnight() {
    let (mut w, me, _) = field();
    // A minute to midnight.
    w.rules.clock.start = 24 * 60 - 1 - (w.tick / 25) as u32;
    act(&mut w, me, Intent::Attack { dir: EAST });
    let a = w.actor(me).unwrap();
    let (spent, strain) = (a.body.spent, a.strain);
    assert_eq!(spent, 50, "the swing's breath");
    assert_eq!(strain, w.rules.strength.blow, "a blow that landed");
    let mut grew = Vec::new();
    for _ in 0..60 {
        w.step(1);
        grew.extend(w.events().filter(|e| matches!(e, Event::Grew { .. })));
    }
    let a = w.actor(me).unwrap();
    assert_eq!(a.fitness, w.rules.fitness.growth.gain(spent, 0));
    assert_eq!(a.strength, w.rules.strength.growth.gain(strain, 0));
    assert!(a.fitness > 0 && a.strength > 0);
    assert_eq!(
        (a.body.spent, a.strain),
        (0, 0),
        "the day's count starts again"
    );
    assert!(grew.is_empty(), "no whole level from one swing");
}

#[test]
fn a_whole_level_reached_is_news() {
    let (mut w, me, _) = field();
    w.set_build(me, 999, 0);
    w.rules.clock.start = 24 * 60 - 1 - (w.tick / 25) as u32;
    act(&mut w, me, Intent::Attack { dir: EAST });
    let mut grew = Vec::new();
    for _ in 0..60 {
        w.step(1);
        grew.extend(w.events().filter(|e| matches!(e, Event::Grew { .. })));
    }
    assert_eq!(
        grew,
        vec![Event::Grew {
            who: me,
            stat: Stat::Strength,
            level: 1
        }]
    );
}

#[test]
fn blows_shoves_and_a_heavy_load_are_strain() {
    let (mut w, me, _) = field();
    let s = w.rules.strength;
    act(&mut w, me, Intent::Attack { dir: EAST });
    act(&mut w, me, Intent::HeavyAttack { dir: EAST });
    assert_eq!(
        w.actor(me).unwrap().strain,
        3 * s.blow,
        "a heavy blow counts twice"
    );
    act(&mut w, me, Intent::Shove { dir: EAST });
    assert_eq!(w.actor(me).unwrap().strain, 3 * s.blow + s.shove);
    // Walking with nothing heavy is no strain; with the stone (over half what you carry), it is.
    // Away from the dead, which follows.
    let walk = |w: &mut World, to: i32| {
        w.intent(
            me,
            Intent::WalkTo {
                to: TilePos::new(to, 0, 0),
                run: false,
            },
        );
        w.step(60);
        assert_eq!(w.actor(me).unwrap().pos, TilePos::new(to, 0, 0));
    };
    let before = w.actor(me).unwrap().strain;
    walk(&mut w, -4);
    assert_eq!(w.actor(me).unwrap().strain, before);
    let here = TilePos::new(-4, 0, 0);
    w.put(here, Item::new(STONE, 1));
    act(
        &mut w,
        me,
        Intent::Take {
            at: here,
            slot: 0,
            def: STONE,
        },
    );
    assert!(w.load(me) * 2 > w.carry(me));
    walk(&mut w, -8);
    assert_eq!(w.actor(me).unwrap().strain, before + 4 * s.haul);
}

#[test]
fn strength_carries_more_hits_harder_and_shoves_down_more_often() {
    let (mut w, me, z) = field();
    let s = w.rules.strength;
    assert_eq!(w.carry(me), 15_000);
    w.set_build(me, 5000, 0);
    assert_eq!(w.carry(me), 15_000 + 5 * s.carry);
    w.set_build(me, TOP, 0);
    let struck = act(&mut w, me, Intent::Attack { dir: EAST });
    let harm = struck.iter().find_map(|e| match *e {
        Event::Struck { damage, .. } => Some(damage),
        _ => None,
    });
    assert_eq!(harm, Some(100 * (1000 + 10 * s.harm as i32) / 1000));
    // A shove that never knocks down, but for Strength's share: at the top, it always does.
    w.rules.combat.knock_down = 0;
    w.rules.strength.knock_down = 1000;
    let shoved = |w: &mut World| {
        act(w, me, Intent::Shove { dir: EAST })
            .iter()
            .find_map(|e| match *e {
                Event::Shoved { down, .. } => Some(down),
                _ => None,
            })
    };
    assert_eq!(shoved(&mut w), Some(true));
    w.set_build(me, 0, 0);
    // It got up, and came back beside you.
    w.step(120);
    assert_eq!(w.actor(z).unwrap().prone_until, 0);
    let pos = w.actor(z).unwrap().pos;
    assert!(pos.steps_to(TilePos::new(0, 0, 0)) == 1, "{pos:?}");
    let dir = dusk_core::Dir8::toward(pos.x, pos.y);
    let down = act(&mut w, me, Intent::Shove { dir })
        .iter()
        .find_map(|e| match *e {
            Event::Shoved { down, .. } => Some(down),
            _ => None,
        });
    assert_eq!(down, Some(false));
}

#[test]
fn fitness_makes_breath_cheaper_and_brings_it_back_faster() {
    // The breath a swing costs as it starts; and, after six, what comes back in three ticks
    // standing still.
    let swing = |fitness: u32| {
        let (mut w, me, _) = field();
        w.set_build(me, 0, fitness);
        let before = w.actor(me).unwrap().body.endurance;
        w.intent(me, Intent::Attack { dir: EAST });
        w.step(1);
        let cost = before - w.actor(me).unwrap().body.endurance;
        for _ in 0..6 {
            act(&mut w, me, Intent::Attack { dir: EAST });
        }
        let after = w.actor(me).unwrap().body.endurance;
        w.step(3);
        let back = w.actor(me).unwrap().body.endurance - after;
        (cost, back, w.rules.fitness)
    };
    let (cost, back, f) = swing(0);
    let (fit_cost, fit_back, _) = swing(TOP);
    assert_eq!(cost, 50);
    assert_eq!(fit_cost, cost * (1000 - 10 * f.cost as i32) / 1000);
    assert_eq!(fit_back, back * (1000 + 10 * f.recover as i32) / 1000);
    assert!(fit_back > back);
}
