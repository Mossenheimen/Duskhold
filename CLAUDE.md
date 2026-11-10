# 12_Duskhold

An open-world fantasy survival RPG in Project Zomboid's style. The game comes first. Later it becomes a fast world for the PPO agent of `11_PPO-Zombie`, for training and maybe NPC AI.

- `CONCEPT.md`: what the game is. Decisions go in its "Decided" section.
- `ARCHITECTURE.md`: how it's built, the build order, and where the setup stands. Open questions are at the end; answered ones move into the text.
- `crates/dusk-core/`: the simulation (no dependencies, no graphics, no files, no threads). `creature.rs` holds what a kind of creature is (a row of numbers in `Defs::creatures`; an actor of it is `ActorKind::Creature(n)`); `combat.rs` holds the one rule for timed actions and the fight; `grab.rs` engagement, grabs, struggling and the drag-down; `items.rs` items, containers that fill from the seed, and the item actions; `body.rs` health, endurance and wounds; `needs.rs` the clock, hunger, thirst and fatigue, sleep, healing and moodles; `light.rs` day and night, each tile's light, and the torches and candles held up; `view.rs` the character's field of view and the fog of war; `skills.rs` skills that grow by real use under risk, repeats that pay less, the free cap and masteries, and what a level does; `barricade.rs` boarding up doors and windows, prying boards off, and the boards the dead beat through before the door.
- `crates/dusk-data/`: loads and checks `data/` (serde + toml), turning a designer's units into the core's.
- `crates/dusk-gen/`: a patch of a region from a seed: the layout (road, hamlet lots), then every tile as a pure function, then the dead.
- `crates/dusk-cli/`: the demo, the benchmark, and the data and world tools.
- `crates/dusk-godot/`: the Godot extension (gdext). Its `DuskWorld` node makes a patch, ticks it, and hands GDScript the map a chunk at a time, the actors, what the dead can see, noises and events. It takes intents back.
- `data/`: the game's numbers and content in TOML: `rules.toml`, `creatures/`, `fixtures.toml`, `items/`, `loot/`, `prefabs/` (buildings as text grids), `regions/`.
- `client/`: the Godot 4.7 project, built in code. It draws what the core says and decides nothing:
  - `main.gd`: input, the camera and the loop;
  - `map_view.gd`: streams chunks round the camera, and draws the fog of war and the sight-cone overlay;
  - `chunk_view.gd`: one chunk's ground, trees, furniture and walls;
  - `actor_view.gd` (poses: the lean of a wind-up, lying down, dead), `tells.gd` (a swing's tell on the ground), `threats.gd` (engagement lines), `bag.gd` (the pack and "within reach" panels), `skills.gd` (the skills panel), `litter.gd` (what lies on the floor), `noise_rings.gd`, `hud.gd` (clock, body panel, moodle column, news, the sleep veil, death screen);
  - `shaders/`: the cut-away is `cut_away.gdshaderinc`, also used for tree crowns and furniture. `fog_of_war.gdshader` is a full-screen pass over everything opaque, before the overlays.
- `play.cmd`: builds the extension and starts the game.
- `research/`: what's behind the decisions.
  - `game_design.md`: the 2026-11-06 design research (combat, loot, athletics, levelling, pacing and more), with its sources. It was accepted and folded into CONCEPT.md the same day.
  - The pure-Python benchmarks of a tick and of its hot spots (any Python 3).

## Commands (PowerShell, from this folder)

```powershell
.\play.cmd                                 # build the extension and play (the first run also imports the project)
.\play.cmd -e                              # the same, but open the Godot editor on client/
.\play.cmd -- --seed=7 --walk=4,5          # game flags after --: --seed, --town, --walk=X,Y, --zoom, --speed, --data=DIR, --do=S:ACTION, --news, --hour=H, --shot=PNG --shot-after=S
cargo test --workspace                     # 95 tests, a few seconds
cargo run -p dusk-cli -- check             # load and check everything in data/
cargo run -p dusk-cli -- prefab cottage    # a building, in its four turns
cargo run --release -p dusk-cli -- gen --seed 7 --png map.png   # a patch: its lots, and a map (--text X,Y,W,H, --ticks N)
cargo clippy --workspace --all-targets     # lints; keep it at zero warnings
cargo fmt --all                            # standard formatting
cargo run -p dusk-cli -- demo              # the scene, printed once a second of game time
cargo run --release -p dusk-cli -- bench --dead 100 --think-every 5   # ticks a second (ARCHITECTURE.md, performance budget)
cargo run --release -p dusk-cli --example python_twin                # research/python_tick_bench.py, line for line in Rust
```

Godot 4.7.2 is in `%LOCALAPPDATA%\Programs\Godot`. `Godot_v4.7.2-stable_win64.exe` is the editor; `Godot_v4.7.2-stable_win64_console.exe` prints to the terminal.

## Conventions

- **Build output** goes to `%LOCALAPPDATA%\cargo-target` (`CARGO_TARGET_DIR`), never into OneDrive. In the Bash tool, export it, and `PATH=$HOME/.cargo/bin:$PATH`, in each command: its shells don't see `setx` changes.
- **Determinism:**
  - no `HashMap` iteration in game logic (`hash::FastMap` is for lookups only);
  - gameplay randomness only through `hash::roll(seed, tick, who, why)`;
  - actors in id order, and one world per thread.
- **Test maps are text grids** (`text.rs`), the same format as prefabs and `World::render`. Their `z` is creature 0, the test shambler (`Creature::default`).
- **A creature is data, never a rule.** A new kind is a `[[creature]]` in `data/creatures/`; if it needs a number no creature has, add the field to `Creature` (with its conversion and check in `dusk-data`), not to `Rules`. In a test, change a kind with `w.creature_mut(n)`, after any `w.defs = ...`: the numbers live in the definitions.
- **The game's numbers live in `data/`,** in a designer's units (tiles a second, seconds, degrees). `dusk-data` converts them into the core's units and checks every reference: a new field needs its conversion and its check there. `Rules::default()` is a fixed set for tests and tools, not the game's tuning.
- **Generation stays order-free:** every tile and edge of a patch is a pure function of the seed, the layout and its place (`Layout::tile`, `north_edge`, `west_edge`), never of other chunks. `dusk-gen`'s tests check it.
- **Leave `11_PPO-Zombie` alone.** Don't edit it from here. Copy what's reused (`ppo.py`, telemetry, the dashboard) and note where it came from.
- **The client:**
  - **Rules live in the core.** A rule the client draws (sight, noise) comes from a core function (`World::sees`), never a GDScript copy of it.
  - **Generated files:** `client/dusk.gdextension` is written by `play.cmd` (it names this PC's build folder) and is gitignored, like `client/.godot/`. Commit the `.uid` files Godot makes next to scripts and shaders.
  - **Close the game before `cargo build`:** a running game holds the DLL. The editor loads a copy, so it reloads on a rebuild.
  - **Checking the look from the Bash tool:**
    - `cmd //c ".\\play.cmd --headless --quit-after 300"` runs the scripts with no window, but it can't compile shaders.
    - `cmd //c ".\\play.cmd -- --shot=<scratchpad>/x.png --shot-after=5"` opens the game for a few seconds on the user's desktop and saves a picture.
  - **Colours:** client colours are picked in sRGB. MultiMesh instance colours go through `srgb_to_linear()`.
