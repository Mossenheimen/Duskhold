//! The Cleric (`faith.rs`): taking up a class before you're under way, faith and prayer at a
//! shrine, turning the dead, consecration, and the talents of faith.

use std::sync::Arc;

use dusk_core::skills::Skill;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Class, Creature, CreatureDef, Defs, Dir8, Event, GetUp, Hallow, Intent,
    Item, ItemDef, ItemKind, Kit, Mind, NoFaith, Part, Power, Pressure, Region, Remains, Talent,
    Tile, TilePos, World, WoundKind,
};

const SHAMBLER: ActorKind = ActorKind::Creature(0);
const WOLF: ActorKind = ActorKind::Creature(1);
const WIGHT: ActorKind = ActorKind::Creature(2);
const CUDGEL: u16 = 0;
const HOLY_WATER: u16 = 1;
const CANDLE: u16 = 2;
// Classes.
const FIGHTER: u8 = 0;
const CLERIC: u8 = 1;
// Talents.
const MENDING: u8 = 1;
const PURIFY: u8 = 2;
const WARDING: u8 = 3;
const STEADY: u8 = 4;
const REBUKE: u8 = 5;
const SUNDER: u8 = 6;
const LAST_RITES: u8 = 7;
/// The altar's fixture number.
const ALTAR: u16 = 1;
/// Ticks in a game minute.
const MINUTE: u32 = 25;

/// A cudgel that always lands for 10 points, holy water and a candle; the shambler, a wolf (of
/// the wild, not the dead) and a wight (turned for half as long). The Fighter has everyone's kit
/// (the cudgel and a flask); the Cleric its own, and 100 points of faith.
fn defs() -> Defs {
    let club = Swing {
        wind_up: 4,
        recovery: 4,
        endurance: 0,
        hit: 10_000,
        damage: (100, 100),
        noise: 2,
    };
    let item = |id: &str, stack: u16, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 100,
        stack,
        kind,
        worth: 0,
    };
    let talent = |id: &str| Talent {
        id: id.into(),
        name: id.into(),
        line: id.into(),
        ..Talent::default()
    };
    let kind = |id: &str, stats: Creature| CreatureDef {
        id: id.into(),
        name: id.into(),
        stats,
    };
    Defs {
        creatures: vec![
            CreatureDef::shambler(),
            kind(
                "wolf",
                Creature {
                    wild: true,
                    ..Creature::default()
                },
            ),
            kind(
                "wight",
                Creature {
                    turned: 500,
                    ..Creature::default()
                },
            ),
        ],
        items: vec![
            item(
                "cudgel",
                1,
                ItemKind::Weapon {
                    swing: club,
                    wear: 0,
                    skill: Some(Skill::Blunt),
                },
            ),
            item("holy_water", 4, ItemKind::Holy),
            item("candle", 10, ItemKind::Light { radius: 2, wane: 0 }),
        ],
        shrines: vec![false, true],
        wield: Some(CUDGEL),
        kit: vec![Item::new(HOLY_WATER, 1)],
        carry: 15_000,
        classes: vec![
            Class {
                id: "fighter".into(),
                name: "Fighter".into(),
                learn: 1000,
                endurance: 1000,
                grabbed: 1000,
                ..Class::default()
            },
            Class {
                id: "cleric".into(),
                name: "Cleric".into(),
                line: "the dead's natural enemy".into(),
                skills: vec![Skill::Devotion],
                learn: 1500,
                endurance: 1000,
                grabbed: 1000,
                talents: (0..8).collect(),
                faith: 100_000,
                turns: true,
                kit: Some(Kit {
                    wield: Some(CUDGEL),
                    pack: vec![Item::new(HOLY_WATER, 2), Item::new(CANDLE, 1)],
                }),
                wear: Vec::new(),
                slots: 0,
                spells: Vec::new(),
            },
        ],
        talents: vec![
            Talent {
                faith: 25_000,
                ..talent("devout")
            },
            Talent {
                mending: true,
                ..talent("mending")
            },
            Talent {
                purify: true,
                ..talent("purify")
            },
            Talent {
                warding: true,
                ..talent("warding")
            },
            Talent {
                steady: 500,
                ..talent("steady")
            },
            Talent {
                turn_time: 2000,
                turn_reach: 4,
                ..talent("rebuke")
            },
            Talent {
                sunder: 150,
                ..talent("sunder")
            },
            Talent {
                last_rites: true,
                ..talent("last_rites")
            },
        ],
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// The data's numbers on a world, its creatures standing still and its needs stilled.
fn settle(w: &mut World) {
    w.defs = Arc::new(defs());
    for n in 0..3 {
        w.creature_mut(n).wander = 0;
    }
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
}

/// A field of grass, health enough that wounds never end a test, and you the first class (the
/// Fighter) at its middle until you choose.
fn field() -> (World, ActorId) {
    let mut w = World::new(17);
    settle(&mut w);
    w.rules.combat.character_health = 1_000_000;
    for y in -20..20 {
        for x in -20..20 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    let me = w.spawn(ActorKind::Character, at(0, 0)).unwrap();
    (w, me)
}

/// The same field, and you a Cleric.
fn cleric() -> (World, ActorId) {
    let (mut w, me) = field();
    assert!(w.choose_class(me, CLERIC));
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

fn count(w: &World, me: ActorId, def: u16) -> u16 {
    let items = &w.actor(me).unwrap().items;
    items.iter().filter(|i| i.def == def).map(|i| i.count).sum()
}

/// Steps until `id` is turned (the turning's strike tick), and how long it's turned for.
fn turned_for(w: &mut World, id: ActorId) -> u64 {
    while !w.is_turned(id) {
        w.step(1);
        assert!(w.tick < 200, "never turned");
    }
    let Mind::Turned { until, .. } = w.actor(id).unwrap().mind else {
        unreachable!()
    };
    until - w.tick
}

fn altar(w: &mut World, x: i32, y: i32) {
    w.grid.set_tile(
        at(x, y),
        Tile {
            fixture: ALTAR,
            solid: true,
            ..Tile::GRASS
        },
    );
}

/// A shambler put down on a tile, to get up an hour later.
fn body(w: &mut World, at: TilePos) -> ActorId {
    let z = w.creature_mut(0);
    z.health = 1;
    z.get_up = GetUp {
        chance: 10_000,
        after: (60, 60),
    };
    let b = w.spawn(SHAMBLER, at).unwrap();
    w.wound(b, WoundKind::Bite, Part::Torso);
    assert!(w.actor(b).unwrap().dead);
    b
}

#[test]
fn you_take_up_a_class_before_youre_under_way_and_its_kit_with_it() {
    let (mut w, me) = field();
    assert_eq!(w.class_of(me).unwrap().name, "Fighter");
    assert_eq!(w.max_faith(me), 0, "a Fighter has no faith");
    assert_eq!(w.actor(me).unwrap().hands.map(|i| i.def), Some(CUDGEL));
    assert_eq!(count(&w, me, HOLY_WATER), 1, "everyone's kit");
    assert!(!w.choose_class(me, 9), "no such class");
    assert!(w.choose_class(me, CLERIC));
    assert!(w.events().any(|e| e
        == Event::ClassChosen {
            who: me,
            class: CLERIC
        }));
    assert_eq!(w.class_of(me).unwrap().name, "Cleric");
    assert_eq!(
        (count(&w, me, HOLY_WATER), count(&w, me, CANDLE)),
        (2, 1),
        "the Cleric's own kit, in place of everyone's"
    );
    assert_eq!(w.faith(me), 100_000, "and its faith, full");
    run(&mut w, MINUTE);
    assert!(!w.choose_class(me, FIGHTER), "under way, it's too late");
    assert_eq!(w.class_of(me).unwrap().name, "Cleric");
}

#[test]
fn the_dead_you_see_within_reach_turn_and_flee() {
    let (mut w, me) = cleric();
    w.creature_mut(0).attack.hit = 0;
    // North of you, facing you; far off to the north; and south, behind a line of rock.
    let near = w.spawn(SHAMBLER, at(0, -3)).unwrap();
    let far = w.spawn(SHAMBLER, at(0, -15)).unwrap();
    for x in -3..=3 {
        w.grid.set_tile(at(x, 2), Tile::ROCK);
    }
    let walled = w.spawn(SHAMBLER, at(0, 4)).unwrap();
    run(&mut w, 10);
    assert!(
        matches!(
            w.actor(near).unwrap().mind,
            Mind::Hunt {
                target: Some(_),
                ..
            }
        ),
        "it saw you"
    );
    w.intent(me, Intent::Turn);
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::Turned {
        who: me,
        turned: 1,
        sundered: 0
    }));
    assert!(w.is_turned(near));
    assert!(!w.is_turned(far), "out of reach");
    assert!(!w.is_turned(walled), "out of sight");
    assert_eq!(w.faith(me), 70_000);
    let before = w.actor(near).unwrap().pos.dist2(at(0, 0));
    run(&mut w, 50);
    assert!(
        w.actor(near).unwrap().pos.dist2(at(0, 0)) > before,
        "it flees"
    );
}

#[test]
fn held_you_can_still_turn_the_dead_and_they_let_go() {
    let (mut w, me) = cleric();
    w.rules.combat.grab = 10_000;
    w.rules.faith.turn_wind_up = 1;
    w.creature_mut(0).attack.hit = 10_000;
    let z = w.spawn(SHAMBLER, at(0, -1)).unwrap();
    while !w.is_held(me) {
        w.step(1);
        w.events().for_each(drop);
        assert!(w.tick < 400, "never held");
    }
    w.intent(me, Intent::Turn);
    let seen = run(&mut w, 10);
    assert!(seen.contains(&Event::Freed { who: me, from: z }));
    assert!(!w.is_held(me));
    assert!(w.is_turned(z));
}

#[test]
fn a_blow_ends_the_turning_and_a_turned_one_isnt_struck_unawares() {
    let (mut w, me) = cleric();
    w.creature_mut(0).health = 1_000_000;
    // Hemmed in by rock, it can't get away: it stands beside you, facing south, blind to you.
    let z = w.spawn(SHAMBLER, at(1, 0)).unwrap();
    for (x, y) in [(1, -1), (2, -1), (2, 0), (2, 1), (1, 1)] {
        w.grid.set_tile(at(x, y), Tile::ROCK);
    }
    w.intent(me, Intent::Turn);
    run(&mut w, 20);
    assert!(w.is_turned(z));
    w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
    let seen = run(&mut w, 10);
    assert!(
        seen.iter()
            .any(|e| matches!(*e, Event::Struck { who, target, .. } if who == me && target == z))
    );
    assert!(
        !seen.iter().any(|e| matches!(*e, Event::SneakAttack { .. })),
        "it knows you're there"
    );
    assert!(!w.is_turned(z), "the blow ended it");
    assert!(matches!(
        w.actor(z).unwrap().mind,
        Mind::Hunt {
            target: Some(_),
            ..
        }
    ));
}

#[test]
fn the_turned_hunt_again_when_their_time_is_up() {
    let (mut w, me) = cleric();
    w.creature_mut(0).attack.hit = 0;
    let z = w.spawn(SHAMBLER, at(0, -3)).unwrap();
    w.intent(me, Intent::Turn);
    assert_eq!(turned_for(&mut w, z), 200);
    run(&mut w, 200 + 10);
    assert!(!w.is_turned(z));
    assert!(matches!(w.actor(z).unwrap().mind, Mind::Hunt { .. }));
}

#[test]
fn an_old_dead_thing_is_turned_for_less_and_the_wild_not_at_all() {
    let (mut w, me) = cleric();
    for n in 0..3 {
        w.creature_mut(n).attack.hit = 0;
    }
    let wight = w.spawn(WIGHT, at(0, -3)).unwrap();
    let wolf = w.spawn(WOLF, at(3, 0)).unwrap();
    w.intent(me, Intent::Turn);
    assert_eq!(turned_for(&mut w, wight), 100, "half the time");
    assert!(!w.is_turned(wolf), "a wolf isn't one of the dead");
}

#[test]
fn turning_wants_a_class_that_turns_and_faith_enough() {
    let (w, fighter) = field();
    assert_eq!(w.can_turn(fighter), Err(NoFaith::NotYours));
    let (mut w, me) = cleric();
    for left in [70_000, 40_000, 10_000] {
        assert_eq!(w.can_turn(me), Ok(()));
        w.intent(me, Intent::Turn);
        run(&mut w, 20);
        assert_eq!(w.faith(me), left);
    }
    assert_eq!(w.can_turn(me), Err(NoFaith::TooLittle), "pray for more");
}

#[test]
fn anyone_may_pray_at_a_shrine_for_its_comfort() {
    let (mut w, me) = field();
    assert_eq!(w.can_pray(me), Err(NoFaith::Nothing), "no shrine");
    altar(&mut w, 0, -1);
    // Midnight, alone in the dark: dread rises.
    w.rules.clock.start = 0;
    run(&mut w, 10 * MINUTE);
    let before = w.dread(me);
    assert!(before > 0);
    assert_eq!(w.can_pray(me), Ok(at(0, -1)));
    w.intent(me, Intent::Pray);
    let seen = run(&mut w, 101);
    assert!(seen.contains(&Event::Prayed {
        who: me,
        faith: false
    }));
    assert!(w.dread(me) < before, "eased");
    assert_eq!(w.faith(me), 0, "a Fighter has none to get back");
}

#[test]
fn prayer_at_a_shrine_brings_a_clerics_faith_back() {
    let (mut w, me) = cleric();
    altar(&mut w, 0, -1);
    w.intent(me, Intent::Turn);
    run(&mut w, 20);
    assert_eq!(w.faith(me), 70_000);
    w.intent(me, Intent::Pray);
    run(&mut w, 99);
    assert_eq!(w.faith(me), 70_000, "not until the prayer's said");
    let seen = run(&mut w, 2);
    assert!(seen.contains(&Event::Prayed {
        who: me,
        faith: true
    }));
    assert_eq!(w.faith(me), 100_000);
}

#[test]
fn with_purify_a_prayer_cures_ghoul_fever() {
    let (mut w, me) = cleric();
    take(&mut w, me, PURIFY);
    altar(&mut w, 0, -1);
    w.catch_fever(me);
    assert_eq!(w.fever_stage(me), 1);
    w.intent(me, Intent::Pray);
    let seen = run(&mut w, 101);
    assert!(seen.contains(&Event::Cured { who: me }));
    assert_eq!(w.fever_stage(me), 0);
}

/// A region whose patch began with 10 of the dead, its pressure still but for the dead.
fn region() -> Region {
    Region {
        pressure: Pressure {
            start: 1000,
            least: 250,
            most: 2000,
            per_day: 0,
            ceiling: 1100,
            per_rise: 20,
            per_burn: 30,
            every: 0,
        },
        kinds: vec![(SHAMBLER, 10_000)],
        chunks: (8, 8),
        dead: 10,
        ..Region::default()
    }
}

#[test]
fn holy_water_consecrates_a_body_and_it_never_gets_up() {
    let (mut w, me) = field();
    w.set_region(region());
    let b = body(&mut w, at(0, 1));
    assert_eq!(w.can_consecrate(me), Ok((b, Hallow::Water)));
    w.intent(me, Intent::Consecrate);
    let seen = run(&mut w, 40);
    assert!(seen.contains(&Event::Consecrated {
        who: me,
        body: b,
        water: true
    }));
    assert_eq!(w.actor(b).unwrap().remains, Remains::Hallowed);
    assert_eq!(count(&w, me, HOLY_WATER), 0, "the flask's used up");
    assert_eq!(
        w.pressure(),
        970,
        "the pressure falls, as for a body burned"
    );
    run(&mut w, 120 * MINUTE);
    assert!(w.actor(b).unwrap().dead, "it never got up");
    assert_eq!(
        w.can_consecrate(me),
        Err(NoFaith::Nothing),
        "nothing left to consecrate"
    );
    // A Fighter with no flask left can't.
    body(&mut w, at(0, 1));
    assert_eq!(w.can_consecrate(me), Err(NoFaith::NoWater));
}

#[test]
fn a_cleric_consecrates_with_faith_once_its_holy_water_is_gone() {
    let (mut w, me) = cleric();
    for with in [Hallow::Water, Hallow::Water, Hallow::Faith] {
        let b = body(&mut w, at(0, 1));
        assert_eq!(w.can_consecrate(me), Ok((b, with)));
        w.intent(me, Intent::Consecrate);
        run(&mut w, 40);
        assert_eq!(w.actor(b).unwrap().remains, Remains::Hallowed);
    }
    assert_eq!(count(&w, me, HOLY_WATER), 0);
    assert_eq!(w.faith(me), 85_000);
}

#[test]
fn a_characters_body_consecrated_lets_its_gear_fall_and_never_rises() {
    let (mut w, me) = field();
    w.rules.combat.character_health = 100;
    let other = w.spawn(ActorKind::Character, at(0, 1)).unwrap();
    w.wound(other, WoundKind::Bite, Part::Torso);
    assert!(w.actor(other).unwrap().dead);
    assert!(matches!(
        w.actor(other).unwrap().remains,
        Remains::Lying | Remains::GetsUp { .. }
    ));
    assert_eq!(w.can_consecrate(me), Ok((other, Hallow::Water)));
    w.intent(me, Intent::Consecrate);
    run(&mut w, 40);
    assert_eq!(w.actor(other).unwrap().remains, Remains::Hallowed);
    let a = w.actor(other).unwrap();
    assert!(a.items.is_empty() && a.hands.is_none(), "nothing kept");
    assert!(
        w.store(at(0, 1)).iter().any(|i| i.def == CUDGEL),
        "its gear lies where it fell"
    );
}

#[test]
fn mending_stops_the_worst_bleeding_and_gives_health_back() {
    let (mut w, me) = cleric();
    assert_eq!(w.can_heal(me), Err(NoFaith::NotYours), "a talent's");
    take(&mut w, me, MENDING);
    assert_eq!(w.can_heal(me), Err(NoFaith::Nothing), "nothing to mend");
    w.wound(me, WoundKind::Laceration, Part::Torso);
    let hurt = w.actor(me).unwrap().body.health;
    w.intent(me, Intent::Heal);
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::Healed { who: me }));
    let a = w.actor(me).unwrap();
    assert!(a.body.wounds.iter().all(|w| w.bleeding == 0), "stopped");
    assert!(a.body.health > hurt, "health back");
    assert_eq!(w.faith(me), 80_000);
    assert!(
        a.skills.xp[Skill::Devotion as usize] > 0,
        "a wound that bled teaches Devotion"
    );
}

/// You, a Cleric with Warding, in a corridor beside a shut door; a shambler at its far end.
const CORRIDOR: &str = "\
+-+-+-+-+-+-+-+
|. . @D. . . z|
+-+-+-+-+-+-+-+
";

fn corridor() -> (World, ActorId) {
    let (mut w, _) = World::from_text(1, CORRIDOR).unwrap();
    settle(&mut w);
    let me = w.character().unwrap();
    assert!(w.choose_class(me, CLERIC));
    take(&mut w, me, WARDING);
    (w, me)
}

/// The blows the shambler beats on the door in 30 seconds of your noise behind it.
fn blows_on_the_door(w: &mut World, me: ActorId) -> usize {
    w.make_sound(at(2, 0), 12, Some(me));
    let seen = run(w, 300);
    seen.iter()
        .filter(|e| matches!(e, Event::Thumped { .. }))
        .count()
}

#[test]
fn the_dead_dont_beat_on_a_warded_door() {
    let (mut w, me) = corridor();
    assert!(blows_on_the_door(&mut w, me) > 0, "unwarded, they do");

    let (mut w, me) = corridor();
    assert_eq!(w.can_ward(me), Ok(Dir8::E));
    w.intent(me, Intent::Ward);
    let seen = run(&mut w, 50);
    assert!(seen.contains(&Event::Warded {
        who: me,
        at: at(3, 0),
        side: Dir8::W
    }));
    assert!(w.is_warded(at(3, 0), Dir8::W) && w.is_warded(at(2, 0), Dir8::E));
    assert_eq!(w.faith(me), 60_000);
    assert_eq!(blows_on_the_door(&mut w, me), 0);
}

#[test]
fn a_ward_fades_after_its_hours() {
    let (mut w, me) = corridor();
    w.intent(me, Intent::Ward);
    run(&mut w, 50);
    let seen = run(&mut w, 8 * 60 * MINUTE);
    assert!(seen.contains(&Event::WardFaded {
        at: at(3, 0),
        side: Dir8::W
    }));
    assert!(!w.is_warded(at(3, 0), Dir8::W));
}

#[test]
fn steady_slows_dread_while_you_hold_a_light() {
    let dread_near_the_dead = |steady: bool| {
        let (mut w, me) = cleric();
        if steady {
            take(&mut w, me, STEADY);
        }
        let slot = w
            .actor(me)
            .unwrap()
            .items
            .iter()
            .position(|i| i.def == CANDLE)
            .unwrap() as u16;
        w.intent(me, Intent::Light { slot, def: CANDLE });
        run(&mut w, 20);
        assert!(w.actor(me).unwrap().light.is_some());
        w.creature_mut(0).attack.hit = 0;
        w.spawn(SHAMBLER, at(0, -4)).unwrap();
        run(&mut w, 3 * MINUTE);
        w.dread(me)
    };
    let (plain, steady) = (dread_near_the_dead(false), dread_near_the_dead(true));
    assert!(plain > 0);
    assert_eq!(steady * 2, plain, "half as fast");
}

#[test]
fn sunder_puts_down_the_weakest_of_the_dead_it_turns() {
    let (mut w, me) = cleric();
    take(&mut w, me, SUNDER);
    for n in [0, 2] {
        w.creature_mut(n).attack.hit = 0;
    }
    let weak = w.spawn(SHAMBLER, at(0, -3)).unwrap();
    w.wound(weak, WoundKind::Laceration, Part::Torso);
    assert_eq!(w.actor(weak).unwrap().body.health, 150);
    let strong = w.spawn(WIGHT, at(3, 0)).unwrap();
    w.intent(me, Intent::Turn);
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::Turned {
        who: me,
        turned: 1,
        sundered: 1
    }));
    assert!(w.actor(weak).unwrap().dead);
    assert!(w.is_turned(strong));
}

#[test]
fn with_last_rites_the_dead_you_put_down_stay_down() {
    let lies = |rites: bool| {
        let (mut w, me) = cleric();
        if rites {
            take(&mut w, me, LAST_RITES);
        }
        let z = w.creature_mut(0);
        (z.health, z.attack.hit) = (100, 0);
        z.get_up = GetUp {
            chance: 10_000,
            after: (60, 60),
        };
        let z = w.spawn(SHAMBLER, at(1, 0)).unwrap();
        while !w.actor(z).unwrap().dead {
            w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
            run(&mut w, 10);
        }
        w.actor(z).unwrap().remains
    };
    assert!(matches!(lies(false), Remains::GetsUp { .. }));
    assert_eq!(lies(true), Remains::Lying);
}

#[test]
fn rebuke_turns_the_dead_for_longer_and_from_farther() {
    let (mut w, me) = cleric();
    assert_eq!(w.turn_reach(me), 8);
    take(&mut w, me, REBUKE);
    assert_eq!(w.turn_reach(me), 12);
    w.creature_mut(0).attack.hit = 0;
    let z = w.spawn(SHAMBLER, at(0, -11)).unwrap();
    w.intent(me, Intent::Turn);
    assert_eq!(turned_for(&mut w, z), 400, "twice as long, 11 tiles off");
}

#[test]
fn devotion_grows_by_turning_the_dead_and_makes_faith_go_further() {
    let (mut w, me) = cleric();
    w.creature_mut(0).attack.hit = 0;
    w.spawn(SHAMBLER, at(0, -3)).unwrap();
    w.spawn(SHAMBLER, at(3, 0)).unwrap();
    w.intent(me, Intent::Turn);
    run(&mut w, 20);
    // Two of the dead turned: 2 XP each, half as much again for a Cleric.
    assert_eq!(
        w.actor(me).unwrap().skills.xp[Skill::Devotion as usize],
        6000
    );
    w.give_xp(me, Skill::Devotion, 1_000_000);
    assert_eq!(w.skill_level(me, Skill::Devotion), 7);
    assert_eq!(w.faith_cost(me, 30_000), 21_600, "28% off at level 7");
}

#[test]
fn a_clerics_powers_are_turning_then_the_talents_it_acts_with() {
    let (w, me) = field();
    assert!(w.powers(me).is_empty(), "no class, no powers");
    let (mut w, me) = cleric();
    assert_eq!(w.powers(me), [Power::Turn]);
    take(&mut w, me, WARDING);
    assert_eq!(
        w.powers(me),
        [Power::Turn],
        "a ward is put on a door, not called on"
    );
    take(&mut w, me, MENDING);
    assert_eq!(w.powers(me), [Power::Turn, Power::Mend]);
}
