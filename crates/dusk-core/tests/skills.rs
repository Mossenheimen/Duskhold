//! Skills: only real use under real risk teaches, repeats pay less, a skill waits at the free cap
//! for a mastery, and a level shows in the blow, the run and the bandage.

use std::sync::Arc;

use dusk_core::skills::Rise;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Event, Intent, Item, ItemDef, ItemKind, Part, Skill, Tile,
    TilePos, World, WoundKind,
};

const CLUB: u16 = 0;
const BANDAGE: u16 = 1;
const ANVIL: u16 = 2;

/// A club that always lands for 50 (a shambler has 300), a bandage, and an 8 kg anvil: more than
/// half of what you can carry, not more than all of it.
fn defs() -> Defs {
    let item = |id: &str, weight, stack, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight,
        stack,
        kind,
    };
    let club = Swing {
        wind_up: 3,
        recovery: 2,
        endurance: 0,
        hit: 10_000,
        damage: (50, 50),
        noise: 3,
    };
    Defs {
        items: vec![
            item(
                "club",
                1000,
                1,
                ItemKind::Weapon {
                    swing: club,
                    wear: 10,
                    skill: Some(Skill::Blunt),
                },
            ),
            item("bandage", 100, 10, ItemKind::Bandage { time: 40 }),
            item("anvil", 8000, 1, ItemKind::Other),
        ],
        wield: Some(CLUB),
        kit: vec![Item::new(BANDAGE, 5)],
        carry: 15_000,
        ..Defs::default()
    }
}

/// A row of floor: you in the middle (with the club and five bandages), a shambler on either
/// side, and a wall at the east one's back. The dead here never land a blow, so nothing breaks
/// off what you do.
fn row() -> (World, ActorId, ActorId, ActorId) {
    let (mut w, placed) = World::from_text(
        5,
        "\
+-+-+-+-+-+
|. z . z|.|
+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs());
    w.rules.combat.shambler_attack.hit = 0;
    w.rules.wander_chance = 0;
    // Spawned once the items are there, so it gets its kit.
    let me = w
        .spawn(ActorKind::Character, TilePos::new(2, 0, 0))
        .unwrap();
    (w, me, placed[0], placed[1])
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

fn xp(w: &World, me: ActorId, skill: Skill) -> u32 {
    w.actor(me).unwrap().skills.xp[skill as usize]
}

fn swing(w: &mut World, me: ActorId, dir: Dir8) -> Vec<Event> {
    act(w, me, Intent::Attack { dir: Some(dir) })
}

#[test]
fn a_blow_on_a_foe_teaches_and_a_swing_at_nothing_doesnt() {
    let (mut w, me, _, east) = row();
    let seen = swing(&mut w, me, Dir8::E);
    assert!(
        seen.iter()
            .any(|e| matches!(e, Event::Struck { target, .. } if *target == east)),
        "the blow landed"
    );
    assert_eq!(xp(&w, me, Skill::Blunt), 1000, "a point of Blunt");
    swing(&mut w, me, Dir8::N); // at the wall: nobody there
    assert_eq!(xp(&w, me, Skill::Blunt), 1000, "the air teaches nothing");
    assert_eq!(
        xp(&w, me, Skill::Blade),
        0,
        "a club teaches Blunt, nothing else"
    );
}

#[test]
fn bare_fists_teach_no_skill() {
    let (mut w, me, _, _) = row();
    act(&mut w, me, Intent::Unwield);
    swing(&mut w, me, Dir8::E);
    assert!(Skill::ALL.iter().all(|&s| xp(&w, me, s) == 0));
}

#[test]
fn the_same_foe_pays_less_each_time_and_a_fresh_one_pays_in_full() {
    let (mut w, me, _, _) = row();
    for _ in 0..3 {
        swing(&mut w, me, Dir8::E);
    }
    // 1 + 0.75 + 0.5625 (rounded down to thousandths).
    assert_eq!(xp(&w, me, Skill::Blunt), 1000 + 750 + 562);
    swing(&mut w, me, Dir8::W);
    assert_eq!(
        xp(&w, me, Skill::Blunt),
        2312 + 1000,
        "the other one is new"
    );
}

#[test]
fn a_foe_thats_down_pays_half() {
    let (mut w, me, _, _) = row();
    // The east one has a wall at its back: a shove can't push it, only knock it down.
    w.rules.combat.knock_down = 10_000;
    act(&mut w, me, Intent::Shove { dir: Some(Dir8::E) });
    swing(&mut w, me, Dir8::E);
    assert_eq!(xp(&w, me, Skill::Blunt), 500);
}

#[test]
fn levels_come_from_xp_and_are_told() {
    let (mut w, me, _, _) = row();
    w.give_xp(me, Skill::Blunt, 9_500);
    assert_eq!(w.skill_level(me, Skill::Blunt), 0);
    let seen = swing(&mut w, me, Dir8::E);
    assert_eq!(w.skill_level(me, Skill::Blunt), 1);
    assert!(seen.contains(&Event::SkillUp {
        who: me,
        skill: Skill::Blunt,
        level: 1
    }));
    let standing = w.skills(me)[Skill::Blunt as usize].1;
    // 10.5 XP: half a point into the 20 between level 1 (10) and level 2 (30), 25 per mille.
    assert_eq!((standing.level, standing.progress), (1, 25));
}

#[test]
fn a_skill_waits_at_the_cap_until_its_mastered() {
    let (mut w, me, _, _) = row();
    let gate = w.rules.skills.levels[7];
    w.give_xp(me, Skill::Blunt, 5_000_000);
    assert_eq!(xp(&w, me, Skill::Blunt), gate, "held at the gate");
    assert_eq!(w.skill_level(me, Skill::Blunt), 7);
    assert!(w.events().any(|e| e
        == Event::SkillAtCap {
            who: me,
            skill: Skill::Blunt
        }));
    assert_eq!(w.skills(me)[Skill::Blunt as usize].1.state, Rise::AtCap);
    // Not at the cap: no mastery.
    assert!(!w.master(me, Skill::Axe));
    assert!(w.master(me, Skill::Blunt));
    assert_eq!(w.skill_level(me, Skill::Blunt), 8);
    assert!(!w.master(me, Skill::Blunt), "once is enough");
    w.give_xp(me, Skill::Blunt, 5_000_000);
    assert_eq!(w.skill_level(me, Skill::Blunt), 10);
    assert_eq!(w.skills(me)[Skill::Blunt as usize].1.state, Rise::Top);
}

#[test]
fn only_three_skills_go_past_the_cap() {
    let (mut w, me, _, _) = row();
    let four = [Skill::Blade, Skill::Blunt, Skill::Axe, Skill::Spear];
    for s in four {
        w.give_xp(me, s, 5_000_000);
    }
    let chosen: Vec<bool> = four.iter().map(|&s| w.master(me, s)).collect();
    assert_eq!(chosen, [true, true, true, false]);
    assert_eq!(w.masteries_left(me), 0);
    assert_eq!(w.skill_level(me, Skill::Spear), 7);
}

#[test]
fn skill_lands_harder_and_wears_the_weapon_less() {
    let (mut w, me, _, east) = row();
    let blow = |seen: &[Event]| {
        seen.iter().find_map(|e| match e {
            Event::Struck { target, damage, .. } if *target == east => Some(*damage),
            _ => None,
        })
    };
    let condition = |w: &World| w.actor(me).unwrap().hands.unwrap().condition;
    let fresh = blow(&swing(&mut w, me, Dir8::E));
    let worn = 1000 - condition(&w);
    w.give_xp(me, Skill::Blunt, 300_000); // level 5
    let before = condition(&w);
    let skilled = blow(&swing(&mut w, me, Dir8::E));
    assert_eq!(fresh, Some(50));
    assert_eq!(skilled, Some(62), "+5% a level");
    assert_eq!((worn, before - condition(&w)), (10, 7), "-5% wear a level");
}

/// Open grass, `w` tiles wide, and you on its west side with the anvil if `loaded`.
fn field(w: i32, loaded: bool) -> (World, ActorId) {
    let mut world = World::new(9);
    world.rules.wander_chance = 0;
    let mut d = defs();
    if loaded {
        d.kit.push(Item::new(ANVIL, 1));
    }
    world.defs = Arc::new(d);
    for y in 0..5 {
        for x in 0..w {
            world.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    let me = world
        .spawn(ActorKind::Character, TilePos::new(1, 2, 0))
        .unwrap();
    (world, me)
}

fn run(w: &mut World, me: ActorId, dir: Dir8, ticks: u32) {
    w.intent(me, Intent::Move { dir, run: true });
    w.step(ticks);
}

#[test]
fn running_teaches_only_under_risk() {
    let (mut w, me) = field(40, false);
    run(&mut w, me, Dir8::E, 60);
    assert_eq!(
        xp(&w, me, Skill::Athletics),
        0,
        "nothing after you, nothing carried"
    );

    let (mut w, me) = field(40, true);
    run(&mut w, me, Dir8::E, 60);
    assert!(xp(&w, me, Skill::Athletics) > 0, "under a heavy load");

    // One of the dead within earshot hears you run, and comes: now it's after you.
    let (mut w, me) = field(40, false);
    w.spawn(ActorKind::Shambler, TilePos::new(1, 4, 0)).unwrap();
    run(&mut w, me, Dir8::E, 60);
    assert!(
        xp(&w, me, Skill::Athletics) > 0,
        "with one of the dead after you"
    );
}

#[test]
fn running_round_in_one_spot_pays_less_than_running_away() {
    let (mut w, me) = field(40, true);
    run(&mut w, me, Dir8::E, 80);
    let away = xp(&w, me, Skill::Athletics);

    let (mut w, me) = field(40, true);
    for i in 0..8 {
        let dir = if i % 2 == 0 { Dir8::E } else { Dir8::W };
        run(&mut w, me, dir, 10);
    }
    let round = xp(&w, me, Skill::Athletics);
    assert!(
        round * 2 < away,
        "back and forth in one spot ({round}) against the same run away ({away})"
    );
}

#[test]
fn athletics_runs_faster_and_cheaper() {
    let (mut w, me) = field(60, false);
    run(&mut w, me, Dir8::E, 40);
    let (slow, spent) = {
        let a = w.actor(me).unwrap();
        (a.pos.x, 1000 - a.body.endurance)
    };
    let (mut w, me) = field(60, false);
    w.give_xp(me, Skill::Athletics, 3_000_000);
    w.master(me, Skill::Athletics);
    w.give_xp(me, Skill::Athletics, 3_000_000);
    assert_eq!(w.skill_level(me, Skill::Athletics), 10);
    run(&mut w, me, Dir8::E, 40);
    let a = w.actor(me).unwrap();
    assert!(a.pos.x > slow, "farther in the same time");
    assert!(1000 - a.body.endurance < spent, "for less breath");
}

#[test]
fn a_bandage_on_a_bleeding_wound_teaches_first_aid_and_skill_bandages_faster() {
    let (mut w, me, _, _) = row();
    w.wound(me, WoundKind::Laceration, Part::LeftArm);
    let ticks = |w: &mut World| {
        w.intent(
            me,
            Intent::Use {
                slot: 0,
                def: BANDAGE,
            },
        );
        let mut n = 0;
        while n < 200 {
            w.step(1);
            n += 1;
            let a = w.actor(me).unwrap();
            if a.action.is_none() && a.intent == Intent::Idle {
                break;
            }
        }
        n
    };
    let slow = ticks(&mut w);
    assert_eq!(xp(&w, me, Skill::FirstAid), 3000);
    w.give_xp(me, Skill::FirstAid, 1_400_000); // to the cap: level 7
    w.wound(me, WoundKind::Laceration, Part::RightArm);
    let quick = ticks(&mut w);
    assert!(
        quick < slow,
        "level 7 bandages in {quick} ticks, level 0 in {slow}"
    );
}
