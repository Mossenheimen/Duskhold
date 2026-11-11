//! Magic items: properties on a weapon or a light, curses that cling, learning by use or at a
//! holdout's counter, and what a container's loot rolls.

use std::sync::Arc;

use dusk_core::magic::NONE;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Event, GetUp, HANDS, Holdout, Intent, Item, ItemDef, ItemKind,
    LootEntry, LootTable, Magic, NoDeal, Part, Property, Tile, TilePos, World, WoundKind,
};

const COIN: u16 = 0;
const CUDGEL: u16 = 1;
const CANDLE: u16 = 2;
const KNIFE: u16 = 3;
// Properties.
const MUFFLED: u8 = 0;
const KEEN: u8 = 1;
const HALLOWED: u8 = 2;
const BRIGHT: u8 = 3;
const CLAMOROUS: u8 = 4;
/// The counter is fixture 1, north of where you start; fixture 2 is a chest of knives.
const COUNTER: TilePos = TilePos::new(0, -1, 0);
const CHEST: TilePos = TilePos::new(1, 0, 0);

fn magic(props: [u8; 2], curse: u8) -> Magic {
    Magic {
        rarity: if props[1] == NONE { 1 } else { 2 },
        props,
        curse,
        known: false,
        curse_shown: false,
        uses: 0,
    }
}

fn defs(kit: Vec<Item>) -> Defs {
    let item = |id: &str, stack: u16, kind: ItemKind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 10,
        stack,
        kind,
        worth: 4000,
    };
    let club = Swing {
        wind_up: 4,
        recovery: 4,
        endurance: 0,
        hit: 10_000,
        damage: (100, 100),
        noise: 10,
    };
    let weapon = ItemKind::Weapon {
        swing: club,
        wear: 10,
        skill: None,
    };
    let prop = |id: &str| Property {
        id: id.into(),
        name: id.into(),
        line: id.into(),
        ..Property::default()
    };
    Defs {
        items: vec![
            item("coins", 100, ItemKind::Other),
            item("cudgel", 1, weapon.clone()),
            item(
                "candle",
                10,
                ItemKind::Light {
                    radius: 2,
                    wane: 10,
                },
            ),
            item("knife", 1, weapon),
        ],
        properties: vec![
            Property {
                noise: 300,
                ..prop("Muffled")
            },
            Property {
                damage: 1500,
                ..prop("Keen")
            },
            Property {
                hallowed: true,
                ..prop("Hallowed")
            },
            Property {
                glow: 3,
                wane: 500,
                on_light: true,
                ..prop("Bright")
            },
            Property {
                noise: 2000,
                curse: true,
                ..prop("Clamorous")
            },
        ],
        loot: vec![LootTable {
            id: "chest".into(),
            rolls: (1, 1),
            entries: vec![LootEntry {
                item: KNIFE,
                weight: 1,
                count: (1, 1),
                magic: 10_000,
            }],
        }],
        containers: vec![None, None, Some(0)],
        holdouts: vec![Holdout {
            id: "abbey".into(),
            name: "the abbey".into(),
            sells: Vec::new(),
            buys: 500,
            repair: 1000,
            wants: Vec::new(),
            supply: 0,
            time: 10,
            bounty: dusk_core::Bounty::default(),
            identify: 5,
            uncurse: 20,
        }],
        counters: vec![None, Some(0), None],
        coin: Some(COIN),
        kit,
        ..Defs::default()
    }
}

/// A field, the counter north of you and a chest east, the needs stilled, no grabs.
fn field(kit: Vec<Item>) -> (World, ActorId) {
    let mut w = World::new(12);
    w.defs = Arc::new(defs(kit));
    w.creature_mut(0).wander = 0;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    for y in -20..20 {
        for x in -20..20 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    for (at, fixture) in [(COUNTER, 1), (CHEST, 2)] {
        w.grid.set_tile(
            at,
            Tile {
                fixture,
                solid: true,
                ..Tile::GRASS
            },
        );
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

fn slot(w: &World, me: ActorId, def: u16) -> u16 {
    let items = &w.actor(me).unwrap().items;
    items.iter().position(|i| i.def == def).unwrap() as u16
}

fn wield(w: &mut World, me: ActorId, def: u16) -> Vec<Event> {
    let s = slot(w, me, def);
    w.intent(me, Intent::Wield { slot: s, def });
    run(w, 10)
}

#[test]
fn a_weapons_properties_work_in_hand_known_or_not() {
    let plain = Item::new(CUDGEL, 1);
    let quiet = Item {
        magic: magic([MUFFLED, KEEN], NONE),
        ..Item::new(KNIFE, 1)
    };
    let (mut w, me) = field(vec![plain, quiet]);
    wield(&mut w, me, CUDGEL);
    let before = w.actor(me).unwrap().hands.unwrap();
    assert_eq!(before.def, CUDGEL);
    // Swing the plain cudgel, then the knife: hear the difference, feel it.
    let swing = |w: &mut World| {
        w.spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0));
        w.intent(
            me,
            Intent::Attack {
                dir: Some(dusk_core::Dir8::S),
            },
        );
        let seen = run(w, 12);
        let noise = seen.iter().find_map(|e| match *e {
            Event::Sound {
                by: Some(b),
                loudness,
                ..
            } if b == me => Some(loudness),
            _ => None,
        });
        let harm = seen.iter().find_map(|e| match *e {
            Event::Struck { who, damage, .. } if who == me => Some(damage),
            _ => None,
        });
        (noise.unwrap(), harm.unwrap())
    };
    let (loud, hard) = swing(&mut w);
    wield(&mut w, me, KNIFE);
    // The first shambler was put out of the way: its tile is the second's.
    let (soft, harder) = {
        let first = w.actors().find(|(_, a)| !a.kind.is_character()).unwrap().0;
        while !w.actor(first).unwrap().dead {
            w.wound(first, WoundKind::Bite, Part::Torso);
        }
        swing(&mut w)
    };
    assert_eq!((loud, soft), (10, 3), "Muffled: a third the noise");
    assert_eq!((hard, harder), (100, 150), "Keen: half as hard again");
    assert!(
        !w.actor(me).unwrap().hands.unwrap().magic.known,
        "and still unknown"
    );
}

#[test]
fn enough_blows_on_a_foe_teach_you_what_it_is() {
    let knife = Item {
        magic: magic([MUFFLED, NONE], NONE),
        ..Item::new(KNIFE, 1)
    };
    let (mut w, me) = field(vec![knife]);
    wield(&mut w, me, KNIFE);
    let held = w.actor(me).unwrap().hands.unwrap();
    assert_eq!(w.defs.label(&held), "knife (magic, unknown)");
    // Blows into the air teach nothing: a shambler to learn on, too tough to fall.
    w.creature_mut(0).health = 1_000_000;
    w.spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    // Swing until 15 have landed (its blows break off a wind-up now and then).
    let mut seen = Vec::new();
    let landed = |seen: &[Event]| {
        let mine = |e: &&Event| matches!(e, Event::Struck { who, .. } if *who == me);
        seen.iter().filter(mine).count()
    };
    while landed(&seen) < 15 {
        assert!(
            !seen.contains(&Event::Identified {
                who: me,
                item: KNIFE
            }),
            "not before 15"
        );
        w.intent(
            me,
            Intent::Attack {
                dir: Some(dusk_core::Dir8::S),
            },
        );
        seen.extend(run(&mut w, 10));
    }
    assert!(seen.contains(&Event::Identified {
        who: me,
        item: KNIFE
    }));
    let held = w.actor(me).unwrap().hands.unwrap();
    assert_eq!(w.defs.label(&held), "Muffled knife");
    assert_eq!(w.defs.lines(&held), vec!["Uncommon", "Muffled: Muffled"]);
}

#[test]
fn a_cursed_weapon_clings_until_the_abbey_lifts_it() {
    let cursed = Item {
        magic: magic([KEEN, NONE], CLAMOROUS),
        ..Item::new(KNIFE, 1)
    };
    let (mut w, me) = field(vec![cursed, Item::new(CUDGEL, 1), Item::new(COIN, 40)]);
    let seen = wield(&mut w, me, KNIFE);
    assert!(seen.contains(&Event::CurseClings {
        who: me,
        item: KNIFE
    }));
    assert!(w.hand_clings(me));
    // It won't be put away, nor changed for the cudgel.
    w.intent(me, Intent::Unwield);
    run(&mut w, 10);
    wield(&mut w, me, CUDGEL);
    assert_eq!(w.actor(me).unwrap().hands.unwrap().def, KNIFE);
    // Its curse shows in its name now; the rest is still unknown.
    let held = w.actor(me).unwrap().hands.unwrap();
    assert_eq!(w.defs.label(&held), "Clamorous knife (magic, unknown)");
    assert_eq!(w.can_uncurse(me, COUNTER), Ok(20));
    w.intent(me, Intent::Uncurse { at: COUNTER });
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::CurseLifted {
        who: me,
        item: KNIFE
    }));
    assert_eq!(w.coins(me), 20);
    w.intent(me, Intent::Unwield);
    run(&mut w, 10);
    assert!(w.actor(me).unwrap().hands.is_none(), "put away at last");
    assert!(
        w.actor(me)
            .unwrap()
            .items
            .iter()
            .any(|i| i.def == KNIFE && i.magic.props[0] == KEEN),
        "its magic kept"
    );
}

#[test]
fn the_abbey_reads_a_magic_item_for_coin() {
    let knife = Item {
        magic: magic([MUFFLED, KEEN], NONE),
        ..Item::new(KNIFE, 1)
    };
    let (mut w, me) = field(vec![knife, Item::new(COIN, 7)]);
    let s = slot(&w, me, KNIFE);
    assert_eq!(w.can_identify(me, COUNTER, s), Ok(5));
    assert_eq!(
        w.can_identify(me, COUNTER, HANDS),
        Err(NoDeal::NothingToRead)
    );
    w.intent(
        me,
        Intent::Identify {
            at: COUNTER,
            slot: s,
        },
    );
    let seen = run(&mut w, 20);
    assert!(seen.contains(&Event::Identified {
        who: me,
        item: KNIFE
    }));
    let knife = w.actor(me).unwrap().items[usize::from(slot(&w, me, KNIFE))];
    assert_eq!(w.defs.label(&knife), "Muffled Keen knife");
    assert_eq!(w.coins(me), 2);
    // Magic adds to its worth: a Rare one by the most.
    assert_eq!(w.defs.worth(&knife), 4000 + 50_000);
}

#[test]
fn what_a_hallowed_weapon_puts_down_stays_down() {
    let holy = Item {
        magic: magic([HALLOWED, NONE], NONE),
        ..Item::new(KNIFE, 1)
    };
    let (mut w, me) = field(vec![holy]);
    w.creature_mut(0).get_up = GetUp {
        chance: 10_000,
        after: (60, 60),
    };
    w.creature_mut(0).health = 100;
    wield(&mut w, me, KNIFE);
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    while !w.actor(z).unwrap().dead {
        w.intent(
            me,
            Intent::Attack {
                dir: Some(dusk_core::Dir8::S),
            },
        );
        run(&mut w, 10);
    }
    run(&mut w, 2 * 60 * 25);
    assert!(w.actor(z).unwrap().dead, "it never got up");
}

#[test]
fn a_bright_light_reaches_farther_and_burns_slower_and_you_know_it_once_lit() {
    let bright = Item {
        magic: magic([BRIGHT, NONE], NONE),
        ..Item::new(CANDLE, 1)
    };
    let (mut w, me) = field(vec![bright]);
    let s = slot(&w, me, CANDLE);
    w.intent(
        me,
        Intent::Light {
            slot: s,
            def: CANDLE,
        },
    );
    let seen = run(&mut w, 5);
    assert!(seen.contains(&Event::Identified {
        who: me,
        item: CANDLE
    }));
    let reach: Vec<i32> = w.carried_lights().map(|(_, r)| r).collect();
    assert_eq!(reach, vec![5], "a candle's 2, and 3 more");
    let before = w.actor(me).unwrap().light.unwrap().condition;
    run(&mut w, 10 * 25);
    let after = w.actor(me).unwrap().light.unwrap().condition;
    assert_eq!(before - after, 50, "ten minutes at half its 10 a minute");
}

#[test]
fn a_containers_magic_comes_from_the_seed_and_fits_its_kind() {
    let (w, _) = field(Vec::new());
    let first = w.store(CHEST);
    let (again, _) = field(Vec::new());
    assert_eq!(first, again.store(CHEST), "the same world, the same chest");
    let knife = first[0];
    assert!(knife.magic.is_magic() && !knife.magic.known);
    // A weapon's properties, never a light's.
    for p in knife.magic.all() {
        assert!(!w.defs.properties[usize::from(p)].on_light);
    }
}
