//! Duskhold's Godot extension: the core, inside the game.
//!
//! `DuskWorld` is a node that owns a `dusk_core::World` and ticks it at 10 ticks a second of game
//! time (times the speed). It hands GDScript what it needs to draw:
//!
//! - the map, a chunk at a time as the client asks: its tiles (ground and fixture) and the edges
//!   that aren't open;
//! - each frame: the actors' positions, smoothed between the last two ticks, their facing and
//!   what they know (the alert marks);
//! - each tick: which tiles the dead can see (their sight cones), the noises made, and what
//!   happened.
//!
//! GDScript sends the player's input back as intents. The client never decides anything.
//!
//! The core runs on the main thread here: a tick of a patch takes well under a millisecond.
//! `ARCHITECTURE.md` moves it onto its own thread once the world is big enough to need it.

use dusk_core::geom::CHUNK_SIZE;
use dusk_core::world::TICKS_PER_SECOND;
use dusk_core::{
    ActorId, ActorKind, Alert, ChunkPos, Dir8, Edge, Event, Ground, Intent, TilePos, World, demo,
};
use dusk_data::Data;
use godot::prelude::*;

struct DuskExtension;

#[gdextension]
unsafe impl ExtensionLibrary for DuskExtension {}

/// The most ticks one frame catches up on. After a long hitch the game slows down instead of
/// jumping ahead.
const MAX_TICKS_PER_FRAME: u32 = 20;

/// How far round the character the sight cones are worked out for the client, in tiles: about
/// what the camera shows zoomed out.
const WATCH_RANGE: i32 = 64;

/// The event kinds `take_events` reports.
const HEARD: i32 = 1;
const SAW: i32 = 2;
const REACHED: i32 = 3;
const NO_WAY: i32 = 4;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct DuskWorld {
    world: Option<World>,
    /// The game's data, loaded the first time a patch is made.
    data: Option<Data>,
    /// Game time not yet ticked, in seconds.
    pending: f64,
    /// While paused, nothing ticks; the player can still look and queue.
    #[var]
    paused: bool,
    /// Game speed: 1 is normal.
    #[var]
    #[init(val = 1.0)]
    speed: f64,
    /// Each actor's position at the last two ticks, for drawing in between.
    before: Vec<(f32, f32)>,
    after: Vec<(f32, f32)>,
    /// Noises made since GDScript last took them: x, y, and how far each carries.
    noises: Vec<Vector3>,
    /// What happened since GDScript last asked, four numbers an event (see `take_events`).
    events: Vec<i32>,
    /// Per tile, row by row: the most alert creature that can see it (see `watched`).
    watched: Vec<u8>,
    /// The map's size in tiles.
    size: (i32, i32),
}

fn positions(world: &World) -> Vec<(f32, f32)> {
    world.actors().map(|(_, a)| a.position()).collect()
}

/// An actor's index in the actor arrays, or -1.
fn index_of(world: &World, who: ActorId) -> i32 {
    world
        .actors()
        .position(|(id, _)| id == who)
        .map_or(-1, |i| i as i32)
}

fn alert_level(alert: Alert) -> u8 {
    match alert {
        Alert::Unaware => 0,
        Alert::Curious => 1,
        Alert::Hunting => 2,
    }
}

/// An event as (kind, who, x, y), or None for the ones GDScript gets another way (sounds).
fn event_row(world: &World, e: Event) -> Option<[i32; 4]> {
    let at = |id: ActorId| world.actor(id).map_or((-1, -1), |a| (a.pos.x, a.pos.y));
    let (kind, who, (x, y)) = match e {
        Event::Heard { who, at: p } => (HEARD, who, (p.x, p.y)),
        Event::Saw { who, target } => (SAW, who, at(target)),
        Event::Reached { who, target } => (REACHED, who, at(target)),
        Event::NoWay { who, to } => (NO_WAY, who, (to.x, to.y)),
        Event::Sound { .. } => return None,
    };
    Some([kind, index_of(world, who), x, y])
}

/// Which tiles the creatures near the character can see, by the same rule they hunt by
/// (`World::sees`). Each tile holds 0 if none can, else 1 + the alert level of the most alert
/// one that can.
fn watch(world: &World, size: (i32, i32), out: &mut Vec<u8>) {
    out.clear();
    out.resize((size.0 * size.1) as usize, 0);
    let Some(me) = world
        .character()
        .and_then(|id| world.actor(id))
        .map(|a| a.pos)
    else {
        return;
    };
    for (id, a) in world.actors() {
        let Some((range, _)) = world.sight(a.kind) else {
            continue;
        };
        if a.pos.steps_to(me) > WATCH_RANGE || !world.is_awake(a.pos) {
            continue;
        }
        let level = 1 + alert_level(a.alert());
        for y in (a.pos.y - range).max(0)..=(a.pos.y + range).min(size.1 - 1) {
            for x in (a.pos.x - range).max(0)..=(a.pos.x + range).min(size.0 - 1) {
                let i = (y * size.0 + x) as usize;
                if out[i] < level && world.sees(id, TilePos::new(x, y, a.pos.z)) {
                    out[i] = level;
                }
            }
        }
    }
}

fn ground_code(ground: Ground) -> i32 {
    match ground {
        Ground::Void => 0,
        Ground::Grass => 1,
        Ground::Dirt => 2,
        Ground::Road => 3,
        Ground::Water => 4,
        Ground::Floor => 5,
    }
}

fn edge_code(edge: Edge) -> i32 {
    match edge {
        Edge::Open => 0,
        Edge::Wall => 1,
        Edge::Door { open: false } => 2,
        Edge::Door { open: true } => 3,
        Edge::Window { open: false } => 4,
        Edge::Window { open: true } => 5,
        Edge::Fence => 6,
    }
}

#[godot_api]
impl INode for DuskWorld {
    fn process(&mut self, delta: f64) {
        if self.paused {
            return;
        }
        let Some(world) = self.world.as_mut() else {
            return;
        };
        let tick = 1.0 / f64::from(TICKS_PER_SECOND);
        self.pending += delta * self.speed;
        let mut ticks = 0;
        while self.pending >= tick && ticks < MAX_TICKS_PER_FRAME {
            self.pending -= tick;
            ticks += 1;
            world.step(1);
            let events: Vec<Event> = world.events().collect();
            for e in events {
                if let Event::Sound { at, loudness, .. } = e {
                    self.noises
                        .push(Vector3::new(at.x as f32, at.y as f32, loudness as f32));
                } else if let Some(row) = event_row(world, e) {
                    self.events.extend(row);
                }
            }
            self.before = std::mem::replace(&mut self.after, positions(world));
        }
        if ticks == MAX_TICKS_PER_FRAME {
            self.pending = 0.0;
        }
        if ticks > 0 {
            watch(world, self.size, &mut self.watched);
        }
    }
}

impl DuskWorld {
    fn start(&mut self, world: World, size: (i32, i32)) {
        self.size = size;
        self.after = positions(&world);
        self.before = self.after.clone();
        self.pending = 0.0;
        self.noises.clear();
        self.events.clear();
        watch(&world, self.size, &mut self.watched);
        self.world = Some(world);
    }
}

#[godot_api]
impl DuskWorld {
    /// Makes a patch of the Hushlands from a seed, from the data in `data/`. Returns "" when
    /// all's well, or what's wrong with the data.
    #[func]
    fn load_patch(&mut self, seed: i64) -> GString {
        if self.data.is_none() {
            match dusk_data::load(&dusk_data::default_dir()) {
                Ok(data) => self.data = Some(data),
                Err(e) => return GString::from(&e.to_string()),
            }
        }
        let Some(data) = &self.data else {
            return GString::new();
        };
        let Some(region) = data.region("hushlands") else {
            return GString::from("data/regions has no hushlands.toml");
        };
        let (world, layout) = dusk_gen::patch(seed as u64, data, region);
        self.start(world, (layout.width, layout.height));
        GString::new()
    }

    /// Loads the little demo town (build step 3's) instead.
    #[func]
    fn load_town(&mut self, seed: i64) {
        self.start(demo::town(seed as u64), (demo::TOWN_W, demo::TOWN_H));
    }

    /// The map's size in tiles.
    #[func]
    fn map_size(&self) -> Vector2i {
        Vector2i::new(self.size.0, self.size.1)
    }

    /// Tiles along a chunk's side.
    #[func]
    fn chunk_size(&self) -> i32 {
        CHUNK_SIZE
    }

    /// The fixtures' ids, by number: index n is fixture number n, and 0 is "" (nothing).
    #[func]
    fn fixture_names(&self) -> PackedStringArray {
        let mut names = vec![GString::new()];
        if let Some(data) = &self.data {
            names.extend(data.fixtures.iter().map(|f| GString::from(&f.id)));
        }
        names.into_iter().collect()
    }

    /// A chunk's tiles, row by row (32 × 32): each is its ground plus 16 × its fixture number.
    /// Ground 0 is nothing, 1 grass, 2 dirt, 3 road, 4 water, 5 a floor.
    #[func]
    fn chunk_tiles(&self, cx: i32, cy: i32) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let origin = ChunkPos { x: cx, y: cy }.origin(0);
        let mut out = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE) as usize);
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let t = world.grid.tile(origin.offset(x, y));
                out.push(ground_code(t.ground) + 16 * i32::from(t.fixture));
            }
        }
        PackedInt32Array::from(&out[..])
    }

    /// The edges a chunk's tiles own that aren't open, as (x, y, side, kind) quadruples in world
    /// tiles. Side 0 is the tile's north edge and 1 its west edge. Kind 1 is a wall, 2 a shut
    /// door, 3 an open door, 4 a shut window, 5 an open window, 6 a fence.
    #[func]
    fn chunk_edges(&self, cx: i32, cy: i32) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let origin = ChunkPos { x: cx, y: cy }.origin(0);
        let mut out = Vec::new();
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pos = origin.offset(x, y);
                for (side, edge) in [(0, world.grid.north(pos)), (1, world.grid.west(pos))] {
                    let kind = edge_code(edge);
                    if kind != 0 {
                        out.extend([pos.x, pos.y, side, kind]);
                    }
                }
            }
        }
        PackedInt32Array::from(&out[..])
    }

    /// Each actor's kind, in the same order as the positions: 0 the character, 1 a shambler.
    #[func]
    fn actor_kinds(&self) -> PackedInt32Array {
        self.world
            .iter()
            .flat_map(|w| w.actors())
            .map(|(_, a)| match a.kind {
                ActorKind::Character => 0,
                ActorKind::Shambler => 1,
            })
            .collect()
    }

    /// What each actor knows of its prey: 0 unaware, 1 curious (heard something: "?"), 2 hunting
    /// (sees its prey: "!").
    #[func]
    fn actor_alert(&self) -> PackedInt32Array {
        self.world
            .iter()
            .flat_map(|w| w.actors())
            .map(|(_, a)| i32::from(alert_level(a.alert())))
            .collect()
    }

    /// Each actor's facing, in eighths of a turn clockwise from north.
    #[func]
    fn actor_facing(&self) -> PackedInt32Array {
        self.world
            .iter()
            .flat_map(|w| w.actors())
            .map(|(_, a)| a.facing as i32)
            .collect()
    }

    /// Each actor's position in tiles, smoothed between the last two ticks.
    #[func]
    fn actor_positions(&self) -> PackedVector2Array {
        let tick = 1.0 / f64::from(TICKS_PER_SECOND);
        let t = (self.pending / tick).clamp(0.0, 1.0) as f32;
        self.before
            .iter()
            .zip(&self.after)
            .map(|(&(x0, y0), &(x1, y1))| Vector2::new(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t))
            .collect()
    }

    /// The character's index in the actor arrays, or -1.
    #[func]
    fn character_index(&self) -> i32 {
        let Some(world) = &self.world else { return -1 };
        world.character().map_or(-1, |me| index_of(world, me))
    }

    /// Which tiles the creatures near you can see, one byte a tile, row by row (`map_size().x` a
    /// row): 0 if none can, else 1 + the alert level of the most alert one that can. It's the
    /// rule they hunt by, so where it's clear, they can't see you.
    #[func]
    fn watched(&self) -> PackedByteArray {
        PackedByteArray::from(&self.watched[..])
    }

    /// The noises made since the last call: x, y, and how far each carries, in tiles.
    #[func]
    fn take_noises(&mut self) -> PackedVector3Array {
        let out = PackedVector3Array::from(&self.noises[..]);
        self.noises.clear();
        out
    }

    /// What happened since the last call, as (kind, who, x, y) quadruples. `who` is the actor's
    /// index in the actor arrays. Kind 1: it heard a noise at (x, y). 2: it saw its prey, at
    /// (x, y). 3: it reached its prey, at (x, y). 4: it found no way to (x, y).
    #[func]
    fn take_events(&mut self) -> PackedInt32Array {
        let out = PackedInt32Array::from(&self.events[..]);
        self.events.clear();
        out
    }

    /// Steers the character by screen direction: `right` +1 or -1, `up` +1 or -1. North is up
    /// and to the right on screen, so W walks north-west and W with D walks north. (0, 0) stops
    /// a keyboard walk but leaves a click-to-walk going.
    #[func]
    fn steer(&mut self, right: i32, up: i32, run: bool) {
        let Some(world) = self.world.as_mut() else {
            return;
        };
        let Some(me) = world.character() else { return };
        let walking_by_keys =
            matches!(world.actor(me).map(|a| a.intent), Some(Intent::Move { .. }));
        let intent = match Dir8::toward(right - up, -right - up) {
            Some(dir) => Intent::Move { dir, run },
            None if walking_by_keys => Intent::Idle,
            None => return,
        };
        world.intent(me, intent);
    }

    /// Walks the character to a tile, finding the way.
    #[func]
    fn walk_to(&mut self, x: i32, y: i32, run: bool) {
        let Some(world) = self.world.as_mut() else {
            return;
        };
        if let Some(me) = world.character() {
            world.intent(
                me,
                Intent::WalkTo {
                    to: TilePos::new(x, y, 0),
                    run,
                },
            );
        }
    }

    /// Ticks played so far.
    #[func]
    fn tick(&self) -> i64 {
        self.world.as_ref().map_or(0, |w| w.tick as i64)
    }
}
