//! Spells: Light (no slot: a light on a spot, fading, and a lure at night), Silence (no sound made
//! inside it), Fog (the dead can't see into it), Fireball (harm within its reach, what it puts down
//! burns, heard far off); their limits (known, slots, the component, range, sight); a blow breaks
//! the words and spends nothing; slots come back with a night's sleep; Arcana and the talents.

use std::sync::Arc;

use dusk_core::senses::Cone;
use dusk_core::{
    ActorId, ActorKind, Class, Defs, Event, Intent, Item, ItemDef, ItemKind, Kit, NoCast, Part,
    Remains, Skill, SpellDef, SpellKind, Talent, Tile, TilePos, Woke, World, WoundKind,
};

const SULPHUR: u16 = 0;
const LIGHT: u16 = 0;
const SILENCE: u16 = 1;
const FOG: u16 = 2;
const FIREBALL: u16 = 3;
const ME: TilePos = TilePos::new(0, 0, 0);

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A spell's numbers, ticks and tiles.
fn spell(id: &str, kind: SpellKind, slots: u8, radius: i32, light: i32) -> SpellDef {
    SpellDef {
        id: id.into(),
        name: id.into(),
        line: String::new(),
        kind,
        slots,
        cast: 10,
        range: 8,
        radius,
        light,
        lasts: 100,
        noise: 2,
        boom: 0,
        damage: (0, 0),
        component: None,
    }
}

/// Sulphur; the four spells (a fireball burns within 2 tiles for 100 points, flashes and is heard
/// 35 tiles off, and uses up a pinch of sulphur); a wizard (3 slots, the four spells, two pinches)
/// and a fighter (none); and the wizard's three talents.
fn defs() -> Defs {
    let fireball = SpellDef {
        cast: 25,
        lasts: 15,
        boom: 35,
        damage: (1000, 1000),
        component: Some(SULPHUR),
        ..spell("fireball", SpellKind::Fireball, 2, 2, 7)
    };
    let wizard = Class {
        id: "wizard".into(),
        name: "Wizard".into(),
        learn: 1000,
        endurance: 1000,
        grabbed: 1000,
        talents: vec![0, 1, 2],
        kit: Some(Kit {
            wield: None,
            pack: vec![Item::new(SULPHUR, 2)],
        }),
        slots: 3,
        spells: vec![LIGHT, SILENCE, FOG, FIREBALL],
        ..Class::default()
    };
    let fighter = Class {
        id: "fighter".into(),
        name: "Fighter".into(),
        learn: 1000,
        endurance: 1000,
        grabbed: 1000,
        ..Class::default()
    };
    let talent = |id: &str| Talent {
        id: id.into(),
        name: id.into(),
        ..Talent::default()
    };
    Defs {
        items: vec![ItemDef {
            id: "sulphur".into(),
            name: "sulphur".into(),
            weight: 50,
            stack: 20,
            kind: ItemKind::Other,
            worth: 0,
        }],
        spells: vec![
            spell("light", SpellKind::Light, 0, 0, 4),
            spell("silence", SpellKind::Silence, 1, 3, 0),
            spell("fog", SpellKind::Fog, 1, 2, 0),
            fireball,
        ],
        classes: vec![wizard, fighter],
        talents: vec![
            Talent {
                slots: 1,
                ..talent("reserve")
            },
            Talent {
                cast: 500,
                ..talent("quick")
            },
            Talent {
                hush: 0,
                ..talent("hushed")
            },
        ],
        ..Defs::default()
    }
}

/// A field of grass at `hour` o'clock with no moon, the needs stilled, and you, a wizard, at
/// (0, 0). The test shambler sees all round and doesn't wander.
fn field(hour: u32) -> (World, ActorId) {
    let mut w = World::new(8);
    w.defs = Arc::new(defs());
    {
        let c = w.creature_mut(0);
        c.wander = 0;
        c.cone = Cone { cos: -10_000 };
    }
    w.rules.clock.start = hour * 60;
    w.rules.day.night = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    for y in -10..10 {
        for x in -10..14 {
            w.grid.set_tile(at(x, y), Tile::GRASS);
        }
    }
    let me = w.spawn(ActorKind::Character, ME).unwrap();
    (w, me)
}

/// Asks for an act and steps until it's over (or never began); its events.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..80 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

fn cast(w: &mut World, me: ActorId, spell: u16, to: TilePos) -> Vec<Event> {
    act(w, me, Intent::Cast { spell, at: to })
}

fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut seen = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

fn sulphur(w: &World, me: ActorId) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .filter(|i| i.def == SULPHUR)
        .map(|i| i.count)
        .sum()
}

#[test]
fn light_costs_no_slot_lights_its_spot_and_fades() {
    let (mut w, me) = field(0);
    assert_eq!(w.light_at(at(5, 0)), 0, "midnight, no moon");
    let seen = cast(&mut w, me, LIGHT, at(5, 0));
    assert!(seen.contains(&Event::Cast {
        who: me,
        spell: LIGHT,
        at: at(5, 0)
    }));
    assert!(
        seen.iter()
            .any(|e| matches!(*e, Event::Sound { at: p, loudness: 2, .. } if p == ME)),
        "the words are heard"
    );
    assert_eq!(w.slots(me), 3, "a cantrip");
    assert!(w.light_at(at(5, 0)) > 0);
    assert!(w.light_at(at(7, 0)) > 0, "it lights 4 tiles round");
    run(&mut w, 100);
    assert!(w.zones().is_empty(), "it's run its time");
    assert_eq!(w.light_at(at(5, 0)), 0);
}

#[test]
fn at_night_the_dead_that_see_the_light_come_to_it() {
    let (mut w, me) = field(0);
    let z = w.spawn(ActorKind::Creature(0), at(9, 0)).unwrap();
    cast(&mut w, me, LIGHT, at(6, 0));
    let seen = run(&mut w, 60);
    assert!(seen.contains(&Event::SawFire {
        who: z,
        at: at(6, 0)
    }));
}

#[test]
fn inside_a_silence_no_sound_is_made() {
    let (mut w, me) = field(12);
    cast(&mut w, me, SILENCE, at(5, 0));
    assert_eq!(w.slots(me), 2);
    w.events().for_each(drop);
    w.make_sound(at(6, 2), 10, None);
    w.make_sound(at(12, 0), 10, None);
    let heard: Vec<TilePos> = w
        .events()
        .filter_map(|e| match e {
            Event::Sound { at, .. } => Some(at),
            _ => None,
        })
        .collect();
    assert_eq!(heard, vec![at(12, 0)], "only the one outside it");
}

#[test]
fn the_dead_cant_see_into_a_fog() {
    let (mut w, me) = field(12);
    let z = w.spawn(ActorKind::Creature(0), at(6, 0)).unwrap();
    assert!(w.sees(z, ME));
    cast(&mut w, me, FOG, ME);
    assert_eq!(w.slots(me), 2);
    let you = w.actor(me).unwrap().pos;
    assert!(!w.sees(z, you), "you're in the fog");
    let from = w.actor(z).unwrap().pos;
    let clear = at(from.x - 3, 5);
    assert!(w.sees(z, clear), "out of it, it sees as well as ever");
}

#[test]
fn a_fireball_burns_everything_in_its_reach_and_what_it_puts_down_burns() {
    // Midnight: the dead don't see you, and stand where they are.
    let (mut w, me) = field(0);
    let hit = w.spawn(ActorKind::Creature(0), at(5, 0)).unwrap();
    let also = w.spawn(ActorKind::Creature(0), at(5, 2)).unwrap();
    let missed = w.spawn(ActorKind::Creature(0), at(9, 0)).unwrap();
    let body = w.spawn(ActorKind::Creature(0), at(4, 0)).unwrap();
    while !w.actor(body).unwrap().dead {
        w.wound(body, WoundKind::Bite, Part::Torso);
    }
    let seen = cast(&mut w, me, FIREBALL, at(5, 0));
    assert_eq!(w.slots(me), 1, "two slots");
    assert_eq!(sulphur(&w, me), 1, "and a pinch of sulphur");
    for who in [hit, also] {
        assert!(seen.contains(&Event::Scorched {
            who: me,
            target: who,
            damage: 1000
        }));
        let a = w.actor(who).unwrap();
        assert!(a.dead);
        assert!(
            matches!(a.remains, Remains::Burning { .. }),
            "put down, it burns"
        );
    }
    assert!(matches!(
        w.actor(body).unwrap().remains,
        Remains::Burning { .. }
    ));
    assert!(!w.actor(missed).unwrap().dead, "out of its reach");
    assert!(
        seen.iter()
            .any(|e| matches!(*e, Event::Sound { at: p, loudness: 35, .. } if p == at(5, 0))),
        "heard far off"
    );
    assert!(w.light_at(at(5, 3)) > 0, "the flash");
}

#[test]
fn a_spell_has_its_limits() {
    let (mut w, me) = field(12);
    assert_eq!(w.can_cast(me, 9, at(3, 0)), Err(NoCast::Unknown));
    assert_eq!(w.can_cast(me, FIREBALL, at(9, 0)), Err(NoCast::TooFar));
    // Something solid and dark between you and the spot.
    w.grid.set_tile(
        at(2, 0),
        Tile {
            solid: true,
            opaque: true,
            ..Tile::FLOOR
        },
    );
    assert_eq!(w.can_cast(me, FIREBALL, at(4, 0)), Err(NoCast::NoSight));
    cast(&mut w, me, FIREBALL, at(4, 3));
    assert_eq!(w.can_cast(me, FIREBALL, at(4, 3)), Err(NoCast::NoSlots));
    // A fresh wizard without the sulphur.
    let (mut w, me) = field(12);
    act(
        &mut w,
        me,
        Intent::Drop {
            slot: 0,
            def: SULPHUR,
        },
    );
    assert_eq!(w.can_cast(me, FIREBALL, at(4, 3)), Err(NoCast::NoComponent));
    // And a fighter knows none.
    let (mut w, me) = field(12);
    assert!(w.choose_class(me, 1));
    assert_eq!(w.can_cast(me, LIGHT, at(3, 0)), Err(NoCast::Unknown));
    assert_eq!(w.max_slots(me), 0);
}

#[test]
fn a_blow_breaks_the_words_and_nothing_is_spent() {
    let (mut w, me) = field(12);
    w.creature_mut(0).attack.hit = 10_000;
    w.spawn(ActorKind::Creature(0), at(1, 0)).unwrap();
    let seen = cast(&mut w, me, FIREBALL, at(4, 4));
    assert!(seen.contains(&Event::SpellBroke {
        who: me,
        spell: FIREBALL
    }));
    assert!(!seen.iter().any(|e| matches!(e, Event::Cast { .. })));
    assert_eq!((w.slots(me), sulphur(&w, me)), (3, 2));
}

#[test]
fn the_slots_come_back_with_a_nights_sleep() {
    let (mut w, me) = field(22);
    cast(&mut w, me, SILENCE, at(3, 0));
    assert_eq!(w.slots(me), 2);
    w.rules.needs.fatigue.rise = 100_000;
    run(&mut w, 25);
    w.rules.needs.fatigue.rise = 0;
    w.rules.needs.fatigue.rest = 100_000;
    act(&mut w, me, Intent::Sleep);
    let seen = run(&mut w, 50);
    assert!(seen.contains(&Event::WokeUp {
        who: me,
        why: Woke::Rested
    }));
    assert!(seen.contains(&Event::SlotsBack { who: me }));
    assert_eq!(w.slots(me), 3);
}

#[test]
fn arcana_learns_from_each_spell_and_casts_faster() {
    let (mut w, me) = field(12);
    let base = w.cast_ticks(me, FIREBALL);
    assert_eq!(base, 25);
    cast(&mut w, me, LIGHT, at(3, 0));
    assert_eq!(
        w.actor(me).unwrap().skills.xp[Skill::Arcana as usize],
        w.rules.skills.cast
    );
    w.give_xp(me, Skill::Arcana, 1_000_000);
    let level = w.skill_level(me, Skill::Arcana);
    assert!(level > 0);
    assert_eq!(
        w.cast_ticks(me, FIREBALL),
        w.rules.skills.cast_ticks(25, level)
    );
    assert!(w.cast_ticks(me, FIREBALL) < base);
}

#[test]
fn the_wizards_talents_add_a_slot_quicken_the_words_and_hush_them() {
    let (mut w, me) = field(12);
    w.gain_deeds(me, 100_000_000);
    for _ in 0..3 {
        let offer = w.talent_offer(me);
        assert!(w.choose_talent(me, offer[0]));
    }
    assert_eq!(w.max_slots(me), 4);
    assert_eq!(w.slots(me), 4, "the new slot comes at once");
    assert_eq!(w.cast_ticks(me, SILENCE), 5, "half the time");
    let seen = cast(&mut w, me, LIGHT, at(3, 0));
    assert!(
        !seen
            .iter()
            .any(|e| matches!(*e, Event::Sound { at: p, .. } if p == ME)),
        "the words aren't heard"
    );
}
