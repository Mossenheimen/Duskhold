//! Duskhold's tools.
//!
//! - `dusk-cli demo`: a little scene, printed once a second of game time.
//! - `dusk-cli bench [--dead N] [--ticks N] [--think-every N]`: times the core on the same task as
//!   `research/python_tick_bench.py`.

use std::time::Instant;

use dusk_core::hash::{Purpose, below, mix64, roll};
use dusk_core::world::TICKS_PER_SECOND;
use dusk_core::{ActorKind, Dir8, Event, Intent, Tile, TilePos, World};

/// Rooms and a corridor above a field. `@` is you; each `z` a shambler.
const DEMO: &str = "\
+-+-+-+-+-+-+-+-+-+-+
|. . . . .|. . . z .|
+         +-+-+-+d+-+
|@ . . . .d. . . . .|
+         +-+-+-+-+-+
|. . . . .|. z . . .|
+-+-+-+-+-+-+-+-+-+-+
|, , z , , , , z , ,|
+                   +
|, , , , , , , , , ,|
+-+-+-+-+-+-+-+-+-+-+
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("demo") => demo(),
        Some("bench") => bench(&args[1..]),
        _ => {
            eprintln!("usage: dusk-cli demo | bench [--dead N] [--ticks N] [--think-every N]");
            std::process::exit(2);
        }
    }
}

fn demo() {
    let (mut w, _) = World::from_text(1, DEMO).expect("the demo map parses");
    let me = w.character().expect("the map places you");
    println!("@ you   z a shambler that hasn't noticed anything   Z one that has");
    println!("| - walls   d an open door   , outdoors   (one frame a second)\n");
    let script = [
        (
            1,
            Intent::Move {
                dir: Dir8::E,
                run: false,
            },
            "You walk east, quietly.",
        ),
        (70, Intent::Idle, "You stop."),
        (
            90,
            Intent::Move {
                dir: Dir8::W,
                run: true,
            },
            "You run back west.",
        ),
        (130, Intent::Idle, "You stop."),
    ];
    for t in 1..=200u64 {
        for (at, intent, line) in script {
            if at == t {
                println!(">> {line}");
                w.intent(me, intent);
            }
        }
        w.step(1);
        // Every footstep is a sound event; the text demo leaves those to the frames.
        let events: Vec<Event> = w
            .events()
            .filter(|e| !matches!(e, Event::Sound { .. }))
            .collect();
        for e in events {
            println!("   {}", describe(&w, e));
        }
        if t % u64::from(TICKS_PER_SECOND) == 0 {
            println!("{} s", t / u64::from(TICKS_PER_SECOND));
            print!("{}", w.render(TilePos::new(0, 0, 0), 10, 5));
        }
    }
}

fn describe(w: &World, e: Event) -> String {
    let name = |id| match w.actor(id).map(|a| (a.kind, a.pos)) {
        Some((ActorKind::Character, _)) => "you".to_string(),
        Some((ActorKind::Shambler, p)) => format!("the shambler at ({}, {})", p.x, p.y),
        None => "something".to_string(),
    };
    match e {
        Event::Heard { who, at } => {
            format!("{} heard something at ({}, {})", name(who), at.x, at.y)
        }
        Event::Saw { who, target } => format!("{} saw {}", name(who), name(target)),
        Event::Reached { who, target } => format!("{} reached {}", name(who), name(target)),
        Event::NoWay { who, to } => {
            format!("{} can't find a way to ({}, {})", name(who), to.x, to.y)
        }
        Event::Sound { at, loudness, .. } => {
            format!("a noise at ({}, {}), carrying {loudness} tiles", at.x, at.y)
        }
    }
}

fn bench(args: &[String]) {
    let dead: usize = flag(args, "--dead").unwrap_or(100);
    let ticks: u32 = flag(args, "--ticks").unwrap_or(20_000);
    // 1 matches the Python benchmark, which thinks every tick; the game's default is 5.
    let think: u64 = flag(args, "--think-every").unwrap_or(1);
    let mut w = open_field(96, dead, 1);
    w.rules.think_every = think;
    w.rules.shambler_sight = 15;
    let me = w.character().expect("placed");
    let start = Instant::now();
    for t in 0..ticks {
        if t % 5 == 0 {
            let r = roll(1, u64::from(t), 0, Purpose::WanderDir);
            w.intent(
                me,
                Intent::Move {
                    dir: Dir8::ALL[below(r, 8) as usize],
                    run: false,
                },
            );
        }
        if t % 10 == 0 {
            let at = w.actor(me).expect("still there").pos;
            w.make_sound(at, 20, Some(me));
        }
        w.step(1);
        w.events().for_each(drop);
    }
    let tps = f64::from(ticks) / start.elapsed().as_secs_f64();
    println!(
        "{dead} dead on a 96 x 96 map, thinking every {think} tick(s): {tps:.0} ticks/s = {:.0}x real time",
        tps / f64::from(TICKS_PER_SECOND)
    );
}

fn flag<T: std::str::FromStr>(args: &[String], name: &str) -> Option<T> {
    let i = args.iter().position(|a| a == name)?;
    args.get(i + 1)?.parse().ok()
}

/// An open field `size` tiles square, 12% of it rock (like the Python benchmark's map), you in
/// the middle and `dead` shamblers scattered about.
fn open_field(size: i32, dead: usize, seed: u64) -> World {
    let mut w = World::new(seed);
    for y in 0..size {
        for x in 0..size {
            let rock = below(roll(seed, 0, (y * size + x) as u64, Purpose::Map), 100) < 12;
            w.grid.set_tile(
                TilePos::new(x, y, 0),
                Tile {
                    solid: rock,
                    ..Tile::EARTH
                },
            );
        }
    }
    let centre = TilePos::new(size / 2, size / 2, 0);
    w.grid.set_tile(centre, Tile::EARTH);
    w.spawn(ActorKind::Character, centre)
        .expect("the centre is clear");
    let (mut placed, mut n) = (0, 0u64);
    while placed < dead {
        n += 1;
        let r = roll(seed, n, 0, Purpose::Place);
        let pos = TilePos::new(
            below(r, size as u32) as i32,
            below(mix64(r), size as u32) as i32,
            0,
        );
        if w.spawn(ActorKind::Shambler, pos).is_some() {
            placed += 1;
        }
    }
    w
}
