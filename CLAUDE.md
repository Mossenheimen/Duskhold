# 12_Duskhold

An open-world fantasy survival RPG in Project Zomboid's style. The game comes first. Later it becomes a fast world for the PPO agent of `11_PPO-Zombie`, for training and maybe NPC AI.

- `CONCEPT.md`: what the game is. Decisions go in its "Decided" section.
- `ARCHITECTURE.md`: how it's built, the build order, and where the setup stands. Open questions are at the end; answered ones move into the text.
- `crates/dusk-core/`: the simulation (no dependencies, no graphics, no files, no threads).
- `crates/dusk-cli/`: the demo and the benchmark.
- `research/`: measurements behind decisions (pure-Python benchmarks of a tick and of its hot spots, run with any Python 3).

## Commands (PowerShell, from this folder)

```powershell
cargo test --workspace                     # 19 tests, about a second
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
- **Test maps are text grids** (`text.rs`), the same format as prefabs and `World::render`.
- **Rules that will become data** (`world::Rules`, `ActorKind::speeds`) say so in a comment; they move to `data/` at build step 4.
- **Leave `11_PPO-Zombie` alone.** Don't edit it from here. Copy what's reused (`ppo.py`, telemetry, the dashboard) and note where it came from.
