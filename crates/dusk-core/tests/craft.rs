//! Crafting: a recipe uses up its makings (from the pack, then the hand), wants its tool to hand,
//! teaches its skill; furniture breaks down for planks with a hammer, loudly, and the tile clears.

use std::sync::Arc;

use dusk_core::craft::{Breaking, Recipe};
use dusk_core::world::Swing;
use dusk_core::{
    ActorId, ActorKind, Area, Defs, Event, Intent, Item, ItemDef, ItemKind, NoBreak, NoCraft,
    Skill, Tile, TilePos, World,
};

const PLANK: u16 = 0;
const FIREWOOD: u16 = 1;
const RAG: u16 = 2;
const TORCH: u16 = 3;
const NAILS: u16 = 4;
const CUDGEL: u16 = 5;
const HAMMER: u16 = 6;
const KNIFE: u16 = 7;
const STAKE: u16 = 8;
const CLUB: u16 = 9;
/// Recipes, by number.
const MAKE_TORCH: u16 = 0;
const MAKE_STAKE: u16 = 1;
const MAKE_CLUB: u16 = 2;
/// Fixture 1: a table, two planks' worth.
const TABLE: u16 = 1;

fn defs(kit: Vec<Item>, wield: Option<u16>) -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 500,
        stack: 20,
        kind,
        worth: 0,
    };
    let weapon = |id: &str, skill| {
        item(
            id,
            ItemKind::Weapon {
                swing: Swing {
                    wind_up: 3,
                    recovery: 3,
                    endurance: 10,
                    hit: 8000,
                    damage: (80, 120),
                    noise: 3,
                },
                wear: 10,
                skill: Some(skill),
            },
        )
    };
    let recipe = |id: &str, makes, uses: Vec<(u16, u16)>, tools: Vec<u16>, skill| Recipe {
        id: id.into(),
        makes: (makes, 1),
        uses,
        tools,
        fire: false,
        time: 20,
        skill,
        xp: 2000,
    };
    Defs {
        items: vec![
            item("plank", ItemKind::Other),
            item("firewood", ItemKind::Other),
            item("rag", ItemKind::Other),
            item(
                "torch",
                ItemKind::Light {
                    radius: 5,
                    wane: 17,
                },
            ),
            item("nails", ItemKind::Other),
            weapon("cudgel", Skill::Blunt),
            weapon("hammer", Skill::Blunt),
            weapon("knife", Skill::Blade),
            weapon("stake", Skill::Spear),
            weapon("club", Skill::Blunt),
        ],
        recipes: vec![
            recipe("torch", TORCH, vec![(FIREWOOD, 1), (RAG, 1)], vec![], None),
            recipe(
                "stake",
                STAKE,
                vec![(PLANK, 1)],
                vec![KNIFE],
                Some(Skill::Carpentry),
            ),
            recipe(
                "club",
                CLUB,
                vec![(CUDGEL, 1), (NAILS, 6)],
                vec![HAMMER],
                Some(Skill::Carpentry),
            ),
        ],
        breaking: Some(Breaking {
            plank: PLANK,
            tools: vec![HAMMER],
            time: 30,
            noise: 10,
            xp: 1000,
        }),
        planks: vec![0, 2],
        kit,
        wield,
        carry: 50_000,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// A room with a table on its east side and you beside it, carrying `kit` and holding `wield`.
fn room(kit: Vec<Item>, wield: Option<u16>) -> (World, ActorId) {
    let mut w = World::new(4);
    w.defs = Arc::new(defs(kit, wield));
    for y in 0..3 {
        for x in 0..5 {
            w.grid.set_tile(at(x, y), Tile::FLOOR);
        }
    }
    w.grid.set_tile(
        at(3, 1),
        Tile {
            fixture: TABLE,
            solid: true,
            ..Tile::FLOOR
        },
    );
    let me = w.spawn(ActorKind::Character, at(2, 1)).unwrap();
    (w, me)
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..300 {
        w.step(1);
        seen.extend(w.events());
        let a = w.actor(me).unwrap();
        if a.action.is_none() && a.intent == Intent::Idle {
            break;
        }
    }
    seen
}

fn count(w: &World, me: ActorId, def: u16) -> u16 {
    w.actor(me)
        .unwrap()
        .items
        .iter()
        .filter(|i| i.def == def)
        .map(|i| i.count)
        .sum()
}

fn carpentry(w: &World, me: ActorId) -> u32 {
    w.actor(me).unwrap().skills.xp[Skill::Carpentry as usize]
}

#[test]
fn a_recipe_uses_up_its_makings_and_makes_something_of_yours() {
    let (mut w, me) = room(vec![Item::new(FIREWOOD, 1), Item::new(RAG, 2)], None);
    assert_eq!(w.can_craft(me, MAKE_TORCH), Ok(()));
    let seen = act(&mut w, me, Intent::Craft { recipe: MAKE_TORCH });
    assert!(seen.contains(&Event::Crafted {
        who: me,
        recipe: MAKE_TORCH
    }));
    assert_eq!(
        (
            count(&w, me, TORCH),
            count(&w, me, FIREWOOD),
            count(&w, me, RAG)
        ),
        (1, 0, 1)
    );
    let torch = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .find(|i| i.def == TORCH)
        .copied();
    assert!(
        !torch.unwrap().found,
        "made, not found: it brings no XP home"
    );
    assert_eq!(w.can_craft(me, MAKE_TORCH), Err(NoCraft::Lacks(FIREWOOD)));
    assert_eq!(w.can_craft(me, 7), Err(NoCraft::NoRecipe));
}

#[test]
fn a_recipe_wants_its_tool_to_hand_keeps_it_and_teaches_its_skill() {
    let (w, me) = room(vec![Item::new(PLANK, 1)], None);
    assert_eq!(w.can_craft(me, MAKE_STAKE), Err(NoCraft::NoTool));
    let (mut w, me) = room(vec![Item::new(PLANK, 1), Item::new(KNIFE, 1)], None);
    let before = carpentry(&w, me);
    act(&mut w, me, Intent::Craft { recipe: MAKE_STAKE });
    assert_eq!(
        (
            count(&w, me, STAKE),
            count(&w, me, PLANK),
            count(&w, me, KNIFE)
        ),
        (1, 0, 1),
        "the knife's kept"
    );
    assert!(carpentry(&w, me) > before, "Carpentry learnt from it");
}

#[test]
fn the_cudgel_in_your_hand_goes_into_a_spiked_club() {
    let (mut w, me) = room(
        vec![Item::new(NAILS, 10), Item::new(HAMMER, 1)],
        Some(CUDGEL),
    );
    assert_eq!(w.can_craft(me, MAKE_CLUB), Ok(()));
    act(&mut w, me, Intent::Craft { recipe: MAKE_CLUB });
    let a = w.actor(me).unwrap();
    assert!(a.hands.is_none(), "the cudgel's gone into it");
    assert_eq!((count(&w, me, CLUB), count(&w, me, NAILS)), (1, 4));
}

#[test]
fn furniture_breaks_down_for_planks_with_a_hammer_and_the_tile_clears() {
    let (w, me) = room(vec![], None);
    assert_eq!(w.can_break(me), Err(NoBreak::NoTool));
    let (mut w, me) = room(vec![Item::new(HAMMER, 1)], None);
    assert_eq!(w.can_break(me), Ok(at(3, 1)));
    let before = carpentry(&w, me);
    let seen = act(&mut w, me, Intent::BreakDown);
    assert!(seen.contains(&Event::BrokeDown {
        who: me,
        at: at(3, 1),
        planks: 2
    }));
    let blows = seen
        .iter()
        .filter(|e| matches!(e, Event::Sound { loudness: 10, .. }))
        .count();
    assert_eq!(blows, 2, "heard as the work starts and as it's done");
    assert_eq!(count(&w, me, PLANK), 2);
    let tile = w.grid.tile(at(3, 1));
    assert!(tile.fixture == 0 && tile.walkable(), "the tile's clear");
    assert!(carpentry(&w, me) > before);
    assert_eq!(
        w.can_break(me),
        Err(NoBreak::Nothing),
        "nothing left to break"
    );

    // On hallowed ground, the sisters' furniture is left be.
    let (mut w, me) = room(vec![Item::new(HAMMER, 1)], None);
    w.hallowed = vec![Area {
        x0: 0,
        y0: 0,
        x1: 5,
        y1: 3,
        z: 0,
    }];
    assert_eq!(w.can_break(me), Err(NoBreak::Nothing));
}
