# Backlog: what's still to build

One list, made on 2026-11-13 from the 32 "not yet" lists in the build log (`HISTORY.md`; the ones built since are left out), `CONCEPT.md`'s systems not yet started, `VISUALS.md` §16's slices to come, and the 2026-11-13 review. When a slice leaves something for later, it goes here, ranked, rather than only into its history entry.

Decisions that are the user's are in `ARCHITECTURE.md`, "Open questions": a playtest first, what comes next, saves, and the repo.

## Queue (most useful first)

Ranking: does it protect or test play (a game that's played needs saves that last, and numbers someone has felt)? Does the game lack it where the player would notice now? Then: does it deepen the loop (scavenge by day, hold by night, bring it home), or prove the next pillar? Size: S an afternoon, M a slice, L a few slices.

| # | Item | Area | Why | Size | From |
|---|---|---|---|---|---|
| 1 | **Saves that survive updates:** a self-describing format with defaults for new fields, or migrations | saves | The format moved from version 1 to 32 in three days, and each move sets the player's world aside (`world.dusk.old`). Postcard can't skip a field it doesn't know. Needed before anyone plays in earnest | M | the review; saves slice |
| 2 | **A playtest night, then tuning** (in `data/` only) | the loop | The concept's first question, whether surviving the Hushlands is tense and fun, isn't answered by play: 46 slices were checked by tests, headless runs and pictures. Needs the user | M | the review; `CONCEPT.md` |
| 3 | **The Esc menu, the title and death screens** (`VISUALS.md` §16, slice 9) | the GUI | Esc with nothing open saves and quits at once; the settings are only on F10; there's no title, so a new world is Ctrl+N | M | `VISUALS.md` |
| 4 | **Sound, a first pass:** footsteps, a groan, a blow landing, a door, the bell; a cue for where a sound came from | sound | "Sound is gameplay" (`CONCEPT.md`), and the game is silent. Every sound is already a ring and an event, so it's playing what the core says | L | `CONCEPT.md`; settings |
| 5 | **Base-building II:** decoy noise-makers, repairs, then walls and doors of your own | the base | The night half of the loop. Decoys are the Hushlands' own base tool in the concept's table, and "damage stays until you repair it" | L | `CONCEPT.md` |
| 6 | **Replays:** the player's intents kept with their ticks; `dusk-cli replay` plays a death back; a fixed-step flag for scripted runs | tools | `ARCHITECTURE.md`'s first decision promises "any bug or death can be replayed". It would also make pictures and `check.cmd`'s runs come out the same every time | M | the review |
| 7 | **The map** (a book panel) | the GUI | Zomboid's map that holds only what you've seen, and your notes. The fog of war already remembers it | M | `CONCEPT.md` |
| 8 | **`main.gd` in parts:** the news (`_read_events`, 431 lines) to `news.gd`; the script runner (188) to `script.gd` as a table of actions; the bridge's shared guard (95 of its 178 functions open the same way) | code | The review's merges still to do | S–M | the review |
| 9 | **Looks as data** (`VISUALS.md` §16, slice 8): `data/looks.toml` for furniture and plants | the look | A person can change a look without code | M | `VISUALS.md` |
| 10 | **The Warren Hills:** the second region, goblins that see and think | world | The concept's next step after the first playable, to prove the first pillar | L+ | `CONCEPT.md` |
| 11 | **The Rogue:** the fourth class (dark, quiet, sneak attacks), with locks and traps | character | The concept's "four to start"; locks and traps serve the base too | L | `CONCEPT.md` |
| 12 | **The Dusk Director, and quests from the holdouts** (rescues, recoveries, caravans) | events | The concept's pacing: build-up, peak and relax, and reasons to go out beyond bounties | L | `CONCEPT.md` |

## By area

**World and regions**
- The other regions (Warren Hills, Webwood, Wolfmark, Cinder Reach, Weeping Moor), each with its family, senses and base; a palette shift each (`VISUALS.md` §16, slice 12).
- The realm: regions round the heartland, chunks made on demand, the population layer (counts off-screen), more than one patch.
- Cellars and upper floors (z from −2 to +3), stairs and ladders.
- Weather: fog, wind, snow and seasons, heat; rain filling barrels; lanterns, which rain doesn't put out.
- Dungeons, the end-game; a relit wardstone lowering the pressure's ceiling.

**The dead and the wild**
- Ghouls eating the dead (an action that can be interrupted); a trail of blood on the ground, rather than a scent from where you stand.
- Rares roaming; the Spectral; family affixes (a Gravecaller raising the dead).
- The dead swinging heavy (a brute).
- The dead finding a window from inside a house; a crash drawing the dead from farther than a thump, as a rule of its own; flow fields that climb.
- Fire that harms the living or spreads to a house; a fireball setting houses alight; ash cleared away.

**You**
- More classes: the Rogue (in the queue); later the Ranger, the Bard, the Paladin, the Warlock.
- Ancestries and traits (`CONCEPT.md`); a class or a trait starting a stat above 0.
- Skills still to come: Lockpicking, Traps, Lore, Alchemy, Smithing, Masonry, Farming, Tracking. Books that multiply a skill's XP; trainers; a trainer re-picking talents for coin.
- XP for places found, days survived and bounties.
- Strength and Fitness falling back without use, or with hunger; Strength in climbing and struggling free.
- The Cleric's other verbs (wards of salt and words on a whole house, cures for others); holy water thrown; the sisters teaching prayers or blessing weapons.
- The Wizard: Knock (locks first), Chill and Warmth (heat-hunters first), Identify, scrolls and learning new spells, other schools; the Fog hiding the dead from you too.
- What you learned and the places you found carrying over to the next character; the holdouts remembering you; the revenant fighting the way you did (a bot trained on your play).

**Items**
- Attunement; the Lore skill and scrolls of identify; Very Rare and Legendary; a broken magic item kept, to be mended; magic on armour.
- Making armour (a hide jerkin from the deer); armour wet and cold in the rain; the Fighter's talents for armour.
- A sack carried in hand; a container's own capacity; a dropped bag's things taken one by one.

**The base and the holdouts**
- Clearing the glass from a frame; shutters; locks.
- Workbenches (smithing, alchemy, scribing), a rain barrel, a garden, traps, a pyre.
- The abbey's sisters as actors: defenders, morale, raids; training and healing for coin; selling the weapon in your hand.
- Rewards for a night survived; the Director choosing how often the Tolling comes; a raid's horns and a hunter's moon (other regions).

**Senses and the GUI**
- The signature HUD: how loud, bright, warm and scented you are.
- Scent trails and heat as fields; heat-hunters seeing warmth.
- A bar on screen showing your powers; a gamepad; text size on its own.
- The figure for creatures; wounds shown closing as they heal.
- Panels fading in and out (§12's 0.12 s); small caps or italics; a prompt for what's within reach (zone G lists it already).

**The look** (`VISUALS.md` §16)
- Slice 8, looks as data; slice 9, screens (both in the queue); slice 10, icons, by kind and then each item's own; slice 11, shapes: heads, shoulders and the thing in hand for people and the dead; slice 12, regions.
- Wear showing: blood, dirt and scorch on gear and the character, damage on bases.
- Poses: a climb, a heavy swing, one turned by the holy symbol; armour and bags drawn on the character and the dead.

**Saves and tools**
- More than one save; the camera and the speed dial saved.
- Golden replays, property tests and criterion benchmarks (`ARCHITECTURE.md`, Tests and tools).
- The Rust half of `check.cmd` on GitHub at each push.
- Data reloaded without a restart in dev builds; mods as a folder of the same files.

**Later** (`ARCHITECTURE.md`)
- The bot (build step 7): `dusk-py`, many worlds stepped at once, observations and masks, `ppo.py`; then trained policies as NPCs.
- Lua for quests and events; co-op (`dusk-server`); PvP.
