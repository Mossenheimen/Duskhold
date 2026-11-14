//! Magic items: properties on a weapon or a light, curses that cling, learning by use or at a
//! holdout's counter, and what a container's loot rolls; a weapon's quality (`quality.rs`), and
//! its card; and the uniques, with their effects.

use std::sync::Arc;

use dusk_core::hash::{Purpose, roll};
use dusk_core::magic::{NONE, UNIQUE};
use dusk_core::quality::{CRUDE, FINE, MASTERWORK, PLAIN};
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Defs, Event, GetUp, HANDS, Holdout, Intent, Item, ItemDef, ItemKind,
    LootEntry, LootTable, Magic, NoDeal, Part, Property, Remains, Tile, TilePos, Unique, World,
    WoundKind,
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
        unique: NONE,
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
    assert_eq!(d.label(&keen), "fine knife (magic, unknown)");
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
    assert_eq!(
        d.lines(&found),
        vec!["Unique: what it does, you don't know yet"]
    );
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
    assert_eq!(
        d.lines(&known),
        vec!["Unique", "Keen: Keen", "Thirst: thirst", "\"It drinks.\""]
    );
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
