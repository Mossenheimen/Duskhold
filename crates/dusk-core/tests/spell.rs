//! Spells: Light (free: a light on a spot, fading, and a lure at night), Silence (no sound made
//! inside it), Fog (the dead can't see into it), Fireball (harm within its reach, what it puts down
//! burns, heard far off), Firebolt (the first on its line burns; a wall stops it), Fire Nova
//! (everyone beside you burns and is thrown back); their limits (known, mana, the component,
//! range, sight); a blow sets the words back, and the spell goes on; mana comes back by the
//! second; Arcana and the talents.

use std::sync::Arc;

use dusk_core::senses::Cone;
use dusk_core::{
    ActorId, ActorKind, Class, Defs, Dir8, Edge, Event, Intent, Item, ItemDef, ItemKind, Kit,
    NoCast, Part, Power, Remains, Skill, SpellDef, SpellKind, Talent, Tile, TilePos, World,
    WoundKind,
};

const SULPHUR: u16 = 0;
const LIGHT: u16 = 0;
const SILENCE: u16 = 1;
const FOG: u16 = 2;
const FIREBALL: u16 = 3;
const BOLT: u16 = 4;
const NOVA: u16 = 5;
const ME: TilePos = TilePos::new(0, 0, 0);

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A spell's numbers, ticks and tiles.
fn spell(id: &str, kind: SpellKind, mana: i32, radius: i32, light: i32) -> SpellDef {
    SpellDef {
        id: id.into(),
        name: id.into(),
        line: String::new(),
        kind,
        mana,
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
/// 35 tiles off, costs 20 mana and uses up a pinch of sulphur); a wizard (30 mana, none coming
/// back unless a test says so, the four spells, two pinches) and a fighter (none); and the
/// wizard's three talents.
fn defs() -> Defs {
    let fireball = SpellDef {
        cast: 25,
        lasts: 15,
        boom: 35,
        damage: (1000, 1000),
        component: Some(SULPHUR),
        ..spell("fireball", SpellKind::Fireball, 20_000, 2, 7)
    };
    let wizard = Class {
        id: "wizard".into(),
        name: "Wizard".into(),
        learn: 1000,
        endurance: 1000,
        staggered: 1000,
        talents: vec![0, 1, 2],
        kit: Some(Kit {
            wield: None,
            pack: vec![Item::new(SULPHUR, 2)],
        }),
        mana: 30_000,
        mana_regen: 0,
        spells: vec![LIGHT, SILENCE, FOG, FIREBALL, BOLT, NOVA],
        ..Class::default()
    };
    let fighter = Class {
        id: "fighter".into(),
        name: "Fighter".into(),
        learn: 1000,
        endurance: 1000,
        staggered: 1000,
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
            spell("silence", SpellKind::Silence, 10_000, 3, 0),
            spell("fog", SpellKind::Fog, 10_000, 2, 0),
            fireball,
            // A bolt: 2 mana, 5 ticks of words, 50 points to the first on its line.
            SpellDef {
                cast: 5,
                lasts: 0,
                damage: (500, 500),
                ..spell("bolt", SpellKind::Bolt, 2_000, 0, 0)
            },
            // A nova: 5 mana, 4 ticks, 10 points to everyone beside you, thrown back.
            SpellDef {
                cast: 4,
                range: 1,
                lasts: 0,
                damage: (100, 100),
                ..spell("nova", SpellKind::Nova, 5_000, 1, 0)
            },
        ],
        classes: vec![wizard, fighter],
        talents: vec![
            Talent {
                mana: 10_000,
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
    assert_eq!(w.mana(me), 30_000, "free");
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
    assert_eq!(w.mana(me), 20_000);
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
    assert_eq!(w.mana(me), 20_000);
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
    assert_eq!(w.mana(me), 10_000, "twenty mana");
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
    assert_eq!(w.can_cast(me, FIREBALL, at(4, 3)), Err(NoCast::NoMana));
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
    assert_eq!(w.max_mana(me), 0);
}

#[test]
fn a_blow_sets_the_words_back_and_the_spell_goes_on() {
    // Alone, the fireball's words take its 25 ticks.
    let (mut w, me) = field(12);
    let (alone, _) = said(&mut w, me, FIREBALL, at(4, 4));
    // With one of the dead beside you landing every blow, each sets them back 3 ticks, and the
    // fireball is cast all the same, later (the second playtest: "hits set a cast back not break
    // it").
    let (mut w, me) = field(12);
    w.creature_mut(0).attack.hit = 10_000;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    w.spawn(ActorKind::Creature(0), at(1, 0)).unwrap();
    w.intent(
        me,
        Intent::Cast {
            spell: FIREBALL,
            at: at(4, 4),
        },
    );
    let (mut backs, mut late) = (0, 0);
    for _ in 0..80 {
        w.step(1);
        let seen: Vec<Event> = w.events().collect();
        backs += seen
            .iter()
            .filter(|e| matches!(e, Event::SpellSetBack { who, .. } if *who == me))
            .count();
        if seen.iter().any(|e| matches!(e, Event::Cast { .. })) {
            late = w.tick;
            break;
        }
    }
    assert!(backs >= 1, "set back");
    assert!(late > alone, "later: {late} against {alone}");
    assert_eq!(w.mana(me), 10_000, "and paid for");
    assert_eq!(sulphur(&w, me), 1);
}

#[test]
fn a_firebolt_burns_the_first_on_its_line_and_a_wall_stops_it() {
    let (mut w, me) = field(12);
    let near = w.spawn(ActorKind::Creature(0), at(2, 0)).unwrap();
    let far = w.spawn(ActorKind::Creature(0), at(4, 0)).unwrap();
    let full = w.actor(near).unwrap().body.health;
    // Aimed at the far one, the near one on the line takes it.
    let seen = cast(&mut w, me, BOLT, at(4, 0));
    assert!(
        seen.contains(&Event::Cast {
            who: me,
            spell: BOLT,
            at: at(2, 0)
        }),
        "told where it stopped"
    );
    assert!(seen.contains(&Event::Scorched {
        who: me,
        target: near,
        damage: 500
    }));
    assert_eq!(w.actor(near).unwrap().body.health, full - 500);
    assert_eq!(
        w.actor(far).unwrap().body.health,
        full,
        "the far one untouched"
    );
    assert_eq!(w.mana(me), 28_000, "two points");
    // Aimed short of one, it flies only to its spot: nobody burns.
    let (mut w, me) = field(12);
    w.spawn(ActorKind::Creature(0), at(4, 0)).unwrap();
    let seen = cast(&mut w, me, BOLT, at(3, 0));
    assert!(seen.contains(&Event::Cast {
        who: me,
        spell: BOLT,
        at: at(3, 0)
    }));
    assert!(!seen.iter().any(|e| matches!(e, Event::Scorched { .. })));
}

#[test]
fn a_fire_nova_burns_everyone_beside_you_and_throws_them_back() {
    let (mut w, me) = field(12);
    let east = w.spawn(ActorKind::Creature(0), at(1, 0)).unwrap();
    let corner = w.spawn(ActorKind::Creature(0), at(-1, 1)).unwrap();
    // Three tiles north: out of its reach, and out of the way of the two thrown (one coming at
    // you holds the tile it's stepping onto, and nobody's thrown onto a held tile).
    let off = w.spawn(ActorKind::Creature(0), at(0, -3)).unwrap();
    let full = w.actor(east).unwrap().body.health;
    // Looked at as the words are said, before the burned ones come back at you.
    let (_, seen) = said(&mut w, me, NOVA, ME);
    for (who, to) in [(east, at(2, 0)), (corner, at(-2, 2))] {
        let a = w.actor(who).unwrap();
        assert_eq!(a.body.health, full - 100, "burned");
        assert_eq!(a.pos, to, "thrown back a tile, away from you");
        assert!(seen.contains(&Event::DrivenBack {
            who: me,
            target: who
        }));
    }
    assert_eq!(
        w.actor(off).unwrap().body.health,
        full,
        "three tiles off, untouched"
    );
    assert_eq!(
        w.actor(me).unwrap().body.health,
        w.rules.combat.character_health,
        "not you"
    );
    assert!(!w.spell_catches(me, NOVA, ME), "it never catches you");
}

#[test]
fn mana_comes_back_by_the_second_and_faster_with_arcana() {
    let (mut w, me) = field(12);
    // The test wizard's mana comes back a point a second.
    let mut defs = (*w.defs).clone();
    defs.classes[0].mana_regen = 1000;
    w.defs = Arc::new(defs);
    assert_eq!(w.mana_regen(me), 1000);
    cast(&mut w, me, FIREBALL, at(4, 3));
    let spent = w.mana(me);
    assert!(spent < 20_000, "{spent}");
    run(&mut w, 10);
    assert_eq!(w.mana(me), spent + 1000, "a point in a second");
    run(&mut w, 400);
    assert_eq!(w.mana(me), 30_000, "and never past the most");
    // Arcana: a twentieth faster a level.
    w.give_xp(me, Skill::Arcana, 1_000_000);
    let level = w.skill_level(me, Skill::Arcana);
    assert!(level > 0);
    assert_eq!(
        w.mana_regen(me),
        1000 * (1000 + 50 * i32::from(level)) / 1000
    );
    // A fighter holds none, and none comes back.
    let (mut w, me) = field(12);
    assert!(w.choose_class(me, 1));
    assert_eq!((w.max_mana(me), w.mana_regen(me)), (0, 0));
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

/// Casts `spell` on `to`, stepping until its words are said; the tick they were said on, and what
/// happened.
fn said(w: &mut World, me: ActorId, spell: u16, to: TilePos) -> (u64, Vec<Event>) {
    w.intent(me, Intent::Cast { spell, at: to });
    for _ in 0..80 {
        w.step(1);
        let seen: Vec<Event> = w.events().collect();
        if seen.iter().any(|e| matches!(e, Event::Cast { .. })) {
            // The tick moves on as a step begins: the words were said on this one.
            return (w.tick, seen);
        }
    }
    panic!("the words were never said");
}

#[test]
fn arcana_makes_a_fireball_harm_more_and_a_spell_last_longer() {
    let (mut w, me) = field(0);
    // No Arcana: the spells' own numbers.
    assert_eq!(w.spell_harm(me, FIREBALL), (1000, 1000));
    assert_eq!(w.spell_lasts(me, SILENCE), 100);
    w.give_xp(me, Skill::Arcana, 1_000_000);
    let level = w.skill_level(me, Skill::Arcana);
    assert!(level > 0);
    // The test rules: a twentieth more harm a level, a tenth longer.
    let harm = 1000 * (1000 + 50 * i32::from(level)) / 1000;
    let lasts = 100 * (1000 + 100 * u32::from(level)) / 1000;
    assert_eq!(w.spell_harm(me, FIREBALL), (harm, harm));
    assert_eq!(w.spell_lasts(me, SILENCE), lasts);
    assert_eq!(w.spell_lasts(me, LIGHT), lasts);
    assert_eq!(
        w.spell_lasts(me, FIREBALL),
        15,
        "a fireball's flash is a moment"
    );
    // Cast, a silence lasts that long...
    let (now, _) = said(&mut w, me, SILENCE, at(-5, 0));
    assert_eq!(w.zones()[0].until, now + u64::from(lasts));
    // ...and a fireball does that harm.
    let z = w.spawn(ActorKind::Creature(0), at(5, 0)).unwrap();
    let (_, seen) = said(&mut w, me, FIREBALL, at(5, 0));
    assert!(
        seen.contains(&Event::Scorched {
            who: me,
            target: z,
            damage: harm
        }),
        "{seen:?}"
    );
}

#[test]
fn the_wizards_talents_add_mana_quicken_the_words_and_hush_them() {
    let (mut w, me) = field(12);
    w.gain_deeds(me, 100_000_000);
    for _ in 0..3 {
        let offer = w.talent_offer(me);
        assert!(w.choose_talent(me, offer[0]));
    }
    assert_eq!(w.max_mana(me), 40_000);
    assert_eq!(w.mana(me), 40_000, "the new mana comes at once");
    assert_eq!(w.cast_ticks(me, SILENCE), 5, "half the time");
    let seen = cast(&mut w, me, LIGHT, at(3, 0));
    assert!(
        !seen
            .iter()
            .any(|e| matches!(*e, Event::Sound { at: p, .. } if p == ME)),
        "the words aren't heard"
    );
}

#[test]
fn a_wizards_powers_are_its_spells() {
    let (mut w, me) = field(12);
    assert!(w.choose_class(me, 0));
    assert_eq!(
        w.powers(me),
        [LIGHT, SILENCE, FOG, FIREBALL, BOLT, NOVA].map(Power::Spell)
    );
}

#[test]
fn a_fireball_catches_its_caster_only_within_its_reach() {
    let (w, me) = field(12);
    // The fireball's reach is 2 tiles: on yourself, beside you and two off, it catches you.
    assert!(w.spell_catches(me, FIREBALL, ME));
    assert!(w.spell_catches(me, FIREBALL, at(2, 0)));
    assert!(
        !w.spell_catches(me, FIREBALL, at(3, 0)),
        "three tiles off, it doesn't"
    );
    assert!(
        !w.spell_catches(me, FIREBALL, at(2, 2)),
        "the reach is round: (2, 2) is 2.8 off"
    );
    // Nothing else harms its caster.
    assert!(!w.spell_catches(me, LIGHT, ME));
    assert!(!w.spell_catches(me, FOG, ME));
}

#[test]
fn a_spell_cast_out_of_reach_walks_you_into_reach_then_casts() {
    let (mut w, me) = field(12);
    // Light reaches 8 tiles: cast on a spot 12 off, you walk until it's in reach, then cast.
    let spot = at(12, 0);
    assert_eq!(w.can_cast(me, LIGHT, spot), Err(NoCast::TooFar));
    w.intent(
        me,
        Intent::Cast {
            spell: LIGHT,
            at: spot,
        },
    );
    let mut from = None;
    for _ in 0..200 {
        w.step(1);
        if w.events().any(|e| matches!(e, Event::Cast { .. })) {
            from = Some(w.actor(me).unwrap().pos);
            break;
        }
    }
    assert_eq!(from, Some(at(4, 0)), "cast from the first tile in reach");
    // A spot no way leads to, walled round: given up, never cast.
    let (mut w, me) = field(12);
    for d in [Dir8::N, Dir8::E, Dir8::S, Dir8::W] {
        w.grid.set_edge(at(11, 0), d, Edge::Wall);
    }
    w.intent(
        me,
        Intent::Cast {
            spell: LIGHT,
            at: at(11, 0),
        },
    );
    let seen = run(&mut w, 100);
    assert!(!seen.iter().any(|e| matches!(e, Event::Cast { .. })));
    assert_eq!(w.actor(me).unwrap().intent, Intent::Idle, "given up");
    // Not mana enough: refused where you stand, no walk.
    let (mut w, me) = field(12);
    cast(&mut w, me, FIREBALL, at(4, 3));
    w.intent(
        me,
        Intent::Cast {
            spell: FIREBALL,
            at: at(12, 0),
        },
    );
    run(&mut w, 30);
    assert_eq!(w.actor(me).unwrap().pos, ME);
}
