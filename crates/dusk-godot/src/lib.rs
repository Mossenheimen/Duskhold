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
    ActionKind, ActorId, ActorKind, Alert, ChunkPos, Dir8, Edge, Event, Ground, Intent, Item,
    ItemKind, NoBoard, NoSleep, NoStash, Part, Rise, Skill, TilePos, View, World, WoundKind, demo,
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

/// A stack as the client gets it: definition, count, condition, and 1 if it's a find.
fn stack_row(i: Item) -> [i32; 4] {
    [
        i32::from(i.def),
        i32::from(i.count),
        i32::from(i.condition),
        i32::from(i.found),
    ]
}

/// The event kinds `take_events` reports.
const HEARD: i32 = 1;
const SAW: i32 = 2;
const REACHED: i32 = 3;
const NO_WAY: i32 = 4;
const STRUCK: i32 = 5;
const MISSED: i32 = 6;
const SHOVED: i32 = 7;
const DIED: i32 = 8;
const DOOR: i32 = 9;
const EXHAUSTED: i32 = 10;
const GRABBED: i32 = 11;
const FREED: i32 = 12;
const STRUGGLED: i32 = 13;
const THUMPED: i32 = 14;
const DOOR_BROKE: i32 = 15;
const DRAGGED_DOWN: i32 = 16;
const TOOK: i32 = 17;
const DROPPED: i32 = 18;
const WIELDED: i32 = 19;
const WEAPON_BROKE: i32 = 20;
const BANDAGED: i32 = 21;
const ATE: i32 = 22;
const DRANK: i32 = 23;
const DRANK_WATER: i32 = 24;
const FELL_ASLEEP: i32 = 25;
const WOKE: i32 = 26;
const LIT: i32 = 27;
const DOUSED: i32 = 28;
const BURNT_OUT: i32 = 29;
const NIGHTFALL: i32 = 30;
const DAYBREAK: i32 = 31;
const SKILL_UP: i32 = 32;
const SKILL_AT_CAP: i32 = 33;
const MASTERED: i32 = 34;
const BOARDED: i32 = 35;
const UNBOARDED: i32 = 36;
const BOARD_BROKE: i32 = 37;
const MADE_HOME: i32 = 38;
const STASHED: i32 = 39;
const PUT: i32 = 40;
const BROUGHT_HOME: i32 = 41;
const LEVEL_UP: i32 = 42;

/// Numbers an event takes in `take_events`.
const EVENT_SIZE: usize = 6;

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
    /// What happened since GDScript last asked, six numbers an event (see `take_events`).
    events: Vec<i32>,
    /// Per tile, row by row: the most alert creature that can see it (see `watched`).
    watched: Vec<u8>,
    /// What the character makes out of the map, and remembers (see `view_tiles`).
    #[init(val = View::new(0, 0))]
    view: View,
    /// The map's size in tiles.
    size: (i32, i32),
    /// Where the data files are; empty for the game's own `data/` (a mod or a test can point
    /// elsewhere). Read the first time a patch is made.
    #[var]
    data_dir: GString,
    /// The region the world is a patch of: what a save says to make again. Empty for a world
    /// that isn't saved (the demo town).
    region: String,
    /// The hour of day the game starts at, for development (`--hour`); -1 for the data's.
    #[var]
    #[init(val = -1)]
    start_hour: i32,
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
fn event_row(world: &World, e: Event) -> Option<[i32; EVENT_SIZE]> {
    let at = |id: ActorId| world.actor(id).map_or((-1, -1), |a| (a.pos.x, a.pos.y));
    let index = |id: ActorId| index_of(world, id);
    // Doors are named by the tile owning the edge, and the side: 0 north, 1 west.
    let side = |d: Dir8| i32::from(d == Dir8::W);
    // (kind, who, target, x, y, extra)
    let (kind, who, target, (x, y), extra) = match e {
        Event::Heard { who, at: p } => (HEARD, Some(who), None, (p.x, p.y), 0),
        Event::Saw { who, target } => (SAW, Some(who), Some(target), at(target), 0),
        Event::Reached { who, target } => (REACHED, Some(who), Some(target), at(target), 0),
        Event::NoWay { who, to } => (NO_WAY, Some(who), None, (to.x, to.y), 0),
        Event::Struck {
            who, target, wound, ..
        } => {
            let extra = wound.map_or(0, |(kind, part)| (kind as i32 + 1) * 100 + part as i32);
            (STRUCK, Some(who), Some(target), at(target), extra)
        }
        Event::Missed { who, at: p } => (MISSED, Some(who), None, (p.x, p.y), 0),
        Event::Shoved { who, target, down } => {
            (SHOVED, Some(who), Some(target), at(target), i32::from(down))
        }
        Event::Died { who } => (DIED, Some(who), None, at(who), 0),
        Event::Door {
            who, at: p, open, ..
        } => (DOOR, Some(who), None, (p.x, p.y), i32::from(open)),
        Event::Exhausted { who } => (EXHAUSTED, Some(who), None, at(who), 0),
        Event::Grabbed { who, target } => (GRABBED, Some(who), Some(target), at(target), 0),
        Event::Freed { who, from } => (FREED, Some(who), Some(from), at(who), 0),
        Event::StruggledInVain { who } => (STRUGGLED, Some(who), None, at(who), 0),
        Event::Thumped {
            who,
            at: p,
            side: s,
        } => (THUMPED, Some(who), None, (p.x, p.y), side(s)),
        Event::DoorBroke { at: p, side: s } => (DOOR_BROKE, None, None, (p.x, p.y), side(s)),
        Event::DraggedDown { who, holders } => {
            (DRAGGED_DOWN, Some(who), None, at(who), holders as i32)
        }
        // An item and how many: 1000 × its definition number + the count.
        Event::Took { who, item, count } => (
            TOOK,
            Some(who),
            None,
            at(who),
            i32::from(item) * 1000 + i32::from(count),
        ),
        Event::Dropped { who, item, count } => (
            DROPPED,
            Some(who),
            None,
            at(who),
            i32::from(item) * 1000 + i32::from(count),
        ),
        Event::Wielded { who, item } => (WIELDED, Some(who), None, at(who), i32::from(item)),
        Event::WeaponBroke { who, item } => {
            (WEAPON_BROKE, Some(who), None, at(who), i32::from(item))
        }
        Event::Bandaged { who, part } => (BANDAGED, Some(who), None, at(who), part as i32),
        Event::Ate { who, item } => (ATE, Some(who), None, at(who), i32::from(item)),
        Event::Drank { who, item } => (DRANK, Some(who), None, at(who), i32::from(item)),
        Event::DrankWater { who } => (DRANK_WATER, Some(who), None, at(who), 0),
        Event::FellAsleep { who, in_bed } => {
            (FELL_ASLEEP, Some(who), None, at(who), i32::from(in_bed))
        }
        // Why: 0 rested, 1 chose to get up, 2 hurt, 3 a noise.
        Event::WokeUp { who, why } => (WOKE, Some(who), None, at(who), why as i32),
        Event::Lit { who, item } => (LIT, Some(who), None, at(who), i32::from(item)),
        Event::Doused { who, item } => (DOUSED, Some(who), None, at(who), i32::from(item)),
        Event::BurntOut { who, item } => (BURNT_OUT, Some(who), None, at(who), i32::from(item)),
        Event::Nightfall => (NIGHTFALL, None, None, (-1, -1), 0),
        Event::Daybreak => (DAYBREAK, None, None, (-1, -1), 0),
        // A skill (by `skill_names`) and a level: 100 × the skill + the level.
        Event::SkillUp { who, skill, level } => (
            SKILL_UP,
            Some(who),
            None,
            at(who),
            skill as i32 * 100 + i32::from(level),
        ),
        Event::SkillAtCap { who, skill } => {
            (SKILL_AT_CAP, Some(who), None, at(who), skill as i32 * 100)
        }
        Event::Mastered { who, skill, level } => (
            MASTERED,
            Some(who),
            None,
            at(who),
            skill as i32 * 100 + i32::from(level),
        ),
        // A door or window, named by its owner tile (as doors are), and the boards left on it.
        Event::Boarded {
            who, at: p, boards, ..
        } => (BOARDED, Some(who), None, (p.x, p.y), i32::from(boards)),
        Event::Unboarded {
            who, at: p, boards, ..
        } => (UNBOARDED, Some(who), None, (p.x, p.y), i32::from(boards)),
        Event::BoardBroke { at: p, boards, .. } => {
            (BOARD_BROKE, None, None, (p.x, p.y), i32::from(boards))
        }
        Event::MadeHome { who, at: p } => (MADE_HOME, Some(who), None, (p.x, p.y), 0),
        Event::Stashed { who, at: p } => (STASHED, Some(who), None, (p.x, p.y), 0),
        Event::Put { who, item, count } => (
            PUT,
            Some(who),
            None,
            at(who),
            i32::from(item) * 1000 + i32::from(count),
        ),
        // In the place of a tile: the item and how many. The extra is the XP, in points.
        Event::BroughtHome {
            who,
            item,
            count,
            xp,
        } => (
            BROUGHT_HOME,
            Some(who),
            None,
            (i32::from(item), i32::from(count)),
            (xp / 1000) as i32,
        ),
        Event::LevelUp { who, level } => (LEVEL_UP, Some(who), None, at(who), i32::from(level)),
        Event::Sound { .. } => return None,
    };
    Some([
        kind,
        who.map_or(-1, index),
        target.map_or(-1, index),
        x,
        y,
        extra,
    ])
}

/// Which tiles the creatures the character sees can see, by the same rule they hunt by
/// (`World::sees`). Each tile holds 0 if none can, else 1 + the alert level of the most alert
/// one that can. Those the character can't see draw nothing: you don't know where they look.
fn watch(world: &World, size: (i32, i32), view: &View, out: &mut Vec<u8>) {
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
        if a.pos.steps_to(me) > WATCH_RANGE || !world.is_awake(a.pos) || !view.sees(a.pos) {
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

/// An edge's kind (`chunk_edges`), plus 16 for each board nailed across it.
fn edge_code(edge: Edge) -> i32 {
    let kind = match edge {
        Edge::Open => 0,
        Edge::Wall => 1,
        Edge::Door { open: false, .. } => 2,
        Edge::Door { open: true, .. } => 3,
        Edge::Window { open: false, .. } => 4,
        Edge::Window { open: true, .. } => 5,
        Edge::Fence => 6,
    };
    kind + 16 * i32::from(edge.boards())
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
            if let Some(me) = world.character() {
                world.look(me, &mut self.view);
            }
            watch(world, self.size, &self.view, &mut self.watched);
        }
    }
}

impl DuskWorld {
    /// Asks the character for a timed action, unless it's dead.
    fn act(&mut self, intent: Intent) {
        let Some(world) = self.world.as_mut() else {
            return;
        };
        if let Some(me) = world.character()
            && world.actor(me).is_some_and(|a| !a.dead)
        {
            world.intent(me, intent);
        }
    }

    /// Loads the game's data, the first time it's needed.
    fn load_data(&mut self) -> Result<(), String> {
        if self.data.is_none() {
            let dir = if self.data_dir.is_empty() {
                dusk_data::default_dir()
            } else {
                std::path::PathBuf::from(self.data_dir.to_string())
            };
            self.data = Some(dusk_data::load(&dir).map_err(|e| e.to_string())?);
        }
        Ok(())
    }

    fn start(&mut self, mut world: World, size: (i32, i32)) {
        if (0..24).contains(&self.start_hour) {
            world.rules.clock.start = self.start_hour as u32 * 60;
        }
        self.size = size;
        self.after = positions(&world);
        self.before = self.after.clone();
        self.pending = 0.0;
        self.noises.clear();
        self.events.clear();
        self.view = View::new(size.0, size.1);
        if let Some(me) = world.character() {
            world.look(me, &mut self.view);
        }
        watch(&world, self.size, &self.view, &mut self.watched);
        self.world = Some(world);
    }
}

#[godot_api]
impl DuskWorld {
    /// Makes a patch of the Hushlands from a seed, from the data in `data/`. Returns "" when
    /// all's well, or what's wrong with the data.
    #[func]
    fn load_patch(&mut self, seed: i64) -> GString {
        if let Err(problem) = self.load_data() {
            return GString::from(&problem);
        }
        let Some(data) = &self.data else {
            return GString::new();
        };
        let Some(region) = data.region("hushlands") else {
            return GString::from("data/regions has no hushlands.toml");
        };
        let (world, layout) = dusk_gen::patch(seed as u64, data, region);
        self.region = region.id.clone();
        self.start(world, (layout.width, layout.height));
        GString::new()
    }

    /// Saves the game to a file (`dusk_core::save`): the seed, and what play has changed, with
    /// the map you remember. Written beside it first and then moved into place, so a crash
    /// half-way leaves the last save whole. Returns "" when all's well, or what went wrong.
    #[func]
    fn save_game(&self, path: GString) -> GString {
        let (Some(world), Some(data)) = (&self.world, &self.data) else {
            return GString::from("there's no world to save");
        };
        let Some(region) = data.region(&self.region) else {
            return GString::from("this world isn't one that saves");
        };
        let bytes = dusk_gen::save(world, region, &self.view.remembered());
        let path = std::path::PathBuf::from(path.to_string());
        let write = || -> std::io::Result<()> {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let beside = path.with_extension("tmp");
            std::fs::write(&beside, &bytes)?;
            std::fs::rename(&beside, &path)
        };
        match write() {
            Ok(()) => GString::new(),
            Err(e) => GString::from(&e.to_string()),
        }
    }

    /// Loads a saved game: its patch made afresh from its seed, then what the save holds put
    /// onto it. Returns "" when all's well, or why not (and the world, if any, is as it was).
    #[func]
    fn load_game(&mut self, path: GString) -> GString {
        if let Err(problem) = self.load_data() {
            return GString::from(&problem);
        }
        let bytes = match std::fs::read(path.to_string()) {
            Ok(bytes) => bytes,
            Err(e) => return GString::from(&e.to_string()),
        };
        let Some(data) = &self.data else {
            return GString::new();
        };
        let loaded = match dusk_gen::load(data, &bytes) {
            Ok(loaded) => loaded,
            Err(e) => return GString::from(&e.to_string()),
        };
        self.region = loaded.layout.region.id.clone();
        self.start(loaded.world, (loaded.layout.width, loaded.layout.height));
        // What you'd seen before, then what you see now.
        self.view.recall(&loaded.memory);
        if let Some(world) = &self.world {
            if let Some(me) = world.character() {
                world.look(me, &mut self.view);
            }
            watch(world, self.size, &self.view, &mut self.watched);
        }
        GString::new()
    }

    /// The world's seed (a loaded game's is its save's).
    #[func]
    fn world_seed(&self) -> i64 {
        self.world.as_ref().map_or(0, |w| w.seed as i64)
    }

    /// Loads the little demo town (build step 3's) instead.
    #[func]
    fn load_town(&mut self, seed: i64) {
        self.region.clear();
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
    /// door, 3 an open door, 4 a shut window, 5 an open window, 6 a fence; plus 16 for each board
    /// nailed across it (kind & 15 is the kind, kind >> 4 the boards).
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

    /// The creatures' names, by their number in the data.
    #[func]
    fn creature_names(&self) -> PackedStringArray {
        let Some(world) = &self.world else {
            return PackedStringArray::new();
        };
        let names = world.defs.creatures.iter();
        names.map(|c| GString::from(c.name.as_str())).collect()
    }

    /// Each actor's kind, in the same order as the positions: 0 the character, and 1 + n for
    /// creature n of `creature_names`.
    #[func]
    fn actor_kinds(&self) -> PackedInt32Array {
        self.world
            .iter()
            .flat_map(|w| w.actors())
            .map(|(_, a)| match a.kind {
                ActorKind::Character => 0,
                ActorKind::Creature(n) => 1 + i32::from(n),
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

    /// What happened since the last call, six numbers an event: (kind, who, target, x, y,
    /// extra). `who` and `target` are indices in the actor arrays (-1: none).
    ///
    /// - 1: `who` heard a noise at (x, y). 2: it saw `target`. 3: it reached `target`.
    /// - 4: `who` found no way to (x, y).
    /// - 5: `who` hit `target`; extra is 0, or a wound: 100 × (1 + its kind in `wound_names`)
    ///   + its part in `part_names`.
    /// - 6: `who` swung at (x, y) and missed. 7: it shoved `target`; extra 1 if it fell.
    /// - 8: `who` died. 9: it opened (extra 1) or shut a door; (x, y) is the tile owning the
    ///   edge. 10: it ran out of breath.
    /// - 11: `who` took hold of `target`. 12: `who` got free of `target`. 13: `who` struggled in
    ///   vain.
    /// - 14: `who` beat on a shut door, and 15: a door gave way; (x, y) is the tile owning the
    ///   edge, extra its side (0 north, 1 west).
    /// - 16: `who` was dragged down by extra of them.
    /// - 17: `who` took, and 18: dropped, an item: extra is 1000 × its definition number + how
    ///   many. 19: `who` took a weapon in hand, and 20: its weapon broke; extra is the item.
    /// - 21: `who` bandaged a wound; extra is the part, in `part_names`.
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

    /// Swings at what's beside you, toward (dx, dy) in tiles; (0, 0) at the nearest of the dead
    /// you can reach, else straight ahead.
    #[func]
    fn attack(&mut self, dx: i32, dy: i32) {
        self.act(Intent::Attack {
            dir: Dir8::toward(dx, dy),
        });
    }

    /// Shoves what's beside you, aimed as for `attack`.
    #[func]
    fn shove(&mut self, dx: i32, dy: i32) {
        self.act(Intent::Shove {
            dir: Dir8::toward(dx, dy),
        });
    }

    /// Opens or shuts the door beside you.
    #[func]
    fn use_door(&mut self) {
        self.act(Intent::Door);
    }

    /// The items' names, by definition number (what the item arrays and events count by).
    #[func]
    fn item_names(&self) -> PackedStringArray {
        let Some(world) = &self.world else {
            return PackedStringArray::new();
        };
        world
            .defs
            .items
            .iter()
            .map(|i| GString::from(&i.name))
            .collect()
    }

    /// Each item's kind, by definition number: 0 other, 1 a weapon, 2 a bandage, 3 food,
    /// 4 drink, 5 a light.
    #[func]
    fn item_kinds(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        world
            .defs
            .items
            .iter()
            .map(|i| match i.kind {
                ItemKind::Other => 0,
                ItemKind::Weapon { .. } => 1,
                ItemKind::Bandage { .. } => 2,
                ItemKind::Food { .. } => 3,
                ItemKind::Drink { .. } => 4,
                ItemKind::Light { .. } => 5,
            })
            .collect()
    }

    /// Each item's weight in grams, by definition number.
    #[func]
    fn item_weights(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        world.defs.items.iter().map(|i| i.weight as i32).collect()
    }

    /// What's in your hand: [definition, condition out of 1000], or nothing.
    #[func]
    fn character_hands(&self) -> PackedInt32Array {
        let held = self
            .world
            .as_ref()
            .and_then(|w| w.character().and_then(|id| w.actor(id)))
            .and_then(|a| a.hands);
        match held {
            Some(h) => PackedInt32Array::from(&[i32::from(h.def), i32::from(h.condition)]),
            None => PackedInt32Array::new(),
        }
    }

    /// Your pack, in order (the slots the item intents name): (definition, count, condition,
    /// 1 if it's a find not yet brought home) four numbers each.
    #[func]
    fn character_items(&self) -> PackedInt32Array {
        let Some(a) = self
            .world
            .as_ref()
            .and_then(|w| w.character().and_then(|id| w.actor(id)))
        else {
            return PackedInt32Array::new();
        };
        let out: Vec<i32> = a.items.iter().flat_map(|&i| stack_row(i)).collect();
        PackedInt32Array::from(&out[..])
    }

    /// [grams you carry, grams you can carry before you're weighed down and can't run].
    #[func]
    fn character_load(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let Some(me) = world.character() else {
            return PackedInt32Array::new();
        };
        PackedInt32Array::from(&[world.load(me) as i32, world.defs.carry as i32])
    }

    /// The containers and floors within your reach, your own tile first: for each, x, y, its
    /// fixture number (0 a bare floor), how many stacks, 1 if it's your stash, then
    /// (definition, count, condition, found) four numbers a stack.
    #[func]
    fn stores_in_reach(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let Some(me) = world.character() else {
            return PackedInt32Array::new();
        };
        let mut out = Vec::new();
        let stash = world.actor(me).and_then(|a| a.stash);
        for (at, fixture, held) in world.stores_in_reach(me) {
            let mine = i32::from(stash == Some(at));
            out.extend([at.x, at.y, i32::from(fixture), held.len() as i32, mine]);
            for i in held {
                out.extend(stack_row(i));
            }
        }
        PackedInt32Array::from(&out[..])
    }

    /// The tiles with something lying on the floor, as (x, y) pairs.
    #[func]
    fn floor_items(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let out: Vec<i32> = world.floors().iter().flat_map(|p| [p.x, p.y]).collect();
        PackedInt32Array::from(&out[..])
    }

    /// Takes the stack at `slot` of what's on (x, y); `def` is what you saw there.
    #[func]
    fn take(&mut self, x: i32, y: i32, slot: i32, def: i32) {
        self.act(Intent::Take {
            at: TilePos::new(x, y, 0),
            slot: slot as u16,
            def: def as u16,
        });
    }

    /// Drops the stack at `slot` of your pack.
    #[func]
    fn drop_item(&mut self, slot: i32, def: i32) {
        self.act(Intent::Drop {
            slot: slot as u16,
            def: def as u16,
        });
    }

    /// Uses the item at `slot` of your pack: puts a bandage on, eats food, drinks a drink.
    #[func]
    fn use_item(&mut self, slot: i32, def: i32) {
        self.act(Intent::Use {
            slot: slot as u16,
            def: def as u16,
        });
    }

    /// Takes the weapon at `slot` of your pack in hand.
    #[func]
    fn wield(&mut self, slot: i32, def: i32) {
        self.act(Intent::Wield {
            slot: slot as u16,
            def: def as u16,
        });
    }

    /// Puts what's in your hand into your pack.
    #[func]
    fn unwield(&mut self) {
        self.act(Intent::Unwield);
    }

    /// Fights free of what holds you.
    #[func]
    fn struggle(&mut self) {
        self.act(Intent::Struggle);
    }

    /// Whether you could shove now. Held, it takes a free hand: with a weapon in one and a light
    /// held up in the other, there's only the struggle.
    #[func]
    fn can_shove(&self) -> bool {
        let Some(world) = &self.world else {
            return false;
        };
        world.character().is_some_and(|me| world.can_shove(me))
    }

    /// The dead on you, two numbers each: the actor's index, and 1 if it has hold of you. The
    /// client draws a line from each (`CONCEPT.md`: engagement you can see).
    #[func]
    fn character_threats(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let Some(me) = world.character() else {
            return PackedInt32Array::new();
        };
        let mut on_me = world.engaged(me);
        for holder in world.holders(me) {
            if !on_me.contains(&holder) {
                on_me.push(holder);
            }
        }
        on_me.sort();
        let mut out = Vec::new();
        for id in on_me {
            let holds = world.actor(id).is_some_and(|a| a.holding == Some(me));
            out.extend([index_of(world, id), i32::from(holds)]);
        }
        PackedInt32Array::from(&out[..])
    }

    /// Each actor's state: 0 standing, 1 knocked down, 2 dead, 3 asleep.
    #[func]
    fn actor_states(&self) -> PackedInt32Array {
        self.world
            .iter()
            .flat_map(|w| w.actors())
            .map(|(_, a)| match (a.dead, a.prone_until > 0, a.body.asleep) {
                (true, _, _) => 2,
                (false, true, _) => 1,
                (false, false, true) => 3,
                (false, false, false) => 0,
            })
            .collect()
    }

    /// Each actor's timed action, four numbers an actor: its kind (0 none, 1 a swing, 2 a shove,
    /// 3 a door, 4 a blow at a shut door, 5 a struggle, 6 taking, 7 dropping, 8 bandaging,
    /// 9 taking a weapon in hand, 10 putting it away, 11 lying down to sleep, 12 drinking from
    /// water, 13 eating, 14 drinking a drink, 15 lighting a light, 16 putting it out, 17 boarding
    /// up, 18 prying a board off), how far through its wind-up it is (0 to 1000; 1000 once it
    /// has struck), and the tile it's aimed at. The client draws the wind-up of a swing or a shove
    /// as its tell.
    #[func]
    fn actor_actions(&self) -> PackedInt32Array {
        let mut out = Vec::new();
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        for (_, a) in world.actors() {
            let at = a.pos.step(a.action.map_or(a.facing, |act| act.dir));
            out.extend(match a.action {
                None => [0, 0, at.x, at.y],
                Some(act) => [
                    match act.kind {
                        ActionKind::Attack
                            if act.dir.split().is_none()
                                && world.grid.edge(a.pos, act.dir).is_shut_door() =>
                        {
                            4
                        }
                        ActionKind::Attack => 1,
                        ActionKind::Shove => 2,
                        ActionKind::Door => 3,
                        ActionKind::Struggle => 5,
                        ActionKind::Take { .. } => 6,
                        ActionKind::Drop { .. } => 7,
                        ActionKind::Use { def, .. } => {
                            match world.defs.item(def).map(|d| &d.kind) {
                                Some(ItemKind::Food { .. }) => 13,
                                Some(ItemKind::Drink { .. }) => 14,
                                _ => 8,
                            }
                        }
                        ActionKind::Wield { .. } => 9,
                        ActionKind::Unwield => 10,
                        ActionKind::Light { .. } => 15,
                        ActionKind::Douse => 16,
                        ActionKind::Sleep => 11,
                        ActionKind::DrinkWater => 12,
                        ActionKind::Board => 17,
                        ActionKind::Unboard => 18,
                        ActionKind::Stash { .. } => 19,
                        ActionKind::Put { .. } => 20,
                    },
                    (act.elapsed * 1000 / act.strike.max(1)).min(1000) as i32,
                    at.x,
                    at.y,
                ],
            });
        }
        PackedInt32Array::from(&out[..])
    }

    /// Your body: [health, endurance (both out of 1000), exhausted (0 or 1), dead (0 or 1),
    /// wounds bleeding, how many have hold of you].
    #[func]
    fn character_body(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let Some((me, a)) = world
            .character()
            .and_then(|id| world.actor(id).map(|a| (id, a)))
        else {
            return PackedInt32Array::new();
        };
        let b = &a.body;
        let bleeding = b.wounds.iter().filter(|w| w.bleeding > 0).count() as i32;
        PackedInt32Array::from(&[
            b.health.max(0),
            b.endurance,
            i32::from(b.exhausted),
            i32::from(a.dead),
            bleeding,
            world.holders(me).len() as i32,
        ])
    }

    /// Your wounds, worst first, as lines for the health panel ("bite, left arm, bleeding").
    #[func]
    fn character_wounds(&self) -> PackedStringArray {
        let Some(a) = self
            .world
            .as_ref()
            .and_then(|w| w.character().and_then(|id| w.actor(id)))
        else {
            return PackedStringArray::new();
        };
        let mut wounds = a.body.wounds.clone();
        wounds.sort_by_key(|w| (std::cmp::Reverse(w.kind), w.part));
        wounds
            .iter()
            .map(|w| {
                let state = match (w.bleeding > 0, w.bandaged) {
                    (true, _) => ", bleeding",
                    (false, true) => ", bandaged",
                    (false, false) => "",
                };
                GString::from(&format!("{}, {}{state}", w.kind.name(), w.part.name()))
            })
            .collect()
    }

    /// The names of the kinds of wound and of the parts of the body, in the order `take_events`
    /// counts them.
    #[func]
    fn wound_names(&self) -> PackedStringArray {
        WoundKind::ALL
            .iter()
            .map(|k| GString::from(k.name()))
            .collect()
    }

    #[func]
    fn part_names(&self) -> PackedStringArray {
        Part::ODDS
            .iter()
            .map(|(p, _)| GString::from(p.name()))
            .collect()
    }

    /// The clock: [day (from 1), hour, minute, game minutes since the start].
    #[func]
    fn clock(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::from(&[1, 0, 0, 0]);
        };
        let (day, hour, minute) = world.clock();
        let played = world.clock_minutes() - u64::from(world.rules.clock.start);
        PackedInt32Array::from(&[day as i32, hour as i32, minute as i32, played as i32])
    }

    /// Your moodles, two numbers each: which (0 hungry, 1 thirsty, 2 tired, 3 injured,
    /// 4 bleeding, 5 out of breath, 6 heavy load, 7 held, 8 asleep) and its level (1 up).
    #[func]
    fn character_moodles(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let Some(me) = world.character() else {
            return PackedInt32Array::new();
        };
        world
            .moodles(me)
            .into_iter()
            .flat_map(|(m, level)| [m as i32, i32::from(level)])
            .collect()
    }

    /// Whether you can lie down to sleep now: 1 beside a bed, 0 on the ground, or why not:
    /// -1 not tired, -2 the dead are about, -3 something has hold of you.
    #[func]
    fn sleep_check(&self) -> i32 {
        let Some(world) = &self.world else {
            return -1;
        };
        let Some(me) = world.character() else {
            return -1;
        };
        match world.can_sleep(me) {
            Ok(in_bed) => i32::from(in_bed),
            Err(NoSleep::NotTired) => -1,
            Err(NoSleep::DeadAbout) => -2,
            Err(NoSleep::Held) => -3,
        }
    }

    /// Lies down to sleep (see `sleep_check`).
    #[func]
    fn sleep(&mut self) {
        self.act(Intent::Sleep);
    }

    /// Gets up.
    #[func]
    fn wake(&mut self) {
        self.act(Intent::Wake);
    }

    /// Whether there's water within your reach to drink from.
    #[func]
    fn water_in_reach(&self) -> bool {
        let Some(world) = &self.world else {
            return false;
        };
        world
            .character()
            .and_then(|id| world.actor(id))
            .is_some_and(|a| world.water_beside(a.pos).is_some())
    }

    /// Drinks your fill from the water beside you.
    #[func]
    fn drink_water(&mut self) {
        self.act(Intent::DrinkWater);
    }

    /// The sky's light now, out of 100: the client lights the scene by it.
    #[func]
    fn daylight(&self) -> i32 {
        self.world.as_ref().map_or(100, |w| w.daylight() as i32)
    }

    /// What the character makes out of the map, per tile, row by row: 0 never seen,
    /// 1 remembered, 2 seen now. As of the last tick.
    #[func]
    fn view_tiles(&self) -> PackedByteArray {
        PackedByteArray::from(&self.view.tiles[..])
    }

    /// Per actor: 1 if the character sees it (it's on a tile seen now), else 0. The character
    /// always sees itself.
    #[func]
    fn actor_seen(&self) -> PackedByteArray {
        let Some(world) = &self.world else {
            return PackedByteArray::new();
        };
        let me = world.character();
        world
            .actors()
            .map(|(id, a)| u8::from(Some(id) == me || self.view.sees(a.pos)))
            .collect()
    }

    /// The light you hold up: [its definition number, its condition (out of 1000), its radius in
    /// tiles], or nothing.
    #[func]
    fn character_light(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let Some(lit) = world
            .character()
            .and_then(|id| world.actor(id))
            .and_then(|a| a.light)
        else {
            return PackedInt32Array::new();
        };
        let radius = match world.defs.item(lit.def).map(|d| &d.kind) {
            Some(ItemKind::Light { radius, .. }) => *radius,
            _ => 0,
        };
        PackedInt32Array::from(&[i32::from(lit.def), i32::from(lit.condition), radius])
    }

    /// Lights the torch or candle at `slot` of your pack and holds it up.
    #[func]
    fn light(&mut self, slot: i32, def: i32) {
        self.act(Intent::Light {
            slot: slot as u16,
            def: def as u16,
        });
    }

    /// Puts out the light you hold up.
    #[func]
    fn douse(&mut self) {
        self.act(Intent::Douse);
    }

    /// Whether you can nail a board on (or, `prying`, pry one off) now: the boards on that door
    /// or window if so, else why not: -1 no door or window beside you (to pry: none boarded),
    /// -2 the door is open, -3 it has all the boards it takes, -4 no hammer, -5 no plank,
    /// -6 not enough nails.
    #[func]
    fn board_check(&self, prying: bool) -> i32 {
        let Some(world) = &self.world else { return -1 };
        let Some(me) = world.character() else {
            return -1;
        };
        let check = if prying {
            world.can_unboard(me)
        } else {
            world.can_board(me)
        };
        match check {
            Ok(side) => world
                .actor(me)
                .map_or(0, |a| i32::from(world.grid.edge(a.pos, side).boards())),
            Err(NoBoard::Nothing) => -1,
            Err(NoBoard::Open) => -2,
            Err(NoBoard::Full) => -3,
            Err(NoBoard::NoHammer) => -4,
            Err(NoBoard::NoPlank) => -5,
            Err(NoBoard::NoNails) => -6,
        }
    }

    /// Nails a board across the door or window beside you (see `board_check`).
    #[func]
    fn board(&mut self) {
        self.act(Intent::Board);
    }

    /// Pries a board off the boarded door or window beside you.
    #[func]
    fn unboard(&mut self) {
        self.act(Intent::Unboard);
    }

    /// The skills' names, in the order `character_skills` and the skill events use.
    #[func]
    fn skill_names(&self) -> PackedStringArray {
        Skill::ALL.iter().map(|s| GString::from(s.name())).collect()
    }

    /// Your skills, three numbers each (by `skill_names`): the level, how far toward the next
    /// (out of 1000), and how it stands: 0 learning, 1 at the free cap (it learns no more unless
    /// mastered), 2 mastered, 3 as high as it goes.
    #[func]
    fn character_skills(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let Some(me) = world.character() else {
            return PackedInt32Array::new();
        };
        world
            .skills(me)
            .into_iter()
            .flat_map(|(_, s)| {
                let state = match s.state {
                    Rise::Rising => 0,
                    Rise::AtCap => 1,
                    Rise::Mastered => 2,
                    Rise::Top => 3,
                };
                [i32::from(s.level), s.progress as i32, state]
            })
            .collect()
    }

    /// Masteries you have still to choose.
    #[func]
    fn masteries_left(&self) -> i32 {
        let Some(world) = &self.world else { return 0 };
        world
            .character()
            .map_or(0, |me| i32::from(world.masteries_left(me)))
    }

    /// Chooses a skill at the free cap (by `skill_names`) as a mastery. False if it can't be.
    #[func]
    fn master(&mut self, skill: i32) -> bool {
        let Some(world) = self.world.as_mut() else {
            return false;
        };
        let (Some(me), Some(&skill)) = (world.character(), Skill::ALL.get(skill as usize)) else {
            return false;
        };
        world.master(me, skill)
    }

    /// What each item brings home, by definition number, in XP points (rounded).
    #[func]
    fn item_worths(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        world
            .defs
            .items
            .iter()
            .map(|i| ((i.worth + 500) / 1000) as i32)
            .collect()
    }

    /// Your deeds: [level, XP, the XP this level began at, the next level's (-1 at the top),
    /// the most health you can have (out of 1000 at level 1), home's x and y (-1 if none)].
    #[func]
    fn character_deeds(&self) -> PackedInt32Array {
        let Some(world) = &self.world else {
            return PackedInt32Array::new();
        };
        let Some((me, a)) = world
            .character()
            .and_then(|id| world.actor(id).map(|a| (id, a)))
        else {
            return PackedInt32Array::new();
        };
        let level = world.level(me);
        let (from, to) = world.rules.deeds.span(level);
        let points = |xp: u32| (xp / 1000) as i32;
        let home = a.home.map_or((-1, -1), |h| (h.x, h.y));
        PackedInt32Array::from(&[
            i32::from(level),
            points(a.deeds),
            points(from),
            to.map_or(-1, points),
            world.max_health(me),
            home.0,
            home.1,
        ])
    }

    /// Whether the container on (x, y) can be your stash now: 0 if so, else why not: -1 not a
    /// container within reach, -2 out under the sky, -3 you haven't slept a night through
    /// anywhere yet, -4 too far from where you sleep.
    #[func]
    fn stash_check(&self, x: i32, y: i32) -> i32 {
        let Some(world) = &self.world else { return -1 };
        let Some(me) = world.character() else {
            return -1;
        };
        match world.can_stash(me, TilePos::new(x, y, 0)) {
            Ok(()) => 0,
            Err(NoStash::NotHere) => -1,
            Err(NoStash::Outdoors) => -2,
            Err(NoStash::NoHome) => -3,
            Err(NoStash::FarFromHome) => -4,
        }
    }

    /// Makes the container on (x, y) your stash (see `stash_check`).
    #[func]
    fn mark_stash(&mut self, x: i32, y: i32) {
        self.act(Intent::Stash {
            at: TilePos::new(x, y, 0),
        });
    }

    /// Puts the stack at `slot` of your pack into the container on (x, y).
    #[func]
    fn put_item(&mut self, x: i32, y: i32, slot: i32, def: i32) {
        self.act(Intent::Put {
            at: TilePos::new(x, y, 0),
            slot: slot as u16,
            def: def as u16,
        });
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
