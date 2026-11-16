//! Each class of weapon's own edge (`edge.rs`): a blade finishes one that's down, a blunt blow
//! knocks down, an axe's leaves it bleeding (not bone), a spear's drives it back a tile; bare
//! hands and the dead's blows have none; and the card shows a weapon's.

use std::sync::Arc;

use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, CardLine, CardStyle, Defs, Dir8, Edges, Event, Intent, Item, ItemDef,
    ItemKind, Skill, Tile, TilePos, WeaponEdge, World,
};

/// The four classes, a weapon each, in this order: its number is its place here.
const CLASSES: [Skill; 4] = [Skill::Blade, Skill::Blunt, Skill::Axe, Skill::Spear];

/// A weapon of each class, each landing every time for 10 points and never wearing; you hold the
/// one of `skill` (or nothing).
fn defs(skill: Option<Skill>) -> Defs {
    let weapon = |skill: Skill| ItemDef {
        id: skill.name().to_lowercase(),
        name: skill.name().to_lowercase(),
        weight: 1000,
        stack: 1,
        kind: ItemKind::Weapon {
            swing: Swing {
                wind_up: 3,
                recovery: 3,
                endurance: 10,
                hit: 10_000,
                damage: (100, 100),
                noise: 3,
            },
            wear: 0,
            skill: Some(skill),
        },
        worth: 0,
    };
    Defs {
        items: CLASSES.map(weapon).to_vec(),
        wield: skill.and_then(|s| CLASSES.iter().position(|&c| c == s).map(|n| n as u16)),
        carry: 30_000,
        ..Defs::default()
    }
}

/// You on grass beside one of the dead (500 points of health), holding a weapon of `skill` (or
/// nothing), the classes' edges `edges`. It never lands a blow, sees all round (no sneak
/// attacks), and never takes hold.
fn beside(skill: Option<Skill>, edges: Edges) -> (World, ActorId, ActorId) {
    beside_with(skill, edges, 5000)
}

/// The same, its health `health`.
fn beside_with(skill: Option<Skill>, edges: Edges, health: i32) -> (World, ActorId, ActorId) {
    let mut w = World::new(1);
    w.defs = Arc::new(defs(skill));
    for x in 0..5 {
        w.grid.set_tile(TilePos::new(x, 0, 0), Tile::GRASS);
    }
    let c = w.creature_mut(0);
    (c.wander, c.health) = (0, health);
    c.attack.hit = 0;
    c.cone = dusk_core::senses::Cone { cos: -10_000 };
    (w.rules.combat.grab, w.rules.combat.parting_grab) = (0, 0);
    w.rules.combat.attack.hit = 10_000;
    w.rules.edges = edges;
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(1, 0, 0))
        .unwrap();
    (w, me, z)
}

/// One blow east, stepping until it lands: what happened on the tick it did.
fn blow(w: &mut World, me: ActorId) -> Vec<Event> {
    w.intent(me, Intent::Attack { dir: Some(Dir8::E) });
    for _ in 0..50 {
        w.step(1);
        let seen: Vec<Event> = w.events().collect();
        if seen
            .iter()
            .any(|e| matches!(*e, Event::Struck { who, .. } if who == me))
        {
            w.intent(me, Intent::Idle);
            return seen;
        }
    }
    panic!("no blow landed");
}

/// What a blow did, from what happened as it landed.
fn harm(seen: &[Event]) -> i32 {
    seen.iter()
        .find_map(|e| match *e {
            Event::Struck { damage, .. } => Some(damage),
            _ => None,
        })
        .unwrap()
}

fn only(edge: WeaponEdge, class: Skill) -> Edges {
    let mut e = Edges::default();
    match class {
        Skill::Blade => e.blade = edge,
        Skill::Blunt => e.blunt = edge,
        Skill::Axe => e.axe = edge,
        _ => e.spear = edge,
    }
    e
}

#[test]
fn a_weapon_has_its_class_edge_and_bare_hands_and_the_dead_none() {
    let edges = Edges {
        blade: WeaponEdge {
            down: 500,
            ..WeaponEdge::default()
        },
        blunt: WeaponEdge {
            fell: 1200,
            ..WeaponEdge::default()
        },
        axe: WeaponEdge {
            bleed: 80,
            ..WeaponEdge::default()
        },
        spear: WeaponEdge {
            push: 3000,
            ..WeaponEdge::default()
        },
    };
    for class in CLASSES {
        let (w, me, z) = beside(Some(class), edges);
        assert_eq!(w.edge(me), edges.of(Some(class)), "{class:?}");
        assert!(!w.edge(me).is_none());
        assert!(w.edge(z).is_none(), "the dead's blows have none");
    }
    let (w, me, _) = beside(None, edges);
    assert!(w.edge(me).is_none(), "bare hands have none");
    assert!(edges.of(Some(Skill::Archery)).is_none());
}

#[test]
fn a_blade_finishes_one_thats_down() {
    // Its first blow knocks it down (a blunt blow's edge, given the blade here), and the next
    // lands on it there: twice the harm, and the blade's half again.
    let felling = WeaponEdge {
        fell: 10_000,
        ..WeaponEdge::default()
    };
    let down_harm = |edge: WeaponEdge| {
        let (mut w, me, z) = beside(Some(Skill::Blade), only(edge, Skill::Blade));
        let first = blow(&mut w, me);
        assert!(first.contains(&Event::Felled { who: me, target: z }));
        harm(&blow(&mut w, me))
    };
    assert_eq!(
        down_harm(felling),
        200,
        "twice, as any blow on one that's down"
    );
    assert_eq!(
        down_harm(WeaponEdge {
            down: 500,
            ..felling
        }),
        300
    );
}

#[test]
fn a_blunt_blow_may_knock_it_down() {
    let edge = WeaponEdge {
        fell: 10_000,
        ..WeaponEdge::default()
    };
    let (mut w, me, z) = beside(Some(Skill::Blunt), only(edge, Skill::Blunt));
    assert!(blow(&mut w, me).contains(&Event::Felled { who: me, target: z }));
    assert!(w.actor(z).unwrap().prone_until > 0);
    // Another class's weapon doesn't.
    let (mut w, me, z) = beside(Some(Skill::Blade), only(edge, Skill::Blunt));
    blow(&mut w, me);
    assert_eq!(w.actor(z).unwrap().prone_until, 0);
}

#[test]
fn an_axe_blow_leaves_it_bleeding_but_bone_has_no_blood() {
    let edge = WeaponEdge {
        bleed: 80,
        ..WeaponEdge::default()
    };
    let (mut w, me, z) = beside(Some(Skill::Axe), only(edge, Skill::Axe));
    let before = w.actor(z).unwrap().body.health;
    let struck = harm(&blow(&mut w, me));
    assert!(w.actor(z).unwrap().body.bleeding());
    for _ in 0..100 {
        w.step(1);
    }
    // Eight seconds of bleeding: 1 point every 20 ticks, four times.
    let bled = w.rules.combat.bleed_damage * 4;
    assert_eq!(w.actor(z).unwrap().body.health, before - struck - bled);
    assert!(!w.actor(z).unwrap().body.bleeding(), "and it stops");
    // Bone has no blood.
    let (mut w, me, z) = beside(Some(Skill::Axe), only(edge, Skill::Axe));
    w.creature_mut(0).bleeds = false;
    blow(&mut w, me);
    assert!(w.actor(z).unwrap().body.wounds.is_empty());
    // And one that bleeds out dies of it: 15 points, 10 off the blow, then 1 every 20 ticks.
    let long = WeaponEdge {
        bleed: 200,
        ..WeaponEdge::default()
    };
    let (mut w, me, z) = beside_with(Some(Skill::Axe), only(long, Skill::Axe), 150);
    blow(&mut w, me);
    assert!(!w.actor(z).unwrap().dead);
    let mut died = false;
    for _ in 0..200 {
        w.step(1);
        died |= w
            .events()
            .any(|e| matches!(e, Event::Died { who } if who == z));
    }
    assert!(died && w.actor(z).unwrap().dead, "it bled out");
}

#[test]
fn a_spear_blow_may_drive_it_back_a_tile_if_theres_room() {
    let edge = WeaponEdge {
        push: 10_000,
        ..WeaponEdge::default()
    };
    let (mut w, me, z) = beside(Some(Skill::Spear), only(edge, Skill::Spear));
    let seen = blow(&mut w, me);
    assert!(seen.contains(&Event::DrivenBack { who: me, target: z }));
    assert_eq!(w.actor(z).unwrap().pos, TilePos::new(2, 0, 0));
    // With another of the dead behind it, there's no room.
    let (mut w, me, z) = beside(Some(Skill::Spear), only(edge, Skill::Spear));
    w.spawn(ActorKind::Creature(0), TilePos::new(2, 0, 0))
        .unwrap();
    let seen = blow(&mut w, me);
    assert!(!seen.iter().any(|e| matches!(e, Event::DrivenBack { .. })));
    assert_eq!(w.actor(z).unwrap().pos, TilePos::new(1, 0, 0));
}

fn styled(card: &[CardLine], style: CardStyle) -> Vec<(&str, &str)> {
    card.iter()
        .filter(|l| l.style == style)
        .map(|l| (l.text.as_str(), l.value.as_str()))
        .collect()
}

#[test]
fn a_weapons_card_shows_its_class_edge() {
    let edges = Edges {
        blade: WeaponEdge {
            down: 500,
            ..WeaponEdge::default()
        },
        blunt: WeaponEdge {
            fell: 1200,
            ..WeaponEdge::default()
        },
        axe: WeaponEdge {
            bleed: 80,
            ..WeaponEdge::default()
        },
        spear: WeaponEdge {
            push: 3000,
            ..WeaponEdge::default()
        },
    };
    let (w, ..) = beside(None, edges);
    let card = |n: u16| w.card(&Item::new(n, 1), None);
    assert_eq!(
        styled(&card(0), CardStyle::Mod),
        [("+50% damage to one that's down", "Blade")]
    );
    assert_eq!(
        styled(&card(1), CardStyle::Mod),
        [("12% chance a blow knocks down", "Blunt")]
    );
    assert_eq!(
        styled(&card(2), CardStyle::Mod),
        [("A blow bleeds 4 damage over 8 s", "Axe")]
    );
    assert_eq!(
        styled(&card(3), CardStyle::Mod),
        [("30% chance a blow drives it back a tile", "Spear")]
    );
    // With no edges, none.
    let (w, ..) = beside(None, Edges::default());
    assert!(styled(&w.card(&Item::new(0, 1), None), CardStyle::Mod).is_empty());
}
