//! Duskhold's tools.
//!
//! - `dusk-cli demo`: a little scene, printed once a second of game time.
//! - `dusk-cli bench [--dead N] [--ticks N] [--think-every N]`: times the core on the same task as
//!   `research/python_tick_bench.py`.
//! - `dusk-cli check`: loads and checks everything in `data/`.
//! - `dusk-cli prefab <id>`: draws a building, and the same turned a quarter at a time.
//! - `dusk-cli gen [--seed N] [--png PATH] [--scale N]`: makes a patch of the Hushlands and says
//!   what's in it; with `--png`, draws its map.

mod png;

use std::path::Path;
use std::time::Instant;

use dusk_core::hash::{Purpose, below, mix64, roll};
use dusk_core::text::tile_symbol;
use dusk_core::world::TICKS_PER_SECOND;
use dusk_core::{ActorKind, Dir8, Edge, Event, Ground, Intent, Tile, TilePos, World};

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
        Some("check") => {
            let data = load_data();
            println!(
                "data/ is fine: {} fixtures, {} prefabs, {} regions",
                data.fixtures.len(),
                data.prefabs.len(),
                data.regions.len()
            );
        }
        Some("prefab") => prefab(args.get(1).map_or("", String::as_str)),
        Some("gen") => gen_patch(&args[1..]),
        _ => {
            eprintln!(
                "usage: dusk-cli demo | bench [--dead N] [--ticks N] [--think-every N] | check \
                 | prefab <id> | gen [--seed N] [--png PATH] [--scale N]"
            );
            std::process::exit(2);
        }
    }
}

/// The game's data, or a message saying what's wrong with it.
fn load_data() -> dusk_data::Data {
    dusk_data::load(&dusk_data::default_dir()).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    })
}

fn prefab(id: &str) {
    let data = load_data();
    let Some(p) = data.prefab(id) else {
        let ids: Vec<&str> = data.prefabs.iter().map(|p| p.id.as_str()).collect();
        eprintln!("no prefab called {id:?}; there are: {}", ids.join(", "));
        std::process::exit(2);
    };
    let symbol = |c: dusk_data::PrefabCell| c.0.map_or(' ', tile_symbol);
    let mut grid = p.grid.clone();
    let mut front = p.front;
    for turn in 0..4 {
        println!("{} turned {}°, its front {front:?}:", p.id, turn * 90);
        println!("{}", grid.render(symbol));
        grid = grid.rotated();
        front = front.turned();
    }
}

fn gen_patch(args: &[String]) {
    let seed: u64 = flag(args, "--seed").unwrap_or(1);
    let scale: usize = flag(args, "--scale").unwrap_or(3);
    let data = load_data();
    let region = data
        .region("hushlands")
        .expect("the Hushlands are in data/");
    let start = Instant::now();
    let (world, layout) = dusk_gen::patch(seed, &data, region);
    let took = start.elapsed();
    // The dead, by kind: "77 dead (69 shambler, 8 ghoul)".
    let mut kinds = vec![0usize; world.defs.creatures.len()];
    for (_, a) in world.actors() {
        if let dusk_core::ActorKind::Creature(n) = a.kind {
            kinds[usize::from(n)] += 1;
        }
    }
    let by_kind: Vec<String> = world
        .defs
        .creatures
        .iter()
        .zip(&kinds)
        .filter(|&(_, &n)| n > 0)
        .map(|(c, n)| format!("{n} {}", c.name))
        .collect();
    let dead = format!(
        "{} dead ({})",
        kinds.iter().sum::<usize>(),
        by_kind.join(", ")
    );
    println!(
        "{} from seed {seed}: {} x {} tiles, made in {:.0} ms; {} lots, {dead}; you start at ({}, {})",
        region.name,
        layout.width,
        layout.height,
        took.as_secs_f64() * 1000.0,
        layout.lots.len(),
        layout.start.x,
        layout.start.y
    );
    // What there is to bring home: every container's finds, by their worth and weight.
    let (mut containers, mut xp, mut grams, mut cures, mut oil) = (0, 0u64, 0u64, 0, 0);
    for y in 0..layout.height {
        for x in 0..layout.width {
            let at = TilePos::new(x, y, 0);
            if world.is_container(at) {
                containers += 1;
                for i in world.store(at) {
                    let def = world.defs.item(i.def);
                    xp += u64::from(def.map_or(0, |d| d.worth)) * u64::from(i.count);
                    grams += u64::from(world.defs.weight(&i));
                    match def.map(|d| &d.kind) {
                        Some(dusk_core::ItemKind::Cure { .. }) => cures += i.count,
                        Some(dusk_core::ItemKind::Oil) => oil += i.count,
                        _ => {}
                    }
                }
            }
        }
    }
    println!(
        "  {containers} containers hold {} XP of finds, {:.0} kg of them; {cures} cure(s) for the fever, {oil} flask(s) of oil for the dead",
        xp / 1000,
        grams as f64 / 1000.0
    );
    // The magic in it, told as if known: in containers, and on the rares.
    let mut magic: Vec<String> = Vec::new();
    for y in 0..layout.height {
        for x in 0..layout.width {
            let at = TilePos::new(x, y, 0);
            if world.is_container(at) {
                magic.extend(
                    world
                        .store(at)
                        .iter()
                        .filter(|i| i.magic.is_magic())
                        .map(|i| told(&world, i)),
                );
            }
        }
    }
    for id in world.rares() {
        let carried = &world.actor(id).expect("a rare").items;
        magic.extend(
            carried
                .iter()
                .filter(|i| i.magic.is_magic())
                .map(|i| format!("{} (a rare's)", told(&world, i))),
        );
    }
    println!("  {} magic item(s): {}", magic.len(), magic.join("; "));
    for id in world.rares().collect::<Vec<_>>() {
        let a = world.actor(id).expect("a rare");
        let worth: u32 = a
            .items
            .iter()
            .map(|i| world.defs.item(i.def).map_or(0, |d| d.worth) * u32::from(i.count))
            .sum();
        let kind = match a.kind {
            ActorKind::Creature(n) => world.defs.creature(n).map_or("?", |c| c.name.as_str()),
            ActorKind::Character => "?",
        };
        println!(
            "  a rare: {} ({kind}, {} health) at ({}, {}), carrying {} XP of finds",
            world.name_of(id).unwrap_or_default(),
            world.stats(id).map_or(0, |c| c.health) / 10,
            a.pos.x,
            a.pos.y,
            worth / 1000
        );
    }
    for lot in &layout.lots {
        let side = if lot.north { "north" } else { "south" };
        println!(
            "  {:<10} at ({:>3}, {:>3}), {side} of the road, its door at ({}, {})",
            lot.prefab, lot.origin.x, lot.origin.y, lot.door.x, lot.door.y
        );
    }
    // --ticks N times the patch running N ticks with you standing still in it.
    if let Some(ticks) = flag::<u32>(args, "--ticks") {
        let mut running = world.clone();
        let start = Instant::now();
        running.step(ticks);
        let tps = f64::from(ticks) / start.elapsed().as_secs_f64();
        println!(
            "  {ticks} ticks: {tps:.0} ticks/s = {:.0}x real time, {} chunks awake",
            tps / f64::from(TICKS_PER_SECOND),
            running.awake_chunks().len()
        );
        println!(
            "  pressure {}% -> {}%: {} of the dead about, {} wanted",
            world.pressure() / 10,
            running.pressure() / 10,
            running.dead_about(),
            running.dead_wanted()
        );
    }
    // --tolling runs the patch to its first Tolling, then two game hours of it: how many of the
    // dead in the bell's reach are at the bell by then, and how fast it ran.
    if args.iter().any(|a| a == "--tolling") {
        tolling(&world);
    }
    // --text X,Y,W,H prints that part of the patch as a text grid (@ you, z the dead).
    if let Some(area) = flag::<String>(args, "--text") {
        let n: Vec<i32> = area.split(',').filter_map(|v| v.parse().ok()).collect();
        if let [x, y, w, h] = n[..] {
            let marks: std::collections::BTreeMap<TilePos, char> = world
                .actors()
                .map(|(_, a)| {
                    let mark = if a.kind == ActorKind::Character {
                        '@'
                    } else {
                        'z'
                    };
                    (a.pos, mark)
                })
                .collect();
            let origin = TilePos::new(x, y, 0);
            print!(
                "{}",
                dusk_core::text::render(&world.grid, origin, w, h, |p| marks.get(&p).copied())
            );
        }
    }
    if let Some(path) = args
        .iter()
        .position(|a| a == "--png")
        .and_then(|i| args.get(i + 1))
    {
        draw_map(
            &world,
            layout.width,
            layout.height,
            scale.max(2),
            Path::new(path),
        );
        println!("drew {path}");
    }
}

/// The patch as a picture, `scale` pixels a tile: each tile's ground or fixture, its north and
/// west edges as lines, the dead in red and you in yellow.
fn draw_map(world: &World, width: i32, height: i32, scale: usize, path: &Path) {
    let (w, h) = (width as usize * scale, height as usize * scale);
    let mut pixels = vec![0u8; w * h * 3];
    let mut put = |x: usize, y: usize, rgb: [u8; 3]| {
        let i = (y * w + x) * 3;
        pixels[i..i + 3].copy_from_slice(&rgb);
    };
    let edge_colour = |e: Edge| match e {
        Edge::Open => None,
        Edge::Wall => Some([225, 220, 210]),
        Edge::Door { .. } => Some([170, 110, 50]),
        Edge::Window { .. } => Some([140, 200, 235]),
        Edge::Fence => Some([190, 160, 110]),
    };
    for ty in 0..height {
        for tx in 0..width {
            let at = TilePos::new(tx, ty, 0);
            let tile = world.grid.tile(at);
            let fill = match (tile.fixture, tile.ground) {
                (1, _) => [38, 66, 40], // a tree (the first fixture in the data)
                (f, _) if f > 0 => [92, 62, 40],
                (_, Ground::Void) => [18, 20, 26],
                (_, Ground::Grass) => [92, 112, 74],
                (_, Ground::Dirt) => [122, 98, 66],
                (_, Ground::Road) => [152, 138, 108],
                (_, Ground::Water) => [58, 88, 122],
                (_, Ground::Floor) => [146, 116, 84],
            };
            let (px, py) = (tx as usize * scale, ty as usize * scale);
            for dy in 0..scale {
                for dx in 0..scale {
                    put(px + dx, py + dy, fill);
                }
            }
            if let Some(c) = edge_colour(world.grid.north(at)) {
                (0..scale).for_each(|dx| put(px + dx, py, c));
            }
            if let Some(c) = edge_colour(world.grid.west(at)) {
                (0..scale).for_each(|dy| put(px, py + dy, c));
            }
        }
    }
    for (_, a) in world.actors() {
        let colour = match a.kind {
            ActorKind::Character => [255, 222, 40],
            ActorKind::Creature(_) => [220, 40, 40],
        };
        let (px, py) = (a.pos.x as usize * scale, a.pos.y as usize * scale);
        for dy in 0..scale {
            for dx in 0..scale {
                put(px + dx, py + dy, colour);
            }
        }
    }
    png::write(path, w as u32, h as u32, &pixels).expect("the picture is written");
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
        Some((ActorKind::Creature(n), p)) => {
            let kind = w.defs.creature(n).map_or("creature", |c| c.name.as_str());
            format!("the {kind} at ({}, {})", p.x, p.y)
        }
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
        Event::Struck {
            who,
            target,
            damage,
            wound,
        } => match wound {
            Some((kind, part)) => format!(
                "{} gave {} a {} on the {}",
                name(who),
                name(target),
                kind.name(),
                part.name()
            ),
            None => format!("{} hit {} ({damage})", name(who), name(target)),
        },
        Event::Missed { who, .. } => format!("{} missed", name(who)),
        Event::Shoved { who, target, down } => format!(
            "{} shoved {}{}",
            name(who),
            name(target),
            if down { " down" } else { "" }
        ),
        Event::Died { who } => format!("{} died", name(who)),
        Event::Door { who, open, .. } => format!(
            "{} {} a door",
            name(who),
            if open { "opened" } else { "shut" }
        ),
        Event::Exhausted { who } => format!("{} ran out of breath", name(who)),
        Event::Grabbed { who, target } => format!("{} took hold of {}", name(who), name(target)),
        Event::Freed { who, from } => format!("{} got free of {}", name(who), name(from)),
        Event::StruggledInVain { who } => format!("{} struggled, in vain", name(who)),
        Event::Thumped { who, at, .. } => {
            format!("{} beat on the door by ({}, {})", name(who), at.x, at.y)
        }
        Event::DoorBroke { at, .. } => format!("the door by ({}, {}) gave way", at.x, at.y),
        Event::DraggedDown { who, holders } => {
            format!("{holders} of them dragged {} down", name(who))
        }
        Event::Took { who, item, count } => format!("{} took {count} of item {item}", name(who)),
        Event::Dropped { who, item, count } => {
            format!("{} dropped {count} of item {item}", name(who))
        }
        Event::Wielded { who, item } => format!("{} took item {item} in hand", name(who)),
        Event::WeaponBroke { who, item } => format!("{}'s item {item} broke", name(who)),
        Event::Bandaged { who, part } => {
            format!("{} bandaged the {}", name(who), part.name())
        }
        Event::Ate { who, item } => format!("{} ate item {item}", name(who)),
        Event::Drank { who, item } => format!("{} drank item {item}", name(who)),
        Event::DrankWater { who } => format!("{} drank from the water", name(who)),
        Event::FellAsleep { who, in_bed } => format!(
            "{} fell asleep{}",
            name(who),
            if in_bed { " in bed" } else { " on the ground" }
        ),
        Event::WokeUp { who, why } => format!("{} woke ({why:?})", name(who)),
        Event::Lit { who, item } => format!("{} lit item {item}", name(who)),
        Event::Doused { who, item } => format!("{} put out item {item}", name(who)),
        Event::BurntOut { who, item } => format!("{}'s item {item} burnt out", name(who)),
        Event::Nightfall => "the sun is going down".to_string(),
        Event::Daybreak => "the sky is lightening".to_string(),
        Event::SkillUp { who, skill, level } => {
            format!("{}: {} rose to {level}", name(who), skill.name())
        }
        Event::SkillAtCap { who, skill } => {
            format!("{}: {} is at the cap", name(who), skill.name())
        }
        Event::Mastered { who, skill, level } => {
            format!("{} mastered {} (now {level})", name(who), skill.name())
        }
        Event::Boarded {
            who, at, boards, ..
        } => format!(
            "{} nailed a board by ({}, {}): {boards} on it",
            name(who),
            at.x,
            at.y
        ),
        Event::Unboarded {
            who, at, boards, ..
        } => format!(
            "{} pried a board off by ({}, {}): {boards} left",
            name(who),
            at.x,
            at.y
        ),
        Event::MadeHome { who, at } => {
            format!("{} made a home at ({}, {})", name(who), at.x, at.y)
        }
        Event::Stashed { who, at } => {
            format!(
                "{} made the container at ({}, {}) a stash",
                name(who),
                at.x,
                at.y
            )
        }
        Event::Put { who, item, count } => format!("{} put away {count} of item {item}", name(who)),
        Event::BroughtHome {
            who,
            item,
            count,
            xp,
        } => format!(
            "{} brought home {count} of item {item}: {} XP",
            name(who),
            xp / 1000
        ),
        Event::LevelUp { who, level } => format!("{} reached level {level}", name(who)),
        Event::BoardBroke { at, boards, .. } => {
            format!("a board by ({}, {}) broke: {boards} left", at.x, at.y)
        }
        Event::Smelled { who, target } => {
            format!("{} smells {}'s blood", name(who), name(target))
        }
        Event::Fevered { who } => format!("{} has caught ghoul fever", name(who)),
        Event::FeverWorse { who, stage } => {
            let how = if stage >= 3 {
                "dying of it"
            } else {
                "weak with it"
            };
            format!("{}'s fever is worse: {how}", name(who))
        }
        Event::FeverTook { who } => format!("the fever takes {}", name(who)),
        Event::Cured { who } => format!("{} took the cure: the fever's gone", name(who)),
        Event::GotUp { who } => format!("{} got up again", name(who)),
        Event::Kindled { who, body } => format!("{} set {} alight", name(who), name(body)),
        Event::Caught { body } => format!("{} caught fire", name(body)),
        Event::Burnt { body } => format!("{} burnt to ash", name(body)),
        Event::Bought { who, item, price } => {
            format!("{} bought item {item} for {price} coin", name(who))
        }
        Event::Sold {
            who,
            item,
            count,
            price,
        } => format!("{} sold {count} of item {item} for {price} coin", name(who)),
        Event::Supplied { who, by } => format!(
            "{} supplied the holdout: the pressure fell {:.1}%",
            name(who),
            f64::from(by) / 10.0
        ),
        Event::Mended { who, item, price } => {
            format!("{} had item {item} mended for {price} coin", name(who))
        }
        Event::TollingTonight => "the bell will toll tonight".to_string(),
        Event::TollingBegan { at } => {
            format!("the Tolling: the bell at ({}, {}) tolls", at.x, at.y)
        }
        Event::Tolled { .. } => "the bell tolls".to_string(),
        Event::TollingEnded => "dawn: the Tolling is over".to_string(),
        Event::Risen { body, revenant } => format!(
            "{}'s body rose as a revenant: {}",
            w.name_of(body).unwrap_or_default(),
            name(revenant)
        ),
        Event::Identified { who, item } => format!("{} learnt what item {item} is", name(who)),
        Event::CurseClings { who, item } => {
            format!("item {item} clings to {}'s hand: cursed", name(who))
        }
        Event::CurseLifted { who, item } => {
            format!("the curse on {}'s item {item} was lifted", name(who))
        }
        Event::TalentChosen { who, talent } => format!(
            "{} chose the talent {}",
            name(who),
            w.defs
                .talents
                .get(usize::from(talent))
                .map_or("?", |t| t.name.as_str())
        ),
        Event::SecondWind { who } => format!("{} caught a second wind", name(who)),
        Event::RainBegan { heavy } => if heavy {
            "it rains hard"
        } else {
            "it starts to rain"
        }
        .to_string(),
        Event::RainEnded => "the rain stops".to_string(),
        Event::RainedOut { who, item } => format!("the rain put out {}'s item {item}", name(who)),
        Event::LightEaten { by, who } => {
            format!("{} put out {}'s light", name(by), name(who))
        }
        Event::BountyClaimed { who, rare, coin } => format!(
            "{} claimed the bounty on {} for {coin} coin",
            name(who),
            w.name_of(rare).unwrap_or_default()
        ),
        Event::Came { who } => {
            format!("{} came into the world", w.name_of(who).unwrap_or_default())
        }
    }
}

fn bench(args: &[String]) {
    let dead: usize = flag(args, "--dead").unwrap_or(100);
    let ticks: u32 = flag(args, "--ticks").unwrap_or(20_000);
    // 1 matches the Python benchmark, which thinks every tick; the game's default is 5.
    let think: u64 = flag(args, "--think-every").unwrap_or(1);
    let mut w = open_field(96, dead, 1);
    w.creature_mut(0).think_every = think;
    w.creature_mut(0).sight = 15;
    let me = w.character().expect("placed");
    // --tolling: the bell tolls a game minute in, in a corner of the field, and draws them all.
    let tolling = args.iter().any(|a| a == "--tolling");
    if tolling {
        let start = (w.rules.clock.start + 1) % 1440;
        w.set_region(dusk_core::Region {
            tolling: dusk_core::Tolling {
                first: 1,
                every: 1,
                start,
                rings: 60,
                toll_every: 2,
                loudness: 40,
                reach: 140,
                door: 12_000,
            },
            bell: Some(TilePos::new(8, 8, 0)),
            ..dusk_core::Region::default()
        });
    }
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
    let during = if tolling { ", the Tolling on" } else { "" };
    println!(
        "{dead} dead on a 96 x 96 map, thinking every {think} tick(s){during}: {tps:.0} ticks/s = {:.0}x real time",
        tps / f64::from(TICKS_PER_SECOND)
    );
}

/// A magic item as it would be told once known: its name, its rarity, and a curse.
fn told(world: &World, item: &dusk_core::Item) -> String {
    let mut known = *item;
    known.magic.known = true;
    let rarity = if known.magic.rarity >= 2 {
        "Rare"
    } else {
        "Uncommon"
    };
    let curse = if known.magic.cursed() { ", cursed" } else { "" };
    format!("{} ({rarity}{curse})", world.defs.label(&known))
}

/// The patch's first Tolling: run to it, then two game hours of it.
fn tolling(world: &World) {
    let (Some(bell), Some(begins)) = (world.region.bell, world.next_tolling()) else {
        println!("  no Tolling in this region");
        return;
    };
    let t = world.region.tolling;
    let (day, minute) = (begins / 1440 + 1, begins % 1440);
    let mut w = world.clone();
    let per = u64::from(w.rules.clock.ticks_per_minute);
    w.step(((begins - w.clock_minutes()) * per) as u32);
    let reach = i64::from(t.reach).pow(2);
    let drawn = |w: &World, near: i64| {
        w.actors()
            .filter(|(_, a)| !a.kind.is_character() && !a.dead)
            .filter(|(_, a)| a.pos.dist2(bell) <= near)
            .count()
    };
    let in_reach = drawn(&w, reach);
    let start = Instant::now();
    let ticks = (120 * per) as u32;
    w.step(ticks);
    let tps = f64::from(ticks) / start.elapsed().as_secs_f64();
    println!(
        "  the Tolling: day {day} at {:02}:{:02}, then every {} days; the bell at ({}, {})",
        minute / 60,
        minute % 60,
        t.every,
        bell.x,
        bell.y
    );
    println!(
        "  {in_reach} of the dead within its reach; two hours on, {} within 6 tiles of it and {} within 15 ({tps:.0} ticks/s)",
        drawn(&w, 36),
        drawn(&w, 225)
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
                    opaque: rock,
                    ..Tile::GRASS
                },
            );
        }
    }
    let centre = TilePos::new(size / 2, size / 2, 0);
    w.grid.set_tile(centre, Tile::GRASS);
    // As a patch is once it's made: each tile's open steps kept for the searches.
    w.grid.index_steps();
    // You don't die in the benchmark, of wounds or by being dragged down: a dead character no
    // longer keeps the world awake, so the dead would stand still and the figure mean nothing.
    w.rules.combat.character_health = 1_000_000_000;
    w.rules.combat.drag_down = u32::MAX;
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
        if w.spawn(ActorKind::Creature(0), pos).is_some() {
            placed += 1;
        }
    }
    w
}
