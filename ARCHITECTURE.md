# Duskhold — engine and architecture

*Draft 3, 2026-11-09: reviewed against the code after build step 5 (see "The 2026-11-09 review", before the build order). `CONCEPT.md` says what the game is; this says how it's built.*

## Decided (2026-11-05)

- **The game comes first.** The bot comes later, to train on the game once there is one, and maybe to drive its NPCs (the user: "the bot is not a primary in this at all. I'm more interested in developing the game"). Four things the bot will need also help the game, so they stay:
  - a deterministic core: any bug or death can be replayed;
  - one rule for timed actions;
  - the game knowing what's possible: the player's greyed-out buttons;
  - the character perceiving only what it should.
- **The stack:** a Rust core and a Godot 4 client, with Python for the bot later.
- **The look:** 3D models under an orthographic camera.
- **The grain:** 10 ticks a second, chunks of 32 × 32 tiles.
- **The setup:** the Rust toolchain and maturin. Where that stands is under "Setup on this PC".

**Default** marks Claude's choices, open to change. **Open** marks questions for the user, listed at the end.

## In one screen

| | Choice |
|---|---|
| Engine | Our own simulation core, `dusk-core`, in **Rust**: all of the game's rules, with no graphics and no I/O |
| The game | A **Godot 4** client that runs the core in-process (a GDExtension, through gdext), draws what the character perceives and sends what the player wants as intents |
| The bot (later) | A **Python** module, `duskhold` (PyO3, built with maturin), that steps many worlds at once on Rust threads into NumPy arrays for training, with `ppo.py` from `11_PPO-Zombie`. Trained policies can then drive NPCs inside the core, with no Python in the game |
| World | Tiles with walls on their edges (Zomboid's way), floors from cellars to attics, chunks of 32 × 32 tiles; full detail near the character, a population layer everywhere else |
| Time | A fixed tick, 10 a second of action time at 1×; a world clock 24× faster (a day is an hour at 1×, Zomboid's default) |
| Determinism | The same build, seed and intents give the same game, tick for tick; a replay is a seed and the intents |
| Content | TOML data files; buildings as text grids; the world generated from a seed out of hand-made pieces |
| Saves | The seed, plus only what changed |
| Look | 3D low-poly models under a fixed orthographic camera at Zomboid's angle |
| Later | Lua (`mlua`) for quests, events and mods; a headless server for co-op |

```mermaid
flowchart LR
  data["data/ (TOML)"] --> core
  gen["dusk-gen: the world from a seed"] --> core
  core["dusk-core: the simulation"] -- "observations, masks, events" --> py["dusk-py: Python module (later)"]
  py -- "actions" --> core
  core -- "the character's view" --> godot["dusk-godot: Godot extension"]
  godot -- "intents" --> core
  core --> cli["dusk-cli: tools"]
  py --> rl["python/: training (later)"]
  godot --> client["client/: the game"]
  core -. "later" .-> server["dusk-server: co-op"]
```

## Why our own core

The concept asks for things a game engine's simulation layer doesn't give:

- an open world in chunks, with a population layer for what's off-screen, and hundreds of creatures sensing and moving at once;
- time skips: sleeping through a night or waiting out the rain runs the world at hundreds of times real speed (Zomboid fast-forwards a sleep the same way);
- deterministic replays: any bug or death can be played back exactly;
- later, the same rules called from Python for the bot, at 1,000× real time or more per core.

So the core is a library with no graphics, and everything else (the game, the tools, the tests, later the bot) calls it. The cost is writing our own world, systems and tools; Godot is only the client.

## Why Rust

Measured on this PC (2026-11-05, Python 3.14.2, 24 logical CPUs; `research/python_tick_bench.py` and `research/python_hotspots_bench.py`):

| Pure Python | Speed |
|---|---|
| A stripped-down tick: 100 dead hear and see the player and step toward it | 18,900 ticks/s (1,890× real time) |
| The same with 500 dead | 4,900 ticks/s (490×) |
| The character's field of view, radius 20 (recursive shadowcasting) | 0.68 ms each |
| A*, 30–50 tiles | 1.57 ms each |

- **The stripped tick leaves out most of the work:** fields (light, scent, heat), bodies and needs, items, AI states, pathfinding, the bot's field of view at each decision. Ten goblins re-planning once a second is one A* per tick: 1.6 ms, which alone caps Python at about 600 ticks/s. A full tick in Python would land around 400–1,700 ticks/s, 40–170× real time per core (estimate). The game at 1× needs only 10 ticks a second. But a night's sleep fast-forwards eight game hours, 12,000 ticks across the whole active area, and Zomboid does that in a few seconds; the bot later wants 1,000× or more.
- **Rust, measured on identical work (2026-11-06):** `crates/dusk-cli/examples/python_twin.rs` ports the Python script line for line. It ran at 3,600,000 ticks/s with 100 dead (195× the Python script's 18,466 that day) and 700,000 with 500 (151× its 4,651). The real core's numbers are under the performance budget below.
- **PyO3 and maturin** make the core an ordinary Python module. NumPy arrays are filled in place (no copies), and the GIL is released while 16–24 worlds step on Rust threads.
- **gdext** puts the same core inside the Godot game, in-process.
- **WebAssembly** puts it in a browser: replays in the dashboard, maybe a browser build one day.
- **Determinism** comes easily: no garbage collector, integer tiles, and full control of iteration order and randomness.

Considered:

| Option | Why not, at least for the core |
|---|---|
| Python | The quickest to change, and the RL stack's own language. But the numbers above put it at 40–170× real time per core, and every region adds work. It stays for everything around the core |
| C# (Godot's own language) | First-class in Godot, but the Python bridge is clumsy (pythonnet, or a C ABI through NativeAOT) |
| C++ | As fast, but less safe, and its Python and Godot bindings are no better |
| Python + Numba | One language, but entities, inventories and AI fit Numba badly, and Numba trails each Python release by months |
| TypeScript (MeowMix's stack) | Runs natively in a browser, but the bot would have to talk to it over a socket |

The costs of Rust:

- Claude writes it; the user mostly judges behaviour (the dashboard, replays) rather than reading code.
- A toolchain to install.
- Compile times (seconds, incremental).
- The borrow checker pushes the design toward ids and arenas, which suits a simulation anyway.

## Why Godot for the game

- **Practical:** free (MIT) and light, with a real UI system for Zomboid's windows (inventory lists, the health panel, right-click menus), an editor, and exports to Windows, Linux and macOS.
- **In-process:** the core runs inside the game's process through gdext.
- **3D under an orthographic camera** (Default). Zomboid itself draws 3D characters over 2D tiles.
  - Characters are rigged low-poly models: gear shows on the body, they can face any way, and animations are shared across models.
  - Torchlight and day and night are real lighting.
  - Walls are cut away in a shader.
  - Free CC0 kits (Kenney, KayKit, Quaternius) cover the first playable.
  - The alternative, 2D sprites, multiplies the art: 8 directions × every animation × every piece of gear.
- **The client never decides anything.** It draws the view the core gives it and turns input into intents. What's visible is always the core's answer, even where the rendered lighting looks a little different from the gameplay light.

Considered: Bevy (Rust throughout, but pre-1.0 churn and a thin UI and editor), a browser client (PixiJS and the core in WebAssembly: playable on a phone, but the UI and lighting built by hand), and Unity (heavier, with licensing).

## The core: `dusk-core`

### The API, in short

```rust
impl World {
    pub fn new(defs: Arc<Defs>, settings: &Sandbox, seed: u64) -> World;           // a new realm
    pub fn from_scenario(defs: Arc<Defs>, scenario: &Scenario, seed: u64) -> World; // a slice for the bot
    pub fn intent(&mut self, actor: ActorId, intent: Intent); // the player, a bot, an NPC: the same call
    pub fn step(&mut self, ticks: u32);
    pub fn view(&self, actor: ActorId, out: &mut View);       // what this actor perceives (the client)
    pub fn observe(&self, actor: ActorId, out: &mut [f32]);   // the same, as named numbers (the bot)
    pub fn mask(&self, actor: ActorId, out: &mut [bool]);     // what it can do right now
    pub fn events(&mut self) -> Drain<'_, Event>;             // hits, XP, deaths, sounds, messages
    pub fn state_hash(&self) -> u64;
    pub fn save(&self, dir: &Path) -> io::Result<()>;
}
```

An `Intent` is anything a controller can ask for:

- idle;
- move (a heading, walking or running), or walk to a tile (pathfinding);
- attack, shove;
- interact (a door, a container, a window);
- use an item (eat, drink, equip, drop, bandage, ...);
- cast, build;
- toggle sneak, run or light;
- cancel.

A player's click names exactly which item; a bot's "pick up food" names the nearest food worth taking, as in ZomboidRL. Both become the same actions.

### World model

- **Tiles:** each tile (x, y, z) has terrain (grass, mud, road, water, ...), a floor, a roof (so sunlight and rain know what's indoors), a light level and a few flags.
- **Edges** (Zomboid's walls): each tile owns its north and west edge. An edge holds nothing, a wall, a door, a window or a fence, with a material and a state (open, locked, barricade level, health). Whether you can step from one tile to the next depends on the edge between them, and a diagonal step needs both of its sides open: no corner cutting, the lesson of ZomboidRL's `paths`.
- **Fixtures** sit on tiles: furniture, containers, fires, wards, traps, webs.
- **Floors:** z runs from −2 (cellars) to +3 (attics), linked by stairs and ladders. Dungeons, later, are separate maps entered through an entrance.
- **Chunks** are 32 × 32 tiles on every floor. A chunk holds its tiles, edges, fixtures and ground items, plus a flag for whether anything changed.
- **Actors** (the character, NPCs, creatures) are plain structs in a generational arena (our own, `arena.rs`: the core has no dependencies), not a general ECS (Default). There are few kinds of entity and we know them all, and plain structs are easier to read, save and keep deterministic.
- **Items are values** (definition, quality, condition, affixes, identified or not, contents) inside an inventory, a container or a tile's ground list. A town holds thousands of them, so they're kept cheap. Only the few that need it carry an id: uniques, attuned items, a revenant's gear.
- **Containers fill from the seed.** What's in a cupboard is a function of the seed and the cupboard's position, so an unopened container stores nothing. It's filled the first time someone looks, and saved from then on.
- **Definitions** (creatures, items, spells, rooms: everything from the data files) are loaded once, read-only, and shared by every world on every thread.

### Time

- **Action time:** a tick is 0.1 s at 1×. Movement, swings and timed actions count ticks.
- **The world clock** runs 24× faster by default: a game day is an hour at 1×, Zomboid's default day length and a sandbox setting. So a tick is 2.4 s of clock time. Day and night, needs, healing, crops, weather and the Long Dusk run on the clock, while a swing still takes a real second. Zomboid works the same way.
- **Speeds:** pause, 1×, 2×, 5× and "faster while safe" for players; uncapped for bots.
- **Not every system runs every tick:**

  | Every | Runs |
  |---|---|
  | tick | actions, movement, combat, endurance and bleeding |
  | 5 ticks | perception and AI for most creatures, the sound and scent fields, the bot's decision (Default) |
  | 25 ticks (a game minute) | needs, sleep, healing; moodles are worked out when asked |
  | 100 ticks | fire, weather, the population layer |
  | clock hour or day | holdout stores, loot respawn, the Long Dusk |

  Each system's turn is staggered by actor id, so a hundred creatures don't all think on the same tick.

### The tick, in a fixed order

The list below is the plan for the whole game. What the core runs today (`World::tick_once`) is its first half: each actor's timed action or else its step, then hearing, then thinking (whose intents act on the next tick), then grips, bodies, needs, lights and drag-downs. A sound made before the hearing pass is heard that tick; one made after it (a groan, as a creature thinks) on the next.

1. **Intents** from controllers (the player's input, bots, NPC and creature AI) start, continue or replace actions.
2. **Actions** progress, pay out, finish or get interrupted (see below).
3. **Movement:** tile-to-tile steps with progress, drawn smoothly. The tile ahead is reserved, so two actors never share a tile, and doorways are real chokepoints.
4. **Combat:** strikes land on their strike tick. Reach, facing, the body part hit, armour, wounds, knockback, grabs and being dragged down all resolve here.
5. **Emissions:** each actor and fixture gives off noise, light, movement, scent, heat and dread for this tick.
6. **Fields** update on their schedules: light, sounds, scent, heat, web vibrations.
7. **Perception:** each creature senses with its family's senses and updates what it remembers.
8. **AI:** creatures decide; their intents take effect next tick.
9. **Bodies and needs:** bleeding, hunger, thirst, fatigue, temperature, afflictions, moodles.
10. **The world:** fire spread, weather, the clock, the dead getting up, the population layer.
11. **Events out:** XP, quest progress, telemetry, deaths.

### One rule for every timed action

Every action has:

- a check for whether it can start here and now;
- a duration in ticks, scaled by skill;
- what it does as it goes (drinking pays out as it goes, as in Zomboid);
- what it does when it's done;
- its noise and light;
- what interrupts it: a hit, a threat within reach once its first half-second is up, or the actor choosing something else.

Choosing the same action again carries on with it; choosing another replaces it. Players can queue actions while paused, as in Zomboid. Creatures follow the same rule: a ghoul eating a corpse is an action, and it can be interrupted.

In ZomboidRL this hold, carry-on and interrupt logic grew one action at a time (eat, drink, bandage, pickup, gear). Here it's written once.

### Senses and fields

- **Emissions** per actor:
  - noise, as a radius;
  - light, from what it carries;
  - movement: speed × weight;
  - scent, left on its trail;
  - heat, from its body, clothes and fires;
  - dread, its own.
- **Light** (`light.rs`, built at step 5): one number per tile, out of 100, worked out when asked. It's the brighter of the sky's light (from the clock; 60% under a roof) and any carried light that reaches the tile in a straight line (full out to half its radius, then less), or a burning body's, or a fire's in a hearth, stove or forge (`fire.rs`). Rain darkens the sky (`weather.rs`). Later: windows letting light in, and braziers. If asking ever costs too much, the sky's part gets cached per chunk.
- **Sight scales with light:** a tile lit at 100 is seen out to an eye's full range, at 50 half as far, and everyone sees what's right beside them (darkvision) in any light.
  - A creature's eyes (`World::sees`): in range, in its cone, lit enough for how far off (or within its darkvision), and in a line of sight.
  - The character's field of view (`view.rs`) is the same rule, with a 160° cone and the tiles beside it all round. It's perception, not simulation: worked out when asked (each frame for the client), into a `View` that also remembers what was seen (the fog of war). It checks a straight line to each tile in range. Symmetric shadowcasting adapted to edge walls is the faster way, for when the bot asks every few ticks.
- **Sounds** are events (where, how loud, what kind). Creatures in range hear them, and every wall on the way costs loudness.
- **Scent** is a coarse grid, one cell per 2 × 2 tiles, with deposits, decay, wind and rain.
- **Heat** comes from emitters plus the ambient temperature, and needs a line of sight to sense.
- **Vibration:** footsteps carry within a radius; webs form a graph that carries every touch.
- **Dread** is each actor's own number. Ghosts sense it within range, through walls.
- **A region is mostly data.** A family's data says which senses it uses, at what ranges and thresholds, and how it moves (walks, climbs webs, passes through walls). A new region adds data and an AI module, not engine changes.

### Creature AI

- **Per family:** a state machine with utility scores (idle, wander, investigate, chase, attack, flee, feed, sleep, go home), configured in data.
- **Intelligence is a budget:**
  - the dead steer straight at what they sense and share flow fields, so the Tolling moves hundreds of them for the cost of one field;
  - goblins plan with A*: loot, steal, ambush, call for help;
  - wolves steer as a pack: circle, flank, rush.
- **Thinking is staggered:** the dead every 5 ticks, goblins every 2, a boss every tick.

### Pathfinding

A* runs over tiles and edges: doors cost time, and closed doors are passable only for those that can open them. Long trips use hierarchical paths across chunks (portals between chunks), crowds share flow fields, and each chunk's change counter invalidates cached paths through it.

### Bodies, items and combat

All the numbers come from the data files. A body has parts with wounds (type, severity, bleeding, bandaged, infected), pain and armour per part; moodles are worked out from these and the needs when asked. An attack is an action: a wind-up, a strike tick and a recovery.

### The open world: chunks and the population layer

- **The active area:** chunks within 3 chunks (96 tiles, Default) of an anchor run in full. Anchors are the character, any bot, and important NPCs.
- **Everywhere else, a population layer** is updated every clock hour. Per chunk it keeps how many of each family there are and which way they're drifting, plus each holdout's stores, defenders and morale. Raids and migrations move these counts between chunks.
- **Waking a chunk** turns its counts into creatures, placed by family rules: the dead in houses and streets, spiders on webs.
- **Putting a chunk to sleep** folds its creatures back into counts. Named ones (rares, revenants, NPCs) stay real.
- **Zomboid works this way** (its "virtual zombies"), which is how its towns stay full without simulating every zombie.

### Determinism and randomness

- **One thread per world.** Its systems run in a fixed order and its actors in id order. Many worlds run in parallel, one per thread.
- **No hash-map iteration in game logic.** B-tree maps, `IndexMap` or sorted vectors instead, and hashing with fixed seeds.
- **Randomness:** world generation draws from ChaCha streams keyed by (seed, layer, chunk). Gameplay rolls are hashed from (seed, tick, actor, purpose), so a new roll in one system doesn't shift the rolls of every other.
- **Numbers:** everything in the game's state is an integer: tiles, health and needs in thousandths, chances out of 10,000, cones as cosines. Floats appear only in `dusk-data` (a designer's units, converted once at load) and in what the client draws.
- **Checks and replays:** tests call `state_hash()` every tick. A replay is the seed, the scenario and the external intents, since everything else follows from them.

### Saves

- **What's saved:**
  - the seed, the data version and the realm's large-scale layout;
  - only the chunks that changed (like Zomboid's `map_X_Y` files);
  - the actors and the clock.
- **Format:** `serde` with `postcard`, a version header and explicit migrations.
- **Everything else comes back from the seed:** unchanged chunks and unopened containers.
- **Autosave** every clock hour.
- **Built** at step 6, slice 1 (`save.rs`), as one file for the one patch; migrations and one file a chunk come when the world outgrows that.

## World generation: `dusk-gen`

- **The large-scale layout comes first,** once per realm, in seconds:
  - the realm: the six regions around the heartland, and their borders;
  - terrain: height and moisture, which give each region its biomes;
  - rivers down the height map;
  - roads between settlements;
  - sites: towns, holdouts, farms, camps, and later dungeon entrances.
- **Chunks are made on demand,** each from its seed, its position and the large-scale layout, so a chunk comes out the same whatever order chunks are made in:
  - town streets and lots;
  - buildings from prefabs, rotated and mirrored to fit;
  - furniture and containers by room;
  - loot by room, region and depth;
  - creature counts by family and density.
- **Prefabs are text** (Default), so Claude can write dozens of houses and a diff shows what changed:
  - edge rows (even rows) alternate `+` corners with horizontal edges;
  - cell rows (odd rows) alternate vertical edges with cells, so cells sit in the odd columns;
  - on an edge, `|` or `-` is a wall, `D` a door, `W` a window and a space is open;
  - the legend maps each cell's letter to a floor, furniture, a container and loot.

```toml
[[prefab]]
id = "hush_cottage"
tags = ["house", "rural", "hushlands"]
grid = """
+-+-+-+-+
|b .|k .|
+ + +D+-+
|. . . .D
+-+W+-+-+
"""
[prefab.legend]
"." = { floor = "planks" }
"b" = { floor = "planks", furniture = "bed", loot = "bedroom" }
"k" = { floor = "planks", furniture = "cupboard", loot = "kitchen" }
```

Other floors are more grids (`grid_up`, `grid_cellar`), and stairs are a legend entry. Tools:

- `dusk-cli gen --seed 42 --png` draws a realm's map;
- `dusk-cli prefab show hush_cottage` draws a prefab;
- `dusk-cli check` validates all the data.

## Data files

```
data/
  creatures/   hushlands.toml  warren_hills.toml  ...
  items/       weapons.toml  armour.toml  food.toml  tools.toml  ...
  affixes.toml  traits.toml  ancestries.toml
  spells/  classes/  regions/  prefabs/  loot/  recipes/  blueprints/
  presets/     sandbox/  training/
```

```toml
[[creature]]
id = "shambler"
family = "dead"
senses = { hearing = 14, sight = 6, darkvision = 0 }   # tiles
speed = { walk = 0.6, chase = 0.9 }                     # tiles a second
health = 30
attacks = [{ kind = "bite", reach = 1.0, windup = 6, damage = 6, grabs = true }]
think_every = 5                                          # ticks
weak_to = ["fire", "blessed"]
```

- **The example above shows the shape, not today's fields.** `data/creatures/hushlands.toml` is the real file. Each `[[creature]]` in it becomes a row of `Defs::creatures` (`creature.rs`; build step 6, slice 0).
- **Loading:** the data is loaded and checked once: types through `serde`, then cross-references (every loot table, item and spell a file names must exist).
- **Reloading:** changed data reloads in dev builds without a restart.
- **Mods:** a mod is a folder of the same files, loaded after the base game's.

## Later: the bot (training, then NPC AI)

Not before the game is playable (build step 7). The core is shaped for it from the start, so adding it later is a binding, not a rewrite.

```python
import duskhold as dh

sim = dh.VecSim(n=16, data="data", scenario="training/hush_street", seed=1)
obs = sim.reset()                       # (16, n_inputs) float32, filled in place
names = sim.obs_names()                 # stable names by group, read by PPO.from_checkpoint
amask, kmask = sim.masks()              # what each world's character can do now
obs, rew, term, trunc, info = sim.step(actions)   # 5 ticks a step, 16 worlds on 16 threads
```

- **Wrappers:** `gymnasium.Env` and `VectorEnv` around it. `ppo.py` is copied from `11_PPO-Zombie` and taught to roll out many worlds at once. The telemetry and the dashboard come along, with a debug map drawn from the core's view.
- **Scenarios are data:** a region, a place (a farmhouse, a town street, a forest camp), the time, the season, a kit, a creature density and a step limit. A sampler mixes them into a curriculum.
- **Observation:** named inputs in groups, built in Rust straight into the shared array, with every sense present from the start:
  - vitals, body, moodles;
  - the signature: how loud, bright, fast, scented, warm and frightened the character is;
  - what it sees: sectors, and the nearest few creatures;
  - what it hears: loudness per sector;
  - the 9 × 9 tiles around it: walls, doors, light, walking distance;
  - the nearest item worth taking of each kind, gear, what's carried;
  - the base: where it is, and whether you're in it;
  - the time, the weather, the region.
- **Actions:** a verb plus a kind, as in ZomboidRL, with masks from the core.
- **Reward:** the game's own events, weighted by a table in the scenario: XP for what's brought home, the alive bonus, wounds, death.
- **Replays:** a seed and the intents, so the dashboard can play any episode back exactly.
- **Throughput target:** 1,000× real time or more per core. ZomboidRL makes 3–6 decisions a second; 16 worlds here would make 30,000 or more. At that point the trainer, on the RTX 3080 Ti, is the bottleneck, which is where we want it.
- **NPC AI from a trained policy:** a policy is a small network (ZomboidRL's has 256 units a layer). Its weights are exported from `ppo.py` to a file the core reads, and the core runs it as one more kind of controller for an NPC or a creature. It uses the same observation builder and masks the bot trained on, and a decision costs tens of microseconds in plain Rust, with no Python and no ML library in the game. Hand-written AI stays the default; a policy takes over a role where it plays better (a rival survivor, a holdout's scavenger).

## The game: `client/` (Godot)

- **The core runs inside the game** (through `dusk-godot`), ticking at the chosen speed, and the renderer draws smoothly between the last two ticks. For now it ticks on the main thread (Default, 2026-11-06). When a tick costs enough to drop frames, it moves onto its own thread and publishes a view after each tick.
- **Getting the extension to Godot** (Default, 2026-11-06): `play.cmd` writes `client/dusk.gdextension`, which names the DLL in the build folder (`%LOCALAPPDATA%\cargo-target`). So the build stays out of OneDrive, and the file isn't committed, because it names this PC's folder.
  - In the editor, `reloadable = true` makes Godot load a copy of the DLL, so `cargo build` reloads the extension while the editor is open.
  - A running game holds the DLL itself: close the game before rebuilding.
- **The view is what the character perceives:** visible and remembered tiles, light, the creatures it can see, items, effects and sounds. The client can't show what the character doesn't know, the same rule as the bot's.
- **The view also carries what the readability pillar draws** (`CONCEPT.md`, "What hunts you, you can see"):
  - the noise rings made this tick, with their reach;
  - each visible creature's sight cone and alert state;
  - your scent trail, heat and tremors;
  - the lines from adjacent enemies (engagement);
  - each attack's tell while it winds up.

  The core computes these anyway, so drawing them costs the client nothing more than a decal.
- **Rendering** (Default: 3D, orthographic, at Zomboid's angle):
  - chunks as instanced meshes, and walls as thin meshes on tile edges;
  - a cut-away shader for walls between the camera and the character, and the floors above hidden when indoors;
  - the core's view as a texture (one texel per tile, sampled smoothly) for the fog of war, drawn by one full-screen pass. The sun and the sky's light follow the core's daylight, and the light the character carries is a real light with shadows;
  - weather particles.
- **UI** (Godot controls): the moodle column, the health panel, inventory windows with weight bars, Zomboid-style right-click menus, progress bars, the map, the speed dial, build mode.
- **Input becomes intents:** click to walk (pathfinding), WASD to steer, hotkeys, context actions, and a queue while paused.
- **Sound** is positional, played from the core's sound events.

## Later: scripting, mods, co-op

- **Lua** (`mlua`; the user knows Lua from ZomboidRL's mod) for quests, events and mods, kept out of the hot path.
- **Co-op:** `dusk-server` runs the core headless. Clients send intents and receive their character's view as deltas. The server decides everything, so a client can only ever show what its character could see anyway.

## Repository layout

```
12_Duskhold/
  CONCEPT.md  ARCHITECTURE.md  CLAUDE.md
  Cargo.toml                 Rust workspace
  crates/
    dusk-core/               the simulation: world, rules, systems (no graphics, no files, no threads)
    dusk-data/               data file types, loading, checks
    dusk-gen/                world generation from a seed
    dusk-godot/              the Godot extension (gdext), step 3
    dusk-cli/                tools: demo, bench; later check, gen, prefab show, replay
    dusk-py/                 the Python module `duskhold` (PyO3, maturin), later
  data/                      the game's content (TOML)
  client/                    the Godot project
  python/                    training, telemetry, the dashboard (from 11_PPO-Zombie), later
  research/                  measurements behind decisions (the Python benchmarks)
  tests/                     golden replays, Python API checks
```

## Tests and tools

- **Rust:** unit tests per system, and property tests for invariants (no actor inside a wall, weights add up, no item in two places).
- **Golden replays:** a scenario, a seed and its intents give a fixed state hash. A change in behaviour changes the hash; an intended change updates it.
- **Determinism:** the same run twice, and on 1 thread vs 16, gives the same hashes.
- **Benchmarks** (`criterion`): ticks a second for fixed scenarios (a quiet farmhouse, a street with 100 dead, the Tolling with 500).
- **Python:** Gymnasium's environment checker, stable input names, masks that agree with what the actions do, and a toy training run.
- **No fake game needed:** ZomboidRL's `lua_harness.py` had to fake the game; here the core is the game.

## Setup on this PC

State on 2026-11-05:

- **Rust 1.99.1** (rustup, stable, MSVC host) in `%USERPROFILE%\.cargo` and `.rustup`, outside OneDrive. New terminals find `cargo` on the PATH.
- **maturin 1.16.0** in the project's own `.venv` (Python 3.14.2), ready for the bot later.
- **Build output** goes to `%LOCALAPPDATA%\cargo-target` (`CARGO_TARGET_DIR`, set for the user with `setx`).
  - Rust's `target/` folder grows to gigabytes of small files, and OneDrive would sync every one of them.
  - It also keeps paths under Windows' 260-character limit (the root CLAUDE.md's warning).
  - `target/`, `.venv/` and Godot's `.godot/` cache are gitignored.
- **Visual Studio 2022 C++ Build Tools** (MSVC 14.44.35207), the linker the MSVC toolchain needs. They were installed by 2026-11-06, after the first try (winget, 2026-11-05) ended with exit code 1602, the admin prompt cancelled. If they're ever missing, `cargo check`, `clippy` and `fmt` still work, but nothing links. The install command, run in PowerShell with the admin prompt approved (about 5 GB):

  ```powershell
  winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--passive --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
  ```
- **Godot 4.7.2** (standard build), installed 2026-11-06 in `%LOCALAPPDATA%\Programs\Godot`, outside OneDrive. It came from the official GitHub release, checked against its SHA-512 sum. `Godot_v4.7.2-stable_win64.exe` is the editor; `Godot_v4.7.2-stable_win64_console.exe` is the same program printing to the terminal (for scripts and `--version`).
- **`play.cmd`** (2026-11-06) builds the extension and starts the game with the console build, so Godot's messages land in the terminal. The first time, it also has Godot import the project (`--headless --import`), which registers the extension in `client/.godot/`. It's a batch file rather than a PowerShell script, so PowerShell's script policy doesn't come into it.

## Performance budget (per world, one thread)

| Part | Budget per tick (average) |
|---|---|
| actions, movement, combat | 10 µs |
| perception and AI for 100 creatures, staggered | 20 µs |
| fields: light, sound, scent | 10 µs |
| the bot's field of view and observation, every 5 ticks | 5 µs |
| bodies, needs, the world | 5 µs |
| **total** | **about 50 µs: 20,000 ticks/s, 2,000× real time** |

The benchmarks check this from the first week. A part over budget gets profiled before it gets more features.

Measured on 2026-11-06 (`dusk-cli bench`, release build). The map is 96 × 96 with 12% rock, you wander about, and a noise every second carries 20 tiles:

| Dead | Thinking every 5 ticks (the game) | Thinking every tick |
|---|---|---|
| 100 | 157,000 ticks/s (15,700×) | 69,000 ticks/s |
| 500 | 16,500 ticks/s (1,650×) | 6,500 ticks/s |

- **100 creatures** run at 8× the budget.
- **500**, a horde the size of the Tolling's, falls short of it. Each shambler that can't walk straight at you plans its own A* path, and every node of a search checks its 8 steps tile by tile across chunk lookups.
- **The planned fixes:**
  - flow fields, so a crowd heading for one place shares a single field;
  - a per-chunk cache of which steps are open, rebuilt when a chunk changes.
- **For gameplay,** 16,500 ticks/s still fast-forwards a night's sleep (12,000 ticks) in under a second.

**Measured again on 2026-11-10, after step 6** (both fixes built). The bench now keeps you alive in the crowd the whole run: a dead character no longer keeps the world awake, and before, the crowd dragged you down part-way, after which it mattered less. So these are harsher than the table above:

| Dead | Thinking every 5 ticks | The Tolling on, none giving up |
|---|---|---|
| 100 | 111,000 ticks/s (11,100×) | 79,000 ticks/s |
| 500 | 23,100 ticks/s (2,310×) | 16,600 ticks/s |

What was done, found by timing each part of a tick (the A* searches were most of a crowded one):

1. **Finding a tile's chunk** was a search of a sorted map on every read of the grid. A dense table over the chunks' box makes it an array read (`Grid`, `Dense`); the sorted index stays, for comparing, hashing and saves. About 20% faster on its own.
2. **The lookup maps** (`hash::FastMap`: who stands where, A*'s bookkeeping) hash a word at a time (`WordHasher`, rustc's FxHash) instead of a byte at a time (FNV, which stays for the state hash and the data's fingerprint). Another 5%.
3. **Each tile's open steps are kept** (`Grid::index_steps`, `steps_from`): a byte a tile, bit k for `Dir8::ALL[k]`, built once a patch is made and kept true as tiles and edges change (the 5 × 5 tiles round a change are worked out again; a chunk put in place, the chunk and its border). A search reads one byte a node where it made about 25 lookups, so an A* search on the bench's map went from 151 µs to 48. A grid never indexed (a test's, a hand-made map) works the steps out each time: the same answer, slower. A test holds the kept steps to the map's through hundreds of edits of every kind.
4. **The same search wasn't made twice.** A creature waiting behind another planned its way again every 15 ticks, but a search doesn't count actors, so from the same tile to the same goal it found the very same way: 99% of all searches were these (56,000 of 56,900 on the bench). It now plans again only when the map itself bars the way (a door shut), and otherwise waits on. Its way is then what's left of the one it planned, which may differ from a fresh search's only where two ways cost the same.

Making a patch takes 45 ms instead of 22 (indexing its steps). The game's own patch runs its Tolling at 190,000 to 210,000 ticks/s (it was 120,000).

## Risks

- **Rust's learning curve** falls on Claude; the user judges behaviour (the dashboard, replays) more than code. Mitigation: small crates, plain structs, and a comment wherever a rule lives.
- **gdext** is community-maintained. Fallback: the core as a separate process talking over shared memory, or through a C ABI.
- **Art:** 3D needs models and animations. CC0 kits first, our own art later.
- **Scope:** the first playable is one small region; everything else is a layer on a working core.
- **Determinism bugs** (iteration order, floats) are caught by the hash tests from the first week.

## The 2026-11-09 review

Asked for by the user after step 5, together with the concept's (`CONCEPT.md`, under Decided).

**The architecture holds.** The four things it bet on have each paid for themselves in steps 2 to 5:

- the core as a library with no graphics: every rule has a test that runs in milliseconds, and the tests found the bugs (running that never tired you, needs paid in the wrong units, a shambler side-stepping its own tile);
- one rule for timed actions: fifteen kinds of action now go through `combat.rs`'s `act`, `begin` and `start`, and each new one was a match arm, not a system;
- rules in the core, drawn by the client: the review's hand rule for shoves reached the screen as one `can_shove()` call;
- numbers in `data/`: the review's retuning of light, noise and the dead's tell changed four TOML files and no code.

**Three debts to pay before more is built on them** (they're step 6's first slices):

1. **The shambler is hard-coded** (paid the same day: step 6, slice 0). `ActorKind::Shambler`, `Mind::Shambler` and a dozen `Rules::shambler_*` fields are the one creature. "A region is mostly data" needs `Defs::creatures` (a table of definitions: speeds, senses, attack, wounds, groan, what it does at a door) and `ActorKind::Creature(n)`. Do it when the ghoul arrives, with the ghoul as the proof: a second creature that needs no new field in `Rules`.
2. **Nothing is saved** (paid 2026-11-10: step 6, slice 1). The saves section above is still a plan. A game day is an hour of play, bases and stashes are the point, and permadeath in a persistent world needs the world to persist. It's first in step 6.
3. **Crowds plan one path each.** 500 dead run at 16,000 ticks a second, under the budget. Flow fields (planned above) are needed before the Tolling, and the groan makes small crowds common now. *(Part paid 2026-11-10, step 6, slice 6: the Tolling's crowd shares one field. What's left: the dead chasing prey still plan one A* each, and on a Tolling night they don't give up, so 500 of them all after you run at 5,800 ticks a second. The next fix is the per-chunk cache of open steps: a step of A* or of a field costs about 0.8 µs now, most of it looking up chunks.)* *(Paid the same day, after slice 7: the steps are kept a byte a tile, chunks are found by a table, and a creature waiting behind another no longer plans the same way again. 500 dead after you all night run at 16,600 ticks a second; see the performance budget.)*

**Smaller things, noted and left:**

- events cross to GDScript as six bare numbers, with the kinds listed twice (in `dusk-godot` and in `main.gd`). A generated constants file would stop them drifting; it hasn't bitten yet;
- `Rules` is `Copy` and passed by value each tick. It's a few hundred bytes; when creature definitions move out of it, it shrinks again;
- the character's view is worked out each frame by a line of sight to every tile in range. Fine for one character; shadowcasting when the bot asks every few ticks (already noted under Senses).

**What the review changed in the game** (93 tests pass):

- **the dead groan** (`Rules::shambler_groan`, the shambler's `groan = 10`): one that catches sight of prey, having been after nothing, makes a sound where it stands, and the others in earshot go to it. One that was already after something doesn't, so a crowd doesn't call itself for ever. For this, the tick now clears its sounds straight after the hearing pass, so a sound made while thinking is heard on the next tick;
- **finishing spares the weapon:** a blow on one that's down doesn't wear it;
- **a shove from a hold needs a free hand** (`World::can_shove`): a weapon in one hand and a light held up in the other leave only the struggle. The held banner says so;
- **numbers** (`data/` only): the night's light 20 → 10; the torch's reach 5 → 7 tiles and the candle's 2 → 3; weapon noise 3 to 9 tiles (was 2 to 6); the shambler's tell 0.8 → 1.2 s (and its recovery 0.8 → 0.6 s, so its pace is about the same);
- **a test on the game's own data** (`at_night_a_torch_is_worth_its_risk_in_the_games_data`): at midnight a torch at least triples how far you see, and shows you to a shambler 8 tiles off that didn't see you before. Tuning that breaks the trade again fails it.

These were set by arithmetic and tests, not by play: nobody has played a night with them yet. The slice histories under step 5 below keep the numbers each slice shipped with.

## Build order

Game first: every step ends with something to look at or play.

1. **Toolchain and workspace.** Done (2026-11-05, Build Tools by 2026-11-06). The workspace has `dusk-core` and `dusk-cli`, with no dependencies yet.
2. **A core that walks.** Done 2026-11-06: 19 tests pass, clippy is clean, and the demo and the benchmarks run. What it has:
   - tiles, walls on edges, chunks and floors;
   - the text grid (the same format for prefabs, test maps and the debug view);
   - the tick;
   - walking and running, tile to tile, with no corner cutting and no two actors on a tile;
   - `WalkTo` with A*;
   - footsteps that walls, shut doors and windows muffle;
   - sight lines that walls and shut doors block and glass doesn't;
   - a shambler that hears and sees, goes after what it noticed, gives up after 30 s and shuffles about meanwhile;
   - how it gets there:
     - straight at it while it's in sight and that step is open;
     - otherwise along a path through doors and round walls, with a small search budget (the dead aren't clever);
     - where there's no way, pressing against the wall;
     - it has only reached you when it could step onto your tile, never diagonally across a door frame;
   - the first demo run found the greedy-only chase (a shambler walking away from its door), and the fix came with 3 tests;
   - a state hash with a determinism test, and tests for each of the above;
   - `dusk-cli demo` (the scene, printed once a second), `dusk-cli bench` (the performance table), and the `python_twin` example (the language comparison).
3. **On screen: the Godot client's skeleton.** Done 2026-11-06: `play.cmd` builds `dusk-godot` (gdext 0.5.5, API 4.6, running on Godot 4.7.2) and starts `client/`, and 21 tests pass. What it has:
   - the demo town (`dusk_core::demo`, 32 × 24 tiles: three houses and a shed, the character and nine of the dead) in 3D under an orthographic camera at Zomboid's angle (30° down, north up and to the right), with floors, furniture, and walls, doors and glass on the edges;
   - the character and the shamblers moving smoothly between ticks, turning to face where they go;
   - WASD steering and click to walk, sent as intents (Shift runs), plus pause, the speed dial (1×, 2×, 5×) and zoom;
   - cut-away walls: whatever stands between the camera and the character is cut away round it, with a dithered edge, while the shadows still come from the whole wall (V lowers every wall);
   - the readability pillar from the first frame:
     - noise rings for every footstep, out to how far it carries;
     - over each creature's head, its alert state: "?" curious (it heard something), "!" hunting (it sees you);
     - on the ground, the tiles each creature can see, tinted by its alert state. They come from `World::sees`, the very rule the dead hunt by, so where the ground is clear, they can't see you;
   - what happened, in the corner (heard, saw, got you), with a red flash and a count when one reaches you: something to play with until combat comes at step 5;
   - the core decided three things for this step (Defaults):
     - **shamblers see in a 120° cone** (8 tiles, 60° either side of their facing), so you can get behind one, as in Zomboid. Your footsteps still turn it: walking carries 3 tiles, running 8;
     - **a creature plans from the tile it's stepping onto:** its new heading takes effect there. Planning from the tile it was leaving made it side-step its own reserved tile and turn away from you;
     - **a shambler whose goal tile is taken turns to face it** rather than going round;
   - the core runs on the game's main thread for now (Default): a tick of the town takes microseconds. It moves onto its own thread when a tick costs enough to drop frames: chunks waking and sleeping (step 4), or the Tolling's crowds;
   - not yet: more than one chunk (the generated land is step 4), the character's own field of view and fog of war (step 5), real models (CC0 kits).
4. **A seeded Hushlands patch** (`dusk-gen`, `dusk-data`). Done 2026-11-06. 32 tests pass, and `play.cmd` drops you on a road 40 tiles from a hamlet. What it has:
   - **The data files** (`data/`), in a designer's units:
     - `rules.toml` (the world, sound, the character);
     - `creatures/hushlands.toml` (the shambler);
     - `fixtures.toml` (what can stand on a tile, solid and/or opaque);
     - `prefabs/hamlet.toml` (six buildings as text grids: cottage, longhouse, farmhouse, barn, shed, chapel);
     - `regions/hushlands.toml` (the patch's size, terrain, road, hamlet and dead).
   - **`dusk-data`** loads them, turns the units into the core's (tiles a second to thousandths, seconds to ticks, a cone's degrees to a cosine) and checks every reference. A mistake fails the load with the file and what's wrong, including a door that opens onto furniture: the first generator test caught one in the longhouse. `Rules::default()` stays as the tests' fixed set.
   - **The core's tiles:** grass, dirt, road, water (nobody walks it, everybody sees across it), floors; a fixture number with `solid` and `opaque` flags (a table blocks walking, a tree blocks both); fences on edges (stop walking, not sight or sound).
   - **Text grids** turn and mirror (`TextGrid::rotated`, `mirrored`), so one building faces any road.
   - **`dusk-gen`** makes a patch (8 × 8 chunks, 256 × 256 tiles) from a seed in about 25 ms:
     - the layout first: a road that bends every 24 tiles, and the hamlet's lots from its middle outward, on alternate sides of the road. Each building is mirrored or not, turned to face the road, and set in a fenced yard with a gate and a dirt path to the road;
     - then every tile and edge is a pure function of the seed, the layout and its place. So a chunk comes out the same in any order (a test checks it), and chunks can be made on demand when the world opens up;
     - the land: ponds under a height noise, woods over a damp one, lone trees in the fields;
     - the dead: 0–2 in each building, and about 1.2 a chunk in the open, none within 16 tiles of the start;
     - tests: the same seed gives the same world; every building's front door can be walked to from the start (three seeds); the dead all stand on walkable tiles.
   - **Sleeping chunks** (Default): the world runs in full within `active_radius` chunks of the character (3, so 7 × 7 chunks). Farther out, creatures stand as they are until you come near. Folding them into counts (the population layer) comes with the open world.
   - **The client streams chunks:** those within 3 of the camera's are built, nearest first and 3 a frame, and those past 4 are dropped. Each chunk is drawn as:
     - its ground, one square with a texel a tile;
     - trees (a trunk and a low-poly crown);
     - furniture (sized boxes by fixture id);
     - walls, doors, windows and fences. Crowns and furniture use the cut-away shader too, so a tree between you and the camera opens like a wall.
   - **Tools:**
     - `dusk-cli check` (the data);
     - `prefab <id>` (a building in its four turns);
     - `gen --seed N`, which lists the lots, and with flags:
       - `--png` draws the map;
       - `--text X,Y,W,H` prints part of it;
       - `--ticks N` times it.
     The PNG writer needs no dependencies (stored deflate blocks).
   - **Measured:** a tick of the whole patch (86 dead, 49 chunks awake, you standing still) takes 1.5 µs: 688,000 ticks/s. The 96 × 96 bench is unchanged: 178,000 ticks/s with 100 dead.
   - **Moved to step 5** (Default): containers and loot, since they need items, and the inventory comes at step 5. Doors are open in the prefabs until opening and shutting them comes, also at step 5.
5. **Survival rules,** in slices, each playable:
   - **Slice 1, fight or flee:** done 2026-11-06; 39 tests pass. What it has:
     - **the one rule for timed actions** (`dusk-core/src/combat.rs`):
       - an action starts between steps, faces its way and pays its endurance;
       - it winds up (the tell), takes effect on its strike tick, then recovers;
       - nobody moves while acting, and a blow during the wind-up breaks it off;
       - exhausted, wind-ups and recoveries take half as long again;
     - **endurance** (`body.rs`): running spends it, walking and standing still give it back. Empty, you're exhausted until it's back to 30: no running, and your blows do half harm. The body pays for the stride actually taken each tick; a test caught running that never tired you, because the tick a step ends counted as rest;
     - **a body:**
       - health out of 100, and wounds on ten parts (Zomboid's panel), each with its own odds of being hit;
       - scratches, lacerations and bites, with damage at once and then bleeding;
       - death at 0. Corpses stay where they fell, and nobody has to walk round them;
     - **your verbs:**
       - swing (aimed at the nearest of the dead you can reach, else ahead; a cudgel's numbers until weapons come with items);
       - shove (back a tile, breaks off what it was doing, 40% down; one that's down takes every blow, at double harm, Zomboid's stomp);
       - open or shut a door (0.5 s; it won't shut on someone stepping through, and the dead can't open one);
     - **the dead fight back:** beside their prey, they swing. A 0.8 s wind-up, then a 50% hit that leaves a scratch (70%), a laceration (25%) or a bite (5%). The dead and the downed see nothing;
     - **every number is in the data** (`rules.toml`: `[endurance]`, `[attack]`, `[shove]`, `[door]`, `[wounds]`; the shambler's `health`, `down`, `attack`, `wounds`), converted and checked by `dusk-data`;
     - **the client:**
       - F swing, Q shove, E door, and a click on one of the dead beside you swings at it;
       - every wind-up draws its tell: the tile it'll land on fills in, and a shambler's body leans in;
       - the knocked-down and the dead lie flat (the dead darker);
       - health and endurance bars, the wounds list and the news;
       - the death screen: "This is how you died.";
     - **measured:**
       - the 96 × 96 bench with 100 dead runs at 99,000 ticks/s (was 178,000): swings are sounds, and a crowd round you fills every listener's ear;
       - 500 dead: 17,000;
       - the whole patch: 404,000;
   - **Slice 2, grabs and doors:** done 2026-11-07; 45 tests pass. What it has:
     - **engaged** (`grab.rs`): a creature is on you when it stands beside you, sees you, and could reach you. The client draws a line from each, thick when it holds you;
     - **taking hold:** a creature's hit has a 15% chance to hold you, plus 15% for each other one on you, plus 20% when you're out of breath. Step out of reach of one on you, and it grabs as you go one time in four;
     - **held:**
       - you can't walk or swing, and get no breath back;
       - its blows always land, and 35% are bites;
       - the hold breaks when the holder is shoved, knocked down or killed, or you're out of its reach;
     - **the way out:** struggle (F while held: 0.5 s, 12 endurance, 70% less 20% for each other holder, half when out of breath), or shove the holder off (Q);
     - **the drag-down:** three holding you at once, or a hold with no breath left, and they drag you down. The death screen says which;
     - **doors the dead beat down:** a shambler with no way to you and a shut door straight ahead beats on it (2–4 a blow, heard 8 tiles off, so the noise draws the rest). At 60 it gives way for good, and the edge is open. You can beat doors down too. The world keeps each door's damage, in its hash;
     - **the client:**
       - engagement lines, and a "held" banner with its way out;
       - the news of grabs, struggles, blows on doors and broken doors;
       - **fixed:** a door opened, shut or broken now redraws its chunk's walls. Slice 1's doors changed in the core but never on screen;
       - dev flags: `--data=PATH` (another data folder: the start of mods) and `--do=S:ACTION` (scripted presses, for pictures);
     - **numbers** in `rules.toml` `[grab]` and `[door]` (strength, thump noise) and the shambler's `thump`;
     - **measured:** 100 dead 130,000 ticks/s, 500 dead 14,700, the patch 400,000;
   - **Slice 3, items:** done 2026-11-08; 52 tests pass. What it has:
     - **items as values** (`dusk-core/src/items.rs`): a definition's number, a count and a condition. The definitions come from `data/items/` (7 weapons, 10 supplies), shared read-only by every world as `Defs` (an `Arc`, the first of the architecture's shared definitions);
     - **containers fill from the seed:**
       - a container's contents are a function of the seed, its place and its loot table (`data/loot/`; the fixture names its table in `fixtures.toml`), worked out whenever it's looked into;
       - they're stored only once something is taken, so an unopened cupboard costs nothing and looking changes nothing (a test checks the state hash);
       - floors hold what's dropped there, and the client draws a bundle on each such tile;
     - **your pack and your hands:** a list with weights, Zomboid's. Over your carry weight (15 kg) you can't run;
     - **the item actions** (the one rule; a blow breaks them off):
       - take (0.4 s plus 0.1 s a kg) from a container or the floor within reach (beside you, no wall between);
       - drop;
       - use: a bandage (2 s) or a rag (3.5 s) stops the worst wound that's bleeding, which then shows as bandaged;
       - take a weapon in hand, or put it away;
       - each intent names the item it expects, so a click on a stale list takes nothing rather than the wrong thing;
     - **weapons:** each has its own swing (wind-up, damage, noise and endurance), from a quick, quiet kitchen knife to a slow, loud wood axe. Each blow that lands wears it, and at 0 it breaks. Nothing in hand, you fight with your fists (2–4 damage);
     - **the start kit** (`rules.toml`): a cudgel in hand and a bandage;
     - **the client:**
       - the pack panel (Tab): your load against what you can carry, what's in your hand, and the buttons;
       - the "within reach" panel: each container and the floor, with Take and Take all;
       - a progress bar for your own timed actions;
       - the news of what you took, dropped, wielded, bandaged or broke;
     - **food and drink** are loot now; eating and drinking come with needs;
     - **measured:** 100 dead 143,000 ticks/s; the patch 450,000;
   - **Slice 4, needs and moodles:** done 2026-11-08; 59 tests pass. What it has:
     - **the clock** (`dusk-core/src/needs.rs`): 24× action time, so a game minute is 25 ticks and a game day an hour at 1×. Day 1 begins at 08:00. Needs, sleep and healing run on it, a game minute at a time;
     - **hunger, thirst and fatigue,** out of 100, each with three levels and one clear effect at each:
       - hunger rises 6 a game hour, thirst 8. First peckish or thirsty; then hungry or parched (your breath comes back at 75% or 50%); then starving or dying of thirst (2 or 4 health a game hour);
       - fatigue rises 4 a game hour awake: drowsy at 45 (you can sleep), tired at 70 (wind-ups 25% slower), falling asleep at 90 (50% slower);
     - **eating and drinking** (the item action "use"): bread takes 40 off hunger in 4 s, an apple 10, dried meat 20, and a bottle of water 40 off thirst. Water beside you (a pond) slakes it all in 3 s;
     - **sleep:**
       - only when drowsy or worse, not held, and with none of the dead after you within 10 tiles;
       - beside a bed, fatigue falls 12.5 a game hour; on the ground, half that. Hunger and thirst rise at half pace;
       - a blow, a grab, any noise that reaches you, or choosing to get up wakes you; so does being rested;
     - **healing:** fed, watered and not bleeding, you get a point of health back a game hour (three asleep). A wound that's stopped bleeding closes in 12 game hours (a scratch), 48 (a laceration) or 96 (a bite);
     - **moodles,** worked out when asked (`World::moodles`): hungry, thirsty, tired, injured, bleeding, out of breath, heavy load, held and asleep, each with its level;
     - **numbers:**
       - `rules.toml`: `[clock]`, `[hunger]`, `[thirst]`, `[fatigue]`, `[sleep]`, `[healing]` and `[water]`;
       - food and drink: `items/supplies.toml`;
       - beds: `fixtures.toml`;
     - **the client:**
       - the clock in the top line;
       - the moodle column on the right: a name for each level, what it does to you, and a colour for how bad it is;
       - Z sleeps or gets up, and says why when you can't. X drinks from water, and the "within reach" panel shows water with a Drink button. The pack has Eat and Drink;
       - asleep, the world is veiled, the sleeper lies down with a "z", and the clock runs at 60× until you wake;
       - the death screen tells starving and thirst apart from blows and bleeding, and counts game time ("You lasted 14 hours 20 minutes");
       - **fixed:**
         - the tells drew a square for every timed action; now they draw one only for swings, shoves and blows on doors;
         - the empty "within reach" panel showed as a dark box at the start, and kept the last game's contents after R;
       - the dev flag `--news` prints the news (for `--headless` runs);
     - **measured:** 100 dead 140,000 ticks/s; the patch 357,000–393,000 over three runs (slice 3 noted 450,000; the needs run once a game minute, for characters only);
   - **Slice 5, day and night, light and the field of view:** done 2026-11-09; 67 tests pass. What it has:
     - **the sky** (`dusk-core/src/light.rs`): full light from 07:00 to 19:00, falling to the night's 20% by 21:00 and rising again from 05:00. Under a roof, 60% of the sky's. The news says when the sun goes down and when the sky lightens;
     - **carried lights:** a torch (lights 5 tiles, burns about a game hour) or a candle (2 tiles, about 4 hours), lit and held up in your other hand, so your weapon stays in hand:
       - 1.5 s to light, 0.5 s to put out;
       - it burns down a game minute at a time and goes out at nothing. One put out early keeps what's left, in a stack of its own;
       - the start kit now has a torch. Sheds and shelves have more;
     - **sight scales with light:** a tile lit at 100 is seen out to an eye's full range, at 50 half as far, and everyone sees what's beside them in any light:
       - by day a shambler sees you 8 tiles off; at night only beside it, unless you carry a light, and then from 8 tiles again. Light is bait (`CONCEPT.md`);
       - indoors by day it sees about 5 tiles;
     - **your field of view** (`view.rs`): a 160° cone, 20 tiles in full light (4 at night under the open sky, 5 with a torch), and the tiles beside you all round, past no wall. What you saw, you remember: the fog of war;
     - **the dead are a quarter faster at night,** the Hushlands' rule (the "more at night" comes with the population layer);
     - **numbers:**
       - `rules.toml`: `[day]`, `[sight]`, and `[items]` light and douse times;
       - the shambler's `darkvision` and `night_speed`;
       - `light = { radius, burn }` on the torch and the candle;
     - **the client:**
       - the sun, the sky's light and the background follow the core's daylight: a low blue moon at night, warm at dusk and dawn;
       - the light you hold up is a real light with shadows;
       - the fog of war is one full-screen pass (`shaders/fog_of_war.gdshader`). It reads each pixel's tile from the depth buffer and the core's view: seen tiles as they are, remembered ones dim and grey, the rest black;
       - creatures you don't see aren't drawn, and neither are their sight cones or tells. "Something has seen you" when one you can't see spots you;
       - L lights a light or puts it out. The pack shows what you hold up, with Light and Put out buttons;
       - the dev flag `--hour=H` starts at that hour;
       - **checked on screen:** at night with the torch lit, and by day after a walk (the seen cone, the remembered strip behind, black beyond). The day shot caught a bug: the shader read "seen" as "remembered" (bytes scaled by 127.5, not 255);
     - **also fixed:** a stack takes only items in the same condition, so a half-burnt candle doesn't join new ones;
     - **measured:** 100 dead 136,000 ticks/s, 500 dead 16,000, the patch 372,000;
   - **Slice 6, skills:** done 2026-11-09; 82 tests pass. What it has:
     - **six skills so far** (`dusk-core/src/skills.rs`): Blade, Blunt, Axe and Spear (each weapon names the one it trains and uses), Athletics and First Aid. The rest of `CONCEPT.md`'s list comes with what trains it: Sneak with sneaking, Carpentry with barricades;
     - **only real use under real risk teaches** (Default, from `CONCEPT.md`):
       - a blow that lands on a foe: 1 XP to the weapon's skill, half on one that's down. A swing at the air or a door, or with bare fists, teaches nothing;
       - a tile run with one of the dead after you within 12 tiles (the same "after you" that stops sleep), or carrying more than half of what you can: 0.2 XP of Athletics;
       - a bandage on a wound that's bleeding: 3 XP of First Aid;
     - **repeats pay less:** the same act on the same foe, or in the same 8 × 8 spot, pays three-quarters of what it paid the time before, until it's been left alone 6 game hours. A shambler stuck in a fence is worth about 4 XP in all, however long you beat it; running back and forth in one spot pays less than half of running the same distance away (a test checks it). Each character remembers its last 32 such acts, in its state hash;
     - **levels** 1 to 10 at 10, 30, 70, 150, 300, 550, 900, 1,400, 2,100 and 3,000 XP. A shambler felled in three blows pays about 2.3, so level 1 takes four or five;
     - **the free cap and three masteries:** every skill rises to 7 freely. There it stops learning and waits; choosing it as a mastery (`World::master`, the panel's button) takes it on to 10. A fourth can't be chosen. It's a choice, not a timed action (Default): like a talent pick, it changes nothing in the world;
     - **what a level does, a little each** (Default): a weapon skill adds 1% to a blow's chance to land and 5% to its harm, and takes 3% off its breath and 5% off the weapon's wear. The wind-up, the tell, stays the same. Athletics runs 2% faster for 4% less breath a level; First Aid bandages 5% faster;
     - **numbers:** `rules.toml` `[skills]`, `[skills.xp]` and `[skills.level]`; each weapon's `skill` in `items/weapons.toml`. `dusk-data` checks that levels rise, shares are shares, and a weapon names a weapon skill;
     - **the client:**
       - K opens the skills panel (`client/skills.gd`): each skill's level, a bar to the next, and "Master" on one waiting at the cap;
       - the news says when a skill rises, when it reaches the cap, and when you master it;
       - the dev flag `--do=S:skills` opens the panel, for pictures;
       - **checked on screen:** the panel after a fight on seed 1, Blunt's bar at the 2.3 XP of three blows;
     - **measured:** 100 dead 144,000 ticks/s, 500 dead 16,800, the patch 412,000 (no change: XP is paid only when something teaches);
   - **Slice 7, barricades:** done 2026-11-09; 89 tests pass. What it has:
     - **boards on the edge** (`dusk-core/src/barricade.rs`): a door or window carries how many boards are across it (`Edge::Door { open, boards }`, `Edge::Window { open, boards }`, as the world model above says), so walking and sight need nothing else. The world keeps what each board has left, in its hash;
     - **boarding up** (B; Defaults, Zomboid's numbers):
       - a shut door or any window, from either side. Each board takes a plank and two nails, with a hammer in hand or in the pack; up to four boards;
       - 4 s a board. The hammering is heard 12 tiles off as it starts and again as the last nail goes in, so the dead come: board up by day. The first picture of it caught a shambler inside the house coming out of its door and round the yard to the window being boarded;
     - **what boards do:**
       - a boarded door won't open;
       - a boarded window can't be seen through, either way. Glass can (pillar 4: a torch inside shows through bare windows);
       - the dead beat on a boarded door as on any shut door. The boards take it first, the last nailed first, 50 points each, then the door's own 60. Two boards make nearly three times the blows;
     - **prying a board off** (N): 3 s with the hammer. You get the plank back; the nails are bent;
     - **Carpentry,** the seventh skill: 2 XP a board, less each time on the same door or window (no farming by nailing and prying). Each level nails 5% faster and makes boards 10% stronger: each board keeps the strength it was nailed with;
     - **seen while testing:** the dead give up on a door after 30 s with nothing heard (`forget_after`), so boards and silence together keep a lone shambler out. A crowd doesn't keep itself at a door yet: one hears another's blows and goes to them, not to you. That's for the Director and sieges;
     - **numbers:** `rules.toml` `[barricade]` (its items and numbers; optional, so a data set with no items still loads), `[skills.xp] board` and `[skills.level] board_time`, `board_strength`. Nails are counted singly now (100 to a stack; sheds hold 6 to 24), and sheds hold more planks;
     - **the client:**
       - planks drawn across boarded doors and windows, tilted a little each. They aren't cut away, as glass isn't: the first picture lost them in the hole cut round the character, right where you nail them;
       - B boards up and N pries a board off, each saying why not when it can't (no hammer, no plank, two nails a board, shut the door first, it won't take another). E on a boarded door says so;
       - the news of each board nailed, pried off or broken;
       - the dev flag `--do=S:board` (and `unboard`). `--data` pointing at a copy of `data/` with a hammer, planks and nails in the kit made the pictures;
     - **measured:** 100 dead 140,000 ticks/s, 500 dead 16,100, the patch 405,000 (no change);
   - **Step 5 is done** with slice 7: everything in the list below is in.
   - bodies and wounds, moodles, needs;
   - items and the inventory window; containers that fill from the seed the first time they're opened, by loot table (moved here from step 4);
   - the one rule for timed actions;
   - combat as an economy (`CONCEPT.md`): endurance shared with running and exhaustion; the verbs, each with a wind-up, a strike tick and a recovery; tells per family; engagement; grabs with a way out;
   - skill XP gated by risk, with repeats paying less;
   - doors to open and barricade;
   - day and night, light and the field of view.
6. **The first playable,** in slices, ordered so that the loop gets a goal and a memory before it gets more content (2026-11-09 review; `CONCEPT.md`, "The core that has to be fun first"):
   - **Slice 0, creatures as data:** done 2026-11-09; 95 tests pass. What it has:
     - **a kind of creature is a row** (`dusk-core/src/creature.rs`): `Defs::creatures` holds a `CreatureDef` each (its id, its name, and its numbers as a `Creature`), and an actor of that kind is `ActorKind::Creature(n)`. The numbers are everything `Rules` knew about the shambler: speeds and the night's pace, sight, cone and darkvision, the groan, how often it thinks and how long it remembers, wandering, its search budget, health, how long it stays down, its swing, the wounds it leaves and what it does to a door;
     - **`Rules` has no creature in it** any more: sixteen fields went. A `Creature` is plain numbers and `Copy`, so the tick copies a kind's out while it holds the actor, as it does the rules;
     - **the loader takes every creature** in `data/creatures/` (it used to refuse any but one called "shambler"), each checked, with ids unique. A region names its dead (`[dead] creature = "shambler"`), and the name is checked;
     - **`Mind::Shambler` is `Mind::Hunt`:** going after the last thing heard or seen is a way of thinking, not a kind. Who fights whom is `ActorKind::hostile_to` (creatures against characters; the families fighting each other comes with the second family);
     - **the proof** (`tests/creature.rs`): a second kind, a "runner" three times as fast that sees twice as far, sees you from where a shambler can't and reaches you in a third of the time, with no rule written for it;
     - **no change in play:** `dusk-cli demo` prints the same 258 lines as before the change, and a test pins the numbers the game's shambler loads with. Gameplay rolls are keyed by actor id, not by kind, so none moved;
     - **for tests and tools:** `World::creature_mut(n)` changes a kind's numbers in one world (`w.creature_mut(0).wander = 0`). They live in the definitions, so set them after `w.defs` is replaced, not before;
     - **the client:** actor kind 1 + n is creature n, and `creature_names()` gives their names. The news still says "a shambler" in its own words; that changes when the ghoul arrives;
     - **measured:** 100 dead 129,000 ticks/s, 500 dead 15,500, the patch 367,000: about 8% under the last figures, for copying a kind's numbers each time an actor moves or thinks. Still six times the budget.
   - **Slice 1, saves:** done 2026-11-10; 99 tests pass. What it has:
     - **a save is the seed plus what changed** (`dusk-core/src/save.rs`): a header (the format's version, the seed, the tick, and a label: the region) and then the state that came from neither the seed nor the data: every actor, sounds not yet heard, what touched containers and floors hold, the harm doors and boards have taken, and the chunks that changed. After a 16-minute walk it's 5 KB;
     - **the grid notes what changed:** `Grid::track_changes` is called once the land is made, and from then a changed tile or edge marks its chunk. A save keeps those chunks whole. A new patch has none; shutting one door makes one, of 64. (A hand-made map never starts tracking, so all of it is saved: it has no seed to come back from.);
     - **loading** (`dusk_gen::load`) reads the header, makes that patch afresh from its seed (23 ms), and restores the save onto it (`World::restore`), which also works out again who stands where;
     - **what isn't saved, on purpose:** the rules and the definitions. They come from `data/`, so a retuned game loads old saves. Items and creatures are saved by number, though, so the header carries a fingerprint of their ids, and a save made with other data is refused, not loaded wrong;
     - **the core still touches no file:** it gives and takes bytes. The Godot extension writes them, beside the save first and then moved into place, so a crash half-way leaves the last save whole;
     - **`serde` and `postcard` are optional** (the core's `save` feature, the user's choice on 2026-11-09 over a hand-written format). `dusk-gen` turns it on; the core alone still builds with no dependencies. Each saved type carries `#[cfg_attr(feature = "save", derive(...))]`;
     - **the map you remember** is the caller's to keep (`View::remembered`, `recall`): the save holds it as opaque bytes, run-length packed (64 KB of tiles to a few hundred bytes);
     - **the tests** (`dusk-gen/tests/save.rs`): a played patch saved and loaded has the same state hash, plays on to the same hashes for 1,000 ticks under the same intents, and saves again to the very same bytes; a save holds only the changed chunk; a cut-short file, another seed's world and other data are each refused, leaving the world as it was;
     - **the game** (Defaults):
       - with no flags it's saved to `user://world.dusk` and carries on from it ("You carry on. Day 1 · 08:16");
       - it saves on each game hour, on F5, on Esc and on closing the window;
       - dying saves once, so there's no going back to before it; a save with you dead in it opens on the death screen;
       - R, alive, asks twice before leaving the world for a new one, which takes the save's place;
       - **any flag makes a trial run,** which neither loads nor saves unless `--save=PATH` says where. So pictures and tests never touch the player's world;
     - **checked in the game:** two headless runs, the second carrying on from the first's save at the right minute;
     - **not yet:** migrations between versions (a save of another version is refused with a message), more than one save, and saving the camera or the speed dial;
     - **measured:** no change (131,000 ticks/s with 100 dead, the patch 367,000).
   - **Slice 2, the stash and bring-it-home XP:** done 2026-11-10; 105 tests pass. What it has:
     - **home is where you slept the night through** (`dusk-core/src/home.rs`): waking rested makes the tile you slept on home. Sleep needs tiredness and none of the dead about, so home moves at most once a game day, and only after a safe night;
     - **why that rule** (Default, from `CONCEPT.md`'s review): with a stash allowed anywhere, you'd mark each house's cupboard as you looted it and never carry anything anywhere. The trip home is the risk the XP pays for;
     - **your stash:** one container under a roof within 8 tiles of home (`World::can_stash`; the reasons why not are `NoStash`). Marking another moves it; the things stay where they are;
     - **finds:** an item that came out of a container's loot is a find (`Item::found`). Put into your stash (`Intent::Put`, the new verb for putting anything into any container within reach), a find pays its worth in XP and becomes an ordinary item, wherever it goes after. What you start with, and what's been home once, pays nothing: there's no selling and buying back, and nothing to farm by taking out and putting in. Finds and ordinary items don't stack together;
     - **worth** is a field on every item (`items/*.toml`, `worth`), chosen for the worth-per-kilo choice: coins 100 XP a kilo, a bandage 30, a wood axe 3, a plank 0.5 (better nailed up than banked);
     - **levels 1 to 10** from deed XP (30, 80, 160, 300, 500, 800, 1,200, 1,800, 2,600). Each brings 5 health, given at once, so level 10 has 145 against 100 (`World::max_health`). That's under `CONCEPT.md`'s 2–3×, on purpose: wounds here are fixed amounts, so 250 health would shrug off a crowd, and pillar 3 is "fragile, not weak";
     - **measured with `dusk-cli gen`,** which now says what a patch holds: seeds 1 to 3 have 240 to 370 XP of finds in 50 to 61 containers, 70 to 90 kg of them. That's five or six full trips, and level 4 or 5 for emptying a patch;
     - **numbers:** `rules.toml` `[deeds]` (levels, health a level, the stash's time, how far from home); each item's `worth`;
     - **the save format** is version 2 (items carry whether they're finds; actors their home, stash and deeds);
     - **the client:**
       - the body panel says your level and XP, and "no home yet" until you have one; the health bar grows with your level;
       - each container within reach has "Make it your stash" (saying why not when it can't be); your stash says so, with "Stash all finds";
       - with your stash within reach, every line of the pack has a Stash button; finds say what they'd bring home ("a find: 8 XP home");
       - the news says when you've made a home, marked a stash, brought something home (and for how much) and reached a level, with a warm glow;
       - **checked on screen:** the level line, at the start of a trial run. The stash's panels weren't caught in a picture (the walk there met the shambler by the road); the core's tests cover them;
     - **not yet:** XP for places found, days survived and bounties (`CONCEPT.md`'s other deeds), and selling at a holdout (slice 5);
     - **measured:** 100 dead 133,000 ticks/s, the patch 386,000 (no change).
   - **Slice 3, the ghoul and ghoul fever:** done 2026-11-10; 114 tests pass. What it has:
     - **the ghoul is a row of data** (`creatures/hushlands.toml`), as slice 0 promised. It needed two new numbers on `Creature`, and no rule:
       - it runs 2.0 tiles a second (2.5 at night) against your walk of 1.5 and your run of 3.0. Walking won't lose one; running will, for the 25 seconds a full bar of endurance lasts;
       - it makes no groan: it doesn't share its prey, so a ghoul comes alone and quietly;
       - it thinks every 3 ticks (a shambler every 5), searches 800 tiles for a way (400) and sees 3 tiles in the dark (1). It's frailer (25 health against 30), and its tell is 1.0 s (the shambler's 1.2);
     - **drawn to blood** (`Creature::blood`, `World::smells`): a creature that sees nothing smells a character within `blood` tiles for each wound that's bleeding, up to three (the ghoul's is 12, so 12, 24 or 36 tiles). Walls and the dark don't stop a scent. It goes to where you are now, not where you were, so a ghoul can't be lost while you bleed. That's what makes the bandage the first move and the run the second;
     - **ghoul fever** (`dusk-core/src/fever.rs`): a wound from a kind that carries it (`Creature::fever`, a chance for each kind of wound: the ghoul's scratch 5%, laceration 25%, bite 75%) may bring it. A shambler's bite is only a bad wound. It's kept as the game minute it was caught, and the stage is worked out from the clock:
       1. **the first day, a fever:** thirst rises half as fast again, and nothing heals;
       2. **the second, weakness:** endurance comes back at half the pace too;
       3. **the third, dying:** 4 health an hour, and at the day's end it kills you, whatever's left. The day's end is checked before the hour's harm, so it's the fever that's named;
     - **the cure** is an item (`ItemKind::Cure`; the feverfew tincture, worth 15 XP brought home), taken at any stage. It's in wardrobes and on shelves: one to four in a patch (eight seeds measured), so a patch always has one to find. The abbey sells it from slice 5;
     - **the region's dead are a mix** (`regions/*.toml`, `[dead] kinds`, each with a share): the Hushlands' are 90% shamblers and 10% ghouls, 7 to 13 a patch. Which kind stands on a tile is a roll of its own, so every place the dead stood before still has one;
     - **numbers:** `rules.toml` `[fever]` (the stage's length, the thirst, the breath, the harm); a creature's `blood` and `fever`; the tincture in `items/supplies.toml` and `loot/hamlet.toml`;
     - **the save format** is version 3 (the body carries its fever);
     - **the client:**
       - the news names the creature ("A ghoul saw you!", "A ghoul gave you a bite on the left arm.") from `creature_names()`, not "a shambler" any more;
       - the ghoul is drawn pale, lower and crouched, so a glance tells it from a shambler;
       - the fever is a moodle with three stages, and the bleeding moodle says ghouls smell it. The news tells of catching it, each stage and the cure, and the death screen says when the fever killed you;
       - a tincture in the pack has a Take button, which says why not with no fever;
       - **a save that won't load is moved aside** (`world.dusk.old`), not saved over. This slice's own check found it: a run with no flags met the old version-2 save, refused it, and saved a new world over it on quitting;
       - **checked on screen:** "A ghoul saw you!" in the news, from a copy of `data/` with only ghouls. The ghoul's look wasn't caught in a picture;
     - **`dusk-cli gen`** says the dead by kind and the cures a patch holds;
     - **not yet:** rising as a revenant after the fever (slice 7); ghouls eating the dead (an action that can be interrupted, as this file says); a trail of blood on the ground, rather than a scent from where you stand;
     - **measured:** 100 dead 122,000 ticks/s, the patch 351,000 (it was 386,000: ghouls think more often and search farther).
   - **Slice 4, the dead get up:** done 2026-11-10; 124 tests pass. What it has:
     - **a body's fate is settled when it falls** (`dusk-core/src/remains.rs`, `Actor::remains`): its kind says whether it gets up and when (`Creature::get_up`, a chance and a span of game hours), and the tick it will is kept on the body (`Remains::GetsUp`). Nothing counts down. The Hushlands' shamblers and ghouls all get up, 12 to 36 game hours later;
     - **getting up:**
       - it's whole again, and on its back for its kind's knock-down time: a window to finish it, as a stomp on one that's down always lands;
       - it waits while someone stands on its tile;
       - it gets up once (`Actor::risen`). Put down again, it stays down, so a second fight is the other answer to the oil. Thrice-Risen, the rare affix, will be the one that gets up twice;
     - **burning** (`Intent::Burn`, `World::can_burn`, whose reasons why not are `NoBurn`):
       - it takes a flask of oil from your pack (`ItemKind::Oil`) and the light you hold up as the flame. The body is the one you face, else the one you stand on, else the first round you;
       - pouring the oil takes 3 s, and the body burns for 20 game minutes, then it's ash (`Remains::Burning`, `Remains::Ash`);
       - a burning body lights 4 tiles round it as a torch does (`World::fires`, added to `light_at`), so a pyre is also a light in the dark;
     - **a pyre:** fire goes from a burning body to the bodies on its tile and the tiles round it, a tile a game minute, but not through a wall or a shut door (the same reach as taking things). So the dead you put down at your door, piled up, burn from one flask. Nothing else catches yet: fire as a system (wood, webs, spreading with the wind) comes with the regions that need it;
     - **the region's pressure** (`dusk-core/src/pressure.rs`, `World::pressure`):
       - it's one number, in thousandths of the dead the patch began with;
       - time raises it to a ceiling (Zomboid's population peaks and stops), each body that gets up raises it (past the ceiling, up to the most), and each body burned lowers it (to the least);
       - **what it does:** while the patch holds fewer of the dead than it says (`dead_about` against `dead_wanted`; bodies that will still get up count), one walks in every so often. It comes onto open ground in a chunk that's asleep, so never where you are;
       - the kinds come by the region's shares, as at the start;
     - **the region in the core** (`dusk_core::Region`): the pressure's numbers, the kinds of the dead, the patch's size and how many dead it began with. `dusk-gen`'s `patch` gives it to the world from the data. Like the rules, it isn't saved; the pressure itself is;
     - **numbers:**
       - `rules.toml` `[burning]` (the pouring time, the burn, the light);
       - a creature's `get_up`;
       - a region's `[pressure]`, in per cent: start 100, least 25, most 200, 5 a day to a ceiling of 150, 2 a rise, 2 a burn, one walks in every 30 game minutes;
       - the flask of lamp oil in `items/supplies.toml`, in kitchens, sheds and barrels: 6 to 24 a patch over eight seeds;
     - **the save format** is version 4 (bodies carry their fate and how often they've got up; the world its pressure). The fires are worked out again from the bodies on loading;
     - **the client:**
       - G burns the body beside you, saying why not: no body, no oil, no flame. A flask in the pack has a "Burn a body" button;
       - a burning body glows and throws a flickering light, and ash is a low dark smear (`DuskWorld.actor_remains`);
       - the dead that walk in get a view as they come (`main.gd`, `_add_new_actors`). The core adds them at the end of its order, so the others keep their places. The bridge's smoothed positions take a newcomer where it is;
       - the news says when one you see gets up again, when you set a body alight, and when the fire spreads. The first body you see fall says it'll get up unless you burn it;
       - the clock line shows the pressure ("the dead 104%");
       - **checked:** headless trial runs with G pressed (it said why not). Fire and ash weren't caught in a picture: making a pile on screen needs a fight a script can't yet stage;
     - **`dusk-cli gen`** says the oil a patch holds, and `--ticks` the pressure before and after (seed 1, a game day with nobody about: 100% to 105%, four of the dead walked in);
     - **not yet:** holy water and consecration (the cleric); bounties and a supplied holdout lowering the pressure (slice 5); a relit wardstone lowering the ceiling (the dungeons); ash that's cleared away (bodies stay in the world, so it grows a little with every arrival); fire that harms the living or spreads to the house;
     - **measured:** 100 dead 125,000 ticks/s, the patch 341,000 (no change worth the name).
   - **Slice 5, the abbey holdout:** done 2026-11-10; 133 tests pass. What it has:
     - **the abbey** (`prefabs/abbey.toml`): a walled close with the chapel, the sisters' dormitory and the stores, its gate open on the road. `dusk-gen` stands it north of the road where it starts (`Layout::place_holdout`, from the region's `[holdout]`), as a lot marked `holdout`: no fence round it, and none of the dead inside. You start just inside its gate; the hamlet is the road's `start` tiles east;
     - **hallowed ground** (`dusk-core/src/holdout.rs`, `World::hallowed`, from the patch's making like the region; not saved). Inside the walls:
       - the dead never step (`move_actor` refuses the step);
       - they don't see, smell or hear anything there (`nearest_visible_character`, `smells` and `hear` skip it), and one after a target there gives it up (`think`). So a crowd that followed you to the gate drifts off;
       - you can sleep with the dead about (`can_sleep`), but a bed there is a guest's: it doesn't make a home (`make_home`);
       - the dead don't walk in there (`walk_in`);
     - **the counter** (`Defs::counters`: a fixture with `trade = "abbey"`; `World::counter_in_reach`). It's a fixture, with the sister drawn behind it, until a holdout's people are actors. Over it, three timed deals (`Intent::Buy`, `Sell`, `Mend`; the reasons why not are `NoDeal`):
       - **buy:** each line has a price and a stock a game day. What's sold is kept by (holdout, line) with the day (`World::sold`, saved), so the stock comes back with the day and nothing ticks;
       - **sell:** anything with a worth, for a share of it in coin (the abbey's is half). A find sold there is brought home: it pays its worth in XP, once (`BroughtHome`, as in the stash). Coin isn't sold. You pay with ordinary coin first, so a found coin keeps its XP as long as it can;
       - **supply:** selling what it's short of (`Holdout::wants`) also lowers the pressure, by its worth (`Supplied`);
       - **mend:** the weapon in hand, good as new, for its share of the weapon's worth by how worn it is, and at least a coin;
     - **coin** is the `coins` item (`Defs::coin`), carried and weighed: a coin is worth 1, as XP brought home or as a price;
     - **numbers** (`data/holdouts.toml`):
       - the abbey pays half an item's worth, and a full repair costs its whole worth;
       - a deal takes 1.5 s;
       - it sells the tincture (30 coin, one a day), bandages 6, oil 5, bread 5, water 4, torches 4 and candles 2;
       - it's short of bread, apples, dried meat, water, oil and candles, at 0.25% of pressure a point of worth;
       - a test holds the game's data to two things: the cure costs 20 coin or more, and no more than half of what the leanest patch's finds fetch (113 coin); and nothing sells back for more than it costs;
     - **the save format** is version 5 (what the holdouts sold, and when);
     - **the client:**
       - at the counter, the "within reach" panel lists the stock with prices, what's left today and Buy buttons, and says what the abbey is short of;
       - the pack's lines get a Sell button with the price, the weapon in hand a Mend button, and the pack's title shows your coin;
       - the news says what you bought, sold and had mended, and when the dusk falls back from supplying the abbey; stepping on and off hallowed ground says so too;
       - **checked on screen:** one picture at seed 1's counter: the stores room, the sister, the trade panel with the day's stock, and Sell on the pack's lines. Headless runs walked there without errors;
     - **not yet:** the sisters as actors, defenders, morale and raids; bounties (they need rare monsters); training and healing for coin; the next character starting here (slice 7 makes death lead to it); selling the weapon in your hand;
     - **measured:** 100 dead 127,000 ticks/s, the patch 349,000 (no change).
   - **Slice 6, the Tolling:** done 2026-11-10; 140 tests pass. What it has:
     - **the calendar** (`dusk-core/src/tolling.rs`, the region's `Tolling`). The Hushlands' first Tolling is on day 3 and then every 5 days. The bell begins at 22:00, tolls every 2 game minutes for an hour, and its pull lasts until dawn. It's all worked out from the clock: nothing is saved. Word comes at dawn on the day (`TollingTonight`), and the HUD counts down to it;
     - **the bell** hangs just inside the hamlet chapel's door (`Layout::bell`, from `[tolling] bell = "chapel"`). Its tolls are sounds (they wake you, and they're drawn), but the dead don't chase them as noise (`World::is_bell`): the field is how they go;
     - **flow fields** (`dusk-core/src/flow.rs`): one Dijkstra map from the bell over the square within its reach, shared by the whole crowd. Each creature after nothing, within reach, steps to the cheapest tile round it that's free (`Flow::step`), else waits. A shut door is a way through at a cost (12 tiles' worth): the way goes through it, and the dead beat it down. The field is planned when the bell begins, and again when the map has changed (`Grid::edits`, a counter that neither hashes nor compares), at most every 5 game minutes;
     - **the bell keeps the world awake round it** (`World::anchors`): the dead walk to it with you away at the abbey;
     - **they don't give up:** on a Tolling night, the dead within the bell's reach don't forget what they're after (`think`). Seen or heard going into a house, you're besieged in it: they beat on the door until it gives, or dawn comes. The field is what says the Tolling is on (`drawn_to_bell`), so nothing works out the calendar on every thought;
     - **found on the way, and fixed:**
       - the longhouse's door opened onto a tile boxed in by chairs and the table, so nobody could get in, you or the dead;
       - the farmhouse's kitchen, mirrored, had a corner nobody could reach;
       - the abbey chapel's front corners were shut in by the altar and pews;
       - a new test holds every floor of every building on six seeds to being walkable from its door;
     - **measured** (`dusk-cli gen --tolling`, seeds 1 to 3): 49 to 54 of the dead are within the bell's reach, and two hours on, 43 to 49 of them are within 15 tiles of it. The patch runs at 120,000 to 126,000 ticks/s through it. `bench --tolling`: 100 dead 43,500 ticks/s, 500 dead 5,800, all of them after you and none giving up (above: the cache of open steps is next);
     - **numbers:** a region's `[tolling]`: the bell's prefab, the first day, days apart, the hour, how long it rings and how often, how far a toll carries (60 tiles), the reach (96), and what a door is worth to the dead;
     - **the save format** doesn't change: the calendar comes from the clock, and the field from the map;
     - **the client:** the clock line counts down ("the Tolling in 2 days", "in 3h 20m", "THE TOLLING, till dawn"), and the news tells of it at dawn on the day, as it begins and as it ends. **Checked:** headless, on a copy of `data/` with the Tolling minutes away: "The bell is tolling!". The crowd at the chapel wasn't caught in a picture;
     - **not yet:** the Director (the pressure choosing how often the event comes); a raid's horns and the hunter's moon (other regions); the dead breaking windows; rewards for a night survived.
   - **Slice 7, death and the revenant:** done 2026-11-10; 146 tests pass. What it has:
     - **names** (`data/names.toml`; `Actor::name`, rolled as a character comes into the world; `World::name_of`). A character also keeps the minute it came and the minute it died (`born`, `died`), so `World::lasted` says how long it lasted;
     - **the body keeps its gear** (`dusk-core/src/legacy.rs`). Unburned, at the next midnight (rules.toml's `[revenant]`) it rises as a **revenant**: a creature of its own (`creatures/hushlands.toml`: twice a shambler's health, faster than your walk, no groan, and it doesn't wander, so it haunts the place it fell). It has the body's name and everything the body carried, and the body is gone (`Remains::Risen`). It waits while someone stands on the body;
     - **gear comes back:**
       - any creature put down drops what it carries where it falls (`shed_gear`), so killing the revenant gets your things back;
       - burning the body before midnight stops it rising, and its gear falls where it lay: items don't burn;
       - a revenant put down stays down;
     - **the next character** (`World::next_character`) comes to the region's start (the abbey's gate, `Region::start`; the nearest free tile if someone's on it). It brings the last one's home and stash, so the base carries over; levels and skills don't. `World::character` is the last to come, so everything that asks for "you" follows the newest;
     - **a dead character no longer keeps the world awake** (`anchors`): the world sleeps round a body until the next one comes near;
     - **the save format** is version 6 (names, the minutes of coming and dying, the risen body);
     - **the client:**
       - the death screen is a recap: who, how long they lasted, how they died (the client's reading, as before), what they carried, and what becomes of the body;
       - R brings the next survivor into the same world, and Shift+R starts a new world. Alive, R still asks twice before abandoning the world;
       - the news says when your old body rises;
       - a risen body isn't drawn, and the revenant is drawn tall, upright and dark;
       - `--do=S:next` brings the next survivor from a script;
       - **checked:** headless on a copy of `data/` that starves you on the hour: the next survivor, Cole Hale, came to the abbey. One picture of the death screen's recap;
     - **found on the way:** `dusk-cli bench` had its character dragged down part-way, after which the crowd stood still once dead characters stopped keeping the world awake. The bench now keeps you alive in the crowd the whole run, which is harsher than before: 100 dead 86,000 ticks/s, 500 dead 12,600 (with the Tolling, 38,000 and 5,300);
     - **not yet:**
       - the revenant fighting the way you did (a bot trained on your play: the stretch goal);
       - class abilities (no classes yet);
       - what you've learned and the places you found carrying over (the map does: it's the client's, and the save keeps it);
       - the holdouts remembering you.
   - **Last, if the loop is fun without them:** magic items and identification, rare affixes, rain, a class. Built so far, ahead of a playtest (the user said go on):
     - **Rares and bounties** (2026-11-11; 156 tests pass):
       - **an affix is data** (`data/affixes.toml`; `dusk-core/src/rare.rs`, `Affix`): what it does to its creature's numbers. Seven so far:
         - the Frenzied (faster, a quicker tell);
         - the Ironhide (2.5 times the health);
         - the Bellwether (its groan carries 20 tiles more);
         - the Hunter (keeps after you four times as long, and follows blood);
         - the Lantern-Eater (lights held up within 4 tiles go out);
         - the Thrice-Risen (gets up once more than its kind);
         - the Plaguebearer (its wounds carry ghoul fever).
         The Spectral (walking through doors) waits: it needs a movement rule;
       - **an actor's numbers are its kind's with its affixes on top** (`World::stats`, used wherever a rule asks what a creature can do). Three hot paths read the kind in place for speed: the thinking beat, the eyes (`sees`), and the legs of all but a rare (`pace`). No affix changes the first two;
       - **a rare** (`World::make_rare`) is one of the region's dead with one or two affixes, a name from `names.toml` (`rare`), the first affix's epithet ("Gaunt the Lantern-Eater"), and a pack of finds from `loot/rares.toml`. Put down, its pack falls where it does, as any creature's now does;
       - **where:** `dusk-gen` makes three a patch (the region's `[rares]`), chosen by the seed among the dead at least 40 tiles from where you start, each with its own name;
       - **bounties** (`holdouts.toml`, `bounty`): the abbey's counter lists each rare, where it was last heard of (the middle of its chunk: rumours, not places), and whether it's been put down. Put down, it's claimed over the counter, once (`Intent::Claim`, `World::claimed`, saved): 25 coin, 30 XP, and the pressure falls 3%. That's the last of the pressure's deeds `CONCEPT.md` names that could be built without dungeons;
       - **the save format** is version 7 (an actor's affixes, the bounties claimed);
       - **the client:**
         - a rare has its name over its head, in gold, and stands a little bigger;
         - the news names it ("Gaunt the Lantern-Eater saw you!"), tells when it puts out your light, and when a bounty's paid;
         - the counter's panel lists the bounties, with Claim once one's put down;
         - the pack's panel now sits above the controls, as the other does;
         - **checked on screen:** the bounty list at seed 1's counter, a second affix shown ("the Thrice-Risen too"). No rare was caught in a picture: they stand 40 tiles out and more;
       - **measured:** at first this cost 10–15%: `stats` was copying a creature's numbers for every actor on every tick, before thinking even checked its beat. Fixed by checking the beat first and reading the kind in place on the hot paths. 500 dead: 22,400 ticks/s (23,300 before); through the Tolling 16,000 (16,600). The quiet patch is 10% slower (277,000; it was 309,000), the same with no rares in it, so most likely the actor's new field: not worth chasing at 27,700× real time;
       - **not yet:** rares roaming, the Spectral, family affixes (a Gravecaller raising the dead). Their packs now hold magic weapons (below).
     - **Magic items and identification** (2026-11-11; 164 tests pass):
       - **an item can be magic** (`dusk-core/src/magic.rs`, `Item::magic`, a small `Copy` block). An Uncommon item has one property and a Rare one two, and some are cursed as well. Two magic items stack only if they're the same in every way;
       - **properties are data** (`data/properties.toml`, `Property`), each one line. On a weapon, in hand:
         - Muffled: a third of the noise;
         - Keen: half as much harm again;
         - Swift: a quarter less tell and recovery;
         - Sure: more blows land;
         - Enduring: a quarter of the wear;
         - Hallowed: what it puts down stays down, as good as burning;
         - Blazing: 4 tiles of light round you, which the dead see by.
         On a light, held up: Bright (3 tiles farther) and Lasting (burns half as fast);
       - **curses are properties marked as curses:** Clamorous, Brittle and Leaden on weapons, Guttering on lights. A cursed weapon clings: once taken in hand it can't be put away or changed (`World::hand_clings`, refused in `begin`), and its curse shows itself then (`CurseClings`). The abbey lifts it for 20 coin (`Intent::Uncurse`); the rest of its magic stays;
       - **the effects go through the rules that already exist:**
         - a weapon's swing (`swing_of` → `magic_swing`) and its wear (`magic_wear`);
         - the light round you (`carried_lights`) and how fast a light burns down (`burn_lights`);
         - Hallowed, in `strike`: the body put down is set to stay down;
       - **unknown until identified:** a magic item found works from the first, but its name says only "(magic, unknown)". You come to know it three ways:
         - the abbey reads it for 5 coin (`Intent::Identify`, a pack slot or `HANDS`);
         - 15 blows that land on the dead with it in hand (`learn_by_use`; blows into the air teach nothing, so you learn it in a fight);
         - a light, as soon as you light it.
         Its name then reads "Muffled Keen knife" (`Defs::label`), with a line for each property (`Defs::lines`);
       - **where they come from:**
         - a loot table's line can give how often its item is magic (`LootEntry::magic`), rolled with the container's seed like the rest of it (`Purpose::Magic`). Weapons in sheds and kitchens are magic one time in ten or so, torches one in twenty;
         - a rare's weapons always are;
         - over six seeds: two to six magic items a patch, nought to three of them in containers, about one in five cursed;
       - **worth:** a magic item is worth its kind's plus 20 XP (Uncommon) or 50 (Rare) (`Defs::worth`). That's what it brings home, what the abbey pays (half), and what mending it costs;
       - **numbers:** rules.toml's `[magic]` (the share that's Rare, how often cursed, the worth, the blows to learn); `holdouts.toml`'s `identify` and `uncurse`; `magic` on loot lines;
       - **the save format** is version 8 (items carry their magic);
       - **the client:**
         - the pack names things as far as you know them (`character_labels`), in blue for Uncommon and gold for Rare, dimmer while unknown, with each property's line under it once known;
         - at the counter: "Read · 5" on unknown magic, and "Lift the curse" on a cursed weapon in hand;
         - a cursed weapon's Put away becomes "It clings", and Wield says why not;
         - the news tells of learning an item, of a curse clinging, and of a curse lifted;
         - `dusk-cli gen` lists a patch's magic as if known;
         - **not checked on screen:** the magic-only parts of the pack (the colours and the lines). The start has no magic and no script can take from a container yet. Headless runs walked the panels with ordinary items without errors;
       - **not yet:**
         - attunement (it caps worn items, and there are none yet);
         - the Lore skill and scrolls of identify;
         - magic on armour;
         - Very Rare and Legendary (the dungeons);
         - a broken magic item that stays, to be mended (today one breaks like any weapon).
     - **Rain** (2026-11-11; 170 tests pass):
       - **the weather comes from the seed and the clock** (`dusk-core/src/weather.rs`, `World::rain`). Time falls into spells (the Hushlands': 4 game hours), each dry, rain or heavy rain by a roll of its own (`Purpose::Weather`). Nothing is saved, like the Tolling's calendar;
       - **what it does, outdoors only** (not under a roof):
         - a light held up may go out, a chance each minute: 2% in rain, 8% in heavy rain (`RainedOut`). It goes back into the pack, as if put out;
         - a sound made outdoors carries 75% as far, or 50% in heavy rain (`World::carries`). That's for the dead hearing you and for a noise waking you;
         - blood outdoors is smelled from half as far, or a fifth (`World::scent_reach`);
       - **the sky is darker** everywhere: 75%, or 55% (`daylight()`), so everyone's sight by daylight is shorter. It's no more night for the dead: they count night by the clear sky's light (`clear_daylight`);
       - **numbers:** a region's `[weather]`: the spell, the share of rain and of heavy rain, and, for light and heavy rain, sound, scent, sky and the chance a light goes out;
       - **the save format** doesn't change;
       - **the client:**
         - rain drawn as streaks falling round the camera, more of them in heavy rain;
         - the sky darkens with the core's daylight;
         - the clock line says "rain" or "heavy rain";
         - the news tells when it begins and ends, and when it puts your light out;
         - **checked on screen:** a picture in heavy rain, from a copy of `data/` where it always rains;
       - **not yet:** fog, wind, snow and seasons; rain filling barrels; being wet and cold; lanterns, which rain doesn't put out.
     - **The Fighter** (2026-11-11; 181 tests pass), the last on this list:
       - **a class is data** (`data/classes.toml`; `dusk-core/src/class.rs`, `Class`). The Fighter has:
         - **skills it learns half as fast again:** Blade, Blunt, Axe and Athletics (`train`);
         - **cheaper endurance:** its running, swings and struggles cost 85% (`endurance_rate`, in `start` and `bodies`);
         - **its signature, shrugging off grabs:** the dead get hold of it three times in four as often (`grabbed_chance`, in `grab_chance`);
         - **ten talents** to pick from.
         Every character is the first class in the data (`spawn`); choosing one waits for a second class. A test world with no classes has classless characters, as before;
       - **talents are data** (`data/talents.toml`, `Talent`), each one line:
         - Tough: 15 health;
         - Tireless: a quarter off endurance;
         - Second Wind: a verb (`Intent::SecondWind`, C). Once between sleeps, full endurance and 10 health back. It comes back on waking rested (`Actor::winded`);
         - Cleave: a blow that puts one down carries on into the next foe beside you, once a swing (`World::cleaving` keeps one cleave from starting another);
         - Shrug: struggling free +20%, and one more must hold you to drag you down;
         - Bash: shoves knock down +35%;
         - Doorwarden: beside a door, +15% to land and half the grabs;
         - Heavy Hand, Edge, Woodsman: a quarter more harm with blunt weapons, blades or axes.
         They work through the rules that already exist: `swing_of` (now `talent_swing` on top of `magic_swing`), `grab_chance`, `struggle`, `drag_downs`, the shove's knock-down, and `max_health`;
       - **picks** (`World::talent_offer`, `choose_talent`, `picks_left`): each level from 2 offers three of the class's talents not yet taken, chosen by the seed (`Purpose::Talent`), the same offer however often it's asked. You pick one when you like: a choice, not a timed action, like a mastery. Its health comes at once;
       - **the save format** is version 9 (an actor's class, talents, and whether its second wind is spent);
       - **the client:**
         - the skills panel (K, now "skills and talents") shows your class, the talents you've taken with their lines, and, with a pick to make, the three on offer with Choose;
         - the news says when a level brings a talent to choose, what you chose, and your second wind;
         - **checked:** a headless run with the panel open. The offer itself wasn't caught on screen: a trial run starts at level 1, with no way yet to give XP from a script;
       - **not yet:**
         - choosing a class at the start (with the Cleric, the second);
         - a class's starting kit (the rules' kit is everyone's);
         - shields, so Brace and Shield Bash as the design names them (Bash is a harder shove for now);
         - a holdout trainer re-picking talents for coin.
   - **The rest of the first playable** (`CONCEPT.md`'s list), in slices: the river; the market town; farms; a fire for your base; the pain and dread moodles.
     - **The river** (2026-11-11; 184 tests pass):
       - **two new grounds** (`Ground::Shallows`, `Ground::Bridge`, `;` and `_` in text grids):
         - **shallows** are walkable. A step into them goes at half pace for everyone, the dead too, and a splash carries half as far again as a footstep (rules.toml's `[shallows]`, `Rules::shallows`; settled as the step starts, `Step::wade`). You can drink standing in them (`water_beside`);
         - **a bridge** is planks over water, walked as a road;
       - **the river** (`dusk-gen`, `Layout::place_river`, `river_at`; a region's `[river]`):
         - **where:** north to south across the patch, east of the hamlet if there's room, else between the holdout and the hamlet, clear of every lot by its margin, wherever it bends;
         - **its course:** its middle line bends by smooth noise (stretched, so it uses its whole sway);
         - **across:** deep water in the middle (5 tiles), a tile of shallows at each bank;
         - **fords:** two rows by the seed, at least 20 tiles from the bridge and 24 from each other, shallow all the way across;
         - **the bridge:** where the road meets the river, the road wins, as bridge planks;
       - **what it does:** the far bank is reached by the bridge, a choke point, or by a ford, slow and loud. The dead go round by the same ways (the paths and the Tolling's field already avoid deep water). At the Tolling, 46 to 50 of the dead in reach still gather within 15 tiles of the bell (seeds 1 to 3);
       - **the save format** is version 10: the land itself changed, so an older save would have a river through it;
       - **the client:** shallows drawn as lighter water, a bridge as dark planks. The controls are now three lines (they ran off the screen), with both panels lifted clear of them;
       - **checked:** the patch's map image shows the river, its bridge and its fords (seeds 1 and 2). A picture at the bridge didn't get there: the walk was caught by a shambler in the hamlet;
       - **measured:** no change to a tick (wading is settled once a step).
     - **The market town** (2026-11-11; 188 tests pass):
       - **the town:** the hamlet grown into a market town of 13 lots. At its heart the chapel (the Tolling's bell) and the tavern face each other across the road, with the smithy and the general store beside them; houses and outbuildings beyond (`regions/hushlands.toml`, `[hamlet]` lots);
       - **where it lies** (`Layout::town_middle`): anywhere along the road that leaves room west of it for the start and the abbey, and east of it for the river. The lots are laid out once about the patch's centre to measure the town (how far it reaches either side of its middle doesn't hang on where it lies), then its middle is rolled within the room that's left. Before, a middle rolled far west would have pushed a town this wide's abbey off the patch;
       - **the shops** (`prefabs/town.toml`), each worth the walk for one thing, in fixtures of their own (`fixtures.toml`) filled from tables of their own (`loot/town.toml`):
         - **the smithy:** a cold forge, an anvil, the smith's room behind, double doors that are hard to hold, and two weapon racks that always hold a weapon, mostly the smith's own work. That's three new forged weapons, each the best of a skill that had only a poor one: the arming sword (blade), the spear (the quietest weapon with real harm in it) and the mace (blunt; two blows fell a shambler nearly every time). They wear at a third to a half of a hand axe's rate, and a fifth of them are magic;
         - **the general store:** aisles of shelves you can't see past, stocked with supplies in quantity (oil, candles, torches, bandages, food, nails); a till of 5 to 20 coins; a storeroom at the back;
         - **the tavern:** the common room with its cold hearth and tables, the bar with its takings, a kitchen corner, two rooms let to travellers, and the cellar's casks, with a door out the back. It brings ale (a drink) and cheese. It was full when the Dusk came: 2 to 5 of the dead are inside;
       - **a building's own dead:** a prefab's `dead`, over the region's `per_house` (`Lot::dead`);
       - **a building's name:** a prefab's `name` (its id if it has none; `Lot::name`). The HUD says which you're in, "in the smithy" (`DuskWorld::place`, by the lots' walls);
       - **the save format** is version 11: the land changed again;
       - **the client:** the shops' furniture, and the HUD's place and title;
       - **fixed on the way:**
         - `--hour` set the clock's start after the world was made, so a character born under it was said to have lived four hours too long. The data's clock now takes the hour first (`DuskWorld::apply_start_hour`);
         - two tests hung on where seed 1's buildings stood. The save tests now take their door from the layout. The river test's far bank is now on the road, since a tile in thick woods can be a clearing the trees shut in;
       - **checked:**
         - by tests: seeds 1 to 8 have one of each shop within 30 tiles of the chapel, with the abbey and the river; every rack holds a weapon, most of them forged; the tavern holds 2 to 5 of the dead (seeds 1 to 6);
         - on screen: a picture in the smithy at noon on seed 2 shows the HUD's "in the smithy". The furniture didn't make the picture: a ghoul followed the scripted walk in and killed the character, and the death recap covers the room;
       - **measured:**
         - the patch runs at about 255k ticks a second (from about 270k: it holds more of the dead);
         - at the Tolling, 45 to 50 of the 52 to 57 dead in reach gather within 15 tiles of the bell (seeds 1 to 3), as before.
     - **The farms** (2026-11-11; 192 tests pass):
       - **a farmstead** (`prefabs/farm.toml`, tag `farm`): the town's old farmhouse and a barn about a dirt farmyard with a well and hay; behind them, a field of tall wheat in pairs of rows with a furrow between. The farmhouse and the barn have left the town for the farms. The fence round it all is the lot's, as a house's yard is, and its gate opens onto its track;
       - **tall wheat** (a fixture that's opaque but not solid): you walk through it, but nobody sees through it, you or the dead. A farm's dead (1 to 3, the prefab's own) may be standing in it;
       - **a well** (a fixture with `water = true`; `Defs::wells`, `is_well`): water to drink from, as a pond is (`World::water_beside`). A farm can be a home with no walk to the river;
       - **where** (`Layout::place_farms`, a region's `[farms]`): three, each 16 to 48 tiles off the road, north or south of it, mirrored or not and turned to face it. A farm goes anywhere that keeps 10 tiles clear of every other lot and of the river's whole course, with a track from its door to the road that crosses no lot. Each has 48 tries; one that finds no room isn't there. All three found room in seeds 1 to 8;
       - **their loot** (`loot/farm.toml`):
         - the barn's sacks hold the harvest: new turnips, apples, dried meat;
         - its tool rack always holds a tool to fight with: the new scythe (a slow, tiring blade that cuts deep), or a pitchfork, a shovel or an axe;
         - the farmhouse's cupboards, wardrobes and barrels fill as a house's do;
       - **the cure, rebalanced:** the farms' bedrooms put five cures for the fever in a patch on average, against the design's one to four. The wardrobes' and shelves' chance of one was cut: now 1 to 5 in seeds 1 to 12 (a mean of 3.25), and never none;
       - **the save format** is version 12: the land changed;
       - **the client:** the farms' furniture (wheat drawn thinner than its tile, since it's walked through); the patch's map image draws crops gold;
       - **checked:**
         - by tests: seeds 1 to 8 have their farms (three each), each with a track at least 16 tiles long that meets the road, clear of every lot and of the river, with one well and at least 60 tiles of wheat that's walked through and not seen through; a well is water to drink from; the data's farms load; a bad `[farms]` is refused;
         - on screen: a picture in a farmyard at noon on seed 1 shows the HUD's "in a farmstead", the well, the hay, and the first row of wheat with nothing seen past it;
       - **measured:**
         - a patch is made in about 55 ms (from 52) and runs at about 243k ticks a second (from 255k: the farms' dead);
         - at the Tolling, 47 to 56 of the 56 to 62 dead in reach gather within 15 tiles of the bell (seeds 1 to 3).
     - **A fire for your base** (2026-11-11; 202 tests pass):
       - **where:** a hearth (the tavern's), a stove (every house and farmhouse) or a forge (the smithy's) takes a fire. That's a fixture with a `fire` reach in `fixtures.toml` (`Defs::hearths`): 6 tiles for a hearth or a forge, 5 for an iron stove. Each keeps a few split logs by it, as a container (`loot/hamlet.toml`'s `logs`: 2 to 4, most of a night's fire);
       - **fuel** is a property of an item, not a kind (`fuel = hours` in `items/*.toml`; `Defs::fuel`, the longest-burning first). A split log (`firewood`, new: 1.5 kg, worth no XP) burns 2 game hours, a plank 1. So a plank is still a plank, and burning one is a real choice against nailing it across a window. Sheds' crates hold logs too;
       - **tending it** (`fire.rs`, `Intent::Tend`, H): with fuel in the pack, a piece goes on (2 s). Once it's laid, if you hold up a light, tending it lights it instead (3 s), the same flame that burns a body. A fire holds 8 game hours at most (`rules.toml`'s `[fire]`). `World::can_tend` says what tending would do, or why not: no fireplace beside you, full, nothing that burns, or laid and wanting a flame;
       - **putting it out** (`Intent::Smother`, J, 2 s): what's left of the fuel stays in it for the next time, so it only wants a flame again;
       - **burning:** a game minute at a time, then out at nothing (`Event::FireOut`). The world keeps only the fires with fuel in them (`World::hearths`, ordered, in the hash and the save);
       - **its light:** a lit fire is a light source in `light_at` beside the carried lights and the burning bodies, as far as its fixture says, and stopped by walls as they are. So a lit room needs no torch in your hand, which leaves that hand free to shove off a grab;
       - **light is bait:** once a game minute at night, each of the dead that's after nothing and sees a fire (`World::sees`: its range, its cone, the fire's own light, a line of sight) comes to the tile in front of it, as to a noise (`Event::SawFire`). A burning body is a fire too. Glass doesn't stop sight, but a boarded window does, so board the windows up before you light it. On hallowed ground, a fire draws nobody;
       - **a Lantern-Eater** puts out a fire it passes, as it does a light held up;
       - **the save format** is version 13 (the fires laid in hearths);
       - **the client:** H and J; a real light with shadows over each lit fire, flickering, with embers glowing on top (`DuskWorld::hearth_fires`); the loot panel's row for the fireplace beside you says how it stands ("Stove · alight, 3h 50m of wood"; `DuskWorld::fire_beside`); the news tells of laying, lighting, putting out, burning out, and the dead seeing the firelight (once in a while); `--do` learns `logs`, `fire` and `smother` for scripted runs;
       - **checked:**
         - by tests: laying the longest-burning fuel first, lighting from a light held up, the fire's reasons why not, what it holds, burning down a minute at a time and going out, putting it out with its fuel kept, lighting its room at night but not through a wall, the dead at night coming to a fire they see through a window (to the tile in front of it), none behind boards and none by day, a Lantern-Eater putting it out; the game's data (fireplaces, logs, fuel, `[fire]`) and nonsense refused; a stove lit in seed 1's patch saved, loaded and burning down the same;
         - in the game: a scripted run in seed 1's farmhouse took three logs from the stove, laid two, lit the torch, lit the fire, put it out and lit it again. A picture at night (21:46, in heavy rain) shows the stove's embers and the kitchen lit round it, its light falling through the doorway. That picture caught the first fire-status line overprinting the level line under the clock, so the status moved to the loot panel. The panel line itself isn't in a picture;
       - **measured:** no change to a tick (about 245k ticks a second with 49 chunks awake); at the Tolling, 47 to 56 of the 56 to 62 dead in reach gather within 15 tiles of the bell (seeds 1 to 3), as before.
     - **Pain and dread** (2026-11-11; 210 tests pass), the last of the first playable's slices:
       - **pain** (`dread.rs`, `World::pain`) is worked out from the wounds when asked, so nothing new is kept. A fresh scratch hurts 10 points, a laceration 20, a bite 30 (`rules.toml`'s `[pain]`). A dressed wound hurts half, and each hurts less the nearer it is to closing: its healing still to come over its whole healing time (all of it while it bleeds);
       - **its moodle:** sore (10) is a warning; in pain (30), everything you do costs a quarter more breath (`endurance_rate`, so swings, shoves and running alike); in agony (60), half as much again, and you can't sleep (`NoSleep::Pain`);
       - **its treatment is the bandage:** a bandage now goes on the worst wound that's bleeding, else the worst not yet dressed, a scratch that never bled among them. First Aid still learns only from a wound that was bleeding;
       - **dread** (`Body::dread`, out of 100) moves each game minute toward what the moment is worth (`dread_due`):
         - 30 for the dark (your tile lit below 25);
         - 20 for each of the dead within 8 tiles that you see or that's after something, up to three. Seen means a line of sight, lit or not: you know what's there;
         - 10 for each level of pain;
         - 10 for each level of hunger past peckish.

         It rises 6 points a game minute and falls 1. Asleep, or on hallowed ground, it ebbs to nothing at 3. So alone in the dark you're uneasy and no worse. A torch or a fire takes the dark's share away, which is how a light steadies you;
       - **its moodle:**
         - uneasy (25): your aim shakes, 90% of your chance to hit (`shaken`, in `swing_of`, so the swing the HUD and the bot read is the shaken one);
         - afraid (50): 80%;
         - frightened (75): 70%, and you can't step nearer the nearest of the dead you dread (`World::fear`, checked as a character's step starts; `Event::Balked`). You can still back away, go round, and fight what's beside you. It takes three of the dead, or two and your pain, in the dark;
       - **news:** `Event::Dread` when its level changes;
       - **the save format** is version 14 (a body's dread);
       - **the client:**
         - the two moodles, with what each level does;
         - dread darkening the screen's edges (`shaders/dread.gdshader`, eased from `DuskWorld::character_dread`), with a throb that quickens as it grows;
         - the news as dread rises, at most once in 30 seconds for each level, since it hovers at a level's edge as the dead come and go from sight; balking, and pain stopping sleep;
       - **checked:**
         - by tests: pain by a wound's kind, halved dressed, fading as it heals; bandages dressing a wound that's stopped bleeding (with no First Aid for it); breath costing a quarter and a half more; no sleep in agony until dressed; dread reaching uneasy alone in the dark and no further, falling under a torch, ebbing asleep and on hallowed ground, fed by hunger; aim at 90% and 80%; frightened among three of the dead at night, balking toward them and backing away; the game's numbers and nonsense refused;
         - in the game: a night walk from the abbey into the town grew uneasy off hallowed ground, then afraid near the dead. A picture of it at 22:22, in heavy rain, shows the "afraid" news, the uneasy moodle (dread had slipped back under the line) and the new title. Seeing the edges darkened in a night that's already dark isn't something a picture shows well, so the vignette was only seen to load and draw;
       - **measured:** about 240k ticks a second (from about 245k: each character's dread is a sweep of the actors once a game minute); at the Tolling, 47 to 56 of the 56 to 62 dead in reach still gather within 15 tiles of the bell (seeds 1 to 3).
     - **Wounds by where they land** (2026-11-11; 216 tests pass), the rest of the first playable's "wounds with clear effects":
       - **how hurt a part is** (`injury.rs`, `World::hurt`): the pain of the wounds on it (`World::wound_pain`, the one rule pain uses), with the hands counted with the arms and the feet with the legs (`Part::group`, `Group`). It's worked out from the wounds, so nothing new is kept and the save format doesn't change. There are two levels (`rules.toml`'s `[injury]`): hurt (10, a fresh laceration) and badly hurt (25, a fresh bite);
       - **what each part does,** to the living only (the dead feel nothing):
         - arms: a swing takes 15% longer, wind-up and recovery, and lands 10% less often; badly hurt, 30% and 20% (`hampered`, in `swing_of`, before dread's shaking);
         - legs: 15% slower (`pace`), and running costs 25% more breath; badly hurt, 40% slower and 50% more. That's 1.8 tiles a second at a run, under a ghoul's 2.0: a badly hurt leg can't outrun one. A test holds the data to it, and it caught the first numbers (0.7, so 2.1) saying otherwise;
         - head: 10% worse aim and 15% less far seen (`sight_range`, in `makes_out` and `look`); badly hurt, 20% and 30%;
         - torso: a wound there hurts half again as much, so pain, and what pain costs, come sooner;
       - **its treatment is the bandage,** as pain's is: a dressed wound hurts half, so it hampers half, and it all eases as the wound heals. A dressed laceration stops hampering as soon as it starts to close; a dressed bite eases to hurt;
       - **the client:** three moodles, hurt arm, limping and head wound, each with two levels and what it does;
       - **checked:**
         - by tests: an arm's slower, less sure swings at each level, and whole again once a laceration's dressed and healing; a leg's slower walk and dearer run; a head's worse aim, and a tile 18 tiles off seen whole and not hurt; a torso laceration hurting 30 against a leg's 20; the dead's swings untouched by their wounds; the game's numbers, a badly hurt leg against a ghoul, and nonsense refused;
         - in the game: a daytime walk into seed 1's tavern was met by its dead, and the news told each wound and where it landed. The one picture caught the death recap after it, listing six wounds (two bites, two lacerations, two scratches) under the new title. The new moodles weren't caught on screen: the column empties on death;
       - **measured:** about 241k ticks a second, as before; the Tolling as before.
   - **Features, after the first playable** (the user, 2026-11-11: "Bot will be done at the very end. Please continue adding features"). In slices, each its own commit: the graveyard and its skeletons; crafting I, the workbench; textures; the wight; sneaking; shields; bows and arrows; cooking.
     - **The graveyard and its skeletons** (2026-11-11; 222 tests pass):
       - **damage by weapon** (`Creature::resist`, `resist_index`, `World::resistance`): a kind of the dead takes so much of a blow by the weapon's skill (blade, blunt, axe, spear), and bare hands strike as blunt. Applied as a character's blow lands, after the rolled harm (`strike`). A blow turned, or told on all the more, is news (`Event::Resisted`). Most kinds take every blow whole (`resist` left out of the data);
       - **the skeleton** (`creatures/hushlands.toml`): the town's old dead, bone and nothing else. It takes 40% of a blade, 30% of a spear, 70% of an axe, and 150% of anything blunt, so a mace puts one down in two blows and a sword takes four to six. It has 24 health, a run between a shambler's and your walk, and the best darkvision of the dead (4 tiles). It has no bite and no fever, makes no groan, and shattered, it stays down;
       - **a kind that never bites doesn't bite holding you:** the held bite ("held, the mouth gets close") now needs a kind whose wounds include bites. A run in the graveyard caught skeletons biting while they held;
       - **a building's own dead** (`Prefab::kind`, `Lot::kind`, `Layout::dead_kind`): a prefab can say what its dead are, over the region's mix. The loader refuses a kind that's no creature;
       - **the graveyard** (`prefabs/graveyard.toml`, the town's fifth lot, so it lies beside the chapel on its side of the road): rows of headstones (`grave`) either side of a path from a gate in its railing to two old tombs (`tomb`, a container). It holds 3 to 6 skeletons, and they're the only skeletons in the patch. The tombs hold grave goods (`loot/graveyard.toml`): coin, candles, and an old sword, mace or spear, 40% of them magic;
       - **the save format** is version 15: the land changed (the graveyard, and the town laid out round it);
       - **the client:** headstones and tombs; the skeleton bone white and upright; news, at most every 15 seconds, when your blow glances off bone or shatters it; the title now names the newest feature;
       - **checked:**
         - by tests: bone takes 40% of a blade, 150% of a mace and of bare hands, with the news of each; most kinds take a blow whole; held, a biting kind bites and bone never does; the skeleton's numbers, against the mace and the sword; the graveyard's kind and dead; a kind that's no creature, or a resistance past 4, refused; seeds 1 to 8 each have a graveyard beside the chapel holding 3 to 6 skeletons, the only skeletons in the patch, and two tombs;
         - in the game: a picture by day from the graveyard's gate (seed 1) shows its headstones, a tomb, the railing, and the skeletons, bone white, one seeing you (!) and one curious (?), with the news of them seeing you. The news of a blow on bone wasn't seen in the game: the scripted fights met the town's shamblers first;
       - **measured:** a patch is made in about 60 ms; it runs at about 218k ticks a second (from about 241k: 104 dead from 96, and skeletons think often); at the Tolling, 54 to 56 of the 63 to 65 dead in reach gather within 15 tiles of the bell (seeds 1 to 3).
     - **Crafting I** (2026-11-11; 228 tests pass):
       - **recipes** (`craft.rs`, `recipes.toml`; `Recipe`, `Defs::recipes`): what each makes, what it uses up (from the pack first, then the hand, so the cudgel you hold goes into the club), the tools it wants to hand (any one, kept), its time and the skill it teaches. Carpentry makes its own recipes quicker, by the same rule as boarding up (`board_ticks`). What you make isn't a find, so it brings no XP home. `World::can_craft` says why not: no tool, or short of an item. The first three:
         - a torch, from a split log and a rag;
         - a sharpened stake (a spear that splinters: damage 9 to 14, wear 3), from a plank, with a knife or an axe;
         - a spiked club (blunt, 11 to 16, between the cudgel and the mace, and good on bone), from a cudgel and 6 nails, with a hammer;
       - **breaking furniture down** (`Intent::BreakDown`; `rules.toml`'s `[crafting]`, `Defs::breaking`; a fixture's `planks`, `Defs::planks`): a table or a bed gives 2 planks, a chair 1, a pew 2, with a hammer or an axe to hand, in 8 seconds less Carpentry. It's heard 14 tiles off as it starts and as it's done, it teaches Carpentry, and the tile is clear (the step index follows, as it does any tile). A container never breaks down (the loader refuses `planks` on one), and nothing on hallowed ground does. So planks, for boards or a fire, come from the house you're holding;
       - **the save format** is version 16 (the new intents and actions);
       - **the client:** a crafting panel (T, `craft.gd`): each recipe, what it takes, and "Make" or what's missing, and the furniture beside you with "Break". A chunk draws its furniture again when a piece is broken up (`refresh_fixtures`). News of what you made and the planks you got; `--do` learns `craft` and `breakdown`;
       - **checked:**
         - by tests: a torch made, its makings gone, and not a find; a stake wanting its knife, keeping it, and teaching Carpentry; the cudgel in hand going into a spiked club; a table broken down for 2 planks with a hammer, heard twice, the tile clear, and Carpentry learnt; nothing to break once it's gone, and none on hallowed ground; the game's recipes, weapons and furniture; a recipe naming no item, and a container with planks, refused;
         - in the game, with a hammer added to the kit in a scratch copy of the data: in seed 1's farmhouse a table broke down into 2 planks and a chair into 1. A picture after the table shows the floor clear, the planks in the pack, the panel's recipes each with what's missing, and the chair offered to break;
       - **measured:** no change to a tick (nothing runs unless asked).
     - **Textures** (2026-11-11; 228 tests pass, no change to the core): every surface is procedural, worked out in the shaders from where a point is (`shaders/surfaces.gdshaderinc`: hashed value noise, four octaves of it, courses of blocks, staggered boards). So nothing is downloaded or loaded, nothing repeats, and no chunk edge shows a seam:
       - **the ground** (`ground.gdshader`): each tile's ground kind now rides in its texel's alpha, as (code + 0.5) / 16, so the shader knows what it's drawing. Grass in patches and tufts, dirt with pebbles, the road in cobbles and worn mortar, floors in staggered planks four to a tile, the bridge in broad planks across it, and water whose light moves;
       - **walls, doors, fences, boards, furniture and trees** (`wall.gdshader`): each MultiMesh instance carries what it's made of in its custom data (`chunk_view.gd`'s surfaces): plaster walls over a plinth of stone blocks, damp low down; wood in boards with their grain (doors, fences, boards, and furniture unless it's listed otherwise); stone (the altar, hearths, forges, wells, graves, tombs), iron with a little shine (stoves, anvils), cloth (beds, sacks), straw (hay, wheat), leaves in clumps, and bark. Boards and trunks use the same shader with the cut-away off (`cuts`);
       - **checked:** two pictures at the seed-1 farm by day (this slice is all looks, so it spent two). The first, from inside the farmhouse, showed the shaders compiled and the floor's planks, but most of the scene outside the character's sight was in the fog's grey, and the grass and plaster read too faint. With their contrast raised, the second, from the farmyard, shows the walls' stone plinth, plank floors, the hay's straw, the fence's boards and mottled grass. The well's stone and the leaves are still faint.
     - **The barrows and their wights** (2026-11-11; 233 tests pass):
       - **three new traits of a kind** (`Creature`; a creature's data): `nocturnal`, only after dark (by day it lies still where it is: no thought, no hunt, `think`); `dread`, what it adds to the dread of the living near it over what any of the dead do (`dread_due`); `lit_harm`, what it takes of a blow standing in the light, its tile lit at half or more (`strike`). And `douses`, the Lantern-Eater's light-eating, can now be a kind's own in the data;
       - **the wight** (`creatures/hushlands.toml`): the night's own. By day it lies in its barrow, and the barrow's dim light is enough to weaken it, so the day is when to go in. After dark it walks faster than your walk and slower than your run, sees 6 tiles in any light, and puts out every light and fire within 4 tiles. Its nearness alone adds 25 dread (one of the dead and a wight in the dark is 75, frightened). In the light it takes twice the harm of a blow. It has twice a shambler's health, and put down, it stays down;
       - **the barrows** (`prefabs/barrow.toml`, a region's `[barrows]`): old burial mounds out in the wilds, two a patch, 30 to 70 tiles off the road down an old track. They're placed by the farms' rule (`Layout::place_outlying`, now shared, with a layer of its own), after the farms and clear of them. A stone chamber, its door long open, a tomb of grave goods at its back, and one wight. Wights spawn nowhere else;
       - **the save format** is version 17 (the land changed);
       - **the client:** the wight night-blue, the tallest of the dead, leaning back; its light-eating is told by the news the Lantern-Eater already has;
       - **fixed on the way:** the barrow's first door was shut (`D`), and the test that every building can be walked into from the start caught it. A path goes through no shut door, and every other prefab's front door starts open;
       - **checked:**
         - by tests: by day it lies still while a day-walking kind hunts you, and at night it hunts you; its nearness frightens (75 at night, against 50 without its own dread); a blow does twice the harm by day and the same at night; the wight's numbers, its speed between your walk and your run, and only in barrows; seeds 1 to 8 have barrows out in the wilds, one wight to each, and wights nowhere else;
         - on screen: a picture inside seed 1's barrow at noon shows the stone chamber with its plinth, the tomb at its back, the HUD's "in a barrow", and dread rising to afraid as one of the dead followed in. The wight itself didn't make the picture: the client draws only the dead you see, and it was out of the character's cone or behind the tomb;
       - **measured:** a patch is made in about 69 ms; it runs at about 217k ticks a second (from about 218k); at the Tolling, 50 to 56 of the 62 to 65 dead in reach gather within 15 tiles of the bell (seeds 1 to 3).
     - **Sneaking** (2026-11-11; 238 tests pass):
       - **a stance, not an action** (`stealth.rs`, `World::sneak`, `Actor::sneaking`; `rules.toml`'s `[sneak]`): no running, half your walking pace (Stealth quickens it), footsteps that carry 40% as far (at least a tile), and the dead see you only half as far off (`spots`, in `nearest_visible_character`). Right beside them, or within their darkvision, they see you as ever. A shambler sees a sneaking you from 4 tiles, not 8;
       - **a sneak attack** (`World::unawares`, `Event::SneakAttack`): a blow on one of the dead that's after nothing, from outside its sight cone. It always lands and does three times the harm, more with Stealth, whether you were sneaking or not;
       - **Stealth** (`Skill::Stealth`, the eighth skill): learnt from a sneak attack that lands, and from a tile sneaked with the dead within 8 tiles and none of them after you (one spot pays less each time, as a run does). Each level quickens sneaking by 3% and adds 10% to a sneak attack's harm;
       - **the save format** is version 18 (a body's stance, and a skill more);
       - **the client:** Ctrl to sneak and to stand; a "Sneaking" moodle saying what it does; you crouch; news of a sneak attack;
       - **five tests changed on the way:** a magic, a skills and two resistance tests struck an idle shambler from beside or behind, which is now a sneak attack. They're about other things, so their shamblers now see all round;
       - **checked:**
         - by tests: sneaking is slower, quieter and never a run; at 6 tiles a shambler sees you standing and not sneaking; a blow from behind on one unaware does 300 and teaches Stealth, while the same blow on one that sees all round does 100 and teaches nothing; sneaking past the dead unseen teaches Stealth and walking doesn't; the data's numbers;
         - on screen: a picture by day outside the abbey shows the Sneaking moodle, the crouch, Ctrl in the controls and the new title, with the cobbled road, patchy grass and leafy crowns of the last slice in daylight;
       - **measured:** about 215k ticks a second (from about 217k).
     - **Shields** (2026-11-11; 244 tests pass):
       - **a new kind of item** (`ItemKind::Shield(Guard)`, `shield.rs`; an item's `shield = { block, endurance, wear, bash }`), carried on the off arm (`Actor::shield`), where a light would be held up. Taking one up puts out the light you held, into the pack; holding up a light puts the shield in the pack (`Intent::Unshield` puts it away). It weighs in your load and falls with your gear;
       - **bracing** (`World::brace`, `Actor::bracing`; Alt, held) is a stance: 60% of your pace (`rules.toml`'s `[shield]`) and no running. A blow from the half you face may be blocked by the shield's chance, more with Shield (`blocks`, its own roll, `Purpose::Block`). A blocked blow does nothing, no wound and no hold, but costs the shield's breath and wears it, and at nothing it breaks (`Event::Blocked`, `WeaponBroke`). Held, nothing's blocked;
       - **a bash:** a shove with a shield on your arm adds the shield's `bash` to its chance to knock down;
       - **Shield** (`Skill::Shield`, the ninth skill): learnt from blocks and bashes, and a level adds 3% to a shield's chance to block;
       - **the shields:** a wooden buckler (blocks half, wears 4 a block, bashes +15%), made from 2 planks and 8 nails with a hammer, or found in sheds; and a heater shield (70%, wears 1.5, +25%), iron-rimmed oak from the tombs' grave goods. It isn't on the smithy's racks: they're promised a weapon, and the test that holds them to it would have caught a shield;
       - **the save format** is version 19 (the arm's shield, the stance, and a skill more);
       - **the client:** "Take up" in the pack for a shield, its line on your arm with "Put away", Alt held to brace (and a word if there's no shield), a Bracing moodle, news of a block (at most every 3 seconds) and of taking one up; `--do` learns `shield` and `brace`;
       - **checked:**
         - by tests: a shield and a light share the arm, each putting the other away; braced, blows from ahead are all blocked, for breath (the least of it while blocking, since it comes back between blows) and wear, while unbraced, or facing away, they land; a shield blocking its last block breaks; a shove that never knocks down does with a shield's bash; bracing is slower and never a run; the data's shields, the buckler's recipe and the numbers;
         - on screen, with a buckler added to the kit in a scratch copy of the data: a picture by day on the road outside the abbey shows the Bracing moodle and the buckler on the arm in the pack panel. It caught the news saying "in hand" for a shield, now "on your arm";
       - **measured:** about 217k ticks a second, as before.
     - **Bows and arrows** (2026-11-11; 248 tests pass), the first ranged combat:
       - **a new kind of item** (`ItemKind::Bow(Ranged)`, `archery.rs`; an item's `bow = { draw, recovery, endurance, hit, damage, noise, range, ammo, breaks }`, its `ammo` named and found once every item is read): taken in hand as a weapon is. Shooting wants the other arm free too: no light held up, no shield (`NoShot::HandsFull`);
       - **a shot** (`Intent::Shoot { at }`, right-click on a tile): the draw is its wind-up and tell, then an arrow from the pack flies the straight line toward the tile, within the bow's range (`walk_line`, now shared). The first one on the line takes it (`World::occupied`). Whatever stops sight stops the arrow: a wall, a shut door, a boarded window. It lands by the bow's chance, less the farther it flies (half at the bow's range), and always on one unawares;
       - **a hit** does the bow's harm, taken as a spear's thrust is (`resistance(.., Spear)`, so bone turns an arrow to 30%), three times on one unawares (a sneak shot). It interrupts as a blow does and can kill. The arrow breaks by the bow's chance (its own roll, `Purpose::Arrow`); otherwise it lies where it fell, to be picked up, as does a miss (`World::put`). A loose is quiet: the bow's noise, at the shooter;
       - **Archery** (`Skill::Archery`, the tenth skill): learnt from shots that land, a blow's XP, and it makes a bow's shots surer and harder as a weapon skill does a blow's (`SkillRules::swing`);
       - **the bow and arrows:** a hunting bow (range 12, 10 to 16, a 0.9 s draw, noise 2, a third of arrows breaking) in shed crates; arrows in sheds and on the store's shelves, and four rough arrows from a plank with a knife or an axe (`recipes.toml`);
       - **the save format** is version 20 (the shot action, and a skill more);
       - **the client:** right-click shoots, and says why not (no bow, no arrows, a hand full, out of range); an arrow's flight drawn as a streak from the bow to where it landed, fading in a moment (`Event::Shot`); "Wield" for a bow in the pack; `--do` learns `bow` (take one up) and `shoot` (at the nearest of the dead you see);
       - **checked:**
         - by tests: an arrow hits the nearer of two on its line, for its harm, teaches Archery, and lies where it fell; a wall stops it, and it falls before the wall; bone turns it to 30%; a bow wants arrows, the other hand free (a torch held up stops it), its range, and a bow in hand; the game's bow, arrows and recipe;
         - in the game, with a bow and arrows added to the kit in a scratch copy of the data: at the graveyard's gate by day, a shot at the nearest skeleton hit. The picture after it shows the news "You hit it", the bow in hand, one arrow fewer in the pack and the loosed one on the ground to take back, with the skeletons closing in. The streak had faded by the picture. On the way, a scripted bow taken up at the walk's start was found to cancel the walk (one intent at a time), so it's taken up on arrival;
       - **measured:** about 208k to 217k ticks a second (nothing new runs in a tick).
     - **Cooking, and food that goes off** (2026-11-11; 253 tests pass):
       - **cooking is crafting at a fire** (`cooking.rs`; a recipe's `fire = true`, `Recipe::fire`): it's made only beside a lit hearth, stove or forge (`World::by_a_fire`, else `NoCraft::NoFire`), its makings used up and a tool to hand if it names one, as any recipe;
       - **Cooking** (`Skill::Cooking`, the eleventh skill): learnt from what's cooked (each recipe's XP), and it makes cooking quicker as Carpentry does carpentry (`SkillRules::cook_ticks`, rules.toml's `[skills.level] cook_time`);
       - **a meal's comfort** (food's `comfort`, points): taken off dread as it's eaten. A stew takes 20, roast mutton 10, a roast turnip 5; raw mutton adds 5 (it sits ill);
       - **food goes off** (food's `keeps`, game days, as `spoils`: condition lost a game hour): each game hour, every food that spoils ages in packs, in containers and on floors, and at none it's gone (`World::spoil`). Gone from your pack, it's news (`Event::Spoiled`). What a container holds before anyone opens it has aged with the world (`Defs::aged`, in `World::fill`): a larder left alone a week holds no meat. Raw mutton keeps 2 days, roast mutton 4, a stew 3, bread 5, apples 10, cheese 14, turnips 20; dried meat keeps;
       - **the food:** raw mutton (8 points; in kitchen cupboards, the tavern cellar, and a joint hung in the farms' barns); roast mutton (35) from raw mutton at a fire; a roast turnip (18) from a turnip; two bowls of stew (40 each) from raw mutton, two turnips and a bottle of water, in a cooking pot (kitchen cupboards) at a fire;
       - **the save format** is version 21 (a skill more);
       - **the client:** "at a fire" on a cooked recipe, "needs a lit fire beside you" when there's none; food in the pack shows fresh, stale or going off once it's begun to age; "Your … has gone off" news. The crafting panel moved to the top right, under the news (eight recipes ran it over the loot panel), with why a recipe can't be made beside its name; `--do` learns `cook` (the last cooked recipe that can be made);
       - **checked:**
         - by tests: a meal can't be cooked with no fire, nor with one laid and not lit, and once it's lit it's made and teaches Cooking, which makes it quicker; raw meat adds dread and a hot meal takes it away; food goes off by the hour in a pack (news) and on a floor, and what keeps, keeps; a larder nobody opened has aged with the world, and its meat is gone in time; the game's foods, recipes and rule;
         - in the game, with stew makings added to the kit in a scratch copy of the data: in seed 1's farmhouse at night, logs laid in the stove and lit from the torch, a stew cooked (two bowls) and then a roast turnip. The picture shows them in the pack (the stew "fresh", an hour on), and the crafting panel with each cooked recipe "at a fire". The first picture showed the panel, three recipes longer, running over the loot panel, so it moved;
       - **measured:** about 217k to 230k ticks a second (spoiling runs once a game hour).
     - **Wildlife and hunting** (2026-11-11; 257 tests pass):
       - **a beast** (`hunting.rs`; a creature's `flees`, tiles): it's after nobody (`Mind::Flee`, which reads as unaware everywhere: no grabs, no "the dead after you" to stop sleep). It grazes (its `wander`); seeing one of the living within `flees` tiles, hearing a noise (your footsteps carry 3 tiles walking, 8 running), or being hit by a blow or an arrow, it's startled (`Event::Startled`) and runs, each step the open one that takes it farthest from the fright and never nearer, until it has had no fright for its `forget_after`. Cornered, it stands. It never strikes;
       - **beasts aren't the dead:** they add no dread, the pressure doesn't count them (`dead_about`), the bell doesn't draw them, and none is made a rare (they're placed after the rares). Code that steers the dead (fire, hearing, getting up) matches `Mind::Hunt` only, so it leaves a beast alone;
       - **the deer** (`creatures/hushlands.toml`): faster than your run (4.0), sees 12 tiles nearly all round (300°), bolts from anyone within 9, and 12 health: most arrows that land drop it, and one unaware (from its blind side) always. A region's `[beasts]` (kinds that flee and their shares, `per_chunk`) puts them out in the open, clear of where you start: 14 to 19 deer a Hushlands patch;
       - **butchering** (a kind's `butcher = { item, count, time }`; `Intent::Butcher`, G): a carcass beside you is cut up with a blade or an axe in hand or pack (`NoButcher::NoBlade`), 20 seconds kneeling over it, for 3 raw venison; nothing's left of it (`Remains::Butchered`). A beast's body never gets up and doesn't burn. Raw venison (10 points, keeps 2 days) roasts at a fire (`recipes.toml`) into roast venison (40, comfort 12);
       - **the save format** is version 22 (the new mind, intent, action and remains);
       - **the client:** a deer on four legs (brown, a long body, its head held high, a white scut) that rolls onto its side when it falls (`actor_view.gd`'s `_build_beast`); "A deer takes fright and bolts!" when you see one startled; G butchers a carcass beside you, else burns a body; "You cut it up: 3 raw venison."; `--do` learns `butcher` and `hunt` (shoot the nearest live beast within a bow's reach). `dusk-cli gen` counts the beasts apart from the dead, and names the one nearest the start;
       - **checked:**
         - by tests: a deer bolts from one it sees near and runs off, after nobody, dreaded by nobody and not counted as the dead; seen from beyond its flees, it grazes on; a blind deer bolts from your running; a carcass is butchered with a blade for its meat, nothing's left after, it won't burn, and without a blade it can't be; the game's deer, venison, recipe and the region's beasts;
         - in the game, with a bow, arrows and a knife added to the kit in a scratch copy of the data: in seed 1, sneaking from the start to 10 tiles north of the nearest deer, one arrow dropped it ("A deer takes fright and bolts! You hit it."). The picture shows it lying where it fell, darkened, with the arrow by it. On the way: walking up (not sneaking), the deer bolted as the path passed it; the scripted `shoot` took the nearest of the dead (a shambler), so `hunt` was added; and at 25 health a seen arrow never dropped a deer, so it's 12;
       - **measured:** about 190k to 223k ticks a second (from about 217k to 230k: 14 to 19 more actors grazing and running).
     - **Wolves** (2026-11-11; 259 tests pass):
       - **a wild hunter** (a creature's `wild`, `World::is_wild`): it hunts you by the dead's own mind (`Mind::Hunt`), and it's as much a dread, but it's of the living world. The pressure doesn't count it, neither the bell nor a fire draws it (`think`'s `belled`, `fire.rs`), and its body neither gets up nor burns: it's skinned (`butcher`). A beast is wild too;
       - **the wolf** (`creatures/hushlands.toml`): a little faster than your run (3.3 to 3.0), so you don't outrun one; darkvision 6, so the night is theirs; it follows blood (8 tiles a wound); a lunge with a shorter tell than the dead's (0.7 s), whose wounds go deep (half lacerations, a fifth bites); 5 points of dread of its own; 20 health. Catching sight of you, it howls (the groan, 18 tiles) and the pack comes. It can't beat a door down. Not nocturnal: a day-sleeping pack that never woke when struck would be free pelts;
       - **packs and herds:** a region's wildlife kinds (`WildKind`) have a `pack`, the fewest and the most together: the first where it's rolled, the rest on open tiles round it. The Hushlands: deer in herds of 1 to 3 (80%), wolves in packs of 2 to 4 (20%), 0.2 a chunk: 17 to 18 deer and 4 to 12 wolves a patch;
       - **a wolf pelt** (`items/supplies.toml`): one from a wolf, cut with a blade or an axe (15 s); worth 6 brought home;
       - **the client:** a wolf on four legs, grey, smaller than a deer, its head low and forward (`_build_beast`'s size and lift); "A wolf sees you, and howls for the pack!", or out of sight "A howl: something out there has seen you." (`DuskWorld.wild_kinds()`: 0 the dead, 1 a beast, 2 a wild hunter). `dusk-cli gen` counts the wild apart from the dead and names the nearest of each wild kind;
       - **checked:**
         - by tests: a wolf that sees you howls, and one out of its sight within the howl comes; wolves aren't counted as the dead but are a dread; put down, one stays down, won't burn, and is skinned; the game's wolf, its pelt, its packs, and that it's no kind of the dead;
         - in the game: seed 1 at dusk in heavy rain, walking to the road by the nearest pack. The news ran "A howl: something out there has seen you", then the pack was on you: scratches, a laceration, a hold, bites. The picture shows three wolves crossing the field toward you. They looked like small dark deer with their heads up, so a wolf's head is now low and forward (not pictured again);
       - **measured:** about 174k to 208k ticks a second (from about 190k to 223k: packs hunting the character standing at the start).
     - **The Cleric** (2026-11-12; 285 tests pass), the second class, and with it choosing one:
       - **choosing a class** (`World::choose_class`, `can_choose_class`): a character comes into the world as the first class in the data, and takes up another before it's under way: in its first game minute, with nothing learnt or done. The new class's kit replaces the one it came with. Every class has a `line` for whoever chooses;
       - **a class's starting kit** (`classes.toml`'s `wield` and `kit`; `Class::kit`, `class::Kit`; `World::equip`, used by `spawn` too): left out, it's everyone's (`rules.toml`'s `[character]`), so the Fighter starts as before. The Cleric starts with a cudgel, two bandages, a torch and two flasks of holy water;
       - **faith** (`dusk-core/src/faith.rs`; `Class::faith`, `Actor::faith`, `World::max_faith`): the Cleric's resource, 100 points, more with Devout. Turning the dead, consecrating without holy water, Mending and Warding spend it, and it comes back only by **prayer at a shrine** (`Intent::Pray`, P; `fixtures.toml`'s `shrine`, `Defs::shrines`): the abbey's altar, or the old chapel's in the heart of town, where the bell is. Anyone may pray, 10 seconds kneeling: it takes 20 points off dread, and the faithful get their faith back in full;
       - **turning the dead** (`Intent::Turn`, U; `Class::turns`), the Cleric's signature: 30 faith, a second holding up the holy symbol (a tell; a blow during it breaks it off), and a word heard 6 tiles off. Each of the dead within 8 tiles that the cleric can see is turned (`Mind::Turned`) for 20 seconds, its kind's share of that (`Creature::turned`: a wight's and a revenant's is half): it lets go of whoever it holds, its swing comes to nothing, and it flees from the cleric as a beast flees (`flight`), striking nobody and hearing nothing. A blow (or an arrow) ends it, and it rounds on whoever struck (`unturn`); it can't be struck unawares. The wild (wolves, deer) aren't the dead, and aren't turned. Held, a cleric can still turn: a third way out of a grab, after the struggle and the shove;
       - **consecration** (`Intent::Consecrate`, O; `ItemKind::Holy`, `Remains::Hallowed`): burning's quiet cousin. Holy water poured on a body beside you (3 seconds), or with none, a cleric's 15 faith, and it never gets up: no fire, no light, nothing spreads, and the pressure falls as for a body burned. A character's body consecrated never rises, and its gear falls where it lies. Holy water is in the chapel's two fonts (a new fixture by its door, `loot/hamlet.toml`'s `font`: one to three flasks each), and the abbey sells two a day;
       - **Devotion** (`Skill::Devotion`, the twelfth skill): learnt from each of the dead turned (a foe each: turning one shambler again and again pays less) and from a bleeding wound mended. A level makes the dead stay turned 10% longer and faith cost 4% less;
       - **ten talents** (`talents.toml`): Tough and Heavy Hand (the Fighter's too); Devout (25 faith); Mending (M: 20 faith stops the worst bleeding wound and gives 15 health back); Purify (a prayer cures ghoul fever too: the cure as a quest to a shrine); Warding (O at a shut door: 40 faith, and for 8 game hours the dead won't beat on it, `World::wards`, saved); Steady (holding a light up, dread rises half as fast: the steadying light); Rebuke (the turned flee twice as long, from 4 tiles farther); Sunder (the dead turned with 15 health or less fall instead); Last Rites (the dead you put down stay down, as with a Hallowed weapon). The loader refuses a talent of faith on a class without faith, and one of turning on a class that doesn't turn;
       - **the save format** is version 23 (faith, the turned mind, hallowed remains, the new intents and actions, the wards, and a skill more). The data's items changed too, so older saves are refused as made with other data;
       - **the client:** a new survivor (a new world, or R after a death) is asked "Who comes to the abbey?", paused, with each class and its line; a number takes it up. A trial run asks nothing: `--class=NAME` picks one, else it's the first. A faith bar under endurance for a class with faith; U, P, O and M, each saying why not; a turned one has a pale "+" over its head; a consecrated body lies pale; holy water's "Consecrate a body" in the pack; news of each. `--do` learns `turn`, `pray`, `consecrate` (or `ward`) and `mend`;
       - **checked:**
         - by tests: a character takes up the Cleric in its first minute with its kit and faith, and not after; the dead in reach and in sight turn and flee, not those out of reach or behind rock; held, the turned let go; a blow ends it and isn't a sneak attack; the turned hunt again when their time's up; a wight is turned for half as long and a wolf not at all; turning wants the class and the faith; anyone may pray for comfort, and a cleric gets its faith back, at the end of the prayer; Purify cures the fever; holy water consecrates a body, which never gets up, and lowers the pressure; a cleric does it with faith once the water's gone; a character's body consecrated lets its gear fall; Mending, Warding (the dead don't beat on a warded door, and the ward fades), Steady, Sunder, Last Rites and Rebuke; Devotion grows by turning and makes faith go further; the game's Cleric, its kit, the shrines, the fonts and the abbey's holy water; nonsense in `[faith]` and in a class refused; on four seeds, the chapel and the abbey each have one shrine and the chapel's two fonts hold holy water; faith, a turned shambler and a ward survive a save;
         - in the game (built on a PC with no Godot, so first run on the next, on pulling it): `--class=cleric` on seed 1 at noon, walking to the graveyard gate. "You hold up the holy symbol! 6 of the dead turn and flee"; then none in sight (they'd fled), then one more, then "Your faith is spent". Walking to the abbey's altar and praying: "You pray. Your faith comes back, and your dread eases." The picture shows six skeletons with a pale "+" over each fleeing into the graveyard, the faith bar at 70 of 100 under endurance, and the Cleric's kit in the pack with "Consecrate a body" on the holy water. Fixed on that run: the news of Devotion rising came before the turning (`turn_the_dead` now tells the turning first, then puts down, turns and teaches), "1 of the dead turn", a Cleric without Mending being told "Mending is a Cleric's talent", and the title still saying "new: wolves". A scripted act replaces a walk under way (one intent at a time), so a script walks first and acts on arrival;
       - **measured:** about 170k to 174k ticks a second, and the slice before it about 168k to 176k on the same PC, runs taken by turns: no cost to see. A tick does a little more for a creature that's turned (it flees instead of thinking) and nothing more for the rest; a ward is a map lookup when the dead would beat on a door;
       - **not yet:** the Cleric's other verbs in `CONCEPT.md` (wards of salt and words on a whole house, cures for others); a turned one's own look (it's marked, not posed); holy water thrown as a weapon; the sisters teaching prayers or blessing weapons for coin.
     - **Herbs and foraging** (2026-11-12; 296 tests pass):
       - **wild plants** (`dusk-core/src/forage.rs`; `Defs::plants` by fixture number, `Plant { item, count, regrow }`, from `fixtures.toml`'s `forage`): yarrow, chamomile, brambles and feverfew, walked through and seen past. The land grows them (`dusk-gen`'s `land`, `Layer::Plant`; a region's `[plants]`, each kind `on` open grass, the woods' floor or by the water, with a chance a tile): yarrow in the open (0.3%), brambles between the trees (0.6%), chamomile on a riverbank or a pond's shore (6%; a shore is low land with a pond within two steps, `by_water`). A prefab's `grows` grows its own in its yard: feverfew round the barrows (6%) and the graveyard (3%), where the dead lie. A patch has about 285 to 325: 105 to 115 yarrow, 75 to 120 chamomile, 70 to 90 brambles, 14 to 18 feverfew (four seeds);
       - **picking** (`Intent::Pick`, Y; `ActionKind::Pick`): 3 quiet seconds over the plant you face, else the one you stand on, else the first round you (`plant_beside`). It gives its count of the herb (yarrow 1 to 2, chamomile 1 to 3, a bramble 1 to 3 handfuls of blackberries, feverfew 1 to 2), and the plant's picked bare (`Defs::bare`, named in `rules.toml`'s `[foraging]`: a tuft of stalks) until it grows back: yarrow and chamomile in 3 game days, brambles in 4, feverfew in 5. `World::regrowing` (saved) is looked at each game hour (`regrow`, `Event::Regrew`); one built over meanwhile stays gone;
       - **Foraging** (`Skill::Foraging`, the thirteenth skill): 1 XP a plant, less picking again in one spot; a level makes one more of the herb 10% likelier (`SkillRules::forage_more`);
       - **remedies** (`items/herbs.toml`, `recipes.toml`): a **yarrow poultice** (2 yarrow and a rag; First Aid) is a bandage with a salve (`ItemKind::Bandage`'s `salve`): the wound it dresses keeps only half the healing it had left, so it closes sooner, and as a wound's pain follows the healing it has left, it hurts half as much as under a bandage. **Chamomile tea** (2 chamomile and a bottle of water, at a fire; Cooking) is a drink with comfort (`ItemKind::Drink`'s `comfort`): 20 points of dread eased. The **feverfew tincture** (3 feverfew and water, at a fire, 30 s; Cooking) makes the cure the patch already had, so the fever's cure can be fetched from the barrows, by day. Blackberries are eaten raw (5 points, keeping 3 days);
       - **the save format** is version 24 (the plants regrowing, a skill more, the new action);
       - **the client:** plants drawn as low clumps of leaves, the herbs with a disc of flowers on top (`chunk_view.gd`'s `PLANTS`), and a tuft of stalks once picked; Y picks, saying why not; beside a kind for the first time, the news says what it's good for ("Chamomile grows here: steeped at a fire, its tea calms. Y to pick it."); "You pick it: chamomile flowers ×2."; the tile redrawn as it's picked and as it grows back (`PICKED` 97, `REGREW` 98; action 43). `DuskWorld.pick_check()` gives the plant's fixture. `--do` learns `pick`. `dusk-cli gen` counts the wild plants by kind and names the nearest of each;
       - **checked:**
         - by tests: a plant picked gives its herb and is left bare, with nothing more to pick; it grows back in its day and not before, and not through what's been built over it; Foraging grows by picking, less in one spot, and finds one more herb more often; a poultice leaves half the healing and less pain than a bandage; a tea calms; the game's plants, herbs, remedies and recipes, feverfew only round the barrows and the graveyard; nonsense in a plant, and plants without `[foraging]`, refused; on four seeds, every plant on grass and off the paths, chamomile by the water, feverfew round a barrow or the graveyard; a plant picked bare stays so, and grows back alike, after a load;
         - in the game: seed 1 at 10:00, walking from the abbey to the pond east of its wall: "Chamomile grows here: steeped at a fire, its tea calms. Y to pick it.", and two picks, "You pick it: chamomile flowers ×2." each. The picture shows the stalks left of one, the pack's "chamomile flowers ×4", and chamomile round about. They read as pale pebbles (small, the flowers hiding the leaves), so the plants are now larger with a narrower disc of flowers (not pictured again);
         - found on the way: a first test put two plants either side of x = 0, which are different 8-tile spots for skill repeats, so the second picking paid in full (the test now picks in one spot); and the first shore was any land within 0.02 of the pond line, which grew chamomile in dry hollows 6 to 8 tiles from any water (now a pond must be within two steps);
       - **measured:** about 173k to 176k ticks a second (from 170k to 174k: nothing to see; `regrow` is an hourly look, and only once something's picked); a patch is still made in about 60 ms.
     - **The look in one place, and a GUI that scales** (2026-11-12; the first visual slice of `VISUALS.md` §16, with parts of the next ones; 296 tests, nothing in the core changed):
       - **tokens** (`client/look.gd`):
         - every colour the client draws with, named as `VISUALS.md` §4 names it (`Look.CALM` for `signal.calm`), merged where the guide said: six reds into `DANGER` and `DANGER_TEXT`, the ambers into `ALERT`, the golds into `GOLD`;
         - the type sizes of §8: 18, 20, 24, 32 and 48 px at the base screen;
         - the GUI's parts: `panel_style`, `text`, `bar`.

         Shaders take their colours as uniforms set from it: the sight tints, the noise ring, dread's edge. The furniture and plant tables are left for the slice that moves looks into data;
       - **a GUI that scales** (`project.godot`): the base screen is 1920 × 1080 with `canvas_items` and `expand`, so the GUI scales with the window and 3D still renders at the screen's own size.
         - The player's own game opens maximised. A trial run keeps 1600 × 900, so its pictures stay alike.
         - `-` and `=` step the GUI's size from 75% to 200% (`content_scale_factor`). Only the player's game keeps it, in `user://settings.cfg`;
       - **the layout** (§10):
         - the status and body down the top left, in one column;
         - the news down the top right, 620 px wide, its text left-aligned, with room kept for six lines so the moodles under it stay put;
         - the banners in sentence case ("Three of them have hold of you");
         - the act under way just under you;
         - the keys behind F1, in four columns over the middle while open. For a world's first game hour, "F1: every key" shows at the bottom centre instead;
         - making (T) on the left middle where skills (K) open, one at a time;
       - **checked:**
         - headless through the new paths: F1, skills and making in turn, a picking;
         - two pictures by the pond east of the abbey. The first showed the scaled GUI with the act's bar under you, and the keys in three columns at the bottom: twelve rows, over both bottom panels and nearly the bar. So the keys went to four columns over the middle, on F1 only, with the one-line reminder in their place. The second shows them so, and the news column;
       - **not yet:** the fonts, moodle tiles, prompts under you, Esc closing panels, a settings screen.
     - **Armour** (2026-11-12; 310 tests pass; `CONCEPT.md`: "Armour protects per body part, as Zomboid's clothing does", and the Fighter's heavy armour, which is loud):
       - **worn, a piece to a place** (`dusk-core/src/armour.rs`; `ItemKind::Armour(Armour)`, `Actor::worn`): a piece covers some of the head, the torso, the arms (with the hands) and the legs (with the feet), as `body::Group` bits. Putting one on (`Intent::Wear`) takes its own seconds, and whatever covered the same places comes off into the pack; taking one off (`Intent::TakeOff`) takes half as long (`[armour] off`);
       - **a blow on a covered place may glance off** (`Event::Glanced`), by the piece's chance against the wound it would have left (`turn`, by `WoundKind::ALL`): no wound, so no bleeding, no fever and no pain from it, and only half its harm (`[armour] bruise`). The piece wears with each (`wear`) and falls apart at nothing (`Event::WeaponBroke`, as a shield's does). A blow on one that's down, or one struck unawares, finds the gaps;
       - **what it costs, worn:** half its weight toward what you carry (`[armour] worn_weight`); heavy pieces make every cost in breath dearer (`endurance`, folded into `endurance_rate`), every footstep carry farther (`noise`: a walk in mail is heard at 5 tiles, not 3) and a closed helm narrows what you see (`sight`, in `sight_range`);
       - **the dead in armour:** a kind may wear a piece (`Creature::wears`, the data's `wears`): the wight comes in barrow mail and leaves it where it falls, a find. A weapon's blow (or an arrow) on a covered place glances as a laceration would, as far as the weapon's kind lets it (`[armour] pierce`: a blade 1.0, an axe 0.8, blunt 0.6, a point 0.4), so "points for armour" is in the data. A revenant rises in the armour its body died in, and a class may come in some (`Class::wear`: the Fighter's padded jerkin and leather cap);
       - **the abbey mends it:** `Mend` mends the weapon in hand and the armour worn, each by its share of its worth;
       - **the items** (`items/armour.toml`): a leather cap, an iron helm (sight 0.85), a padded jerkin (torso and arms; the abbey sews one a day, 12 coin), a leather jerkin and leather bracers (sheds), riding boots (wardrobes, with the cap and the jerkin), a mail shirt (the smithy's anvil, now a container with the smith's last piece: mail, a helm or bracers; noise 2, 15% more breath) and barrow mail (the wight's, neck to knee). Mail turns about 47% of a shambler's blows, barrow mail about 66%, the padded jerkin about 22%;
       - **the save format** is version 25 (`Actor::worn`);
       - **the client:** the pack lists what you wear ("Worn: mail shirt (100%) · torso, arms", with Take off), armour in the pack says where it goes and has Wear, and with nothing in hand the first piece worn has Mend at a counter. News: "The blow glances off your mail shirt." (every 3 s at most), "Your blow glances off the wight's barrow mail. A point would go through." (every 15 s), "You put on…", "…falls apart!" (`GLANCED` 99, `WORE` 100, `TOOK_OFF` 101; actions 44 and 45; item kind 11). `DuskWorld.item_covers()`, `character_worn()`, `actor_worn()` (for drawing it, later), `wear()`, `take_off()`. `--do` learns `wear`;
       - **checked:**
         - by tests: a piece goes on in its time and swaps out what covered the same places; a covered blow glances (no wound, half the harm) and wears the piece; only what a piece covers is turned; a piece worn through falls apart and the blows wound again; worn, it weighs half, costs breath, clinks and a helm narrows sight; a blade's blows on the dead in armour glance and a spear's go through; a blow on one that's down finds the gaps; a kind comes in its armour and leaves it where it falls; a revenant rises in yours; the abbey mends it; a class comes in its armour and another's kit replaces it; the game's armour, the wight's mail, the Fighter's and the anvil load; armour is a trade in the game's numbers (padded < mail < barrow, never all; heavy pieces are heard and tire you, but you can still sneak in them; barrow mail worn leaves more than half your carry); nonsense in a piece, or a creature wearing what isn't armour, refused; on six seeds the smithy's anvil holds a piece of armour;
         - in the game: a scratch copy of the data with mail and a helm in the kit. "You put on the mail shirt.", "You put on the iron helm."; the picture shows both worn with Take off, the jerkin and cap they replaced in the pack with Wear, and the load at 10.6 kg with the two worn at half. A headless walk into the tavern's dead in mail: "The blow glances off your mail shirt." among lacerations and a bite that got through;
       - **measured:** about 167k to 191k ticks a second (from 173k to 176k: nothing to see; a step looks at the armour only if something's worn);
       - **not yet:** armour drawn on the character and on the wight (`actor_worn` is there for it); magic on armour; the Fighter's talents for it; armour that's wet and cold in the rain; making armour (a hide jerkin from the deer).
     - **Climbing** (2026-11-12; 321 tests pass; `CONCEPT.md`, athletics: "climb fences, windows and low walls (a timed action that costs endurance); vault"):
       - **a climb is a step, only slower** (`dusk-core/src/climb.rs`; `Step::climb`): walking straight into an edge you can't walk through but can climb (`Edge::climbable`: an open window with no boards across it, or a fence) climbs it. It takes its own ticks whatever your pace (`[climb] window` 1.6 s, `fence` 1.0 s; running at a fence you vault it in 0.6 of that), so `progress` counts ticks and nothing carries over. Nobody climbs a corner. Its seconds cost breath as running does (the stride is a hard one), so armour, hurt legs, the Fighter and Athletics all count with no number of its own. It lands with a scrape (`[climb] noise` 4 tiles, quieter sneaking, louder in mail), teaches Athletics as a run does (under risk), and is slower weighed down (`heavy`, twice) or out of breath (half again); Athletics quickens it as it eases a run;
       - **windows open:** the door key (E, `Intent::Door`) opens and shuts the window beside you as it does a door (the same time and noise), never on someone climbing through, and a boarded one stays as it is. Every window starts shut, so a house you've opened a window in has a way in;
       - **the dead climb too,** by their kind (`Creature::climb`, per mille of your time: the shambler 2.5 times as long, the skeleton 2, the wight and the revenant 1.5, the ghoul as quick as you; wolves and deer never), but only after something (`Mind::Hunt` with a target): wandering, one stays on its side of the fence. A kind that `tumble`s (the shambler and the skeleton) lies on the far side for its `down` time as it lands, so the window you stand by is where you finish them. The ghoul, the wight and the revenant land on their feet;
       - **ways that climb** (`path::find_way`, `ClimbCost`): for one that climbs, a search may go through an open window or over a fence, at the step plus the ground it could have covered at its pace in the climb's time; `find_path` climbs nothing, as before. The grid's step cache keeps each tile's climbs a byte above its steps (`Grid::climbs_from`, `can_climb`), worked out again with every edit. A shambler's straight chase takes a fence straight ahead as open. The flow fields (a crowd to the bell, the dead to a fire) climb nothing yet;
       - **the save format** is version 26 (`Step::climb`);
       - **the client:** one climbing is lifted over the sill or the fence and set down (`DuskWorld.actor_climbs()`, `CLIMB_LIFT`). News: "You opened the window.", and when one of the dead you can see tumbles in, "A shambler tumbles in over the sill. Finish it while it's down!" (every 10 s at most; `CLIMBED` 102: 1 a window, 2 it fell; the door event's extra adds 2 for a window). The key help says E is "door or window" and that walking into an open window or a fence climbs it;
       - **checked:**
         - by tests: a fence is climbed in its ticks and for breath; a run vaults it quicker; a shut window isn't climbed, the door key opens it, and then it is, in its own time; a boarded window isn't, even open; weighed down, a climb takes twice as long; a way somewhere goes over the fence when the gate is far; the dead climb over after you and tumble down on your side; a kind that doesn't climb stays behind the fence; wandering, the dead stay in a fenced pen; the step cache's climbs stay true to the map through every edit (fences and open windows among the edges now); the game's climbing and each kind's climb load; a tumble without a climb, or a climb of nothing, refused;
         - in the game: headless runs on seed 7, one walking from the abbey into a fenced yard, with no script errors;
       - **measured:** about 162k to 197k ticks a second (from 167k to 191k: nothing to see; how an actor climbs is worked out only as a step starts for one going somewhere);
       - **not yet:** climbing low walls and up onto things; the dead breaking glass to come in, and the glass cutting you; the flow fields climbing; a climb's own pose (it's lifted, not drawn climbing); falling, with a heavy load or out of breath.
     - **The dead break glass** (2026-11-12; 327 tests pass; Zomboid's dead smash windows, and boarding them up is what holds a house):
       - **a window is beaten as a door is** (`Edge::beatable`: a shut door, or a window shut on its glass or with boards across it; `World::thump`): one of the dead that climbs, finding no way to you, beats on the window in its way as on a shut door (`in_the_way`, was `shut_door_toward`; wards are a door's alone), the boards first, the last nailed first (`barricade.rs`). Then the glass (`World::beat_glass`, its harm kept with the doors' in `door_damage`): it gives at `[climb] glass` (8 points, two to four of a shambler's blows; a door takes 60), with a crash heard 15 tiles off (`smash`), and the dead come in over the sill as they climb. A kind that doesn't climb leaves windows be. Your blows break glass too (swing at a shut window);
       - **broken for good** (`Edge::Window::broken`, `Edge::broken_window()`, `x` in a text grid): open and climbable when it has no boards, letting sound through, and it can't be shut (the door key does nothing). Boards can still go across it, and they're what keeps a window now;
       - **the glass left in the frame cuts** one of the living climbing through, a time in four (`[climb] cut`, `Purpose::Glass`): a laceration on an arm, a hand or a leg, so it bleeds, and a ghoul smells it (`Event::Climbed::cut`);
       - **the save format** is version 27 (`Edge::Window::broken`);
       - **the client:** a broken window keeps a rim of glass top and bottom (edge code 7, `SHARDS`). News: "Glass shatters!" (`WINDOW_BROKE` 103), "Something is beating on a window." (the thump's extra adds 2 for a window; no longer said of your own blows), "The broken glass cut you as you climbed through. Bandage it!" (`CLIMBED`'s extra adds 4). F's key help says a swing at a shut window smashes it. `--do` learns `swing_n`, `swing_e`, `swing_s` and `swing_w`, a swing one way;
       - **checked:**
         - by tests: the dead beat the glass out of a shut window and climb in; boards keep a window, the board breaking first, then the glass; the dead that don't climb leave windows be; a broken window can't be shut, and its glass cuts (an arm, a hand or a leg) or doesn't, by its chance; your blow breaks the glass with a crash of `smash` tiles; a broken window renders back as `x`; the game's glass numbers load;
         - in the game: a scratch copy of the data with no dead, beasts or rares (seed 7, `--walk=158,135 --speed=4 --do=18:swing_e`): "Glass shatters!" at the first blow, the next blows at the empty frame, the door key doing nothing; the picture shows the rim of glass left at the sill. A walk with the graveyard's skeletons about: "A skeleton tumbles in over the fence. Finish it while it's down!";
       - **measured:** about 162k to 194k ticks a second (nothing to see: a window is looked at only where a chase finds no way);
       - **not yet:** clearing the glass from a frame; the dead finding a window from inside a house as well as from out; shutters; the noise of a crash drawing the dead from farther than a thump does, as a rule of its own.
     - **Core mechanics** (the user, 2026-11-12: "Forgo those builds for now. Continue with core mechanics"): the verbs, the body and the pack before more content.
       - **The heavy swing** (2026-11-12; 333 tests pass; `CONCEPT.md`'s verbs: "heavy attack: a longer wind-up, and it knocks down"):
         - **the verb** (`Intent::HeavyAttack`, `ActionKind::Heavy`, `combat::Heavy`, rules.toml's `[heavy]`): a swing with your weight behind it, aimed as a swing is. Its wind-up is twice the weapon's (the longer tell, and a blow taken during it breaks it off), its recovery half as long again, and it costs twice the breath: six in a row leave you out of breath. Not while held. Only the living swing so;
         - **what it does:** 1.75 times the harm (`harm`, before what bone, the light and a sneak attack make of it), and half the time it knocks down what it hits for its kind's `down` time (`knock_down`; `Event::Felled`), letting go of whatever it held, so the next blow is a finish (always lands, double). It's two blows' wear on the weapon (`wear`), and it beats a door, boards or a window's glass harder (the thump's harm by `harm` too);
         - **the save format** is version 28 (the new intent and action kind);
         - **the client:** Shift+F, or Shift-click one of the dead beside you (`DuskWorld.heavy_attack()`). It leans in and tells as a swing does (action 46; the tell on the ground too). News: "Your heavy blow knocks it down. Finish it!" (`FELLED` 104). The key help lists it; `--do` learns `heavy`. The tell of a blow at a door now also shows at a window (`Edge::beatable`);
         - **checked:**
           - by tests: a heavy swing's wind-up, length and breath are the swing's by `[heavy]`; it does 1.75 times the harm and twice the wear; it knocks down by its chance (always at 1, never at 0), and a plain swing never does; it breaks a door in fewer blows; held, it doesn't start; the game's `[heavy]` loads, and a wind-up quicker than a swing's is refused;
           - in the game: headless runs on seed 7 with `--do=…:heavy`: heavy swings at the air ("You missed."), and "You're out of breath." after six in a row;
         - **measured:** about 160k to 192k ticks a second (nothing to see);
         - **not yet:** the dead swinging heavy (a brute); a heavy swing's own pose and sound.
       - **Strength and Fitness** (2026-11-12; 340 tests pass; `CONCEPT.md`, athletics: "Fitness and Strength grow over days of real exertion, as Zomboid's do... Returns diminish within a day"):
         - **two slow stats** (`dusk-core/src/fitness.rs`; `Actor::strength`, `fitness`, in thousandths of a level, 0 to 10). Everyone starts at 0, which is the game's balance, so nothing got weaker; they rise and don't fall back;
         - **the day's work:** every point of breath spent counts toward Fitness (`Body::spent`, counted in `Body::spend`, so running, swings, climbs and struggles all count, as paid after armour and the rest); blows that land (a heavy one twice), shoves that find someone, and steps carrying more than half what you can count toward Strength (`Actor::strain`, `World::strain`);
         - **at midnight** (`World::grow`, from `needs`) the day's work turns into growth (`Growth::gain`): at most `most` (0.6 levels) a day, half that at `half` (1500 points of breath, 100 of strain), and less the higher the stat already is. So a hard day gives a third of a level or so, and a week of it beats the same work in one day. The count starts again; a whole level reached is news (`Event::Grew`);
         - **what they do,** for every thousandth: Strength carries 1 kg more a level before you're weighed down (`World::carry`, now read wherever `defs.carry` was), does 3% more harm a level with a blow or on a door (`strong_harm`), and adds 2% a level to a shove's chance to knock down (`strength_knock_down`); Fitness takes 3% a level off every cost in breath (`fitness_cost`, in `endurance_rate`) and brings breath back 5% a level faster (`fitness_recovery`, in `recovery`). At 10: 25 kg, 30% more harm; 30% cheaper breath, half again as quick back;
         - **the data:** rules.toml's `[fitness]` and `[strength]` (dusk-data `fitness.rs`). `World::set_build` sets them, for tests and tools, as `creature_mut` does;
         - **the save format** is version 29 (`Actor::strength`, `fitness`, `strain`, `Body::spent`);
         - **the client:** the skills panel (K) opens on a Body section: Strength and Fitness, each with its level and a bar toward the next (`DuskWorld.character_build()`: both, and the day's strain and breath so far). News at midnight: "You feel stronger: Strength 2." / "You feel fitter: Fitness 2." (`GREW` 105: the stat × 100 plus the level). The pack's weight line reads your own carry;
         - **checked:**
           - by tests: a day gives less for each more of work, never more than its most, less the higher the stat, and a week of work beats the same in a day; a swing's breath and a landed blow's strain turn into Fitness and Strength at midnight, by `gain`, and the count starts again; a whole level reached is news; a blow, a heavy blow (twice), a shove and steps under a heavy load are strain, a light walk isn't; Strength carries more, hits harder and makes a shove knock down; Fitness makes a swing cheaper and breath come back faster, by the rules' shares; the game's `[strength]` and `[fitness]` load, and a greedy `most` is refused;
           - in the game: a scratch copy of the data with the clock at 23:54 and Fitness quick to grow: three swings, then at midnight "You feel fitter: Fitness 1."; the picture shows the Body section over the skills, Fitness 1 and most of the way to 2;
         - **measured:** about 156k to 189k ticks a second (seed 7 157k to 160k, from 160k to 162k: at the noise);
         - **not yet:** the stats falling back without use, or with hunger; a class or a trait starting above 0; Strength in climbing and struggling free.
       - **Bags** (2026-11-12; 353 tests pass; `CONCEPT.md`, items: "Inventory as in Zomboid: lists with weights, not a grid; bags with capacities"):
         - **pockets and a bag** (`dusk-core/src/bag.rs`): with no bag you carry what fits in your pockets (`Defs::pockets`, 5 kg). A bag on your back (`Actor::bag`, one at a time) holds its own capacity more, and what's in it weighs less on you by its share (`Bag { capacity, reduce }`, `ItemKind::Bag`). Each stack of the pack is in one or the other (`Item::bagged`), so every item verb still finds things by their slot;
         - **taking is bounded:** from a floor, a container or a counter, a stack goes into the bag while there's room, then the pockets, as many as fit (`World::can_take`, `room_for`); with no room, none, and the take never starts (`NoTake::NoRoom`; `NoDeal::NoRoom` at a counter). What comes to you otherwise (made, picked, butchered, coin for a sale, put away from your hand, armour taken off) is never refused: into the bag while there's room, then the pockets, over full if need be (`World::carry_in`, now the one way into a pack);
         - **reaching into the bag is slower:** using a thing from it, taking it in hand, lighting it or putting it on takes 1 s more (`Defs::from_bag`). A stack moves between the pockets and the bag, as many as fit, in 0.5 s (`Intent::Shift`, `can_shift`). So the pockets are for what you need quickly;
         - **putting a bag on** (`Intent::Wear`, as armour; 1.5 s, `Defs::bag_on`): the one you wore comes off into the pack, and what was in it moves into the new one (`don_bag`). A bag you take with a bare back and no room for it in your pockets goes straight on;
         - **a dropped bag falls with what's in it** (`Intent::DropBag`, in the drop's moment): the bag lies on the ground full (`Item::holds`, a number in `World::held`, which keeps its contents; saved and hashed), weighing all it holds. Taken up with a bare back, it goes back on with all of it, taking as long as its weight; with a bag on already, what's in it comes into yours as far as it fits, then the bag itself, and what doesn't fit stays in it (`take_full_bag`). A body's bag comes off full with its gear (`take_gear`), so a revenant carries yours, and drops it full when it's put down;
         - **the bags** (`items/bags.toml`): a satchel (0.5 kg, holds 6 kg, a quarter lighter: wardrobes, the general store, and the abbey stitches one a day for 8 coin), a knapsack (1 kg, 12 kg, 40% lighter: sheds, the store), a grain sack (0.4 kg, 15 kg, no lighter: the barns) and a traveller's pack (1.8 kg, 20 kg, half as heavy: only off the rares). The wardrobes' share of tinctures falls a little (seed 1's cures 3 to 2; seeds 7 and 11 unchanged);
         - **the data:** `[character] pockets`, `[items] from_bag`, `bag_on` and `shift`, and an item's `bag = { holds, lighter }` (dusk-data `items.rs`; a bag holds above 0 and at most 100 kg, lighter by 0 to 0.9). `Defs::default()` has no pocket limit, so tests that aren't about bags carry what they like;
         - **the save format** is version 30 (`Item::bagged`, `holds`, `Actor::bag`, `World::held`);
         - **the client:** the pack panel lists your pockets ("Pockets · 2.1 / 5.0 kg", red when over full), then "On your back: knapsack · 6.3 / 12.0 kg in it, 40% lighter" with Drop, each stack under the one it's in with "To bag" or "To pockets"; a bag in the pack has Wear. Take says why not ("No room for it: your pockets are full. A bag would hold more."), as Buy does, and Take all stops when nothing more fits. A full bag on the ground reads "satchel · 0.5 kg · 4.7 kg in it". `DuskWorld.character_bag()`, `character_pockets()`, `character_bagged()`, `take_check()`, `store_holds()`, `shift_check()`, `shift()`, `drop_bag()`; item kind 12. `--do` learns `dropbag`, `tobag` and `take`, and `wear` puts on a bag too;
         - **checked:**
           - by tests: the pockets take as many of a stack as fit and the rest stay, and with no room a take never begins; a bag holds more, the bag first, and what's in it weighs its share less; using a thing from the bag takes `from_bag` longer; a stack moves across as far as it fits, and nowhere with no bag; a dropped bag falls with what's in it, weighs all of it, takes as long to take up as its weight and goes back on with all of it; a full bag taken with one on empties into yours, the satchel after, or as much as fits, keeping the rest; a new bag takes over the old one's load, and the old one goes in; putting a club away into full pockets isn't refused; a bag with no room in full pockets goes straight on a bare back; a full bag on the ground lasts a save, the same world; a body's bag rises with its revenant and falls full when it's put down; the game's bags, pockets and times load, and a bag that holds nothing is refused;
           - in the game: a scratch copy of the data with a satchel and two planks in the kit, seed 7 at noon: "You put on the satchel.", three stacks moved into it, "You dropped the satchel.", taken up again: "You put on the satchel." The picture shows the pockets empty and the satchel back on holding 4.7 of its 6 kg (the bandage, the torch and the planks, each with To pockets), the pack at 7.0 kg with what's in the satchel a quarter lighter;
         - **measured:** about 156k to 160k ticks a second (seed 7 159k to 160k, from 157k to 160k: at the noise);
         - **not yet:** a bag in hand (a sack carried, taking the hand, Zomboid's way to haul more); a container's own capacity (a cupboard holds anything); a full bag's contents seen and taken one by one where it lies; bags in the drawing (a pack on the back).
       - **Wet and cold** (2026-11-12; 362 tests pass; `CONCEPT.md`, the senses: "Heat: your body, your clothes (wet means cold ... on the way to hypothermia), your fires", and the weather's moodles):
         - **wet** (`dusk-core/src/cold.rs`; `Body::wet`, out of 100): out in the rain you get wet, 2 points a game minute (soaked through in 50 minutes), 5 in heavy rain (20), and wading the shallows 10. Out of it you dry: half a point a minute in the open when it's dry, a point under a roof, 5 by a fire;
         - **cold** (`Body::chill`, out of 100) moves a game minute at a time toward what the moment is worth (`World::cold_due`), as dread does: half a point a minute colder, a point warmer, 3 by a fire. It's worth the night out of doors (30 at its darkest, by the sky's light), the wet on you (40 soaked through), the rain on you (10, heavy 20); less a roof (20), a bed asleep (20), and what you wear: armour has a `warmth` now (a padded jerkin 15, a leather jerkin 10, a cap and boots 5, metal little), and soaked through it keeps half of it. By a lit hearth, stove or forge (within 3 tiles, in sight of it: `World::by_fire`), nothing;
         - **what it does:** chilly (25) is a warning. Cold (50): you shiver, your aim (a swing's and a bow's: `numbed`) is 85% of itself and everything costs 15% more breath (`cold_cost`, in `endurance_rate`). Freezing (75): 70% and 30%, nothing heals, health drains 3 points a game hour, there's no lying down (`NoSleep::Cold`) and it wakes a sleeper (`Woke::Cold`). So a dry night out in a jerkin is nothing, a rainy one soaked is cold, a night of heavy rain is freezing, and a roof or a fire is the cure;
         - **the moodles and the news:** Chilly, Cold, Freezing and Damp, Wet, Soaked (`Moodle::Cold`, `Wet`); `Event::Cold` and `Event::Wet` for each new level (`COLD` 107, `WET` 106): "You're getting chilly.", "You're cold, and shivering...", "You're freezing! ... Get to a fire." (the screen flashes), "You're warm again.", and "You're soaked through." Lying down says "You're too cold to sleep."; woken, "The cold woke you, shivering.";
         - **the data:** rules.toml's `[cold]` (dusk-data `cold.rs`), armour's `warmth` (points, 0 to 50). `World::set_weathered` sets both, for tests and tools;
         - **the save format** is version 31 (`Body::wet`, `chill`);
         - **checked:**
           - by tests: light rain wets you two points a minute and heavy rain soaks you through (news), and a roof dries you a point a minute; wading soaks you in dry weather; what the cold is worth: the night, a jerkin's 15 off, soaked through all the wet's worth and half the jerkin, half wet, a roof, light and heavy rain; it drifts half a point a minute colder, no colder than it's worth, a new level is news, and a point a minute warmer under a roof; a lit stove warms you (nothing's due) three points a minute and dries you five, and doesn't reach the yard; chilly leaves your aim and breath alone, cold and freezing take theirs off a swing's chance and put theirs on its breath; freezing takes an hour's harm and heals nothing, and there's no sleeping; it wakes you; the game's numbers and the jerkin's warmth load, and a shiver that saves breath is refused;
           - in the game: a scratch copy of the data with heavy rain always and the cold rising ten times as fast, seed 7 at 23:00 in the abbey's yard: "You're getting chilly.", "You're cold, and shivering...", "You're soaked through...", "You're freezing!..." The picture shows Freezing and Soaked at the top of the moodle column, red, with their hints, the jerkin and the cap worn and the rain coming down;
         - **measured:** seed 7 about 155k ticks a second (from 159k to 160k), seed 1 149k (from 156k); with the cold's minute switched off seed 1 ran 151k to 153k, so it costs about a percent, and the rest moves between runs;
         - **not yet:** a cold region (the Wolfmark), seasons, or heat; warmth from a fire you carry (a torch) or a burning pyre; wet clothes that weigh more; heat-hunters seeing warmth.
     - **The Wizard and its first spells** (2026-11-12; 373 tests pass; the user: "pick loot/items/equipping, or spells"; `CONCEPT.md`: the Wizard, "Spells are timed actions, stated by their limits", "Spells work on the sense fields", "Components are loot", "Resting restores slots"):
       - **spells are data** (`data/spells.toml`, dusk-data `spells.rs`; `SpellDef`, `Defs::spells`): what each does (`SpellKind`: light, silence, fog, fireball), the slots it costs, its words (seconds), its range, how far it reaches round the spot, the light it gives and for how long, the noise of its words and of its burst, its harm, and the component it uses up. A class knows some (`Class::spells`) and holds slots (`Class::slots`);
       - **casting** (`dusk-core/src/spell.rs`; `Intent::Cast { spell, at }`, `ActionKind::Cast`): a timed action at a tile within range that you can see (`World::can_cast`: `NoCast` unknown, held, no slots, no component, too far, out of sight). The words are its wind-up and its tell, drawn on the spot it's cast on, and heard as far as they carry; a blow during them breaks the spell (`interrupt`, `Event::SpellBroke`) and nothing is spent. On the strike the slots and the component go (`Event::Cast`);
       - **the four:**
         - **Light** (a cantrip, no slot; 1 s, 8 tiles): lights 4 tiles round the spot for two minutes (`World::spell_lights`, in `light_at`); at night the dead that see it come to it as to a fire (`draw_to_fires`): a lure;
         - **Silence** (1 slot; 1.5 s, 6 tiles): for 30 s no sound is made within 3 tiles of the spot (`World::hushed`, in `emit`): not a footstep, a blow, a hammer or a groan. Board up a window inside one, and nothing hears;
         - **Fog** (1 slot; 1.5 s, 6 tiles): for 30 s the dead can't see into or out of 2 tiles round the spot (`fogged`, in `sees`);
         - **Fireball** (2 slots and a pinch of sulphur; 2.5 s, 8 tiles): everything within 2 tiles of the spot takes 25 to 35 (you too), what it puts down burns and the bodies lying there catch fire (`set_alight`), so none of them gets up and the dusk falls back; heard 35 tiles off, and a flash lights 7 tiles for a moment (`Event::Scorched`);
       - **lasting spells** are `World::zones` (`Zone`: kind, spot, reach, the tick it ends), faded each tick (`fade_spells`), saved and hashed;
       - **slots** (`Actor::slots`): the class's (3), and a talent's; all back once you've slept the night through (`rest_slots`, `Event::SlotsBack`), as the Second Wind is;
       - **Arcana**, the fourteenth skill: each spell cast teaches it (`[skills.xp] cast`), and each level takes 5% off the words (`arcana_cast`, `SkillRules::cast_ticks`);
       - **the Wizard** (`classes.toml`): fragile (its breath goes a tenth quicker, the dead get hold of it more easily), learns Arcana and First Aid half as fast again, comes with a quarterstaff, a wool cloak (warm: 15) and two pinches of sulphur. Its talents: Tough, Steady, Arcane Reserve (a slot more), Quick Casting (words a third quicker), Far Reach (3 tiles farther), Hushed Words (heard a quarter as far) (`Talent::slots`, `cast`, `range`, `hush`). Sulphur lies in sheds and the general store, and the abbey sells three a day;
       - **the save format** is version 32 (`Actor::slots`, `World::zones`, the fourteenth skill, the new intent and action);
       - **the client:** the class choice offers the Wizard; a Spells bar under endurance (`magic.arcane`); I opens the spells panel (zone J, one book panel at a time): each spell with its key, cost, words and range, its line and a Ready button. 4 to 7 ready a spell; the next click casts it on that spot, right-click lets it go, and a refusal says why (`cast_check`). On the map (`zones.gd`): Light a pale violet light with a mote over the spot, Silence a violet ring at its reach, Fog a low grey cloud. News: "You cast Fireball.", "The blow breaks your words: the Fireball is lost. Nothing's spent.", "Rested, your spell slots are all back.", "Your own fireball burns you!". `DuskWorld.spell_names()`, `spell_lines()`, `spell_info()`, `character_spells()`, `character_slots()`, `cast_check()`, `cast()`, `spell_zones()`; events `CAST` 108 to `SLOTS_BACK` 111; action 47. `--do` learns `spells`, `cast_light`, `cast_silence`, `cast_fog`, `cast_fireball` (on the nearest of the dead you see, else 3 tiles east);
       - **checked:**
         - by tests: Light costs no slot, its words are heard, it lights its spot and 2 tiles off, and fades when it's run its time; at night the dead that see it come to it; inside a Silence a sound isn't made, outside it is; the dead see you, then not in a Fog, and still see out of it elsewhere; a fireball costs two slots and a pinch, burns the two in its reach to death and they burn, a body there catches fire, one out of reach is untouched, it's heard 35 tiles off, and the flash lights; the limits: no such spell, too far, a dark solid thing between, no slots left, no sulphur, and a Fighter knows none; a blow breaks the words and nothing's spent; the slots come back with a night's sleep (news); a cast teaches Arcana, and a level casts quicker; the three talents add a slot at once, halve the words and hush them; the game's Wizard, its spells and Arcana load, and a fog that reaches nowhere is refused;
         - in the game: `--class=wizard`, seed 7 at 22:00 in the abbey's yard: "You cast Light.", "You cast Fireball.", "You cast Silence.", then "Not slots enough for the Fog...". The picture shows the spells panel (0 of 3 slots, each spell with its key, cost, words and range), the Silence's violet ring on the ground, the Light's pale violet on the walls, the quarterstaff in hand, the wool cloak worn and one pinch of sulphur left;
       - **measured:** seed 7 about 155k ticks a second, seed 1 150k: as before (with no spell lasting, sight and sound ask nothing more);
       - **not yet:** Knock (locks first), Chill and Warmth (heat-hunters first), Identify, scrolls and learning new spells, other schools; the Fog hiding the dead from you too; a fireball setting houses alight.
7. **Last of all, the bot:** `dusk-py` and training with `ppo.py`, then trained policies as NPC controllers.

## Open questions

None right now (2026-11-06). New ones get listed here.
