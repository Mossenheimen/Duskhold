//! A fire for your base: laying it and lighting it, what it holds, burning down and going out,
//! putting it out, the light it gives, and the dead that see it in the night.

use std::sync::Arc;

use dusk_core::fire::Fire;
use dusk_core::{
    ActorId, ActorKind, Defs, Dir8, Edge, Event, Intent, Item, ItemDef, ItemKind, Mind, NoFire,
    Tend, Tile, TilePos, World,
};

/// Ticks in a game minute.
const MINUTE: u32 = 25;
const FIREWOOD: u16 = 0;
const PLANK: u16 = 1;
const CANDLE: u16 = 2;
/// Fixture 1: a stove.
const STOVE: u16 = 1;

/// Split logs (two game hours each) and planks (one), a candle, and a stove that lights 4 tiles
/// round it; you start with `kit`.
fn defs(kit: Vec<Item>) -> Defs {
    let item = |id: &str, kind| ItemDef {
        id: id.into(),
        name: id.into(),
        weight: 1000,
        stack: 10,
        kind,
        worth: 0,
    };
    Defs {
        items: vec![
            item("firewood", ItemKind::Other),
            item("plank", ItemKind::Other),
            item("candle", ItemKind::Light { radius: 2, wane: 4 }),
        ],
        hearths: vec![0, 4],
        fuel: vec![(FIREWOOD, 120), (PLANK, 60)],
        kit,
        carry: 30_000,
        light_time: 5,
        douse_time: 5,
        ..Defs::default()
    }
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y, 0)
}

/// The stove, in the front room.
const STOVE_AT: TilePos = TilePos::new(3, 3, 0);

/// A cottage, from `hour` o'clock: grass to the north, the front room with a window in its north
/// wall and the stove in it, and a back room through an open door. You stand beside the stove
/// with `kit`, and need nothing.
fn cottage(hour: u32, kit: Vec<Item>) -> (World, ActorId) {
    let (mut w, _) = World::from_text(
        3,
        "\
+ + + + + + + +
 , , , , , , , \n\
+ + + + + + + +
 , , , , , , , \n\
+-+-+-+W+-+-+-+
|. . . . . . .|
+             +
|. . . . . . .|
+-+-+-+-+-+d+-+
|. . . . . . .|
+-+-+-+-+-+-+-+
",
    )
    .unwrap();
    w.defs = Arc::new(defs(kit));
    w.rules.clock.start = hour * 60;
    w.rules.needs.hunger.rise = 0;
    w.rules.needs.thirst.rise = 0;
    w.rules.needs.fatigue.rise = 0;
    w.creature_mut(0).wander = 0;
    w.grid.set_tile(
        STOVE_AT,
        Tile {
            fixture: STOVE,
            solid: true,
            ..Tile::FLOOR
        },
    );
    let me = w.spawn(ActorKind::Character, at(2, 3)).unwrap();
    (w, me)
}

/// Steps the world, keeping its events.
fn run(w: &mut World, ticks: u32) -> Vec<Event> {
    let mut seen = Vec::new();
    for _ in 0..ticks {
        w.step(1);
        seen.extend(w.events());
    }
    seen
}

/// Asks for an action and runs until it's done.
fn act(w: &mut World, me: ActorId, intent: Intent) -> Vec<Event> {
    w.intent(me, intent);
    let mut seen = Vec::new();
    for _ in 0..200 {
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

/// Lights the candle in your pack and holds it up.
fn hold_up_candle(w: &mut World, me: ActorId) {
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == CANDLE)
        .unwrap() as u16;
    act(w, me, Intent::Light { slot, def: CANDLE });
    assert!(w.actor(me).unwrap().light.is_some());
}

/// Lays two logs in the stove, lights it from the candle and puts the candle out.
fn light_the_stove(w: &mut World, me: ActorId) {
    act(w, me, Intent::Tend);
    act(w, me, Intent::Tend);
    hold_up_candle(w, me);
    let seen = act(w, me, Intent::Tend);
    assert!(seen.contains(&Event::FireLit {
        who: me,
        at: STOVE_AT
    }));
    act(w, me, Intent::Douse);
}

/// A cottage at `hour` with its stove alight, and you gone through to the back room, the door
/// shut behind you: nothing outside can see you.
fn lit_cottage(hour: u32) -> (World, ActorId) {
    let (mut w, me) = cottage(hour, vec![Item::new(FIREWOOD, 2), Item::new(CANDLE, 1)]);
    light_the_stove(&mut w, me);
    w.intent(
        me,
        Intent::WalkTo {
            to: at(5, 4),
            run: false,
        },
    );
    run(&mut w, 120);
    assert_eq!(w.actor(me).unwrap().pos, at(5, 4));
    act(&mut w, me, Intent::Door);
    assert_eq!(w.grid.edge(at(5, 4), Dir8::N), Edge::door(false));
    (w, me)
}

#[test]
fn a_fire_is_laid_with_the_longest_burning_fuel_and_lit_from_the_light_you_hold() {
    // Nothing that burns: nothing to lay.
    let (w, me) = cottage(8, vec![Item::new(CANDLE, 1)]);
    assert_eq!(w.can_tend(me), Err(NoFire::NoFuel));
    assert_eq!(w.can_smother(me), Err(NoFire::Nothing), "nothing alight");

    let (mut w, me) = cottage(
        8,
        vec![
            Item::new(PLANK, 1),
            Item::new(FIREWOOD, 1),
            Item::new(CANDLE, 1),
        ],
    );
    // A log goes on before a plank, whatever comes first in the pack.
    assert_eq!(w.can_tend(me), Ok((STOVE_AT, Tend::Lay { item: FIREWOOD })));
    let seen = act(&mut w, me, Intent::Tend);
    assert!(seen.contains(&Event::Laid {
        who: me,
        at: STOVE_AT,
        item: FIREWOOD
    }));
    assert_eq!(count(&w, me, FIREWOOD), 0);
    assert_eq!(w.hearth(STOVE_AT).fuel, 120);
    assert!(!w.hearth(STOVE_AT).lit);
    // Laid but with no flame, tending it lays on what's left: the plank.
    act(&mut w, me, Intent::Tend);
    assert_eq!((count(&w, me, PLANK), w.hearth(STOVE_AT).fuel), (0, 180));
    // Nothing more to lay, and no flame: that's what it wants.
    assert_eq!(w.can_tend(me), Err(NoFire::NoFlame));
    hold_up_candle(&mut w, me);
    assert_eq!(w.can_tend(me), Ok((STOVE_AT, Tend::Light)));
    act(&mut w, me, Intent::Tend);
    assert!(w.hearth(STOVE_AT).lit);
    assert_eq!(w.can_smother(me), Ok(STOVE_AT));

    // Away from the stove, there's nothing to tend.
    let (mut w, me) = cottage(8, vec![Item::new(FIREWOOD, 1)]);
    w.intent(
        me,
        Intent::WalkTo {
            to: at(0, 2),
            run: false,
        },
    );
    run(&mut w, 60);
    assert_eq!(w.can_tend(me), Err(NoFire::Nothing));
}

#[test]
fn a_fire_holds_so_much_burns_down_a_minute_at_a_time_and_goes_out() {
    let (mut w, me) = cottage(8, vec![Item::new(FIREWOOD, 3), Item::new(CANDLE, 1)]);
    w.rules.fire = Fire {
        most: 240,
        ..Fire::default()
    };
    act(&mut w, me, Intent::Tend);
    act(&mut w, me, Intent::Tend);
    assert_eq!(w.hearth(STOVE_AT).fuel, 240);
    // Two logs fill it: laid and cold, what it wants is a flame; alight, it's full.
    assert_eq!(w.can_tend(me), Err(NoFire::NoFlame));
    hold_up_candle(&mut w, me);
    act(&mut w, me, Intent::Tend);
    assert_eq!(w.can_tend(me), Err(NoFire::Full));
    act(&mut w, me, Intent::Douse);
    let fuel = w.hearth(STOVE_AT).fuel;
    run(&mut w, 60 * MINUTE);
    assert_eq!(
        w.hearth(STOVE_AT).fuel,
        fuel - 60,
        "a game minute at a time"
    );
    // Burnt down by a log's worth, there's room for one again.
    run(&mut w, 60 * MINUTE);
    assert_eq!(w.can_tend(me), Ok((STOVE_AT, Tend::Lay { item: FIREWOOD })));
    let seen = run(&mut w, (fuel - 120) * MINUTE);
    assert!(seen.contains(&Event::FireOut { at: STOVE_AT }));
    assert_eq!(
        (w.hearth(STOVE_AT).fuel, w.hearth(STOVE_AT).lit),
        (0, false)
    );
}

#[test]
fn put_out_a_fire_keeps_what_it_has_left_for_the_next_time() {
    let (mut w, me) = cottage(8, vec![Item::new(FIREWOOD, 2), Item::new(CANDLE, 1)]);
    light_the_stove(&mut w, me);
    run(&mut w, 10 * MINUTE);
    let seen = act(&mut w, me, Intent::Smother);
    assert!(seen.contains(&Event::Smothered {
        who: me,
        at: STOVE_AT
    }));
    let left = w.hearth(STOVE_AT);
    assert!(!left.lit && left.fuel > 200);
    run(&mut w, 30 * MINUTE);
    assert_eq!(w.hearth(STOVE_AT), left, "out, it doesn't burn down");
    // Laid already, it only wants a flame.
    hold_up_candle(&mut w, me);
    assert_eq!(w.can_tend(me), Ok((STOVE_AT, Tend::Light)));
}

#[test]
fn a_fire_lights_its_room_in_the_night_but_not_through_a_wall() {
    let (mut w, me) = cottage(22, vec![Item::new(FIREWOOD, 2), Item::new(CANDLE, 1)]);
    let (room, back) = (at(0, 2), at(0, 4));
    let dark = (w.light_at(room), w.light_at(back));
    light_the_stove(&mut w, me);
    assert!(
        w.actor(me).unwrap().light.is_none(),
        "no light in your hand"
    );
    assert!(
        w.light_at(room) >= 50,
        "the room's far corner, lit: {}",
        w.light_at(room)
    );
    assert_eq!(w.light_at(at(3, 2)), 100, "full beside it");
    assert_eq!(
        w.light_at(back),
        dark.1,
        "the back room, through a wall: as dark"
    );
    assert!(dark.0 < 20);
}

/// One of the dead out on the grass, facing the window, with the stove alight beyond it: the
/// events of the next two game minutes, and what it's after then.
fn watcher(w: &mut World) -> (ActorId, Vec<Event>, Option<TilePos>) {
    let z = w.spawn(ActorKind::Creature(0), at(3, 0)).unwrap();
    let seen = run(w, 2 * MINUTE);
    let target = match w.actor(z).unwrap().mind {
        Mind::Hunt { target, .. } => target,
        Mind::None | Mind::Flee { .. } => None,
    };
    (z, seen, target)
}

#[test]
fn in_the_night_the_dead_that_see_a_fire_come_to_it() {
    let (mut w, _) = lit_cottage(22);
    let (z, seen, target) = watcher(&mut w);
    assert!(
        seen.contains(&Event::SawFire {
            who: z,
            at: STOVE_AT
        }),
        "{seen:?}"
    );
    // To the tile in front of it, nearest the window.
    assert_eq!(target, Some(at(3, 2)));
}

#[test]
fn a_boarded_window_hides_a_fire_and_by_day_nobody_heeds_one() {
    let (mut w, _) = lit_cottage(22);
    w.grid.set_edge(
        at(3, 2),
        Dir8::N,
        Edge::Window {
            open: false,
            boards: 1,
        },
    );
    let (_, seen, target) = watcher(&mut w);
    assert!(!seen.iter().any(|e| matches!(e, Event::SawFire { .. })));
    assert_eq!(target, None, "behind the boards, nobody saw it");

    let (mut w, _) = lit_cottage(12);
    let (_, seen, target) = watcher(&mut w);
    assert!(!seen.iter().any(|e| matches!(e, Event::SawFire { .. })));
    assert_eq!(target, None, "by day, a fire is nothing to them");
}

#[test]
fn a_lantern_eater_puts_out_a_fire_it_passes() {
    let (mut w, _) = lit_cottage(22);
    w.creature_mut(0).douses = 8;
    let (z, seen, _) = watcher(&mut w);
    assert!(seen.contains(&Event::Smothered {
        who: z,
        at: STOVE_AT
    }));
    let left = w.hearth(STOVE_AT);
    assert!(!left.lit && left.fuel > 0, "out, with its fuel still in it");
}
