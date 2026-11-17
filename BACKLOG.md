# Backlog: what's still to build

One list, made on 2026-11-13 from the 32 "not yet" lists in the build log (`HISTORY.md`; the ones built since are left out), `CONCEPT.md`'s systems not yet started, `VISUALS.md` §16's slices to come, and the 2026-11-13 review. When a slice leaves something for later, it goes here, ranked, rather than only into its history entry.

Decisions that are the user's are in `ARCHITECTURE.md`, "Open questions". The first playtest was on 2026-11-16, the second on 2026-11-17 (the user's `Notes.md`): their notes are the queue's first nine, in the user's words where they're quoted. The inventory as Path of Exile 2's, the first playtest's seventh, was built on the other PC (2026-11-17).

## Queue (most useful first)

Ranking: since the playtest, what got in the way of play comes first, the quickest first; then what makes the fight readable and fair; then the Diablo and Path of Exile ways the user asked for (spells, loot, the inventory, the map); then the rest as it was. Size: S an afternoon, M a slice, L a few slices.

| # | Item | Area | Why | Size | From |
|---|---|---|---|---|---|
| ~~1~~ | *(Built 2026-11-16, 06205d4.)* **The playtest's quick fixes:** the sound rings and the dead's sight on the ground no longer drawn (a setting brings each back); Space shoves, Esc pauses and opens the menu (Resume, Settings, Save and quit: `VISUALS.md` §16, slice 9's menu); T holds a torch up; running from the start, Shift switching between walking and running, and quicker on your feet; the death screen without what the body carried, in plainer, darker words; windows no longer showing through the unexplored dark | the GUI, the feel | "The sound circle is annoying to see during gameplay", "I also do not need to see enemy range/agro tiles", "Spacebar should be push, not pause. Escape should be pause/menu", "Using a torch should be bound to T", "Move speed is pretty slow. Can we also start with running on and make run/walk a toggle?", "I do not care what my body had when I die", "Windows can be seen from far away on the screen in unexplored areas" | M | the playtest |
| ~~2~~ | *(Built 2026-11-17.)* **Combat you can read:** Zomboid's combat stance (the right mouse button held: on guard, slower, facing the cursor, stepping back allowed; a left click swings that way; a shield braced in it, a bow drawn); damage numbers where blows land; health bars over what you fight | combat | "Holding right click should put you in a combat stance ... and swing with left click"; "I'd also like to see damage numbers and health bars during combat" | M–L | the playtest |
| ~~3~~ | *(Built 2026-11-17.)* **The second playtest's quick fixes:** claiming a bounty takes no time; you walk through chairs; taking a thing in hand or putting it on is at once out of a fight (it takes its time in one); "Make it your stash" offered only where it can be; an XP bar on the main screen, and the bars at the top left gone (the globes say it); the death screen a panel with Restart and New World buttons; a potion of life closing your wounds and curing ghoul fever | the GUI, the feel | "Claiming a bounty shouldnt take any time", "You should be able to walk through chairs", "Equipping items shouldnt take time out of combat, but should take time when in combat", "What is with the option to make anything your stash? ... obnoxious when you see it everywhere", "I want to see an xp bar for the character level on the main ui. Also, get rid of the upper left health/endurance", "The death menu should have a panel and buttons for Restart / New World. No keybind shown", "Potions should heal/clear wounds and ghoul fever" | M | the second playtest (`Notes.md`) |
| 4 | **No grabs: a blow slows you a moment; tuning the dead** (in the core and `data/`): the dead no longer take hold of you or drag you down; a blow that lands slows you briefly instead; then the dead's harm and health until a few in a row can be put down | the fight | "Being stuck, or 'grabbed', by monsters is not fun and unfair. You should only be slowed briefly when hit by monsters, no grapple/freeze"; the first: "Enemies may also be overtuned" | M | both playtests |
| 5 | **Click one of the dead to go at it, and hold to keep at it:** a click on one goes after it wherever it walks and strikes when it's in reach (Diablo's); held, it strikes again until you let go | the fight | "If I click a monster to attack it, the character should go attack whereever the monster is, even when moving ... And if I click and hold, it should continue to attack till I release" | M | the second playtest |
| 6 | **Spells that hit hard:** more spells that harm, and none at this level wanting a reagent (higher ones may); a blow on you sets the words back a little rather than breaking them; the Wizard's spells made for the fight (harm, areas, missiles: Path of Exile's and Diablo's), the utility taking a back seat; the dead getting up again playing a smaller part | magic | "Spells are underwhelming. I don't so much care about stopping zombies from getting up, etc." | L | the playtest |
| 7 | **Walls that stay:** the see-through round you larger, and working at corners and half-way through doors; the low walls, which the user likes, on by themselves indoors; no wall or door vanishing as you walk behind it, you seen through it instead (Diablo's and Path of Exile's outline), and walls and doors the same from every side; the fog of war soft at its edges, not in tiles | the look | "Objects shouldn't disappear, like walls, when you walk behind them ... The transparency feature is not well built and needs work"; "Can the fog of war be less blocky on the edges?" | M–L | the playtest |
| 8 | **Loot on the ground, as Diablo's:** the grid only for a chest, a crate or a counter; what lies on the ground named on the screen where it lies, as you come near (the second playtest); each thing drawn where it lies; a key (Alt) shows every name on the ground in its rarity's colour, and a click on one takes it; the dead drop theirs as they fall | loot | "I like diablo/PoE loot drop style, more than zomboid. I like to see the loot on the ground. I also want a button that will show loot names on the ground" | L | the playtest |
| 9 | **Who comes to the abbey, as a screen:** a panel with the classes to pick from, a name to type, an ancestry to pick (ancestries first: `CONCEPT.md`), each with its words and what it starts with; laid out from research into games' character screens | the GUI | "The intro menu (Who comes to the Abbey) should be a panel with class selection, name input, race pick, etc.. It should also be improved by doing some research on game ui" | L | the second playtest |
| 10 | **The map:** M the world map, as far as you've seen it; Tab switching between no mini-map, a small one in a corner, and a see-through map over the screen (Diablo's and Path of Exile's); the pack moving to I | the GUI | "We will also need a map/mini-map at some point. M should open the map/world map. Tab should switch from no mini-map, small corner mini-map, and transparent map overlaid on the screen" | M–L | the playtest; `CONCEPT.md` |
| 11 | **Sound, a first pass:** footsteps, a groan, a blow landing, a door, the bell; a cue for where a sound came from (the rings, no longer drawn, were the only one) | sound | "Sound is gameplay" (`CONCEPT.md`), and the game is silent. Every sound is already an event, so it's playing what the core says | L | `CONCEPT.md`; settings |
| 12 | **Replays:** the player's intents kept with their ticks; `dusk-cli replay` plays a death back; a fixed-step flag for scripted runs | tools | `ARCHITECTURE.md`'s first decision promises "any bug or death can be replayed". It would also make pictures and `check.cmd`'s runs come out the same every time | M | the review |
| 13 | **`main.gd` in parts:** the news (`_read_events`) to `news.gd`; the script runner to `script.gd` as a table of actions; the bridge's shared guard | code | The review's merges still to do; `main.gd` is past 2,500 lines | S–M | the review |
| 14 | **The rest of the assets, in the kit** (`VISUALS.md` §5): items in hand and on the floor (each item's look and its icon are built, 2026-11-17: §16, slice 10); the bodies, heads, shoulders and what's in hand (slice 11) | the look | The user's ask, 2026-11-13: assets in one style throughout. Loot on the ground (#6) wants them, as the inventory (#7) did | L | the user; `VISUALS.md` |
| 15 | **The Warren Hills:** the second region, goblins that see and think | world | The concept's next step after the first playable, to prove the first pillar | L+ | `CONCEPT.md` |
| 16 | **The Rogue:** the fourth class (dark, quiet, sneak attacks), with locks and traps | character | The concept's "four to start"; locks and traps serve the base too | L | `CONCEPT.md` |
| 17 | **The Dusk Director, and quests from the holdouts** (rescues, recoveries, caravans) | events | The concept's pacing: build-up, peak and relax, and reasons to go out beyond bounties | L | `CONCEPT.md` |

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
- The bar, after its first cut (2026-11-16): a spell dragged from the spells panel onto it, and more than four powers on keys once a class has more; a potion's time left on its moodle and its slot; a cast's or a drink's progress on its slot.
- What you learned and the places you found carrying over to the next character; the holdouts remembering you; the revenant fighting the way you did (a bot trained on your play).

**Items**
- Weapons, after quality, the uniques, Path of Exile's rares and each class's edge (2026-11-13 to 16; the user: "take inspiration from Diablo and PoE 2"): an edge growing with the class's skill (a Blade master's finish, an Axe master's longer bleed), if the flat ones aren't felt; a spear's reach (a blow from two tiles off), which the fight's one-tile swing doesn't have yet; the bleeding seen on the dead (drops on the ground behind it, with the look's other effects); properties and uniques that add to an edge ("+1 s bleeding", "+5% chance to drive back"), as Path of Exile's mods add to an implicit.
- The card and properties, after the numbers (2026-11-14), armour's, bows' and shields' magic and uniques, rolls within a tier, and Arcana's levels doing more (2026-11-15): a spell's reach growing with Arcana too (Silence's and Fog's), if their time alone isn't felt; Dex and Int only if the body gets more stats than Zomboid's two.
- Rares, after the first cut: a tier weighted by how far out (the best less often, as Path of Exile's), not even odds; more light properties, so a Rare light has three; depth as well as distance, once there are dungeons; a holdout's crafting currency (re-rolling a property, Path of Exile's orbs) once there's a smith.
- Uniques, after the first six: the barrows' own grave goods, richer than the graveyard's (a fixture of its own, so a table of its own); a unique's numbers rolled in a range, as Path of Exile's; each unique once in a world (where it lies decided by the seed); a drink of health seen (motes from the struck to you); uniques on lights; a unique's thorns or calm on rolled properties too (Path of Exile's "reflects" mods), if the uniques' alone feel too rare.
- Quality on tools and bags; making and mending raising it (a smith).
- Attunement; the Lore skill and scrolls of identify; Very Rare and Legendary; a broken magic item kept, to be mended.
- Making armour (a hide jerkin from the deer); armour wet and cold in the rain; the Fighter's talents for armour.
- A sack carried in hand; a dropped bag's things taken one by one. *(A container's own capacity is its grid, since 2026-11-16.)*
- The inventory, after its screen (2026-11-17): Shift-click splitting a stack; dragging as well as click-to-pick; a key that tidies a grid; a thing put down on another swapping them (Path of Exile's), not refused; the figure's own silhouette behind its slots; a counter's wares in tabs by kind, and a sale undone before you walk away; items' icons in the news and in E's list; the inventory and its cards at high contrast checked with a person who needs them.
- Potions and the belt, after the first six (2026-11-16): brewing them (Alchemy, at a workbench, from what's foraged); greater potions found farther out, as magic's tiers are; a belt worn as gear, with more loops or deeper ones (Diablo II's); loops arranged by hand; a potion thrown (holy water on the dead, oil alight).

**The base and the holdouts**
- Clearing the glass from a frame; shutters; locks.
- Workbenches (smithing, alchemy, scribing), a rain barrel, a garden, traps, a pyre.
- Doors and walls of your own, after the first (2026-11-16): a wall of planks' own look in the kit (it's a door's wood now); stone walls (Masonry), stronger and slower; a gate in a fence; a wall taken down for its planks; a door with a lock (with the Rogue's locks); a window of your own.
- Decoys, after the clatterbox (2026-11-16): thrown, to land where you aim (Zomboid's noise-makers), not only set where you stand; set off by a tripwire (with traps) or the morning; the abbey's counter selling them; its own look in the kit (with items' looks); its beat shown before it starts (a wind-up ticking in the litter), so you know it's the one that's wound.
- The abbey's sisters as actors: defenders, morale, raids; training and healing for coin; selling the weapon in your hand.
- Rewards for a night survived; the Director choosing how often the Tolling comes; a raid's horns and a hunter's moon (other regions).

**Senses and the GUI**
- The signature HUD: how loud, bright, warm and scented you are.
- Scent trails and heat as fields; heat-hunters seeing warmth.
- Combat you can read, after its first pass (2026-11-17): Diablo's bar at the top of the screen for the one under the mouse (its name, its health); numbers for a block, a miss or a sneak attack's, each its own; the dead's bars only once they're after you.
- The GUI's dress, after its first pass (2026-11-17): the figure's empty slots as faint silhouettes (a helm, a glove, a boot); the moodles' tiles and the status line dressed too; glints along the frames.
- The second playtest's quick fixes, after them (2026-11-17): a level gained flashing along the bar's XP; "in a fight" (equipping takes its time) shown, a mark or a moodle, so a slow swap isn't a surprise.
- A gamepad; text size on its own; the title screen (`VISUALS.md` §16, slice 9's rest: the menu is in the queue's first).
- The figure for creatures; wounds shown closing as they heal.
- Panels fading in and out (§12's 0.12 s); small caps or italics; a prompt for what's within reach (zone G lists it already).

**The look** (`VISUALS.md` §16)
- Slice 9, screens (the menu in the queue's first, the title screen under the GUI above); slice 11, shapes: heads, shoulders and the thing in hand for people and the dead (in the queue's assets); slice 12, regions, each with its own materials in `looks.toml`. *(Slice 10, items' icons drawn from their looks, is built: 2026-11-17.)*
- The kit, after slice 8: a fixture in a corner with two walls turns to the north or west one, which isn't always the right one (a wardrobe beside a door); a fixture across a chunk's edge from what it faces doesn't find it.
- The terrain, after its first pass: fallen logs and stumps in the woods; stones along a shore; the strewn grass worn away where paths are walked.
- Wear showing: blood, dirt and scorch on gear and the character, damage on bases.
- Poses: a climb, one turned by the holy symbol (it's marked, not posed); armour and bags drawn on the character and the dead.
- Effects still to come: a window's glass flying as it breaks; the Tolling's bell; a burning body's sparks; a fireball's flames seen through trees between it and the camera (crowns aren't cut away for a spell's spot).

**Saves and tools**
- More than one save; the speed dial saved (the zoom is kept since 2026-11-13).
- Saves, after they outlast updates (2026-11-16): the whole land saved, not only the chunks that changed, so a new prefab or a moved lot leaves a world in play as it was (its layout too: place names, the bell); an id renamed in the data found by its old one (`was = [...]`), so a save naming it still loads; the first migration step, when `save::VERSION` first moves (with its own golden save).
- Golden replays, property tests and criterion benchmarks (`ARCHITECTURE.md`, Tests and tools).
- The Rust half of `check.cmd` on GitHub at each push.
- Data reloaded without a restart in dev builds; mods as a folder of the same files.

**Later** (`ARCHITECTURE.md`)
- The bot (build step 7): `dusk-py`, many worlds stepped at once, observations and masks, `ppo.py`; then trained policies as NPCs.
- Lua for quests and events; co-op (`dusk-server`); PvP.
