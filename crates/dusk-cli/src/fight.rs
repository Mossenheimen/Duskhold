//! `dusk-cli fight`: the game's own data, a character of a class on an open field in daylight, a
//! few of the dead coming at it from round about, many times over; how often it puts them all
//! down, how long that takes, and what it costs. For tuning the dead (`BACKLOG.md`, the second
//! playtest: "Enemies may also be overtuned. I struggled to kill a few in a row").
//!
//! The character stands and fights: whenever it's free, it swings at the one on it that's
//! winding up a blow, else the first on it; held (only with `--grabs`), it struggles. With
//! `--cast` (spells by id, `firebolt,fire_nova`), it casts first when it can: a nova with two or
//! more on it, the others at the nearest (never a fireball that would catch it). It drinks no
//! potion and binds no wound: a fair, plain player's worst case, not their best.

use dusk_core::{ActorId, ActorKind, Dir8, Event, Intent, SpellKind, Tile, TilePos, World};

use crate::{flag, load_data};

/// How the fights went, summed over the trials.
#[derive(Default)]
struct Tally {
    won: u32,
    seconds: f64,
    health_left: f64,
    worst: Option<f64>,
    blows: u32,
    landed: u32,
    grabs: u32,
    dragged: u32,
    breathless: u32,
    casts: u32,
}

pub fn fight(args: &[String]) {
    let data = load_data();
    let dead: usize = flag(args, "--dead").unwrap_or(3);
    let trials: u64 = flag(args, "--trials").unwrap_or(200);
    let kind_id: String = flag(args, "--kind").unwrap_or_else(|| "shambler".into());
    let class_id: String = flag(args, "--class").unwrap_or_else(|| "fighter".into());
    let grabs = args.iter().any(|a| a == "--grabs");
    let cast: String = flag(args, "--cast").unwrap_or_default();
    let spells: Vec<u16> = cast
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|id| {
            data.defs
                .spells
                .iter()
                .position(|s| s.id == id)
                .map(|n| n as u16)
                .unwrap_or_else(|| {
                    eprintln!("no spell {id:?} in data/spells.toml");
                    std::process::exit(2);
                })
        })
        .collect();
    let Some(kind) = data.defs.creature_kind(&kind_id) else {
        eprintln!("no creature {kind_id:?} in data/creatures");
        std::process::exit(2);
    };
    let Some(class) = data.defs.classes.iter().position(|c| c.id == class_id) else {
        eprintln!("no class {class_id:?} in data/classes.toml");
        std::process::exit(2);
    };
    let mut t = Tally::default();
    for seed in 1..=trials {
        let mut w = World::with_defs(seed, data.rules, data.defs.clone());
        if grabs {
            // As they were before the second playtest (rules.toml's [grab], "It was").
            let c = &mut w.rules.combat;
            (c.grab, c.grab_per_extra, c.grab_tired, c.parting_grab) = (1500, 1500, 2000, 2500);
        }
        one_fight(&mut w, kind, class as u8, dead, &spells, &mut t);
    }
    let n = trials as f64;
    let won = f64::from(t.won).max(1.0);
    let weapon = {
        let mut w = World::with_defs(1, data.rules, data.defs.clone());
        w.grid.set_tile(TilePos::new(0, 0, 0), Tile::GRASS);
        let me = w.spawn(ActorKind::Character, TilePos::new(0, 0, 0));
        me.and_then(|me| {
            w.choose_class(me, class as u8);
            w.actor(me)?.hands.map(|h| w.defs.label(&h))
        })
        .unwrap_or_else(|| "fists".into())
    };
    println!(
        "{dead} {kind_id} against a {class_id} ({weapon}){}{}, {trials} fights: won {:.0}%, in {:.1} s; \
         health left {:.0} (worst {:.0}); blows at you {:.1}, landing {:.1}; out of breath in {:.0}%; \
         grabbed {:.1} times, dragged down in {:.0}%; spells cast {:.1}",
        if grabs { ", grabs on" } else { "" },
        if cast.is_empty() {
            String::new()
        } else {
            format!(", casting {cast}")
        },
        100.0 * f64::from(t.won) / n,
        t.seconds / won,
        t.health_left / won,
        t.worst.unwrap_or(0.0),
        f64::from(t.blows) / n,
        f64::from(t.landed) / n,
        100.0 * f64::from(t.breathless) / n,
        f64::from(t.grabs) / n,
        100.0 * f64::from(t.dragged) / n,
        f64::from(t.casts) / n,
    );
}

/// One fight: `dead` of `kind` on a ring four tiles out, heard and coming.
fn one_fight(
    w: &mut World,
    kind: ActorKind,
    class: u8,
    dead: usize,
    spells: &[u16],
    t: &mut Tally,
) {
    const SIZE: i32 = 25;
    for y in 0..SIZE {
        for x in 0..SIZE {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    w.grid.index_steps();
    let centre = TilePos::new(SIZE / 2, SIZE / 2, 0);
    let me = w
        .spawn(ActorKind::Character, centre)
        .expect("the middle is clear");
    w.choose_class(me, class);
    let mut foes = Vec::new();
    for k in 0..dead {
        let angle = std::f64::consts::TAU * k as f64 / dead as f64;
        let at = TilePos::new(
            centre.x + (4.0 * angle.cos()).round() as i32,
            centre.y + (4.0 * angle.sin()).round() as i32,
            0,
        );
        if let Some(id) = w.spawn(kind, at) {
            foes.push(id);
        }
    }
    w.make_sound(centre, 12, Some(me));
    let mut breathless = false;
    for tick in 1..=1200u32 {
        if let Some(intent) = choose(w, me, &foes, spells) {
            w.intent(me, intent);
        }
        w.step(1);
        for e in w.events().collect::<Vec<_>>() {
            match e {
                // Every blow the dead swing is at you: landed, or missed.
                Event::Struck { target, .. } if target == me => {
                    t.blows += 1;
                    t.landed += 1;
                }
                Event::Missed { who, .. } if foes.contains(&who) => t.blows += 1,
                Event::Grabbed { target, .. } if target == me => t.grabs += 1,
                Event::DraggedDown { who, .. } if who == me => t.dragged += 1,
                Event::Cast { who, .. } if who == me => t.casts += 1,
                _ => {}
            }
        }
        let a = w.actor(me).expect("still there");
        breathless |= a.body.exhausted;
        if a.dead {
            break;
        }
        if foes.iter().all(|&f| w.actor(f).is_none_or(|f| f.dead)) {
            let health = f64::from(a.body.health) / 10.0;
            t.won += 1;
            t.seconds += f64::from(tick) / 10.0;
            t.health_left += health;
            t.worst = Some(t.worst.map_or(health, |w: f64| w.min(health)));
            break;
        }
    }
    t.breathless += u32::from(breathless);
}

/// What a plain player does now, if anything: a spell when one can be cast (a nova with two or
/// more on it, the rest at the nearest), else swing at the one about to strike, else the first on
/// it; struggle when held.
fn choose(w: &World, me: ActorId, foes: &[ActorId], spells: &[u16]) -> Option<Intent> {
    let a = w.actor(me)?;
    if a.dead || a.action.is_some() || matches!(a.intent, Intent::Cast { .. }) {
        return None;
    }
    if w.is_held(me) {
        return Some(Intent::Struggle);
    }
    let on_me = w.engaged(me);
    let nearest = foes
        .iter()
        .filter_map(|&f| w.actor(f).filter(|f| !f.dead).map(|f| f.pos))
        .min_by_key(|p| p.dist2(a.pos));
    for &spell in spells {
        let Some(s) = w.spell(spell) else { continue };
        let at = match s.kind {
            SpellKind::Nova if on_me.len() >= 2 => a.pos,
            SpellKind::Nova => continue,
            _ => match nearest {
                Some(p) => p,
                None => continue,
            },
        };
        if w.can_cast(me, spell, at).is_ok() && !w.spell_catches(me, spell, at) {
            return Some(Intent::Cast { spell, at });
        }
    }
    let target = on_me
        .iter()
        .copied()
        .filter_map(|id| w.actor(id).map(|o| (id, o)))
        .max_by_key(|(id, o)| {
            let coming = o
                .action
                .filter(|act| act.winding_up())
                .map_or(0, |act| 1 + act.elapsed);
            (coming, std::cmp::Reverse(id.index()))
        })?;
    let (dx, dy) = (target.1.pos.x - a.pos.x, target.1.pos.y - a.pos.y);
    Some(Intent::Attack {
        dir: Dir8::nearest(dx, dy),
    })
}
