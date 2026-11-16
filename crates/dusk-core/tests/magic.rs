//! Magic items: properties on a weapon or a light, curses that cling, learning by use or at a
//! holdout's counter, and what a container's loot rolls; a weapon's quality (`quality.rs`), and
//! its card; the uniques, with their effects; Rares as Path of Exile's: tiers by how far out,
//! families, and made-up names; and the card as numbers, with what a weapon in hand gives
//! (Strength, Fitness, levels in skills) and its fire and holy harm; and armour as weapons are:
//! made well or badly, magic, known once it's worn; and bows and shields, each with its own.

use std::sync::Arc;

use dusk_core::archery::{NoShot, Ranged};
use dusk_core::hash::{Purpose, roll};
use dusk_core::magic::{NONE, UNIQUE};
use dusk_core::quality::{CRUDE, FINE, MASTERWORK, PLAIN};
use dusk_core::shield::Guard;
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Armour, CardLine, CardStyle, Defs, Event, Fits, GetUp, HANDS, Holdout,
    Intent, Item, ItemDef, ItemKind, LootEntry, LootTable, Magic, NoDeal, Part, Property, Raises,
    Remains, Skill, ThingNames, Tile, TilePos, Unique, World, WoundKind,
};

const COIN: u16 = 0;
const CUDGEL: u16 = 1;
const CANDLE: u16 = 2;
const KNIFE: u16 = 3;
const CAP: u16 = 4;
const ARROW: u16 = 5;
const BOW: u16 = 6;
const BUCKLER: u16 = 7;
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
        unique: NONE,
        props: [props[0], props[1], NONE, NONE],
        name: [NONE; 2],
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
            // On the head: turns four scratches in ten, a quarter of cuts; wears 3% a blow
            // turned; 5 warmth.
            item(
                "cap",
                1,
                ItemKind::Armour(Armour {
                    covers: 1,
                    turn: [4000, 2500, 1500],
                    wear: 30,
                    noise: 0,
                    endurance: 0,
                    sight: 1000,
                    time: 10,
                    warmth: 5000,
                }),
            ),
            item("arrow", 50, ItemKind::Other),
            // Shoots 6 tiles, sure up close, an arrow broken every other hit.
            item(
                "bow",
                1,
                ItemKind::Bow(Ranged {
                    swing: Swing { noise: 2, ..club },
                    range: 6,
                    ammo: ARROW,
                    breaks: 5000,
                }),
            ),
            // Blocks half the blows from ahead; 6 breath and 4% of it a block.
            item(
                "buckler",
                1,
                ItemKind::Shield(Guard {
                    block: 5000,
                    endurance: 60,
                    wear: 40,
                    bash: 1500,
                }),
            ),
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
                fits: Fits::Light,
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
            level: 0,
            entries: vec![LootEntry {
                item: KNIFE,
                weight: 1,
                count: (1, 1),
                magic: 10_000,
                unique: 0,
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
    field_of(12, kit)
}

/// The same field, from another seed.
fn field_of(seed: u64, kit: Vec<Item>) -> (World, ActorId) {
    let mut w = World::new(seed);
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
    // It sees all round: struck from behind unawares, a blow would be a sneak attack
    // (`stealth.rs`), and this is about the knife.
    w.creature_mut(0).cone = dusk_core::senses::Cone { cos: -10_000 };
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
    assert_eq!(w.defs.label(&held), "knife (uncommon, unknown)");
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
    // Its card says what it does, as numbers: a third of the noise.
    let card = w.card(&held, None);
    assert_eq!(styled(&card, CardStyle::Mod), [("-70% noise", "Muffled")]);
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
    assert_eq!(w.defs.label(&held), "Clamorous knife (uncommon, unknown)");
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
        assert_eq!(w.defs.properties[usize::from(p)].fits, Fits::Weapon);
    }
}

fn made(def: u16, quality: u8) -> Item {
    Item {
        quality,
        ..Item::new(def, 1)
    }
}

#[test]
fn quality_is_rolled_by_its_chances_and_most_are_plain() {
    let rules = dusk_core::QualityRules::default();
    let mut count = [0u32; 4];
    for k in 0..10_000 {
        count[usize::from(rules.roll(roll(7, 0, k, Purpose::Quality)))] += 1;
    }
    // Out of 10,000: crude 2,500, fine 1,500, masterwork 500, plain the 5,500 left.
    let near = |n: u32, want: u32| n.abs_diff(want) < 200;
    assert!(near(count[usize::from(CRUDE)], 2500), "{count:?}");
    assert!(near(count[usize::from(PLAIN)], 5500), "{count:?}");
    assert!(near(count[usize::from(FINE)], 1500), "{count:?}");
    assert!(near(count[usize::from(MASTERWORK)], 500), "{count:?}");
    // With no chances, everything is plain.
    let none = dusk_core::QualityRules {
        chance: [0; 4],
        ..dusk_core::QualityRules::default()
    };
    assert!((0..500).all(|k| none.roll(roll(7, 0, k, Purpose::Quality)) == PLAIN));
}

#[test]
fn quality_changes_a_blows_harm_its_wear_and_worth_and_goes_before_the_name() {
    let d = defs(Vec::new());
    // The cudgel: 100 harm, 10 condition off a blow, worth 4,000.
    let card = |q: u8| d.card(&made(CUDGEL, q)).unwrap();
    assert_eq!(card(PLAIN).swing.damage, (100, 100));
    assert_eq!(card(PLAIN).wear, 10);
    assert_eq!(card(CRUDE).swing.damage, (80, 80));
    assert_eq!(card(CRUDE).wear, 15, "crude wears half as fast again");
    assert_eq!(card(MASTERWORK).swing.damage, (125, 125));
    assert_eq!(card(MASTERWORK).wear, 6);
    assert_eq!(d.worth(&made(CUDGEL, CRUDE)), 2000);
    assert_eq!(d.worth(&made(CUDGEL, MASTERWORK)), 12_000);
    assert_eq!(d.label(&made(CUDGEL, PLAIN)), "cudgel");
    assert_eq!(d.label(&made(CUDGEL, FINE)), "fine cudgel");
    // Seen at a glance, as magic isn't: "a fine knife", magic unknown; then known, its words first.
    let mut keen = Item {
        magic: magic([KEEN, NONE], NONE),
        ..made(KNIFE, FINE)
    };
    assert_eq!(d.label(&keen), "fine knife (uncommon, unknown)");
    keen.magic.known = true;
    assert_eq!(d.label(&keen), "Keen fine knife");
    // Not a weapon, no card.
    assert_eq!(d.card(&Item::new(CANDLE, 1)), None);
}

#[test]
fn a_cards_magic_is_only_what_you_know_but_a_blow_has_all_of_it() {
    let keen = Item {
        magic: magic([KEEN, NONE], NONE),
        ..made(KNIFE, MASTERWORK)
    };
    let (mut w, me) = field(vec![keen]);
    // On its card, unknown: as made, masterwork's harm, and no more.
    assert_eq!(w.defs.card(&keen).unwrap().swing.damage, (125, 125));
    let mut known = keen;
    known.magic.known = true;
    assert_eq!(
        w.defs.card(&known).unwrap().swing.damage,
        (187, 187),
        "Keen on top: 125 half as hard again"
    );
    // In hand it strikes with all of it, known or not.
    wield(&mut w, me, KNIFE);
    w.creature_mut(0).cone = dusk_core::senses::Cone { cos: -10_000 };
    w.spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0));
    w.intent(
        me,
        Intent::Attack {
            dir: Some(dusk_core::Dir8::S),
        },
    );
    let harm = run(&mut w, 12).iter().find_map(|e| match *e {
        Event::Struck { who, damage, .. } if who == me => Some(damage),
        _ => None,
    });
    assert_eq!(harm, Some(187));
    // A curse that's shown itself is on the card too: Clamorous, twice the noise.
    let loud = Item {
        magic: Magic {
            curse_shown: true,
            ..magic([MUFFLED, NONE], CLAMOROUS)
        },
        ..made(KNIFE, PLAIN)
    };
    assert_eq!(w.defs.card(&loud).unwrap().swing.noise, 20);
}

#[test]
fn a_chests_weapons_are_made_well_or_badly_by_the_seed() {
    let mut count = [0u32; 4];
    for seed in 0..120 {
        let (w, _) = field_of(seed, Vec::new());
        let (again, _) = field_of(seed, Vec::new());
        let knife = w.store(CHEST)[0];
        assert_eq!(knife.quality, again.store(CHEST)[0].quality);
        count[usize::from(knife.quality)] += 1;
    }
    assert!(
        count.iter().all(|&n| n > 0),
        "every grade turns up: {count:?}"
    );
    assert!(
        count[usize::from(PLAIN)] > count[usize::from(CRUDE)],
        "most are plain: {count:?}"
    );
}

/// The test's uniques, both made on the knife: Redhand, fine and Keen, that drinks a quarter of
/// the harm it does; and Everything, plain, with every other effect.
fn uniques() -> Vec<Unique> {
    let effect = |name: &str| Property {
        id: name.into(),
        name: name.into(),
        line: name.to_lowercase(),
        ..Property::default()
    };
    vec![
        Unique {
            id: "redhand".into(),
            name: "Redhand".into(),
            item: KNIFE,
            quality: FINE,
            props: vec![KEEN],
            effect: Property {
                drain: 250,
                ..effect("Thirst")
            },
            lore: "It drinks.".into(),
        },
        Unique {
            id: "everything".into(),
            name: "Everything".into(),
            item: KNIFE,
            quality: PLAIN,
            props: Vec::new(),
            effect: Property {
                cleave: true,
                pyre: true,
                terror: 10_000,
                fell: 10_000,
                hearten: 10_000,
                ..effect("All")
            },
            lore: String::new(),
        },
    ]
}

/// One of the uniques (by number) as you'd carry it, known or not.
fn unique(u: u8, known: bool) -> Item {
    Item {
        quality: if u == 0 { FINE } else { PLAIN },
        magic: Magic {
            rarity: UNIQUE,
            unique: u,
            known,
            curse_shown: known,
            ..Magic::MUNDANE
        },
        ..Item::new(KNIFE, 1)
    }
}

/// The field from a seed, with the uniques, and the chest's line as `chest` makes it.
fn unique_field(seed: u64, kit: Vec<Item>, chest: impl Fn(&mut LootEntry)) -> (World, ActorId) {
    let (mut w, me) = field_of(seed, kit);
    let mut d = (*w.defs).clone();
    d.uniques = uniques();
    chest(&mut d.loot[0].entries[0]);
    w.defs = Arc::new(d);
    // It sees all round: no blow here is a sneak attack (`stealth.rs`).
    w.creature_mut(0).cone = dusk_core::senses::Cone { cos: -10_000 };
    (w, me)
}

/// Swings south until a blow of yours lands: what happened meanwhile.
fn land_a_blow(w: &mut World, me: ActorId) -> Vec<Event> {
    let mut seen = Vec::new();
    while !seen
        .iter()
        .any(|e| matches!(e, Event::Struck { who, .. } if *who == me))
    {
        w.intent(
            me,
            Intent::Attack {
                dir: Some(dusk_core::Dir8::S),
            },
        );
        seen.extend(run(w, 10));
    }
    seen
}

#[test]
fn a_unique_is_named_once_known_and_worth_the_most() {
    let (w, _) = unique_field(12, Vec::new(), |_| {});
    let d = &w.defs;
    let found = d.unique_item(0).unwrap();
    assert!(found.found && !found.magic.known);
    assert_eq!(
        (found.def, found.quality),
        (KNIFE, FINE),
        "made as its data says"
    );
    assert_eq!(d.label(&found), "fine knife (unique, unknown)");
    // Its card says it's a unique, and not what it does.
    let card = w.card(&found, None);
    assert_eq!(
        styled(&card, CardStyle::Kind),
        [("Weapon \u{b7} Unique", "")]
    );
    assert!(styled(&card, CardStyle::Effect).is_empty());
    let note = styled(&card, CardStyle::Note);
    assert!(note[0].0.starts_with("Not known yet"), "{note:?}");
    // Unknown, its card shows how well it was made, and none of its magic.
    assert_eq!(d.card(&found).unwrap().swing.damage, (110, 110));
    let known = Item {
        magic: Magic {
            known: true,
            curse_shown: true,
            ..found.magic
        },
        ..found
    };
    assert_eq!(d.label(&known), "Redhand (fine knife)");
    // Known: its own name, its kind under it; how well it was made and its property, as
    // numbers; its effect in words, and its story.
    let card = w.card(&known, None);
    assert_eq!(styled(&card, CardStyle::Title), [("Redhand", "")]);
    assert_eq!(
        styled(&card, CardStyle::Kind),
        [("Fine knife", ""), ("Weapon \u{b7} Unique", "")]
    );
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [
            ("+10% damage", "Fine"),
            ("-15% wear", "Fine"),
            ("+50% damage", "Keen")
        ]
    );
    assert_eq!(styled(&card, CardStyle::Effect), [("Thirst: thirst", "")]);
    assert_eq!(styled(&card, CardStyle::Lore), [("\"It drinks.\"", "")]);
    assert_eq!(
        d.card(&known).unwrap().swing.damage,
        (165, 165),
        "fine, and Keen"
    );
    // The knife's worth as fine, and a unique's on top: more than a Rare's.
    assert_eq!(d.worth(&known), 4000 * 1500 / 1000 + 150_000);
}

#[test]
fn a_tables_unique_chance_gives_one_made_on_what_it_drew() {
    let always = |e: &mut LootEntry| {
        e.unique = 10_000;
        e.magic = 0;
    };
    // Both are made on the knife: from place to place (here, seed to seed), the chest gives each.
    let mut seen = [false; 2];
    for seed in 0..40 {
        let (w, _) = unique_field(seed, Vec::new(), always);
        let got = w.store(CHEST);
        assert_eq!(got.len(), 1);
        let u = got[0];
        assert_eq!(u.magic.rarity, UNIQUE);
        assert!(u.found && !u.magic.known, "a find, unknown");
        let made = &w.defs.uniques[usize::from(u.magic.unique)];
        assert_eq!((u.def, u.quality), (made.item, made.quality));
        seen[usize::from(u.magic.unique)] = true;
    }
    assert_eq!(seen, [true, true]);
    // None is made on the cudgel: it's only a cudgel.
    let (w, _) = unique_field(12, Vec::new(), |e| {
        always(e);
        e.item = CUDGEL;
    });
    assert_eq!(w.store(CHEST)[0].magic, Magic::MUNDANE);
}

#[test]
fn a_unique_that_drinks_gives_back_a_share_of_the_harm_it_does() {
    // The same fight twice: Redhand, unknown, and a Redhand that doesn't drink. What's between
    // your health in the two is what it drank.
    let fight = |drinks: bool| {
        let (mut w, me) = unique_field(12, vec![unique(0, false)], |_| {});
        if !drinks {
            let mut d = (*w.defs).clone();
            d.uniques[0].effect.drain = 0;
            w.defs = Arc::new(d);
        }
        // Hurt first, so there's health to give back; and a shambler too tough to fall.
        w.rules.combat.wounds[WoundKind::Scratch as usize].damage = 400;
        w.wound(me, WoundKind::Scratch, Part::Torso);
        w.creature_mut(0).health = 1_000_000;
        wield(&mut w, me, KNIFE);
        w.spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
            .unwrap();
        let mut harm = Vec::new();
        while harm.len() < 3 {
            harm.extend(land_a_blow(&mut w, me).iter().filter_map(|e| match *e {
                Event::Struck { who, damage, .. } if who == me => Some(damage),
                _ => None,
            }));
        }
        (w.actor(me).unwrap().body.health, harm)
    };
    let ((drank, harm), (dry, same)) = (fight(true), fight(false));
    assert_eq!(harm, same, "the same blows");
    assert!(harm.iter().all(|&h| h == 165), "fine and Keen: {harm:?}");
    assert_eq!(
        drank - dry,
        3 * (165 * 250 / 1000),
        "a quarter of each blow's harm"
    );
}

#[test]
fn a_uniques_blows_topple_and_frighten_and_its_kills_burn_ease_dread_and_carry_on() {
    // A tough one: struck, it's knocked down, and it breaks and flees.
    let (mut w, me) = unique_field(12, vec![unique(1, true)], |_| {});
    w.creature_mut(0).health = 1_000_000;
    wield(&mut w, me, KNIFE);
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    let seen = land_a_blow(&mut w, me);
    assert!(seen.contains(&Event::Felled { who: me, target: z }));
    assert!(seen.contains(&Event::Terrified { who: me, target: z }));
    assert!(w.is_turned(z), "it flees, as from the holy symbol");

    // Two weak ones, in the dark. Dread is let rise, then held: only a kill moves it now.
    let (mut w, me) = unique_field(12, vec![unique(1, true)], |_| {});
    w.rules.dread.dark_below = 101;
    let minute = w.rules.clock.ticks_per_minute;
    run(&mut w, 10 * minute);
    let before = w.dread(me);
    assert!(before >= 20_000, "{before}");
    (w.rules.dread.rise, w.rules.dread.fall) = (0, 0);
    // Nor does it shake your aim: every blow lands, the one carried on too.
    w.rules.dread.aim = [1000; 3];
    w.creature_mut(0).health = 1;
    w.creature_mut(0).get_up = GetUp {
        chance: 10_000,
        after: (60, 60),
    };
    wield(&mut w, me, KNIFE);
    let a = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    let b = w
        .spawn(ActorKind::Creature(0), TilePos::new(-1, 0, 0))
        .unwrap();
    let seen = land_a_blow(&mut w, me);
    // One blow put the first down, and carried on into the second.
    for z in [a, b] {
        assert!(w.actor(z).unwrap().dead);
        assert!(seen.contains(&Event::Kindled { who: me, body: z }));
        assert!(
            matches!(w.actor(z).unwrap().remains, Remains::Burning { .. }),
            "alight where it fell"
        );
    }
    assert_eq!(w.dread(me), before - 2 * 10_000, "ten points a kill");
}

/// The field from a seed, its chest's knife always magic (always Rare, or never), plainly made,
/// never cursed; a weapon's properties in families of three tiers (harm, pace, aim), one in a
/// family of one at the second tier (breath), and two with no tier; and words for Rares' names.
fn tiered(seed: u64, rare: bool) -> (World, ActorId) {
    let (mut w, me) = field_of(seed, Vec::new());
    let mut d = (*w.defs).clone();
    let prop = |id: &str, family: &str, tier: u8| Property {
        id: id.into(),
        name: id.into(),
        line: id.into(),
        family: family.into(),
        tier,
        ..Property::default()
    };
    d.properties = vec![
        prop("sharp", "harm", 1),
        prop("keen", "harm", 2),
        prop("cruel", "harm", 3),
        prop("quick", "pace", 1),
        prop("swift", "pace", 2),
        prop("darting", "pace", 3),
        prop("true", "aim", 1),
        prop("sure", "aim", 2),
        prop("unerring", "aim", 3),
        prop("muffled", "", 0),
        prop("blazing", "", 0),
        Property {
            endurance: 500,
            ..prop("poised", "breath", 2)
        },
    ];
    d.thing_names = ThingNames {
        first: vec!["Grim".into(), "Pale".into()],
        second: [
            vec!["Fang".into()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec!["Brand".into()],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ],
    };
    d.magic.rare = if rare { 10_000 } else { 0 };
    d.magic.cursed = 0;
    d.quality.chance = [0; 4];
    w.defs = Arc::new(d);
    (w, me)
}

#[test]
fn a_rare_has_three_or_four_properties_each_of_its_own_family_and_a_made_up_name() {
    let mut counts = [0; 5];
    for seed in 0..40 {
        let (w, _) = tiered(seed, true);
        let knife = w.store(CHEST)[0];
        assert_eq!(knife.magic.rarity, 2);
        let props: Vec<&Property> = w.defs.properties_of(&knife).collect();
        counts[props.len()] += 1;
        let mut families: Vec<&str> = props.iter().map(|p| p.family_of()).collect();
        families.sort_unstable();
        families.dedup();
        assert_eq!(families.len(), props.len(), "at most one of a family");
        // The knife needs no skill: its second word is any other weapon's.
        let name = w.defs.thing_name(&knife).expect("a made-up name");
        assert!(name == "Grim Brand" || name == "Pale Brand", "{name}");
        // Unknown, it says how rare it is; known, it's called by its name.
        assert_eq!(w.defs.label(&knife), "knife (rare, unknown)");
        let known = Item {
            magic: Magic {
                known: true,
                ..knife.magic
            },
            ..knife
        };
        assert_eq!(w.defs.label(&known), format!("{name} (knife)"));
    }
    assert!(counts[3] > 0 && counts[4] > 0, "{counts:?}");
    assert_eq!(counts[0] + counts[1] + counts[2], 0, "{counts:?}");
}

#[test]
fn how_far_out_a_find_was_made_sets_the_best_tier_it_rolls() {
    // Where you come in is `far` tiles west of the chest; and a table may reach `more` further.
    let best = |far: i32, more: u8| {
        let mut best = 0;
        for seed in 0..40 {
            let (mut w, _) = tiered(seed, true);
            w.region.start = Some(TilePos::new(1 - far, 0, 0));
            let mut d = (*w.defs).clone();
            d.loot[0].level = more;
            w.defs = Arc::new(d);
            let knife = w.store(CHEST)[0];
            let tier = w.defs.properties_of(&knife).map(|p| p.tier).max();
            best = best.max(tier.unwrap());
        }
        best
    };
    assert_eq!(best(0, 0), 1, "where you come in, only the first tier");
    assert_eq!(best(100, 0), 2, "out about the hamlet, the second");
    assert_eq!(best(200, 0), 3, "far out, the third");
    assert_eq!(
        best(0, 1),
        2,
        "a rare's pack, or a tomb, reaches one further"
    );
    let (w, _) = tiered(12, true);
    assert_eq!(
        w.find_level(TilePos::new(500, 0, 0), 2),
        3,
        "never past the third"
    );
}

#[test]
fn an_uncommon_has_one_property_or_two_and_its_lines_give_each_ones_tier() {
    let mut counts = [0; 5];
    for seed in 0..40 {
        let (w, _) = tiered(seed, false);
        let knife = w.store(CHEST)[0];
        assert_eq!(knife.magic.rarity, 1);
        assert_eq!(knife.magic.name, [NONE; 2], "no made-up name");
        assert_eq!(w.defs.label(&knife), "knife (uncommon, unknown)");
        counts[w.defs.properties_of(&knife).count()] += 1;
    }
    assert!(counts[1] > 0 && counts[2] > 0, "{counts:?}");
    assert_eq!(counts[0] + counts[3] + counts[4], 0, "{counts:?}");
    // Known, it's called by its properties; a tier shows after the name, and none for no tier.
    let (w, _) = tiered(12, false);
    let d = &w.defs;
    let known = Item {
        magic: Magic {
            rarity: 1,
            props: [1, 9, NONE, NONE],
            known: true,
            ..Magic::MUNDANE
        },
        ..Item::new(KNIFE, 1)
    };
    assert_eq!(d.label(&known), "keen muffled knife");
    // On its card, each in its words (these have no numbers), its tier beside its name.
    let card = w.card(&known, None);
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [("Keen", "keen II"), ("Muffled", "muffled")]
    );
    // Poised: half the breath a blow.
    let poised = Item {
        magic: Magic {
            props: [11, NONE, NONE, NONE],
            ..known.magic
        },
        ..known
    };
    let swing = d.card(&poised).unwrap().swing;
    let blow = Swing {
        endurance: 100,
        ..swing
    };
    assert_eq!(d.made_swing(&poised, blow, false).endurance, 50);
}

/// A card's lines of one style: their words, and what's to their right.
fn styled(card: &[CardLine], style: CardStyle) -> Vec<(&str, &str)> {
    card.iter()
        .filter(|l| l.style == style)
        .map(|l| (l.text.as_str(), l.value.as_str()))
        .collect()
}

/// One of a card's numbers, by its name: its value, and whether it's marked as changed.
fn number<'a>(card: &'a [CardLine], name: &str) -> (&'a str, bool) {
    let l = card
        .iter()
        .find(|l| l.text == name)
        .unwrap_or_else(|| panic!("no {name} on {card:?}"));
    (l.value.as_str(), l.style == CardStyle::Changed)
}

#[test]
fn a_weapons_card_gives_its_numbers_marked_where_its_making_or_magic_changed_them() {
    // The cudgel: 10 harm, a blow every 0.8 s that always lands, 10 tiles of noise, 1% wear.
    let (mut w, me) = field(vec![Item::new(CUDGEL, 1)]);
    let plain = w.card(&Item::new(CUDGEL, 1), None);
    assert_eq!(styled(&plain, CardStyle::Title), [("Cudgel", "")]);
    assert_eq!(styled(&plain, CardStyle::Kind), [("Weapon", "")]);
    for (name, value) in [
        ("Damage per second", "12.5"),
        ("Damage", "10"),
        ("Attacks per second", "1.25"),
        ("Wind-up", "0.4 s"),
        ("Chance to hit", "100%"),
        ("Noise", "10 tiles"),
        ("Wear", "1% a blow"),
        ("Condition", "100%"),
    ] {
        assert_eq!(number(&plain, name), (value, false), "{name}");
    }
    assert!(styled(&plain, CardStyle::Mod).is_empty());
    // A masterwork knife, Keen: harder, and each says so as numbers, its word to the right.
    let mut keen = Item {
        magic: magic([KEEN, NONE], NONE),
        ..made(KNIFE, MASTERWORK)
    };
    keen.magic.known = true;
    let card = w.card(&keen, None);
    assert_eq!(number(&card, "Damage"), ("18.7", true));
    assert_eq!(number(&card, "Damage per second"), ("23.4", true));
    assert_eq!(number(&card, "Wear"), ("0.6% a blow", true));
    assert_eq!(number(&card, "Noise"), ("10 tiles", false));
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [
            ("+25% damage", "Masterwork"),
            ("-40% wear", "Masterwork"),
            ("+50% damage", "Keen")
        ]
    );
    // Unknown, its making shows and its magic doesn't; a curse that has shown itself does.
    let mut cursed = Item {
        magic: magic([KEEN, NONE], CLAMOROUS),
        ..made(KNIFE, PLAIN)
    };
    cursed.magic.curse_shown = true;
    let card = w.card(&cursed, None);
    assert_eq!(number(&card, "Damage"), ("10", false));
    assert_eq!(number(&card, "Noise"), ("20 tiles", true));
    assert!(styled(&card, CardStyle::Mod).is_empty());
    assert_eq!(
        styled(&card, CardStyle::Curse),
        [("+100% noise", "Clamorous (curse)")]
    );
    // In the pack, set against the cudgel in hand.
    wield(&mut w, me, CUDGEL);
    assert_eq!(
        styled(&w.card(&keen, Some(me)), CardStyle::Note),
        [("Against your cudgel: +10.9 damage per second", "")]
    );
    // A thing that's no weapon, armour or light, and not magic, has none.
    assert!(w.card(&Item::new(COIN, 5), None).is_empty());
}

/// The field, its knife training Blade, and these properties after the fixture's (the first's
/// number is returned).
fn field_with(props: Vec<Property>) -> (World, ActorId, u8) {
    let (mut w, me) = field(Vec::new());
    let mut d = (*w.defs).clone();
    if let ItemKind::Weapon { skill, .. } = &mut d.items[usize::from(KNIFE)].kind {
        *skill = Some(Skill::Blade);
    }
    let first = d.properties.len() as u8;
    d.properties.extend(props);
    w.defs = Arc::new(d);
    (w, me, first)
}

#[test]
fn a_weapon_in_hand_makes_you_stronger_fitter_and_better_at_skills_never_past_ten() {
    let prop = |name: &str| Property {
        id: name.to_lowercase(),
        name: name.into(),
        ..Property::default()
    };
    let (mut w, me, first) = field_with(vec![
        Property {
            strength: 2000,
            ..prop("Mighty")
        },
        Property {
            fitness: 1000,
            ..prop("Hale")
        },
        Property {
            raises: Raises::Own,
            levels: 2,
            ..prop("Seasoned")
        },
        Property {
            raises: Raises::Skill(Skill::Arcana),
            levels: 5,
            ..prop("Sorcerous")
        },
    ]);
    let knife = Item {
        magic: Magic {
            rarity: 2,
            props: [first, first + 1, first + 2, first + 3],
            ..magic([NONE, NONE], NONE)
        },
        ..Item::new(KNIFE, 1)
    };
    w.give(me, knife);
    let carried = w.carry(me);
    // Arcana learnt as far as it goes freely: 7.
    w.give_xp(me, Skill::Arcana, 10_000_000);
    assert_eq!(w.skill_level(me, Skill::Arcana), 7);
    wield(&mut w, me, KNIFE);
    // In hand, known or not: two levels of Strength (it carries more), one of Fitness, two of
    // Blade, its own skill; and Arcana never past 10.
    assert_eq!((w.strength(me), w.fitness(me)), (2000, 1000));
    assert_eq!(w.carry(me), carried + 2 * w.rules.strength.carry);
    assert_eq!(w.skill_level(me, Skill::Blade), 2);
    assert_eq!(w.skill_level(me, Skill::Arcana), 10);
    // What's learnt is still its own: the knife gives levels, never XP, nor a way past the cap.
    let (_, arcana) = w
        .skills(me)
        .into_iter()
        .find(|(s, _)| *s == Skill::Arcana)
        .unwrap();
    assert_eq!(arcana.level, 7);
    assert_eq!(w.actor(me).unwrap().strength, 0);
    // Its card says so, once it's known: its own skill by its name.
    let known = Item {
        magic: Magic {
            known: true,
            ..knife.magic
        },
        ..knife
    };
    assert_eq!(
        styled(&w.card(&known, None), CardStyle::Mod),
        [
            ("+2 to Strength", "Mighty"),
            ("+1 to Fitness", "Hale"),
            ("+2 to Blade", "Seasoned"),
            ("+5 to Arcana", "Sorcerous")
        ]
    );
    // Put away, it gives nothing.
    w.intent(me, Intent::Unwield);
    run(&mut w, 10);
    assert_eq!((w.strength(me), w.skill_level(me, Skill::Blade)), (0, 0));
}

#[test]
fn fire_and_holy_harm_land_on_top_of_a_blow_and_a_beast_takes_no_holy_harm() {
    let (mut w, me, first) = field_with(vec![
        Property {
            id: "burning".into(),
            name: "Burning".into(),
            fire: (20, 20),
            ..Property::default()
        },
        Property {
            id: "holy".into(),
            name: "Holy".into(),
            holy: (30, 30),
            ..Property::default()
        },
    ]);
    let knife = Item {
        magic: magic([first, first + 1], NONE),
        ..Item::new(KNIFE, 1)
    };
    w.give(me, knife);
    wield(&mut w, me, KNIFE);
    // It sees all round, so no blow is a sneak attack (`stealth.rs`), and it won't fall.
    w.creature_mut(0).cone = dusk_core::senses::Cone { cos: -10_000 };
    w.creature_mut(0).health = 1_000_000;
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    let harm = |seen: &[Event]| {
        seen.iter().find_map(|e| match *e {
            Event::Struck { who, damage, .. } if who == me => Some(damage),
            _ => None,
        })
    };
    let seen = land_a_blow(&mut w, me);
    assert_eq!(
        harm(&seen),
        Some(100 + 20 + 30),
        "the knife's 10, fire's 2, holy 3"
    );
    assert!(seen.contains(&Event::Seared {
        who: me,
        target: z,
        fire: 20,
        holy: 30
    }));
    // One of the living world (a wolf): the fire, and none of the holy harm.
    w.creature_mut(0).wild = true;
    let seen = land_a_blow(&mut w, me);
    assert_eq!(harm(&seen), Some(100 + 20));
    assert!(seen.contains(&Event::Seared {
        who: me,
        target: z,
        fire: 20,
        holy: 0
    }));
    // Its card, once known: lines of their own among its numbers, and as properties.
    let known = Item {
        magic: Magic {
            known: true,
            ..knife.magic
        },
        ..knife
    };
    let card = w.card(&known, None);
    assert_eq!(styled(&card, CardStyle::Fire), [("Fire damage", "2")]);
    assert_eq!(
        styled(&card, CardStyle::Holy),
        [("Holy damage", "3 on the dead")]
    );
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [
            ("Adds 2 fire damage", "Burning"),
            ("Adds 3 holy damage on the dead", "Holy")
        ]
    );
}

#[test]
fn a_piece_of_armour_is_made_well_or_badly_its_magic_turns_more_and_its_known_once_worn() {
    let worn = |name: &str| Property {
        id: name.to_lowercase(),
        name: name.into(),
        fits: Fits::Armour,
        ..Property::default()
    };
    let (mut w, me, first) = field_with(vec![
        Property {
            ward: 1000,
            ..worn("Warded")
        },
        Property {
            warmth: 6000,
            ..worn("Lined")
        },
        Property {
            strength: 1000,
            ..worn("Brawny")
        },
        Property {
            raises: Raises::Skill(Skill::Stealth),
            levels: 2,
            ..worn("Sly")
        },
        Property {
            steps: 2,
            curse: true,
            ..worn("Clanking")
        },
    ]);
    let cap = Item {
        quality: FINE,
        magic: Magic {
            rarity: 2,
            props: [first, first + 1, first + 2, first + 3],
            curse: first + 4,
            ..magic([NONE, NONE], NONE)
        },
        ..Item::new(CAP, 1)
    };
    // Unknown: how well it was made shows (fine: 5% more of every blow turned, slower wear), its
    // magic doesn't.
    let card = w.card(&cap, None);
    assert_eq!(number(&card, "Turns scratches"), ("45%", true));
    assert_eq!(number(&card, "Wear"), ("2.5% a blow turned", true));
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [("+5% chance to turn blows", "Fine"), ("-15% wear", "Fine")]
    );
    assert!(styled(&card, CardStyle::Curse).is_empty());
    let note = styled(&card, CardStyle::Note);
    assert!(
        note[0].0.starts_with("Not known yet: put it on"),
        "{note:?}"
    );
    // Put on, it shows what it does, its curse with it; and it works on you.
    w.give(me, cap);
    let s = slot(&w, me, CAP);
    w.intent(me, Intent::Wear { slot: s, def: CAP });
    let seen = run(&mut w, 30);
    assert!(seen.contains(&Event::Identified { who: me, item: CAP }));
    let piece = w.actor(me).unwrap().worn[0];
    assert!(piece.magic.known && piece.magic.curse_shown);
    assert_eq!(w.strength(me), 1000);
    assert_eq!(w.skill_level(me, Skill::Stealth), 2);
    let card = w.card(&piece, None);
    for (name, value) in [
        ("Turns scratches", "55%"),
        ("Turns lacerations", "40%"),
        ("Turns bites", "30%"),
        ("Warmth", "11"),
        ("Footsteps", "+2 tiles"),
    ] {
        assert_eq!(number(&card, name), (value, true), "{name}");
    }
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [
            ("+5% chance to turn blows", "Fine"),
            ("-15% wear", "Fine"),
            ("+10% chance to turn blows", "Warded"),
            ("+6 warmth", "Lined"),
            ("+1 to Strength", "Brawny"),
            ("+2 to Stealth", "Sly")
        ]
    );
    assert_eq!(
        styled(&card, CardStyle::Curse),
        [("+2 tiles to every footstep", "Clanking (curse)")]
    );
    // Its curse is a cost it carries: it doesn't cling. Taken off, it gives nothing.
    w.intent(me, Intent::TakeOff { piece: 0, def: CAP });
    run(&mut w, 30);
    assert!(w.actor(me).unwrap().worn.is_empty());
    assert_eq!((w.strength(me), w.skill_level(me, Skill::Stealth)), (0, 0));
}

#[test]
fn a_chests_armour_is_made_well_or_badly_and_its_magic_is_armours() {
    let mut fine = 0;
    for seed in 0..60 {
        let (mut w, _) = field_of(seed, Vec::new());
        let mut d = (*w.defs).clone();
        d.properties.push(Property {
            id: "warded".into(),
            name: "Warded".into(),
            fits: Fits::Armour,
            ward: 1000,
            ..Property::default()
        });
        d.loot[0].entries[0].item = CAP;
        d.magic.cursed = 0;
        w.defs = Arc::new(d);
        let cap = w.store(CHEST)[0];
        assert_eq!(cap.def, CAP);
        assert!(cap.magic.is_magic() && !cap.magic.known);
        for p in cap.magic.all() {
            assert_eq!(w.defs.properties[usize::from(p)].fits, Fits::Armour);
        }
        fine += usize::from(cap.quality != PLAIN);
    }
    assert!(fine > 0, "some made better or worse than plain");
}

/// A property for `fits`, by its name.
fn on(fits: Fits, name: &str) -> Property {
    Property {
        id: name.to_lowercase(),
        name: name.into(),
        fits,
        ..Property::default()
    }
}

#[test]
fn a_bow_is_made_well_or_badly_its_magic_shoots_farther_and_burns_and_shots_teach_it() {
    let bow = |name: &str| on(Fits::Bow, name);
    let (mut w, me, first) = field_with(vec![
        Property {
            range: 2,
            ..bow("Far")
        },
        Property {
            fire: (20, 20),
            ..bow("Kindled")
        },
        Property {
            raises: Raises::Own,
            levels: 1,
            ..bow("Fletcher's")
        },
        Property {
            arrows: 500,
            ..bow("Sparing")
        },
    ]);
    let mut d = (*w.defs).clone();
    d.magic.learn = 1;
    w.defs = Arc::new(d);
    let made = Item {
        quality: MASTERWORK,
        magic: Magic {
            rarity: 2,
            props: [first, first + 1, first + 2, first + 3],
            ..magic([NONE, NONE], NONE)
        },
        ..Item::new(BOW, 1)
    };
    // Unknown: how well it was made shows (masterwork: a quarter harder; a bow doesn't wear),
    // its magic doesn't.
    let card = w.card(&made, None);
    assert_eq!(number(&card, "Damage"), ("12.5", true));
    assert_eq!(number(&card, "Range"), ("6 tiles", false));
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [("+25% damage", "Masterwork")]
    );
    let note = styled(&card, CardStyle::Note);
    assert!(note[0].0.contains("land 1 shots"), "{note:?}");
    // In hand, known or not: two tiles farther, and a level of Archery, its own skill.
    w.give(me, made);
    w.give(me, Item::new(ARROW, 20));
    wield(&mut w, me, BOW);
    assert_eq!(w.skill_level(me, Skill::Archery), 1);
    assert_eq!(w.can_shoot(me, TilePos::new(0, 8, 0)), Ok(()));
    assert_eq!(w.can_shoot(me, TilePos::new(0, 9, 0)), Err(NoShot::TooFar));
    // An arrow that lands burns, and the first that lands teaches you what the bow is.
    let c = w.creature_mut(0);
    (c.health, c.attack.hit) = (1_000_000, 0);
    c.cone = dusk_core::senses::Cone { cos: -10_000 };
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 3, 0))
        .unwrap();
    let mut seen = Vec::new();
    for _ in 0..12 {
        let at = w.actor(z).unwrap().pos;
        w.intent(me, Intent::Shoot { at });
        seen.extend(run(&mut w, 30));
        if seen
            .iter()
            .any(|e| matches!(e, Event::Struck { who, .. } if *who == me))
        {
            break;
        }
    }
    assert!(seen.contains(&Event::Seared {
        who: me,
        target: z,
        fire: 20,
        holy: 0
    }));
    assert!(seen.contains(&Event::Identified { who: me, item: BOW }));
    // Its card now: farther, burning, fewer arrows broken, each marked; its properties as
    // numbers, in their order.
    let held = w.actor(me).unwrap().hands.unwrap();
    let card = w.card(&held, None);
    assert_eq!(number(&card, "Range"), ("8 tiles", true));
    assert_eq!(number(&card, "Arrows break"), ("25% on a hit", true));
    assert_eq!(styled(&card, CardStyle::Fire), [("Fire damage", "2")]);
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [
            ("+25% damage", "Masterwork"),
            ("Adds 2 fire damage", "Kindled"),
            ("+2 tiles of range", "Far"),
            ("+1 to Archery", "Fletcher's"),
            ("-50% arrows broken", "Sparing")
        ]
    );
}

#[test]
fn a_shield_is_made_well_or_badly_its_magic_blocks_more_and_its_known_once_taken_up() {
    let shield = |name: &str| on(Fits::Shield, name);
    let (mut w, me, first) = field_with(vec![
        Property {
            block: 1000,
            ..shield("Steadfast")
        },
        Property {
            bash: 1000,
            ..shield("Spiked")
        },
        Property {
            endurance: 800,
            ..shield("Deft")
        },
        Property {
            raises: Raises::Own,
            levels: 1,
            ..shield("Warden's")
        },
        Property {
            wear: 3000,
            curse: true,
            ..shield("Rotten")
        },
    ]);
    let made = Item {
        quality: FINE,
        magic: Magic {
            rarity: 2,
            props: [first, first + 1, first + 2, first + 3],
            curse: first + 4,
            ..magic([NONE, NONE], NONE)
        },
        ..Item::new(BUCKLER, 1)
    };
    // Unknown: how well it was made shows (fine: 5% more blocked, slower wear), its magic
    // doesn't.
    let card = w.card(&made, None);
    assert_eq!(number(&card, "Chance to block"), ("55% braced", true));
    assert_eq!(number(&card, "Wear"), ("3.4% a block", true));
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [("+5% chance to block", "Fine"), ("-15% wear", "Fine")]
    );
    let note = styled(&card, CardStyle::Note);
    assert!(
        note[0]
            .0
            .starts_with("Not known yet: take it up on your arm"),
        "{note:?}"
    );
    // Taken up, it shows what it does, its curse with it; and a level of Shield, its own skill.
    w.give(me, made);
    let seen = wield(&mut w, me, BUCKLER);
    assert!(seen.contains(&Event::Identified {
        who: me,
        item: BUCKLER
    }));
    let on_arm = w.actor(me).unwrap().shield.unwrap();
    assert!(on_arm.magic.known && on_arm.magic.curse_shown);
    assert_eq!(w.skill_level(me, Skill::Shield), 1);
    let card = w.card(&on_arm, None);
    for (name, value) in [
        ("Chance to block", "65% braced"),
        ("Bash", "+25% to knock down"),
        ("Breath a block", "4.8"),
        ("Wear", "10.2% a block"),
    ] {
        assert_eq!(number(&card, name), (value, true), "{name}");
    }
    assert_eq!(
        styled(&card, CardStyle::Mod),
        [
            ("+5% chance to block", "Fine"),
            ("-15% wear", "Fine"),
            ("+10% chance to block", "Steadfast"),
            ("+10% chance a bash knocks down", "Spiked"),
            ("+1 to Shield", "Warden's"),
            ("-20% breath a block", "Deft")
        ]
    );
    assert_eq!(
        styled(&card, CardStyle::Curse),
        [("+200% wear", "Rotten (curse)")]
    );
    // Its curse is a cost it carries: it doesn't cling. Put away, it gives nothing.
    w.intent(me, Intent::Unshield);
    run(&mut w, 10);
    assert!(w.actor(me).unwrap().shield.is_none());
    assert_eq!(w.skill_level(me, Skill::Shield), 0);
}

#[test]
fn a_chests_bows_and_shields_are_made_well_or_badly_and_their_magic_is_their_own() {
    for (def, fits) in [(BOW, Fits::Bow), (BUCKLER, Fits::Shield)] {
        let mut better = 0;
        for seed in 0..60 {
            let (mut w, _) = field_of(seed, Vec::new());
            let mut d = (*w.defs).clone();
            d.properties.push(Property {
                range: 2,
                ..on(Fits::Bow, "Far")
            });
            d.properties.push(Property {
                block: 1000,
                ..on(Fits::Shield, "Steadfast")
            });
            d.loot[0].entries[0].item = def;
            d.magic.cursed = 0;
            w.defs = Arc::new(d);
            let found = w.store(CHEST)[0];
            assert_eq!(found.def, def);
            assert!(found.magic.is_magic() && !found.magic.known);
            for p in found.magic.all() {
                assert_eq!(w.defs.properties[usize::from(p)].fits, fits);
            }
            better += usize::from(found.quality != PLAIN);
        }
        assert!(better > 0, "some made better or worse than plain");
    }
}

/// The uniques' field with one more unique, made plainly on `def` with `effect`, and that unique
/// as you'd carry it, known.
fn field_with_unique(def: u16, effect: Property) -> (World, ActorId, Item) {
    let (mut w, me) = unique_field(12, Vec::new(), |_| {});
    let mut d = (*w.defs).clone();
    let u = d.uniques.len() as u8;
    d.uniques.push(Unique {
        id: effect.id.clone(),
        name: effect.name.clone(),
        item: def,
        quality: PLAIN,
        props: Vec::new(),
        effect,
        lore: String::new(),
    });
    w.defs = Arc::new(d);
    let made = Item {
        magic: Magic {
            rarity: UNIQUE,
            unique: u,
            known: true,
            curse_shown: true,
            ..Magic::MUNDANE
        },
        ..Item::new(def, 1)
    };
    (w, me, made)
}

/// Puts on what you carry of `def`.
fn put_on(w: &mut World, me: ActorId, def: u16) {
    let s = slot(w, me, def);
    w.intent(me, Intent::Wear { slot: s, def });
    run(w, 30);
    assert!(w.actor(me).unwrap().worn.iter().any(|i| i.def == def));
}

/// The harm `whom` took back off `me`'s thorns, a blow at a time, among `seen`.
fn thorned(seen: &[Event], me: ActorId, whom: ActorId) -> Vec<i32> {
    seen.iter()
        .filter_map(|e| match *e {
            Event::Thorned {
                who,
                target,
                damage,
            } if who == me && target == whom => Some(damage),
            _ => None,
        })
        .collect()
}

#[test]
fn a_unique_pieces_thorns_hurt_whatever_strikes_you_and_can_put_it_down() {
    let sting = Property {
        thorns: (20, 20),
        ..on(Fits::Armour, "Sting")
    };
    // Its card says so in numbers, with no words of its own.
    let (w, _, piece) = field_with_unique(CAP, sting.clone());
    assert_eq!(
        styled(&w.card(&piece, None), CardStyle::Effect),
        [("Sting: 2 harm back on whatever strikes you", "")]
    );
    // A tough one that lands every blow: each, turned or not, costs it 2 points.
    let (mut w, me, piece) = field_with_unique(CAP, sting.clone());
    w.give(me, piece);
    put_on(&mut w, me, CAP);
    let c = w.creature_mut(0);
    (c.health, c.attack.hit) = (1_000_000, 10_000);
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    let seen = run(&mut w, 60);
    let blows = seen
        .iter()
        .filter(|e| matches!(e, Event::Struck { who, .. } if *who == z))
        .count();
    assert!(blows >= 1, "it struck you");
    assert_eq!(thorned(&seen, me, z), vec![20; blows]);
    let left = w.actor(z).unwrap().body.health;
    assert_eq!(left, 1_000_000 - 20 * blows as i32);
    // A weak one: its first blow puts it down.
    let (mut w, me, piece) = field_with_unique(CAP, sting);
    w.give(me, piece);
    put_on(&mut w, me, CAP);
    let c = w.creature_mut(0);
    (c.health, c.attack.hit) = (15, 10_000);
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    let seen = run(&mut w, 60);
    assert_eq!(thorned(&seen, me, z), [20]);
    assert!(w.actor(z).unwrap().dead);
    assert!(seen.contains(&Event::Died { who: z }));
}

#[test]
fn a_calming_unique_piece_slows_your_dread() {
    let vigil = |calm: u32| Property {
        calm,
        ..on(Fits::Armour, "Vigil")
    };
    let (w, _, piece) = field_with_unique(CAP, vigil(500));
    assert_eq!(
        styled(&w.card(&piece, None), CardStyle::Effect),
        [("Vigil: -50% dread taken", "")]
    );
    // Three minutes in the dark, in a cap that calms or one that doesn't.
    let dread_after = |calm: u32| {
        let (mut w, me, piece) = field_with_unique(CAP, vigil(calm));
        w.give(me, piece);
        put_on(&mut w, me, CAP);
        w.rules.dread.dark_below = 101;
        let minute = w.rules.clock.ticks_per_minute;
        run(&mut w, 3 * minute);
        w.dread(me)
    };
    let (plain, calmed) = (dread_after(1000), dread_after(500));
    assert!(plain > 0);
    assert_eq!(calmed * 2, plain, "half as fast");
}

#[test]
fn a_piercing_unique_bows_arrow_flies_on_past_one_it_puts_down() {
    let shoot = |pierces: bool| {
        let (mut w, me, bow) = field_with_unique(
            BOW,
            Property {
                cleave: pierces,
                line: if pierces {
                    String::new()
                } else {
                    "none".into()
                },
                ..on(Fits::Bow, "Pierce")
            },
        );
        if pierces {
            assert_eq!(
                styled(&w.card(&bow, None), CardStyle::Effect),
                [(
                    "Pierce: An arrow that puts one down flies on into the next",
                    ""
                )]
            );
        }
        w.give(me, bow);
        w.give(me, Item::new(ARROW, 5));
        wield(&mut w, me, BOW);
        // Two weak ones on a line south, that never see you: every arrow lands.
        let c = w.creature_mut(0);
        c.health = 1;
        c.cone = dusk_core::senses::Cone { cos: 10_001 };
        let a = w
            .spawn(ActorKind::Creature(0), TilePos::new(0, 2, 0))
            .unwrap();
        let b = w
            .spawn(ActorKind::Creature(0), TilePos::new(0, 4, 0))
            .unwrap();
        w.intent(
            me,
            Intent::Shoot {
                at: TilePos::new(0, 2, 0),
            },
        );
        let seen = run(&mut w, 30);
        assert!(w.actor(a).unwrap().dead, "the first, shot at");
        (w.actor(b).unwrap().dead, seen, me)
    };
    // It flies on from where the first fell, and puts the second down too.
    let (second, seen, me) = shoot(true);
    assert!(second);
    let (from, to) = (TilePos::new(0, 2, 0), TilePos::new(0, 4, 0));
    assert!(seen.contains(&Event::Shot { who: me, from, to }));
    // A bow that doesn't pierce stops at the first.
    let (second, ..) = shoot(false);
    assert!(!second);
}

#[test]
fn a_unique_shield_stings_frightens_and_fells_what_it_blocks() {
    let (mut w, me, shield) = field_with_unique(
        BUCKLER,
        Property {
            thorns: (20, 20),
            terror: 10_000,
            fell: 10_000,
            ..on(Fits::Shield, "All")
        },
    );
    let does = [
        "100% chance a block sends the dead fleeing",
        "100% chance a block knocks down",
        "2 harm back on whatever you block",
    ];
    assert_eq!(
        styled(&w.card(&shield, None), CardStyle::Effect),
        [(format!("All: {}", does.join(", ")).as_str(), "")]
    );
    w.give(me, shield);
    wield(&mut w, me, BUCKLER);
    let c = w.creature_mut(0);
    (c.health, c.attack.hit) = (1_000_000, 10_000);
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(0, 1, 0))
        .unwrap();
    // Face it, and brace.
    w.intent(
        me,
        Intent::Move {
            dir: dusk_core::Dir8::S,
            run: false,
        },
    );
    w.step(1);
    w.intent(me, Intent::Idle);
    w.step(1);
    w.brace(me, true);
    let mut seen = Vec::new();
    while !seen.iter().any(|e| matches!(e, Event::Blocked { .. })) {
        assert!(seen.len() < 10_000, "a blow blocked");
        seen.extend(run(&mut w, 10));
    }
    // The blow it blocked: 2 points back, and it breaks and flees, knocked down.
    assert_eq!(thorned(&seen, me, z), [20]);
    assert!(seen.contains(&Event::Terrified { who: me, target: z }));
    assert!(seen.contains(&Event::Felled { who: me, target: z }));
    assert_eq!(w.actor(z).unwrap().body.health, 1_000_000 - 20);
}
