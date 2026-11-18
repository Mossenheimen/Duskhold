# Duskhold: the look

This guide covers how Duskhold looks and how its screens are laid out. It sets the camera, light, colour, shapes, surfaces and the drawn senses. It also sets the type, the GUI's layout and parts, and how the AI and people work on the look together.

The AI builds most of the look from this guide. A person steers it by changing a number, dropping in a reference picture, or making an asset to a template (§14).

Read §1 always, then the section for what you're changing. Keep this guide and the code in step: a change to the look changes this guide in the same commit, as `ARCHITECTURE.md` is kept.

**Words used here:**

- **Must:** break it only by changing this guide first.
- **Should:** the rule. Break it only with a reason in the commit.
- **Default:** chosen by the AI and not yet confirmed by the user, as in `CONCEPT.md`. The user may change it.
- **Today:** what the code does now, where it differs from the target. §16 lists the work to close those gaps.
- **Sizes:** all sizes are pixels at the base screen of 1920 × 1080, and everything scales with the screen (§10).

---

## 1. The look in brief

Duskhold looks at its world the way Project Zomboid does, but with a fantasy dusk over it:

- **The view:** a small, lived-in place seen from above at an angle, through an orthographic camera.
- **The things:** simple low-poly shapes with quiet surfaces, all made in code.
- **Day and night:** by day the world is muted and cool. By night it's truly dark, and the warm light of a torch or a fire is what keeps you safe.
- **The dead:** each kind reads by its shape before its colour.
- **The senses:** everything the dead can sense of you is drawn on the ground.
- **The GUI:** quiet. Dark glass panels with plain text sit at the edges, and the middle of the screen belongs to the world.

**Pillars** (each later section serves these):

1. **Readable before beautiful.** At a glance you can tell who's there, where they are, what they know of you, and what they're about to do. Value (light against dark) and silhouette come first, colour second. This is the game's own pillar, "what hunts you, you can see" (`CONCEPT.md`).
2. **The world is dim, and what matters is lit.** Ground, walls and furniture sit lower in value and saturation than actors, signals and light, so the things that move and threaten stand forward.
3. **Plain and lived-in.** The world is made of ordinary things a little worn: tables, sacks, wheat, a well. Nothing is shiny. Magic is rare, and it's the only thing that glows by itself.
4. **One language for the senses.** Sound is a pale ring and sight is a tint on the ground. Danger is red, a wary state amber, and calm pale. A colour means the same thing in the world and in the GUI, every time.
5. **A quiet GUI.** Show what matters now, and keep the rest a key away. Nothing stays in the middle of the screen.
6. **Made of numbers.** Every look is a set of parameters a person can change in one place (§14). Procedural comes before painted, and any painted or modelled asset follows a template.

### What we take from Zomboid, and what we don't

| Take | Leave |
|---|---|
| The angled, orthographic view; walls cut away round you; truly dark nights and warm torchlight | Text too small for high-resolution screens, and a GUI that doesn't scale (Build 42 had to enlarge its icons) |
| **Moodles:** a column of icons on a severity-tinted tile, with the detail on hover | Many always-on buttons and windows. Players ask for ways to hide parts of the HUD, so ours shows less by default |
| A health panel with a body diagram of the wounds | Deep right-click menus for every act. Ours names the act and its key beside you (§11, prompts) |
| Two inventory lists: yours, and what's within reach, with weights | Fiddly transfers between them. "Take all", "Stash finds" and sorting are first-class here |
| Death told as a story ("this is how you died") | |

---

## 2. Camera and scale

- **Must:**
  - An orthographic camera, pitched 30° down and turned 45° (`main.gd`: `CAMERA_PITCH`, `CAMERA_YAW`). Every asset is made for this view.
  - 1 tile = 1 metre = 1 world unit.
- **Zoom:** `camera.size` is how many metres of the world fit the screen's height. The default is 14, the range 6 to 40.
  - **Today** (2026-11-13), as Zomboid's: the wheel zooms a step of 12% a notch (a trackpad's finer scroll a part of one, and its pinch too), and so do `-` and `=`; the camera eases to it, centred on you. The player's own game keeps the zoom it was left at (`user://settings.cfg`). Zoomed out, what you've never seen stays near-black (the fog of war), so early on the far view is mostly dark.
  - At the default on a 1080p screen, a tile is a diamond about 109 × 55 px (2:1, classic isometric), and a person stands about 100 px tall.
  - **Must:** everything reads at 14 and 20. Fine detail may show only at 6 to 10.
- **Cut-away:** walls near you and near the cursor are cut away with a dithered edge, so you see inside, and a shadow-only copy keeps the shadows right. V lowers every wall to 0.35 m.
- **The sun** comes from the south-west, so shadows fall up and to the right, away from the faces turned to the camera.
- **Fog of war, three states** (`shaders/fog_of_war.gdshader`):
  - **Seen now:** full colour.
  - **Remembered:** 75% grey, at 30% brightness. Things that don't move (walls, furniture, plants) stay drawn in memory. Actors are drawn only while you see them, which is Darkwood's lesson: the unseen space is the horror.
  - **Never seen:** near-black.

---

## 3. Light and time

Light here is a rule, not just a mood: the dead see by it (`light.rs`). So the picture must never lie about light.

**Must:** what the core counts as lit is drawn lit, and a light's radius on screen is its radius in the core.

| Time | Sun (colour, strength) | Ambient (colour, strength) | Sky |
|---|---|---|---|
| Midnight | moon-blue `#8C9EF2`, 0.06 | `#47548C`, 0.16 | `#030306` |
| Noon | warm white `#FFF0D6`, 1.1 | `#94A1B8`, 0.6 | `#12141A` |
| Dusk and dawn | blends toward `#FF9E5C` | between | between |

- **Night must be dark enough that a torch changes what you can see**, since light is a trade (`CONCEPT.md`). Unlit ground at midnight is barely there. The GUI and the drawn senses stay fully readable, because they're information, not light.
- **Indoors** is darker than outdoors by day, as the core rules.
- **Warm lights** are the safe colour:
  - torch `#FFB86B`, with a flicker of 6% at 13 Hz plus 4% at 31 Hz, in brightness only;
  - fire `#FF8F3D`, and a weapon's fire damage on its card;
  - embers and a burning body `#FF6B1A`, emissive.
- **Cold light** belongs to the night and the dead: moonlight, and the wight's dark.
- **Rain** darkens the sky and sends streaks past the camera; lights outdoors gutter.
- **Dread** darkens the screen's edges violet-black (`shaders/dread.gdshader`), pulsing when you're afraid. The pulse must stay slower than 3 Hz, and it stops when flashes are reduced (§13).

---

## 4. Colour

Colours are named tokens, and code uses the names (§14). In code each token is a constant in `client/look.gd`, named by its last word in capitals: `signal.calm` is `Look.CALM`, `moodle.mild` is `Look.MILD`, `ui.panel` is `Look.PANEL`. The few a setting changes (the alert states, and `ui.panel`) are static variables instead, and what copies one copies it again when `Look.version` moves on (§13). The ground is `Look.GROUND[code]`, and `moodle.good` is `Look.REST`.

The world's own colours are data: `data/looks.toml`'s `[material]` table, by the same names (`ground.grass`, `build.wall`), where the looks of things take their materials too (§5, "The kit"). `Look.take_materials` brings the ground's, the buildings' and `world.litter` into `look.gd` as the looks are read; until then they're the values below.

**Today** (2026-11-15): every colour the client draws with is a token: the world's in `looks.toml` (since slice 8, §16), the rest in `look.gd`, merged as the tables below say. *(Before slice 8, furniture and plants were tables of their own in `chunk_view.gd`.)*

### The world (the Hushlands)

| Token | Hex | Used for |
|---|---|---|
| `ground.void` | `#12141A` | nothing: off the map |
| `ground.grass` | `#5E6B4F` | the Hushlands' grey-green |
| `ground.dirt` | `#6E5940` | paths, yards, farm earth |
| `ground.road` | `#857A66` | the road |
| `ground.water` | `#334D61` | deep water |
| `ground.shallows` | `#4D6B78` | water you can wade |
| `ground.floor` | `#78593D` | plank floors |
| `ground.bridge` | `#664A2E` | darker planks over water |
| `build.wall` | `#A8A191` | plaster and stone walls |
| `build.door` | `#614026` | doors, and walls of planks you put up (a door's wood, a wall's full height: `chunk_view.gd`'s `PLANK_WALL`; their own look is the kit's to come) |
| `build.board` | `#9E7A4D` | boards nailed up |
| `build.fence` | `#73614D` | fences and railings |
| `tree.trunk` | `#4D3B2B` | trunks |
| `tree.leaves` | `#3D5436` | crowns, lighter or darker a little per tree |
| `world.litter` | `#9E855C` | a bundle dropped on a floor |
| `light.rain` | `#B3C7E6`, 35% | a streak of rain past the camera |
| `light.shot` | `#D9CCA6` | an arrow's flight |

**Should:** world colours keep a saturation under about 35% and stay in the middle values, so actors and signals stand forward (pillar 2).

### Materials: what things are made of

Each is a colour and a surface (§6), in `data/looks.toml`'s `[material]`. Every part of every look takes one, so the world draws from one small palette.

| Group | Materials (hex, surface) | Used for |
|---|---|---|
| `wood` | `pale` `#93795A`, `oak` `#5E4229`, `dark` `#46301F`, `weathered` `#6E6252` (wood) | furniture, racks, a well's posts. **Must:** the floor's planks (`ground.floor`) sit between `pale` and `oak` in value, so furniture always stands off the floor, lighter or darker |
| `stone` | `pale` `#9E998F`, `grey` `#858789`, `old` `#787873`, `dark` `#6B6661`, `black` `#4D4542` (stone) | the altar and font, wells and headstones, tombs, hearths, a sooted forge |
| `metal` | `iron` `#38383D`, `black` `#2B2B30`, `steel` `#8C9196`, `bronze` `#8C7046` (iron) | stoves, anvils, hoops; an edge; bell bronze, the Hushlands' accent (§15) |
| `cloth` | `linen` `#B8AE9A`, `wool` `#94785F`, `sacking` `#998561`, `habit` `#5C6175` (cloth) | bedding, blankets, sacks, the sisters' grey |
| `straw` | `hay` `#BDA861`, `wheat` `#B89E52`, `twine` `#857043` (straw) | bales, sheaves, their binding |
| `tree` | `trunk` `#4D3B2B`, `birch` `#A39C8E` (bark); `leaves` `#3D5436`, `leaves_dark` `#304430`, `leaves_pale` `#4F6342`, `pine` `#2E4234` (foliage) | trunks and crowns: a crown's lobes shaded below and lit on top, so it reads round without smooth shading |
| `plant` | `yarrow` `#6B805C`, `chamomile` `#738F59`, `feverfew` `#667A4D`, `bramble` `#2B3B29`, `tuft` `#64714F` (foliage); `picked` `#756945`, `reed` `#5B6B47` (straw); `berry` `#47233A`, `reed_head` `#5E4A33` (plain) | wild plants' leaves, grass grown long, reeds; a bramble's blackberries, wine-dark so they show on its leaves |
| `flower` | `white` `#EDEDDE`, `cream` `#F5EDAD`, `yellow` `#F0E06B` (plain) | the flowers on a plant you can pick: bright, as what's worth picking should be |
| `world` | `soot` `#2B2724`, `earth` `#5C4A36` (plain) | a hearth's mouth and a forge's coals; a grave's mound |
| `clay`, `glass`, `wax` | `clay.red` `#8A5A44`, `glass.green` `#4F6352`, `wax.candle` `#D9CFB4` (plain) | jars, bottles and candles on shelves and the altar |

### Actors

Each look is a colour, a height and a stoop. The colour is the second cue; the shape is the first (§5).

| Token | Hex | Height, stoop | Note |
|---|---|---|---|
| `actor.you` | `#BF5233` | 1.55 m, upright | rust: the one warm body in a cold world |
| `actor.shambler` | `#8F9980` | 1.45 m, 14° forward | grey-green: the wall of the dead |
| `actor.ghoul` | `#D1C7B3` | 1.3 m, 28° forward | pale and crouched: run |
| `actor.revenant` | `#733D33` | 1.6 m, 6° | your own colour gone bad, in your gear |
| `actor.skeleton` | `#E0DBC7` | 1.6 m, upright | bone white and upright: bring a hammer |
| `actor.wight` | `#384057` | 1.8 m, leaning back | night-blue, the tallest: the dark's own |
| `actor.deer` | `#8C613D` | 1.2 m, on four legs | brown, head up |
| `actor.wolf` | `#66666B` | 0.95 m, on four legs | grey, head low and forward |
| `actor.scut` | `#EBE6D9` | | a deer's white tail, flashing as it bolts |
| `actor.ash` | `#211F1C` | | a body burnt to ash |
| `actor.blood` | `#8C1A14` | | a blow landing on flesh: dark, so it's never mistaken for `signal.danger` |

Ghouls and skeletons are close in colour on purpose. Their posture tells them apart: low and crouched against tall and upright. A new kind must pass the silhouette test (§5) before it gets a colour.

### Signals: the same meaning everywhere

| Token | Hex | Means | Merged from (2026-11-12) |
|---|---|---|---|
| `signal.calm` | `#F5E699` | one of the dead that isn't after you (its sight on the ground) | |
| `signal.alert` | `#FFA733` | heard something: "?" | `#FFBF40` mark, `#FF9E2E` sight, `#FFBF4D` out of breath |
| `signal.danger` | `#FF3B2B` | sees you or is after you: "!"; its tell; the lines of the dead on you | six reds, `#FF291A` to `#FF594D` |
| `signal.danger_text` | `#FF6A5C` | danger written as text: the held banner (it keeps 4.5:1 contrast) | `#FF594D` |
| `signal.grip` | `#BF0D0D` | a hold on you: a thick dark line | |
| `signal.sound` | `#E6F2FF` | a noise ring, at 80% | |
| `signal.holy` | `#FFF2B3` | turned ("+"), consecrated, hallowed; the abbey's counter; a weapon's holy damage on its card | `#D9E6FF` counter header |
| `signal.rest` | `#99B8FF` | asleep ("z"), restful moodles | |
| `signal.yours` | `#F2EBCC` | your own tell, and the bar of the act under way | `#D9D9BF` act bar |
| `signal.good` | `#BFE699` | can be made now, healed | |
| `signal.gold` | `#FFC74D` | XP, a level, a mastery, a find's worth, your stash | `#FFD98C` level, `#FFCC4D` flash, `#FFD173` stash |
| `signal.lore` | `#BFCCE6` | what a known magic item's properties do: their numbers on its card | |
| `signal.dread` | `#05000A` | the screen's edges darkening with dread | |
| `magic.uncommon` | `#8CBFFF` | an uncommon magic item's name | |
| `magic.rare` | `#FFCC59` | a rare magic item's name, and a rare monster's | |
| `magic.unique` | `#E39A5B` | a unique's name (copper, as Path of Exile's), and what its effect does in the world: the dead sent fleeing; on its card, its effect and its story | |
| `magic.changed` | `#8FA8FF` | a number on a card that its quality or its magic changed (Path of Exile's blue), and what a weapon in hand gives a skill or a stat in the skills panel ("+2") | |
| `magic.cursed` | `#E07A7A` | a curse's numbers on a card; it says "(curse)" in words too, so colour isn't the only cue | |
| `magic.arcane` | `#B3A6FF` | a Wizard's: its mana bar, a Light spell's light, a Silence's ring, a readied spell in the panel and on the bar (its slot's edge), a spell's glyph and cost on the bar | |
| `magic.fog` | `#B8BEC7` at 50% | a Fog spell's cloud | |

### Bars and moodles

| Token | Hex | Used for |
|---|---|---|
| `bar.health` | `#BF2924` | health |
| `bar.endurance` | `#D9AD38` | endurance |
| `bar.faith` | `#D1D6FF` | a Cleric's faith |
| `bar.skill` | `#8CB8E6` | skill progress |
| `bar.capped` | `#F2D973` | a skill waiting at the free cap |
| `moodle.mild` | `#F2D973` | the first level of a bad moodle |
| `moodle.bad` | `#FF9940` | the second |
| `moodle.worst` | `#FF4D40` | the third |
| `moodle.good` | `#99B8FF` | a restful or good moodle (= `signal.rest`) |
| `moodle.ink` | `#0D0F14` | a moodle's glyph on its tile. White fails 3:1 on three of the four tints (1.4:1 on mild); this is 5.8:1 on worst and 9:1 to 13.6:1 on the rest |
| `ui.hurt_text` | `#FFD1C7` | your wounds, listed under the bars |
| `body.whole` | `#5E6573` | a part of the body diagram with no wound (3.3:1 on the panel's own colour). A wounded part takes its worst wound's moodle tint: a scratch `moodle.mild`, a laceration `moodle.bad`, a bite `moodle.worst` |
| `body.dressing` | `#F2EEE6` | the bands round a bandaged part, each on a dark `ui.outline` edge so it shows on any tint |
| `potion.haste` | `#8CE0B0` | a potion of haste's flask, on the bar (since 2026-11-16) |
| `potion.iron` | `#A6B0BF` | a potion of iron skin's: steel |
| `potion.courage` | `#F2A07A` | a potion of courage's: dawn |

A potion's flask takes its bar's colour where it has one: life `bar.health`, mana `magic.arcane`, vigour `bar.endurance` (`Look.POTIONS`, by the core's potion code). Its name and its count are beside it, and its card on hover, so the colour isn't alone.

### The GUI

| Token | Value | Used for |
|---|---|---|
| `ui.panel` | `#140F0B`, 90% | every panel's ground, under its grain and frame (97% in high contrast, §13). A cold `#0D0F14` at 72% until the GUI's dress (2026-11-17) |
| `ui.heading` | `#E3C68C` | headings and banners, in small capitals (§8): gold |
| `ui.button_text` | `#EBDDBE` | a button's word on its bronze |
| `ui.edge` | white, 10% | a panel's 1 px edge (Default) |
| `ui.text` | `#FFFFFF` | text |
| `ui.text_dim` | white, 70% | hints, secondary lines, what can't be done yet |
| `ui.outline` | black, 85% | a 6 px outline on any text drawn straight on the world |
| `ui.track` | black, 50% | the empty part of a bar |
| `ui.veil` | `#050814`, 60% | sleep |
| `ui.dim` | black, 50% | the world behind a choice, death, or a menu |
| `ui.hurt` | `#B30000`, up to 35% | the flash when you're hurt |
| `ui.cell_ground` | white, 6% | an empty cell of the inventory's grids, and a slot of its figure (§11, Grid) |
| `ui.cell_edge` | black, 60% | the lines between a grid's cells |
| `ui.tile_ground` | `#262B36`, 90% | a thing's tile over its cells; a magic one's takes its rarity's colour at 22%, and its edge in it. Since the GUI's dress: its ground is a glow instead (below) |

### The GUI's dress (since 2026-11-17)

The user: "work on styling the ui more ... turn the inventory/etc. into a more artistic rendering (again, similar to Diablo/poe)". Nothing on the GUI is flat colour any more: it's drawn (`client/art.gd`), from SVG drawn by hand in `client/icons/ui/` and a grain and glows made as the game starts. Its own colours are the art's: bronze (`#8E6B3B` to a lit `#CFA96A`), gold (`#C9A466`, a rule's), and garnet (`#B3261C`, the frame's stones).

| Piece | What it is | Where |
|---|---|---|
| Frame | `ui.panel` under a soft warm grain, darker toward its edges, in a bronze band with a gold hairline inside it and a plate set with a garnet at each corner (`frame.svg`, nine pieces, its corners 24 px) | every panel (`Look.panel_style`): the inventory, what's within reach, the book panels, a card, the menu, the settings, the keys, the bar's base |
| Well | a square sunk into the ground, shadowed at its top and left and lit at its foot (`well.svg`); an edge lit 2 px in `signal.good` or `signal.danger` (where what you hold would go), or `magic.arcane` (a spell readied) | the inventory's cells and the figure's slots; the bar's slots; a bar's track |
| Plate | bronze with a gold rim (`button.svg`): lighter under the mouse, darker pressed, dim when it can't be pressed; or a header pointed at both ends (`banner.svg`), made a rarity's colour | every button; a card's title |
| Glow | a thing's rarity's colour welling up behind its icon, at 55%, edged in it at 70%; a plain thing's `ui.heading` at 16%, unedged | a thing's tile in the inventory |
| Fill | a bar's colour, lit along its top 40% and shaded along its foot | the bars |
| Rule | a gold line fading out either side of a garnet in a gold lozenge (`divider.svg`) | between a card's parts, under a panel's title |
| Globe | a liquid in a glass ball, filled as you're full, its surface stirring, lit from the upper left, in a bronze ring (`shaders/globe.gdshader`) | either side of the bar (§10, zone K; §11) |

The inventory's panels (§10, zone H) are on `ui.panel` at high contrast's 95% always, as Path of Exile's are: the news and moodles never show through them.

### Colour rules

- **Must: never colour alone.** Every state shown by colour is also shown by a glyph, a shape, text, or a step in value. The alert states have their marks ("?", "!", "+", "z"). On the ground, the sight tints also step in strength: calm faint, alert medium, danger strong. Moodles carry their name, and magic items say their rarity in words on hover.
- **Must: contrast** (WCAG, as used by game accessibility guides):
  - text 4.5:1 against what's behind it;
  - large text (24 px and up) and parts of the GUI that carry meaning (a bar's fill against its track, a focus ring) 3:1;
  - 7:1 in high-contrast mode.

  White on `ui.panel` over the grass is about 15:1. `signal.danger_text` is about 5:1, and plain `signal.danger` (about 4:1) isn't used for small text.
- **Colour-blind mode** (Default, §13) swaps the three alert states for an Okabe–Ito set that stays distinct across the common colour-vision deficiencies:
  - calm sky blue `#56B4E9`;
  - alert yellow `#F0E442`;
  - danger vermillion `#D55E00`, and danger written as text a lighter vermillion, `#F07F45` (7.1:1 on a panel, where `#D55E00` itself is 5:1).

  The holds stay a heavier line, so they don't depend on hue.

  **Today** (2026-11-12): as here (`Look.set_look`). The marks over heads, what the dead see on the ground, their tells, the lines of the dead on you, the held banner and "out of breath" all take the swap at once.
- **A region shifts the world's colours, never the signals** (§15).

---

## 5. Shapes and models

- **Must: the silhouette test.** Every creature kind can be told from every other by shape alone at zoom 14.
  - Draw them all flat black on white in a row: height, stoop, width, legs and head carry it.
  - Run the test before a new kind gets a colour.
  - Today the kinds part by height (0.95 to 1.8 m), stoop (28° forward to leaning back) and legs.
- **Built in code from primitives**, which is the toy-diorama look:
  - people and the dead are capsules that lean into a swing (the tell), lie down when knocked down, and darken when dead. Climbing a sill or a fence, one is lifted over and set down on the far side, at most 0.55 m up (`main.gd`, `CLIMB_LIFT`);
  - beasts get four legs and a head at their own height (`actor_view.gd`, `_build_beast`);
  - furniture, trees and plants are made of the kit's parts (below), from `data/looks.toml`, and so is every thing you carry (its icon is drawn from its look: §11, Icons);
  - walls, doors and windows are boxes on a tile's edge: a shut window is a pane between the sill and the lintel, an open one the empty frame, and a broken one keeps a 0.1 m rim of glass top and bottom, the shards that cut (`chunk_view.gd`, `SHARDS`).
- **Size bands**, so a scene reads at a glance:
  - knee height (to 0.6 m): plants, crates, sacks;
  - waist (0.6 to 1.1 m): tables, beds, counters, altars;
  - head (1.1 to 2 m): shelves, wardrobes, people, wheat;
  - over your head: walls (2.2 m) and trees (about 3 m).
- **Should: detail comes from parts, not texture.** A head, shoulders and the thing in hand carry it, never painted detail at a scale the camera can't show.

### The kit

Everything made in code is made of the same few parts, in the same few materials, so the world looks made by one hand. A look is a list of parts in `data/looks.toml`; `dusk-data` reads and checks it (`looks.rs`), and `client/looks.gd` bakes each look once into one flat-shaded mesh that everything drawn of it shares.

- **Four shapes:** a box; a cylinder, upright, 8 sides unless it says otherwise, tapering to a cone; a ball, 8 sides and half as many rings; and a prism, a ridge. Every round shape starts half a side round from +x, so an even-sided one turns a flat face to each axis. **Must:** `looks.rs` and `looks.gd` make the same corners (dusk-data's `unit_points`), so a silhouette worked out in Rust is the one on screen.
- **Flat shading, always:** every face its own normal, so a cylinder is eight facets and a crown is a cut gem. That's what makes it all one style.
- **Parts:** at least 4 cm thick, so they read at zoom 14 (a centimetre is about a pixel there). A look's origin is the middle of its base, +y up, its front toward −z, in metres. `mirror` and `repeat` make the four legs of a table one line.
- **Materials, not colours:** a part names a material (§4, "Materials"), never a hex of its own. A new colour is a new material, in this guide and `looks.toml`.
- **Budgets:** a fixture's look under 500 triangles (checked as it loads); an item's under 300, and 2.5 m any way; an actor's under 1,500.
- **Turned to fit** (`chunk_view.gd`): a fixture stands as its look's `placed` says. `wall`: its back to a wall, window or fence of its tile, the far walls first (north, west, east, then south: `DuskWorld.chunk_walls()`), else as `fixed`. `fixed`: its front south, to the camera's side. `loose`: a quarter turn by its place, and up to 10° more. `faces` and `backs` come first: a chair turns its front to the table beside it, a pew to the altar, a counter its back to the sister behind it; one in a row of its own kind faces across its row.
- **A little different from tile to tile**, the same every time: a variant picked by the tile, a size within `grow`, and a tint lighter or darker by up to `vary` (4.5% unless it says).
- **Cut away** round the character like the walls, part by part: a trunk is never cut, a crown is (`cut = false`).
- **The wild:** a tree is one of four by its tile (a broadleaf, a taller one, a pine, a birch), its crown in lobes; each plant you can pick has a shape of its own with its flowers bright on top (yarrow's flat white heads on stems, chamomile's low mound of daisies, feverfew's upright bush, a bramble's dark tangle of canes and berries), so it reads from across a field; wheat stands close and tall, as solid as it is to sight.
- **What's strewn** (`[[scatter]]`): tufts of long grass, stones, pebbles on bare earth, reeds in the shallows, on tiles with no fixture, by a chance and a count for each ground. Only a look: nothing strewn stops you or can be taken. **Must:** it stays quiet, a shade off the ground it's on, knee-high or less, never in flower (what flowers is something you can pick), under 60 triangles, casting no shadow and never cut away.
- **What you carry** (`[[item]]`, since 2026-11-17): every item has a look (`dusk-data` checks each does), at its own size, standing as it lies in its cells (`pack.rs`): its long side up them, its front toward −z. A weapon stands point or head up, a shield and a pelt face you, and what's flat (a poultice) stands up to be seen. Each potion is a flask of its own shape as well as its colour (a round flask for life, a tall vial for mana, a cone for haste, a square bottle for iron skin, a long neck for vigour, a teardrop for courage), so they're told apart without it. Its materials are livelier than the world's, as a thing in your hand should be, but a potion's liquid is its bar's colour and nothing borrows a signal's. An icon is seen close, so a part may be thinner than 4 cm here. **Must:** a look longer one way than the other runs the way its cells do (checked), so its icon fills them.
- **Seen whole:** `.\play.cmd gallery.tscn -- ...` lays out every look on plain ground under the game's camera and light (§14, Picture routes).
- **When models come from outside** (Blender, or a person's own), each one:
  - is glTF 2.0 (`.glb`), in `client/models/<what>/<id>.glb` with a snake_case id;
  - has its origin at the centre of its feet or base, +Y up, and faces −Z (Godot's forward);
  - is at 1 unit = 1 m;
  - stays within about 1,500 triangles for an actor and 500 for furniture or an item;
  - has no image textures: vertex colours, or the surface codes (§6), keep it in the palette;
  - uses flat or angle-smoothed shading (30°), no baked lighting;
  - is checked in the game at zoom 14 and 20, by day and at midnight, before it's kept.

---

## 6. Surfaces

Surfaces are procedural, drawn by shaders in world space so tiles join without seams (`shaders/surfaces.gdshaderinc`: `sf_noise`, `sf_fbm`, `sf_blocks`, `sf_boards`). Every surface is drawn by one function there, `sf_surface`, for the walls' shader and the kit's (`looks.gdshader`) alike; what moves (`local`) draws it in its own space, so a pattern never slides over it. A material says its surface (§4, "Materials"); its code is `codes.gd`'s `Surface`.

| Code | Surface | Used on |
|---|---|---|
| 0 | plain | anything that needs no texture |
| 1 | plaster, over a stone plinth under 0.5 m | house walls |
| 2 | wood (boards) | the woods: furniture, fences, doors |
| 3 | stone (blocks) | the stones: the altar, forge, hearth, well, graves, tombs |
| 4 | foliage | tree crowns, plants |
| 5 | straw | wheat, hay |
| 6 | cloth | the cloths: beds, sacks, the sister |
| 7 | iron | the metals: the stove, the anvil, hoops |
| 8 | bark | trunks |

- **Should: patterns stay quiet**, moving value by no more than about ±12%, so an actor standing on them still reads.
- **To add a surface:**
  1. Write it into `sf_surface`, under the next code.
  2. Give it that code in `dusk-data`'s `Surface` and `codes.rs`'s `surface` group (and write `codes.gd` again).
  3. Give a material that surface in `data/looks.toml`.
  4. Check it at noon and at midnight.
- The ground is one colour per kind (§4), with soft noise, and the kind travels in the texel's alpha (`ground.gdshader`). The borders between natural grounds (grass, dirt, water, shallows) wander up to a quarter of a tile off the tiles' edges, by noise, so a shore or a path isn't a staircase; what's built (a floor, the road, a bridge) keeps to the edges. Each chunk's ground carries a tile of its neighbours' all round (`DuskWorld.chunk_grounds()`), so a border wanders across a chunk's edge without a seam. **Must:** only the look wanders: a tile is what the core says it is.

---

## 7. The senses drawn

These marks are information, so they ignore the dark: they're drawn unshaded and on top, and they read at midnight. Each has exactly one meaning.

| Sense | How it's drawn | Token |
|---|---|---|
| A sound | A thin ring growing to the sound's real reach (the core's), then fading; louder sounds are thicker. One ring per sound, so the ring is exactly how far it carried, as Mark of the Ninja draws its sounds | `signal.sound` |
| A creature's sight | Its seen tiles tinted on the ground, by what it knows of you, at 30 to 45%. Only while you see the creature. Never a full-screen wash | calm, alert, danger |
| What it knows | A glyph over its head: "?" heard you, "!" sees you, "+" turned, "z" asleep. Billboarded, outlined; it pops when it changes | alert, danger, holy, rest |
| A tell (a swing winding up) | The attacker glows and leans; your own tell is pale | `signal.danger`, `signal.yours` |
| The dead on you | A thin line from each one that's engaged; a thick dark line for a hold | `signal.danger` at 55%, `signal.grip` |
| Smell (a wound bleeding) | Future: a faint dark trail behind you while you bleed, as far as the scent carries | `signal.danger`, low |
| Light | The light itself: a warm pool round a torch or a fire, as far as the core says it reaches | torch, fire |
| A body burning, ash, hallowed | Embers that glow and light the ground; a dark smear of ash; a body consecrated lies pale | ember, holy |
| Your own state | Edge darkening for dread; a red flash for a hurt; a gold flash for a level; a veil for sleep | §4 |
| A spell on its spot | Light: a pale light with a glowing mote over the spot, reaching as far as the core says. Silence: a ring at its reach. Fog: a low cloud as wide as it reaches. A spell's words: the tell, on the spot it's cast on | `magic.arcane`, `magic.fog`, `signal.yours` |
| A power called on, a skill used | The body's pose: casting leans back, turning holds the symbol high, praying and consecrating kneel, picking, butchering and tending a fire bend or kneel, a heavy swing rears back before it lunges. A power glows in its colour with motes rising (arcane, holy, good, endurance), and lands as a burst (§12, Effects) | `magic.arcane`, `signal.holy`, `signal.good`, `bar.endurance`, `signal.gold`, ember |

**Should:** a new sense or state reuses a token above. A new colour needs a new row here first.

---

## 8. Type

- **Default typefaces:**
  - **Alegreya Sans** for everything in the GUI;
  - **Alegreya**, the serif from the same family, only for titles: the game's name, screen titles ("Who comes to the abbey?"), region names, and the death screen.

  Both are under the SIL Open Font License, so they ship in `client/fonts/` with their licence. If Alegreya Sans reads poorly at 18 px, swap it for Noto Sans.

  **Since 2026-11-17** (the GUI's dress): headings and banners in **Alegreya SC**, the family's small capitals, in `ui.heading`'s gold; titles in **Cinzel**'s capitals, the Roman letters Diablo's titles are cut in. Both SIL OFL, in `client/fonts/` with their licences (`Look.sized`).

  **Today** (2026-11-12): both in, in `client/fonts/` with their licences. Alegreya Sans Regular is the project's default font, so everything takes it; Alegreya Sans Medium is for headings (`type.heading`) and banners (`type.banner`); the serif, Alegreya, is for titles (`type.title`): the class choice and the death screen. `Look.sized` gives each size its face. Alegreya Sans read well at 18 px, so Noto Sans wasn't needed.
- **Must:** the body text is a plain sans. A decorative face never carries information (Microsoft XAG 101).
- **Sizes** at the base screen. Microsoft's minimum for PC is 18 px at 1080p, measured from the top of the tallest letter to the bottom of the lowest.

| Role | Size | Used for |
|---|---|---|
| `type.small` | 18 | hints, legends, item weights: the smallest text allowed |
| `type.body` | 20 | news, panel rows, moodle names, prompts |
| `type.heading` | 24 | panel titles, the status line |
| `type.banner` | 32 | held, paused, asleep |
| `type.title` | 48 (serif) | the class choice, the death screen, the title screen |

  **Today** (2026-11-12): these sizes are in (`Look.SMALL` to `Look.TITLE`). Before, the GUI used 12 to 20 px on a 1600 × 900 window that didn't scale.
- **Text over the world** (no panel behind it) always has the 6 px `ui.outline`.
- **Sentence case**, never whole lines in capitals; a one- or two-word label may be.

  So a banner reads "Paused", in `type.banner`. (The held banner read "Three of them have hold of you", in `signal.danger_text`, until the second playtest took grabs out.)
- **Left-aligned** in its box, even when the box sits on the right edge. Centred only for one-line banners and titles.
- **Lines of 80 characters at most** (news, tooltips, descriptions).
- **Numbers:** counts as "×2", weights as "0.6 kg", times as "3h 50m", and a percentage only for condition. Figures are lining and tabular (the fonts are imported with OpenType `lnum` and `tnum`): Alegreya Sans's own old-style figures sit low among the letters, and a ticking clock or weight would shift.
- **Words:** plain and short, in the game's voice (`CONCEPT.md`). An act and its key are said together: "[E] Pick the yarrow".

---

## 9. The GUI: what the research says, and our rules

| What the research says | Our rule |
|---|---|
| Show only what the player needs now, and hide the rest. Too much on screen is the first HUD mistake | **Progressive disclosure:** the pack, skills, making and the keys each open on a key. Hints show only what's possible here and now |
| Critical, short-lived information belongs near the eye-line. The brain filters out the corners, and edges suit only what the player has learnt to expect | Nothing stays in the middle. The urgent (held, the act under way, what you can do here) sits **just below or above you**, and the habitual (status, news, moodles, panels) sits **at the edges** |
| Read in peripheral vision: large, bold, high-contrast; values as brightness or size, not as small text or hue; neighbours that look different; motion and flashes catch the eye, but too strong pulls it | Bars over numbers. A low bar pulses in **brightness**, not colour. A new or worsened moodle pops once. Health, endurance and faith get different colours and icons. One flash at a time |
| A study of shooter displays: players did best with health on a HUD, and with ammunition and the way to go drawn in the world | Body and needs are on the HUD. The senses and threats are drawn in the world (§7) |
| Lists are simpler than grids and work with keyboard and mouse, but get long. Grids suit games where packing is the play | **Grids** (Path of Exile's: the first playtest, then the user, 2026-11-16: "an inventory system that looks like PoE 2"). Packing is part of the play now, beside weight: your pockets, your bag and each container are grids, and a thing takes up its size (`pack.rs`). Weight stays on every card and in the pack's title |
| Separate news of what happened from an explanation of why you can't do something; let the player clear or quiet repeated notices | **News** (what happened) goes in the log; a **refusal** (why not) shows once, at the act. Repeats are rate-limited, as dread and blocks already are |
| Fixed-size GUIs break at high resolutions; text should scale to 200% | **Everything scales** with the screen, plus a GUI scale from 75% to 200% (§13) |

---

## 10. The GUI: layout

**Must:**

- The base screen is 1920 × 1080. Godot's `canvas_items` stretch with `expand` aspect scales the GUI with the window, and leaves 3D at the screen's own resolution (Godot, "Multiple resolutions").
- Every panel keeps 24 px from the screen's edges.
- Nothing persistent sits in the middle half of the screen.

**Today** (2026-11-12): as here.

- The player's own game opens maximised, filling the screen. A trial run keeps a 1600 × 900 window, so its pictures stay alike.
- `-` and `=` make the GUI a step smaller or larger (§13).

```
 1920 × 1080: the base. Everything scales with it.
┌──────────────────────────────────────────────────────────────────────────────┐
│ A  the Hushlands · Day 2 · 21:40 · 1× · rain          D  News (6 lines, fade)│
│ B  Level 3 · 120 / 160 XP                                You pick it: ...    │
│    (body diagram: wounds by part)                        A howl: something...│
│                                                                              │
│                                                       E  Moodles             │
│                                                          ▣ Hungry            │
│                   C   Paused                             ▣ Uneasy            │
│                                                          ▣ Bleeding          │
│ J  Skills (K)                                                                │
│    or Making (B):                                                            │
│    one at a time            · · ·  the world  · · ·                          │
│                             (nothing stays here)                             │
│                                      @   you                                 │
│                          F   [E] Pick chamomile                              │
│                              Picking it  ▓▓▓▓▓░░                             │
│                                                                              │
│ G  Within reach                                       H  The inventory (I)   │
│    Stove · alight, 3h 50m                                (open: Path of      │
│    Crate              [Take all]                          Exile 2's grids)   │
│                           I  [F1] every key                                  │
│              (health)  K  [1][2][3][4]  [5][6][7][8]  (mana): the bar        │
└──────────────────────────────────────────────────────────────────────────────┘
```

| Zone | What | Where, how big | Shows | Today |
|---|---|---|---|---|
| **A** Status | region, day and time, speed, the pressure, the Tolling, rain, where you are | top left; one line, `type.heading` | always | as here since 2026-11-13 (it was two lines: a title with the newest feature, and the seed). A trial run adds its seed at the end, for pictures and bugs |
| **B** Body | level and XP in words; out of breath; wounds; the body diagram | top left, under A | always | since 2026-11-17 (the second playtest: "get rid of the upper left health/endurance, since we have the hot bar circles") with no bars: the globes say health and mana, faith or breath, and the bar its XP (zone K). Out of breath and the wounds in words beside the diagram |
| **C** Banner | the one thing you must answer now: paused, asleep | top centre, 18% down; `type.banner` | while it holds | as here; held too, until the second playtest took grabs out (2026-11-17) |
| **D** News | what just happened, newest at the bottom; 6 lines, each lives 6 s and fades over 1 s | top right; 620 wide, text left-aligned | when there's news | as here |
| **E** Moodles | needs and states, worst first | right edge, under D; 44 px tiles with names, the detail on hover | while any is on | as here |
| **F** Here and now | what you can do beside you: E's prompt, and what holding E offers besides; while E is held, all of it, numbered; and the act under way with its bar | centred just under you, about 60% down | when there's a prompt or an act | as here |
| **G** Within reach | the world's things beside you: containers, water, a fire, a counter, plants | bottom left; 420 wide, at most 45% of the height, scrolling | when anything is in reach, and the inventory is shut | as here. While the inventory is open, its left side shows the same as grids, and this hides |
| **H** The inventory | Path of Exile 2's: what you hold and wear round a figure (in hand on the left, the off arm on the right, the head over the body, the bag on your back beside the head, arms and legs at the body's foot), the belt's four loops under it; then your pockets' grid and the bag's side by side, twelve cells across, each thing its icon over its cells; weight and coin in its title. On the left while it's open, under zones A and B: a tab for each container and the ground within reach, each a grid, and a holdout's counter as a vendor (its wares a grid, its services and bounties over them) | top right, 24 px in; 50 px cells, about 650 wide; zones D and E move to its left, under zone A, while it's open | I or Tab; Esc, I or Tab shuts it | since 2026-11-17 (the user, 2026-11-16: "an inventory system that looks like PoE 2"; `client/inventory.gd`). It was a list (Zomboid's), open from the start |
| **I** Keys | every key in its groups ("The keys", below), with the legend of the drawn senses. For a world's first game hour, a line at the bottom centre, above the bar, instead: "[F1] every key" | over the middle of the screen, three columns of groups, while it's open; zone F hides under it | on F1 | as here |
| **J** Book panels | Strength, Fitness, skills and talents (K), making (B), spells (P); later the character and the map | left middle; 460 wide; **one at a time** | on their key | as here |
| **K** The bar | in a frame of its own, between the globes (§11, Globe), with XP along its top and, for a class whose right globe holds mana or faith, breath along its foot (§11, XP and breath on the bar): your powers on the number keys, then the belt's loops and their potions, each its own icon (§11, Bar slot): each slot's glyph, key, and a spell's mana or a loop's count; a click uses it, hovering opens its card; a power dragged onto another swaps them, the order kept for your class | bottom centre, 24 px up; 60 px slots 8 apart, the belt 16 further on | always, alive (not while a class is being chosen) | since 2026-11-16 (the user: "a hot bar for spells, as well as a potion belt"), as here |

**Rules:**

- **Yours on the right, the world's on the left:** the inventory is H, what's within reach is G (or the inventory's left side, while it's open). Moving a thing between them always moves it across the screen.
- **The middle is the world's.** Only world-anchored marks, zone F and banner C may sit near you.
- **Panels don't stack.** Opening a book panel (J) closes the other one, and the keys (I), which would lie over it; opening the keys closes the book panel. A modal screen dims the world (`ui.dim`) and pauses the game.
- **Esc** closes the top panel first, and only then opens the menu.

  **Today** (2026-11-12): Esc closes E's list, lets a readied spell go, then closes the keys, then the book panel. Since the first playtest (2026-11-16: "Escape should be pause/menu"), with none of those open it opens the menu, and the world waits under it (`client/menu.gd`); it no longer saves and quits. Since 2026-11-17 it first puts back what the inventory holds, then shuts the inventory.

### The keys

Fewer keys, in groups, with the left hand on WASD. Why, and what Zomboid, Valheim, Don't Starve and the rest do, is in `research/controls.md`.

| Group | Keys | Where the hand is |
|---|---|---|
| **Move and fight** | WASD walk (and climb); Shift switches between walking and running (running from the start, since the first playtest); C sneak; F swing; Shift+F or Shift-click a heavy swing (Shift held so switches nothing); Space shove; click walk; click one of the dead: go after it and swing once it's in reach, held: swing on till you let go (Diablo's, since the second playtest; Shift-click: heavy swings); right-click held: on guard, Zomboid's stance (you face the cursor, slower, no running; a click swings that way, or shoots the bow in your hand at the spot; a shield on your arm braces), since 2026-11-17; the wheel zooms | the left hand, round WASD; the mouse |
| **Here** | **E** does the likeliest thing beside you; **E held** lists all of it, numbered (a number or a click picks one, E or Esc closes it); T a light (dead, the next survivor); Z sleep | the left hand |
| **Your powers and belt** | 1 to 4: a Cleric's turning and Mending, a Fighter's Second Wind, a Wizard's spells, in that order until you drag them into another on the bar (zone K); 5 to 8: the belt's potions | the left hand, over WASD |
| **Panels** | I or Tab the inventory; B making; K skills; P spells; F1 the keys; F10 the settings | the right hand, in calm moments |
| **The game** | F2, F3, F4 the speeds; F5 save; V low walls; - and = zoom (as the wheel does); Ctrl+- and Ctrl+= the GUI's size; Ctrl+N a new world (asked twice); Esc closes what's open, else opens the menu, which pauses | the F keys; two hands for what can't be undone |

**Rules:**

- **The world's verbs are E's, not keys of their own.** A new thing to do beside you is a new line in E's list (`client/prompts.gd`), never a new key.
- **E's choice is the one its prompt shows.** Both come from `Prompts.first`, over the core's own checks. The door or window you face comes first, then a body, a plant, an altar, a fire, and water when you're thirsty; a door beside you that you don't face comes last, with no prompt, as E did before. What's done on purpose (boarding up, prying off, a ward, putting a fire out) is never E's alone: holding E offers it, and the line under the prompt names it.
- **Every key cap is read from the InputMap** (`Prompts.keyed`): the prompts, the news, the moodles' tips and the keys panel. Remapping (§13) changes them all.
- **One action a key.** A remapped key taken from another action leaves that one without it, so two actions never share a key by accident. The one that once shared R, the next survivor, is now what the light's key does when you're dead.

**Today** (2026-11-12): as here.

### Screens

| Screen | What's on it | Rules |
|---|---|---|
| Title | the name in the serif, a dusk scene behind (the abbey, a slow camera); Carry on, New world, Settings, Quit | a key or a click in reach of everything; carrying on is one click, since the game saves itself |
| Who comes to the abbey? | each class, its line and its kit; a number or a click takes one | dims and pauses the world; today as text in the middle |
| Paused | the world stilled under a dim; "Paused" banner; Space to go on | nothing else changes. **Since 2026-11-16** the menu is the pause: there's no key of its own |
| Menu (Esc) | Resume, Settings, Save and quit | centred; dims. **Today** (2026-11-16): as here (`client/menu.gd`), "Paused" over it, the world waiting; a trial run's last button is Quit |
| Settings | the GUI, text, colour, flashes, keys, sound, the game (§13) | every setting saved; changes show at once. **Today** (2026-11-12): on F10 (`client/settings.gd`), over the world dimmed (`ui.dim`) and paused, on a ground at 95% so nothing shows through: the GUI's size, the four switches, and all 34 keys. Esc closes it |
| Death | "This is how you died": the cause, the days, the deeds, where the body lies; Restart (the next survivor, in this world) and New World | a framed panel (§4), the name of who died as its title, a rule under it; the world behind, dimmed, the body in view; no key shown. **Today** (2026-11-17, the second playtest: "The death menu should have a panel and buttons for Restart / New World. No keybind shown"): as here (`hud.gd`'s `_build_death`, `death_chose`); the keys that did it before still do, unlisted |

---

## 11. The GUI: parts

- **Panel:**
  - Background `ui.panel` with a 1 px `ui.edge` and 4 px corners (Default). **Since 2026-11-17** a Frame (§4, the GUI's dress): the ground under a grain in a bronze frame, 18 px across and 16 down inside it.
  - Padding is 12 px across and 10 px down, with 6 px between rows.
  - Its title, in `type.heading`, says what it holds and what it weighs.
  - A panel never grows past 45% of the screen's height; past that it scrolls.
- **Bar:**
  - A label 110 px wide, then a 240 × 18 bar: the fill's token on a `ui.track` back. **Since 2026-11-17** the fill lit along its top, in a well.
  - Numbers show on hover, or always with "show numbers" on (§13). **Today** (2026-11-12): as here, "80 / 100" over the fill, outlined.
  - Under 25%, the fill pulses in brightness once a second.
- **Button:**
  - Its word, then its key: "Take all [T]". **Since 2026-11-17** on a bronze plate, its word in `ui.button_text`.
  - A click target is at least 32 × 32 px, with 8 px between targets.
  - It never relies on a hover to say what it does.
- **Health bar** (over the dead; since 2026-11-17, `client/bars.gd`): 56 × 7 px, 2.1 m over the feet of each of the dead you see that's hurt, and of every rare you see; `bar.health` lit along its top on a dark track, in a 1 px bronze line (gold for a rare, whose name is over it already). Path of Exile's: a whole one stays bare.
- **Globe** (since 2026-11-17; Diablo's and Path of Exile's): 116 px across, either side of the bar's frame, their feet on its foot. Health on the left, in `bar.health`; on the right mana in `magic.arcane` (a Wizard's), else faith in `bar.faith` (a Cleric's), else breath in `bar.endurance`. Filled as you're full, its surface stirring (still, with less motion); under a quarter it throbs in brightness, as a low bar does. Its name and numbers ("Health 72 / 100") show on hover, or always with numbers on (§13). **Since the second playtest** (2026-11-17: "get rid of the upper left health/endurance, since we have the hot bar circles") they're the only ones: the bars at the top left are gone.
- **XP and breath on the bar** (since 2026-11-17, the second playtest: "I want to see an xp bar for the character level on the main ui"): XP, 8 px high, along the top of the bar's frame, in `signal.gold`, how far into this level, on a track with a faint gold edge so it reads with none in it yet (the level and its numbers on hover); breath, 6 px, in `bar.endurance`, along the frame's foot, only while the right globe holds mana or faith. Kept apart on purpose: the first picture had breath right under the XP, and gold over amber read as one full bar.
- **Key caps:** a key always appears in brackets in the same face as its line, such as "[E] Pick chamomile". Remapping a key changes every place it's shown. The keys panel's own column of keys is the one place without brackets: it's a table of them.
- **Item row:**
  - Name ×count, weight, then the actions on the right.
  - A find carries a small `signal.gold` dot until it's brought home.
  - A magic item's name is in its rarity's colour, with the rarity in words on hover.
  - Hovering a weapon, a bow, a shield, armour, a light or any magic item opens its card (Tooltip, below).
  - A unique's name is in `magic.unique`, and says it's a unique in words too: "fine hand axe (unique, unknown)", then "Redhand (fine hand axe)"; a unique bow, shield or piece of armour the same ("The Wasp (fine wooden buckler)").
  - What you wear, and the shield on your arm, are called as far as you know them, in their magic's colour, as the pack's things are.
  - Unknown, any magic thing says its rarity in words beside its colour: "(uncommon, unknown)", "(rare, unknown)". A Rare known is called by its made-up name, its kind after: "Sorrow Fang (masterwork arming sword)"; an Uncommon by its properties' words, "Keen fine sword".
  - On the card, a property is its numbers, its word and tier to the right: "+80% damage", "Cruel III". A number that rolled in its tier gives the range after it, as Path of Exile 2's does: "+87(64–96)% damage" (since 2026-11-15).
  - Condition shows under 100%, and freshness for food.
  - The rows group by kind (weapons, food and drink, cures, light, materials), and sort by kind, weight or name.
- **Moodle:**
  - A 44 px tile in its severity's colour (`moodle.mild`, `moodle.bad`, `moodle.worst`, or `moodle.good`), with a 1 px `ui.outline` edge, and its glyph of 32 px in `moodle.ink`. The glyph is white in its file and inked as it's drawn: white on the tints fails the 3:1 a part that carries meaning needs (§4).
  - Its name sits beside it in `type.body`, and what it does and what eases it shows on hover, a line each.
  - A new or worsened moodle pops once (scale 1.25 back to 1 over 0.3 s).
  - The column is worst first, and 250 px wide, so the tiles stay in one line however long the names are.

  **Today** (2026-11-12): as here (`hud.gd`; `Look.tile_style`). A pop always plays; reducing motion waits for the settings (§13).
- **Bar slot** (zone K):
  - 60 × 60 px on `ui.panel` with a 1 px `ui.edge` (`ui.text_dim` under the mouse); the readied spell's edge is 2 px of `magic.arcane`.
  - Its glyph in the middle, 36 px: a spell's in `magic.arcane`, the Cleric's powers in `signal.holy`, the Second Wind in `bar.endurance`, a potion's flask in what it does (§4). It's dimmed to 35% while it can't be used now (not mana enough, no component, an empty loop); an empty slot shows only a trace (12%).
  - Its key at the top left, in `type.small` and `ui.text_dim`; at the bottom right, a spell's mana in `magic.arcane` or a loop's count in `ui.text`, both outlined.
  - A click uses it, as its key does. Hovering it (0.4 s) opens its card beside it: a spell's numbers (mana, component, words, range, damage or how long it lasts, then what it does), a potion's own card. The powers' cards open to the left, the belt's to the right, never over the bar.

  **Today** (2026-11-16): as here (`client/hotbar.gd`).
- **Mark over a head:**
  - A glyph 0.45 m above the head, billboarded, in `type.banner` scaled to the zoom, with an outline.
  - It pops when it changes.
- **Prompt (zone F):**
  - A key cap, a verb and a thing: "[E] Butcher the deer", "[E] Pick chamomile".
  - E's prompt first, then, dimmed, what holding E offers besides ("Hold [E]: board it up"). A prompt that can't be done says why in `ui.text_dim`.

  **Today** (2026-11-12): `client/prompts.gd` words the core's own checks, the ones E asks before it acts, so a prompt never promises what E won't do. E's prompt is for:
  - the door or window you face;
  - a carcass (butcher it) or a body (burn it, consecrate it);
  - a wild plant, an altar, a hearth, stove or forge;
  - water, only when you're thirsty.

  A door beside you that you don't face isn't prompted: they're everywhere in a town. E still opens it when nothing else is E's, and then what's easily missed beside you (a body you can't yet burn, say) shows dimmed, without a key. While an act is under way, its bar has zone F to itself.

  **E held** (0.35 s) turns zone F into a list of everything here, numbered "[1] Shut the door", with what can't be done dimmed and its reason ("[2] Board it up: you need a hammer"), and a last line saying how to pick: a number, a click, or Esc. It's worked out afresh as you move, and closes when you pick, press E or Esc, or there's nothing left in it.
- **Tooltip:**
  - After 0.4 s of hovering, in a panel, at most 70 characters a line.
  - It never hides what it describes.

  **Today** (2026-11-12): the moodles' detail is the first. It's our own panel rather than Godot's tooltip, so it takes `ui.panel` and opens beside its row, to the left. Godot's own delay (`gui/timers/tooltip_delay_sec`) is 0.4 s too, for any built-in tooltip later.

  **A thing's card** (2026-11-13; as Path of Exile's and Diablo's since 2026-11-14, the user: "more numerical, except for special effects"): the core writes it (`card.rs`), the inventory asks for it (`inventory.gd`; `bag.gd` for what's within reach while the inventory is shut), the HUD's panel draws it, line by line, over every panel. A weapon's, a bow's, a shield's, armour's or a light's, and any magic thing's, in the pack, worn, in hand or within reach. It opens to the pack's left, and to the right of what's within reach, so it never covers the panel it came from. Its ground is solid (`ui.panel` at high contrast's 95%): a card is dense numbers over the world. It's 400 px wide; its notes wrap. Since 2026-11-17 a thing the core writes no card for (food, a bandage, coins, a bag) has a plain one: its weight, its cells, how fresh or how far burnt, what's in a bag, a find's worth; and every card ends with what a right-click does ("Right-click: Eat") and, at a counter, what it sells for.
  - **Its title,** in `type.heading` and its rarity's colour: the name as far as you know it. A known Rare's made-up name or a unique's own has its kind under it ("Sorrow Fang", "Masterwork arming sword"). Then what it is and its rarity in words, in `ui.text_dim`: "Blade weapon · Rare".
  - **Its numbers,** a line each, the name dim at the left and the value at the right, as it was made and as far as you know its magic, before your skill and Strength. A value its quality or magic changed is in `magic.changed`, the rest white; fire damage in fire's colour, holy damage in `signal.holy`. A weapon's: damage per second first (Diablo's), then damage, fire and holy damage, attacks per second, wind-up (the tell), chance to hit, breath a blow, noise, wear a blow, condition. A bow's: damage per second, damage, fire damage, shots per second, draw, chance to hit, range, breath, noise, arrows broken. A shield's: block, breath, bash, wear. (Since 2026-11-15 both marked where their making or magic changed them.) Armour's: where it covers, the chance it turns each kind of wound, warmth, weight worn, its cost in breath, footsteps and sight, time to put on, wear (since 2026-11-15 marked where its making or magic changed them, as a weapon's are). A light's: how far it lights, how long it burns.
  - **Its class's edge** (since 2026-11-16; Path of Exile's implicit): a weapon's, as a property is, in numbers with its class at the right, between its numbers and how it was made: "+50% damage to one that's down   Blade", "12% chance a blow knocks down   Blunt", "A blow bleeds 4 damage over 8 s   Axe", "30% chance a blow drives it back a tile   Spear".
  - **How it was made** (Diablo II's superior weapons): a fine, masterwork or crude weapon's, bow's, shield's or piece of armour's quality as numbers, its word at the right: "+25% damage   Masterwork", "+10% chance to turn blows   Masterwork", "+5% chance to block   Fine" (a bow has no wear line: it doesn't wear).
  - **Its magic,** each property as numbers in `signal.lore` ("+80% damage", "+2 to Strength", "Adds 3 to 6 fire damage"; each number that rolled with its tier's range after it, "+87(64–96)% damage"), its word and tier dim at the right ("Cruel III"); a curse in `magic.cursed`, "Leaden (curse)". A property that's more than numbers is in words (Hallowed, Blazing). Then a unique's effect, in words, and its story, in `magic.unique`.
  - **Notes,** dim: what you don't know of it yet ("Not known yet: have a holdout read it, or strike 15 blows with it"); how it stands against the weapon in your hand, or the piece you wear where it goes, as numbers in words, not green and red ("Against your cudgel: +50.6 damage per second, +1.00 attacks a second, +15% chance to hit"); what it pays as a find.
  - Parts are set apart by a 1 px rule in `ui.edge`.
- **Icons:**
  - SVG on a 64 × 64 canvas: a solid white silhouette, no text inside, legible at 32 px.
  - Moodles and the bar's powers get a glyph each; each item's icon is drawn from its own look (§5).
  - Icons scale with the text, up to 200%.

  **Today** (2026-11-12): the moodles' 19 glyphs are in `client/icons/moodles/`, drawn here, with nothing downloaded; since 2026-11-16, 25, with a potion's six, and the bar's (`icons/powers/`: Light, Silence, Fog, Fireball, turning the dead, Mending, the Second Wind; `icons/potion.svg`, a flask), inked as they're drawn. Every texture is imported at twice its canvas with mipmaps (`project.godot`'s `[importer_defaults]`), so a glyph stays sharp from 75% to 200%. Their `.import` files are committed, and `play.cmd` imports any icon not yet imported.

  **Items' icons** (since 2026-11-17, `client/icons.gd`): each item's look on a small stage of its own, nothing of the game's in it: orthographic, from the front, 24° above and 28° round to its right, a key light from the upper left and a fill from the right, in the kit's own shader. It's fitted to the item's cells with 8% to spare and drawn once into a texture, 96 px a cell (sharp to 200%), one a frame as they're first asked for, and kept. With no window (`--headless`) none is drawn, and a tile shows the item's name. `.\play.cmd gallery.tscn -- --icons` shows them all (§14).
- **Grid** (the inventory's, since 2026-11-17; Path of Exile 2's):
  - Cells 50 px square on `ui.cell_ground`, a 1 px `ui.cell_edge` between them; the figure's slots the same, 2 cells by as many as their thing takes up, named faintly while empty.
  - A thing's tile covers its cells on `ui.tile_ground`; a magic thing's in its rarity's colour at 22%, its 1 px edge in it, so rarity shows before the name. Its icon fills the tile 3 px in, its shape kept. How many, at the top left in `type.small`, outlined, when there's more than one; a find's `signal.gold` dot at the top right; a ware's price at the bottom right in `signal.gold`.
  - Held, a thing is drawn under the mouse, centred on it, and faint (35%) where it was. Where it would go is marked over the cells: `signal.good` at 35% where the core's own check says it fits, `signal.danger` at 40% where it won't; a slot takes a 2 px edge in the same. A click where it can't go says why in words ("No room for it there."), so the colour isn't alone.
  - What's worn over more than one place shows in its first and faintly in the rest (a jerkin in Body, and faint in Arms).
- **Body diagram:**
  - A front silhouette, 120 px high, its parts tinted by their worst wound: scratch, laceration or bite, plus bleeding and dressed.
  - Hovering a part lists its wounds.
  - It sits in zone B, beside the bars.

  **Today** (2026-11-12): `client/body_view.gd`, on a small panel beside the bars. It's a front view, your right on the left, as a doctor's chart has it, with the core's ten parts: head, torso, each arm, hand, leg and foot, 2 px apart so each tint reads on its own.
  - A wounded part takes its worst wound's moodle tint, and an unhurt one `body.whole`.
  - A bleeding part pulses in brightness once a second, as a low bar does.
  - A bandaged part is wrapped in `body.dressing` bands.

  So bleeding and dressed aren't told by colour alone, and the wounds stay listed in words under the bars. Hovering a part (after 0.4 s) opens its detail to the diagram's right: the part, then each wound on it ("laceration, bleeding"), or "unhurt". A wound landing while you hover updates it at once.

---

## 12. Motion and feedback

| What | How long | How |
|---|---|---|
| A panel opening or closing | 0.12 s | fades in or out |
| A news line | lives 6 s, fades over 1 s | as today |
| A mark or a moodle changing | 0.25 to 0.3 s | pops: bigger, then back |
| Hurt, a level, healed | 0.6 s; a level 1.2 s | a flash at the screen's edge: `ui.hurt`, `signal.gold`, `signal.good` |
| A low bar | 1 s a beat | brightness |
| Dread when afraid | slower than 3 Hz | the edge vignette breathes |

- **Must: every act answers within a tenth of a second.** Its tell, its bar, or its refusal shows at once.
- **Must:** nothing flickers faster than 3 Hz, and any flash can be turned off (§13). **Today** (2026-11-12): "Fewer flashes, less motion" turns off the hurt and level flashes, dread's breathing (it still darkens), and the pops of moodles and of marks over heads. The bleeding part's slow pulse on the body diagram stays: it's how bleeding is told without colour.
- **Should:** one flash at a time; a newer one replaces the last.

### Effects

A power or a spell as it happens: glowing motes, rings and a flash of light, brief, in the colour of what it means. Magic is the only thing that glows by itself (§1), so a glow in the world always says "a power".

- **One colour, one meaning:** arcane for a Wizard's (the words, a Light, a Silence), holy for a Cleric's (turning, prayer, consecration, a ward), good for Mending, endurance for the Second Wind (the bar it fills), gold for a skill or a level rising, fire and ember for a fireball. Smoke is ash's colour.
- **Short:** a burst lives at most 1.8 s, a wave 0.9 s; a power's glow and its motes last while the act does.
- **Where you are:** only your own powers make effects, so nothing is drawn where you can't see.
- **Fewer flashes, less motion** (§13): half as many motes, and no flash of light.

**Today** (2026-11-13): `client/effects.gd`, from the core's events, and each actor's pose and glow (`actor_view.gd`'s `POSES`).
- **Spells:** a Light's pale motes rise from its spot; a Silence's ring draws in to it; a Fog puffs out; a Fireball flies from the hand as a trail of fire, then flashes, with flames, embers falling, smoke rising and an ember ring out to its reach. The words broken: the motes fall away. While the words are said, the Wizard glows arcane, leaning back, motes rising round it.
- **The Cleric's:** turning sends a holy wave out to as far as it reaches (`DuskWorld.turn_reach()`, its talents in it); prayer and consecration lift holy motes from the body, a ward puts them on the door. Mending lifts motes in good's green.
- **The Fighter's:** the Second Wind blows a breath out round you.
- **Skills and levels:** gold motes round your feet as a skill rises a level or is mastered; a level of your own, more, and a gold ring.
- **Damage numbers** (since 2026-11-17; the first playtest: "I'd also like to see damage numbers and health bars during combat"; `client/numbers.gd`): each blow that lands, on you or on one you see, its harm in Alegreya SC's bold rising 0.8 m from the chest and fading over its last third (0.9 s in all): white on the dead, `signal.danger_text` on you; whole from ten up, a tenth under. With less motion it fades where it is. Drawn over everything, at the marks' size.
- **Aiming a readied spell** (`client/aim.gd`): a faint arcane ring round you as far as you can cast it, and at the tile under the mouse a ring as far as it would reach there: arcane where it can be cast, or where a click would take you closer first (too far, unseen: since 2026-11-16, as Diablo's), grey where it can't (no mana, no component), danger's red where a fireball would burn you too (`DuskWorld.spell_catches()`). Zone F says the same in words, so the colours aren't alone: "[Click] Cast the Fireball here: it would burn you too", "[Click] Go closer and cast the Fog here", "Fireball: not mana enough", and "[Right-click] let it go".
- **The lasting spells, alive** (`client/zones.gd`): a Light's motes drift round it; a Silence's edge sheds arcane dust; a Fog is a faint cloud full of drifting haze. Each fades over its last 3 s: the light dims, the ring and the cloud thin, the motes stop coming.
- **Smoke, fog, dust and blood are lit as the world is** (only light-points glow by themselves): dark at night unless a light reaches them, so a Fog with a Light in it glows arcane.
- **The fight's** (`main.gd`'s `_fight_effect`): a blow landing flicks blood (`actor.blood`), more for a wound, and bone chips off a skeleton (the bloodless); your shield taking a blow throws sparks toward the one who struck, armour turning one a few (`signal.yours`, the colour of your own); a heavy blow felling one throws up dust and a ring of it round its feet (a unique's that topples, too); a unique's blow sending one of the dead fleeing throws a ring off it, and motes, in `magic.unique`; a unique's thorns biting whatever struck you (or whatever you blocked) throw sharp motes off it in the same colour, out fast and gone; a sneak attack bursts sharply where it lands; bone turned or shattered chips. The dead's: grave dust as one gets up, the dark off your old body as it rises a revenant, a curl of smoke as a wight eats your light. A door or the boards on it giving way: splinters, if you see the spot.
- **Motes** are soft dots made in code (a radial gradient), drawn additively as light or blended as smoke (`Effects.mote`), CPU particles, so a run with no window draws them too.
- **Checked** by `check.cmd`: every effect made once with no window (`client/tests/check_effects.gd`), and none left after three seconds.

---

## 13. Settings for everyone

These come from the Game Accessibility Guidelines' basic set and Microsoft's Xbox Accessibility Guidelines. Every setting is saved.

- **GUI scale:** 75% to 200% (`content_scale_factor`), and **text size** up to 200% on its own.

  **Today** (2026-11-12): Ctrl+- and Ctrl+= step the GUI's size from 75% to 200% (since 2026-11-13; `-` and `=` alone zoom, as Zomboid's do), and so do the settings screen's buttons. Text size on its own waits: every size is set as a label is made, so it needs them all made again.
- **Colour-blind mode:** off, or the Okabe–Ito set for the alert states (§4).
- **High contrast:** panels at 95%, and every text outlined.
- **Reduce flashes and motion:** no screen flashes and no vignette breathing; marks and moodles change without popping.
- **Show numbers** on the bars.
- **Keys:** every key can be remapped, and the GUI says the remapped key everywhere.

  **Today** (2026-11-12): all five, on the settings screen (F10, `client/settings.gd`), each showing at once and kept in `user://settings.cfg` by the player's own game (a trial run keeps nothing; `--look=` turns them on for pictures).
  - High contrast's panels change in place (`Look.restyle`). Its outline on every text is a theme on the window, so a text built later has it too, and a text's own outline stays.
  - A key is remapped by clicking it and pressing the new one: Esc keeps the old one. A modifier alone (Shift, Alt) is bound when it's let go, and Ctrl held with a key goes with it ("Ctrl+S"). The new key replaces an action's first, so walking keeps its arrows. "Back to the defaults" puts every key back. Only the changes are kept, so a later default reaches anyone who didn't change that key.
- **Game speed:** already there (pause, 1×, 2×, 3×).
- **Sound:** separate volumes for effects, the world and music, once there's sound. Nothing is told by sound alone: every sound is also a ring (§7) and, when it matters, a line of news.

---

## 14. Working on the look: the AI and you

### One place for every value

- **Tokens:** `client/look.gd` holds every token in §4 and §8 as named constants, and the GUI's parts as functions:
  - `panel_style()`, a panel's ground;
  - `text()`, a label's size and colour, outlined when it's drawn on the world;
  - `bar()`, a bar's fill on its track;
  - `tile_style()`, a moodle's tile in its tint.

  Each script that draws preloads it (`const Look = preload("res://look.gd")`), as the client's scripts are. **Must:** code uses a token, never a new inline colour. A new colour is a new token in this guide and in `look.gd`.
- **Looks of things:** `data/looks.toml` holds the world's materials (§4) and every fixture's look, as parts of the kit (§5): furniture, trees and plants, each turned to fit its tile, with variants; what's strewn on the ground; and since 2026-11-17 every item's, which its icon is drawn from. A person can tune them without code, and a mod can add its own. *(Since slice 8, 2026-11-15; the bodies of people and the dead are still `actor_view.gd`'s `LOOKS`, for slice 11.)* A table, for one:

```toml
[[fixture]]
id = "table"
placed = "fixed"            # "wall", "fixed" or "loose" (§5, The kit)
parts = [
  { shape = "box", size = [0.98, 0.06, 0.98], at = [0, 0.73, 0], material = "wood.oak" },
  { shape = "box", size = [0.08, 0.7, 0.08], at = [0.4, 0.35, 0.4], material = "wood.dark", mirror = "xz" },
]
```

  The file's head says what each field does. `dusk-data` checks every look as the data loads (`cargo run -p dusk-cli -- check`): every fixture has one, every material it names exists, no shape is too thin, too big or past its triangles.

### The loop

1. **Say what you want.** A plain sentence is enough: "the night is too blue", "I can't tell a ghoul from a skeleton", "the pack is too big". The AI turns it into tokens and rules, and says which.
2. **The AI changes this guide** if a rule or token changes, **then the code**.
3. **The AI shows it:** one picture at the route that shows the change (below), by day and at night if light is involved.
4. **You look**, and say yes, or what's wrong.
5. **The AI commits** the guide and the code together.

### Ways a person can help

| You do | Where | Then the AI |
|---|---|---|
| Change a number or a colour | `client/look.gd` or `data/looks.toml`, then `.\play.cmd` | keeps it, and updates this guide |
| Collect reference pictures (screens of games, paintings, photos of the look you want) | `art/reference/`, with one line each in its `README.md` saying what to take from it | works from them, and names what it took |
| Draw an icon | SVG, 64 × 64, a white silhouette (§11), in `client/icons/<what>/<id>.svg` | wires it in |
| Make a model | `.glb` to §5's rules, in `client/models/<what>/<id>.glb` | wires it in, and pictures it at zoom 14 and 20, day and night |
| Pick a font | an OFL font in `client/fonts/`, with its licence | swaps it in, and checks it at 18 px |
| Review a picture | say yes, or what's wrong in a few words | changes it, or files it in §16 |

The [Game UI Database](https://www.gameuidatabase.com/) is a free library of thousands of game screens, searchable by screen, layout and colour. It's the place to find "a pack screen like this one" when words fall short.

### Picture routes

These are standard pictures for checking the look. Each takes no special data. A picture opens a window for a few seconds, so take one per change, not many.

| Shows | Command (`.\play.cmd -- ...`) |
|---|---|
| Day, open land, plants by the water | `--seed=1 --hour=10 --speed=3 --walk=57,119 --do=12:pick --shot=PNG --shot-after=14` |
| Noon, the dead turned, faith | `--seed=1 --class=cleric --hour=12 --speed=5 --walk=122,117 --do=11:turn --shot=PNG --shot-after=11.7` |
| Dusk, rain, a wolf pack | `--seed=1 --hour=18 --speed=2 --walk=27,135 --shot=PNG --shot-after=20` |
| Midnight in town, dread | `--seed=1 --hour=22 --speed=5 --walk=118,118 --shot=PNG --shot-after=14` |
| The class choice: the title's serif, the GUI's sans | `--seed=7 --hour=12 --choose --shot=PNG --shot-after=3` |
| A prompt under you, by the abbey's altar | `--seed=7 --hour=12 --walk=74,114 --shot=PNG --shot-after=10` |
| The keys opened over the spells panel: the panel closes | `--seed=7 --hour=12 --class=wizard --do=1:spells,2:keys --shot=PNG --shot-after=3` |
| The body diagram in a fight: wounds, a bandage, a part's detail (each run's fight goes its own way) | `--seed=7 --hour=12 --speed=4 --walk=118,84 --do=12:bandage,14:body --shot=PNG --shot-after=14.5` |
| The settings screen, two switches on | `--seed=7 --hour=12 --look=colour_blind,show_numbers --do=1:settings --shot=PNG --shot-after=2.5` |
| Every setting on, in a fight at the near farm | `--seed=7 --hour=12 --speed=4 --walk=118,84 --look=colour_blind,high_contrast,reduce_motion,show_numbers --shot=PNG --shot-after=14` |
| A wall of planks put up by the pond, a door's wood the wall's full height, beside the plastered walls | `--seed=1 --hour=12 --walk=57,119 --give=hammer,plank_wall --do=12:putup --zoom=14 --shot=PNG --shot-after=24` |
| A Wizard's spells as Arcana makes them: the panel with a Sorcerous sword in hand (+3), each spell's time or harm as yours ("lasts 39 s", "28.7 to 40.2 damage") | `--seed=7 --hour=12 --class=wizard --give=sword:masterwork:keen+swift+sorcerous --do=1:wield,2:spells --shot=PNG --shot-after=3` |
| A Wizard's spells at night: the panel, a Light, a Silence | `--seed=7 --hour=22 --class=wizard --do=1:cast_light,4:cast_fireball,7:cast_silence,9:spells --shot=PNG --shot-after=10` |
| A fireball flying (its trail, the ember ring where it lands; the spot three tiles east is under trees) | `--seed=7 --hour=22 --class=wizard --do=1:cast_fireball --shot=PNG --shot-after=3.75` |
| A Wizard's words: the arcane glow and its motes | `--seed=7 --hour=22 --class=wizard --do=1:cast_fireball --shot=PNG --shot-after=2.6` |
| A Cleric kneeling in prayer at the abbey's altar, glowing holy | `--seed=7 --hour=22 --class=cleric --walk=74,114 --do=9:use --shot=PNG --shot-after=12` |
| A weapon's card, as Path of Exile's: a masterwork Rare sword in the pack, its numbers marked where changed, its fire damage, its properties as numbers with their tiers, set against the cudgel in hand | `--seed=7 --hour=12 --give=sword:masterwork:cruel+darting+mighty+searing,hand_axe:crude --do=1:card --shot=PNG --shot-after=2.5` |
| What a weapon in hand gives: the skills panel's "+3" beside Arcana (a Sorcerous sword taken in hand), and the staff's card on its solid ground, set against the sword | `--seed=7 --hour=12 --class=wizard --give=sword:masterwork:keen+swift+sorcerous --do=1:wield,2:skills,2.5:card --shot=PNG --shot-after=3.5` |
| A weapon's rolled numbers: a masterwork Rare sword, Cruel, Burning, Darting and Unerring, each number with its tier's range after it | `--seed=7 --hour=12 --give=sword:masterwork:cruel+darting+unerring+burning --do=1:card --shot=PNG --shot-after=2.5` |
| A bow's card: a masterwork Rare hunting bow, its made-up name, its harm, range and arrows changed and marked, its fire in fire's colour, its properties as numbers | `--seed=7 --hour=12 --give=hunting_bow:masterwork:flaming+farflung+hunters+frugal --do=1:card --shot=PNG --shot-after=2.5` |
| A piece of armour's card: a masterwork Rare iron helm, its made-up name, its turning changed and marked, its properties as numbers, set against the cap you wear | `--seed=7 --hour=12 --give=iron_helm:masterwork:warded+fleeced+powerful+fleet --do=1:card --shot=PNG --shot-after=2.5` |
| A unique's card: The Last Vigil in the pack, its name in copper, its effect and its story | `--seed=7 --hour=12 --give=last_vigil,hand_axe:crude --do=1:card --shot=PNG --shot-after=2.5` |
| The inventory (Path of Exile 2's) in the near farm's kitchen: the figure and what's on it (a jerkin in Body, faint in Arms), the belt, the pockets and a traveller's pack side by side, what was taken from the stove on its cell, the stove and the cupboard as tabs on the left, and a masterwork Rare sword's card | `--seed=7 --hour=12 --speed=4 --walk=118,84 --give=travellers_pack,sword:masterwork:keen+swift+sorcerous --do=11:wear,12.5:inventory,13:takein,13.5:card --shot=PNG --shot-after=14.5` |
| The abbey's counter as a vendor: what it sells today as a grid of icons with their prices, what it's short of, the bounties posted | `--seed=7 --hour=12 --walk=80,120 --give=sword:masterwork:keen+swift+sorcerous,bread,wolf_pelt --do=4:inventory --shot=PNG --shot-after=5.5` |
| A fireball aimed one tile off: the range ring, the burst's ring in danger's red, and zone F's words | `--seed=7 --hour=22 --class=wizard --aim=1,1 --do=1:power4 --shot=PNG --shot-after=2` |
| The lasting spells at night, alive: a Light inside a Fog, a Silence round them | `--seed=7 --hour=22 --class=wizard --aim=0,4 --do=1:cast_light,3.5:cast_silence,6.5:cast_fog --shot=PNG --shot-after=10` |

The moodles in every tint, with a detail open, need a copy of `data/` whose needs climb fast and then hold (`CLAUDE.md`, something timed). In the copy's `rules.toml`, set:
- `[hunger]`: `rise = 600.0`, `levels = [5, 10, 95]`;
- `[thirst]`: `rise = 600.0`, `levels = [2, 4, 6]`;
- `[fatigue]`: `rise = 600.0`, `levels = [5, 70, 90]`.

Then run `--seed=7 --hour=12 --data=COPY --do=1:sneak,5:moodle --shot=PNG --shot-after=6`. It shows dying of thirst, hungry, drowsy and sneaking, and the worst one's detail.

**The looks gallery** (`client/gallery.tscn`): every look in `data/looks.toml` on plain ground, under the game's camera, sun and sky, each named, with the character beside each row for scale. No world is made, so it's quick, and it's the place to check a look before walking the world to find it. It's a scene of its own, so it's named before the `--`:

| Shows | Command (`.\play.cmd gallery.tscn -- ...`) |
|---|---|
| Every look at once | `--zoom=24 --cols=7 --shot=PNG --shot-after=2` |
| A house's furniture, close | `--zoom=8 --cols=3 --only=bed,wardrobe,cupboard,table,chair,stove,crate,barrel,shelf --shot=PNG --shot-after=2` |
| The abbey's and the shops' | `--zoom=9.5 --cols=4 --only=pew,altar,counter,sister,forge,anvil,rack,till,shop_shelf,bar,cask,hearth --shot=PNG --shot-after=2` |
| The farm, the graveyard and the wild | `--zoom=10 --cols=4 --only=wheat,well,sack,hay,tools,grave,tomb,font,tree,yarrow,chamomile,feverfew,bramble,picked_plant --shot=PNG --shot-after=2` |
| The wild close, and what's strewn: the four trees, the plants, wheat, tufts, stones, pebbles and reeds | `--zoom=9 --cols=5 --only=tree,yarrow,chamomile,feverfew,bramble,picked_plant,wheat,tuft,stone,pebbles,reeds --shot=PNG --shot-after=2` |
| Every item's icon, as the inventory draws it (§11, Icons), at its cells, the shortest first | `--icons --shot=PNG --shot-after=4` (with `--only=` item ids, some) |
| Any of them at midnight, by a torch | add `--daylight=0 --torch` |

A copy of `data/` with no dead (its region's `[dead]` `per_chunk` and `per_house`, its `[beasts]` `per_chunk` and its `[rares]` `count` all 0, and each prefab's `dead = [0, 0]`) lets the game walk anywhere for a picture of the looks where they stand: `--seed=7 --hour=12 --speed=4 --zoom=12 --data=COPY --walk=118,84 --shot=PNG --shot-after=16` is the near farm's farmhouse and yard. A walk much over 40 tiles finds no way (the path's budget), so it starts near the abbey.

### Done means

- [ ] It uses tokens: no new inline colours or sizes.
- [ ] It reads at zoom 14 and 20, and by day and at midnight if light touches it.
- [ ] A new creature passes the silhouette test.
- [ ] No meaning is carried by colour alone.
- [ ] Text is at least 18 px at the base and meets its contrast (§4).
- [ ] Nothing new stays in the middle of the screen.
- [ ] Every act still answers within a tenth of a second.
- [ ] This guide says what the code now does.

---

## 15. Regions to come

Each region shifts the world's colours (ground, foliage, sky and night tint) and adds one accent of its own. **Must:** the signals (§4) never change, and an accent never borrows `signal.danger` or `signal.holy`. All of these are Defaults.

| Region | The world | Its accent |
|---|---|---|
| Hushlands | grey-green grass, slate and plaster, dusk-blue nights | bell bronze |
| Warren Hills | ochre and red-brown earth, scrub | goblin-torch orange |
| Webwood | cold teal-green, pale silk | sickly violet |
| Wolfmark | pine, snow, cold blue | amber eyes |
| Cinder Reach | ash grey, charcoal | ember |
| Weeping Moor | fog teal, peat brown | ghost-light pale cyan |

---

## 16. Getting there

These are the visual slices, in order. Each one ends with a picture or two and this guide brought up to date. All are Defaults.

1. *(Done 2026-11-12.)* **Tokens, and a GUI that scales:** `client/look.gd` took every inline colour; the 1920 × 1080 base with `canvas_items` and `expand`; the GUI's size on `-` and `=`.
2. *(Done 2026-11-12.)* **Type:** the fonts in: Alegreya Sans (Regular, and Medium for headings and banners) and the serif Alegreya for titles, with lining, tabular figures. *(The sizes of §8, and the held banner in sentence case, came first.)*
3. *(Done 2026-11-12.)* **Moodles as tiles:** glyphs, severity tints, a pop on change, the detail on hover.
4. *(Done 2026-11-12.)* **Here and now:** prompts under you (zone F). *(The act's bar under you, and the keys behind F1, came first.)*
5. *(Done 2026-11-12.)* **Book panels:** Esc closes panels first, and the keys (F1) don't open over a book panel. *(Making on the left, one book panel at a time, came first.)*
6. *(Done 2026-11-12.)* **The body diagram** in zone B.
7. *(Done 2026-11-12.)* **Settings:** colour-blind mode, reduce flashes, high contrast, show numbers, and key remapping; since 2026-11-16, the sound rings and what the dead see on the ground, both off unless asked for. *(The keys' layout came first, 2026-11-12: §10, "The keys".)*
8. *(Done 2026-11-15.)* **Looks as data:** `data/looks.toml`, the world's materials and every fixture's look made of the kit's parts (§5); the looks gallery (§14). Then the terrain in it, the same day: four trees, each plant its own shape, wheat you can't see through, what's strewn on the ground, and borders between grounds that wander (§6). *(The bodies' looks wait for slice 11.)*
9. **Screens:** title, the class choice as a screen, and death as a story.
10. *(Done 2026-11-17.)* **Icons:** each item's icon drawn from its own look (§5, What you carry; §11, Icons), with no glyphs by kind on the way; and the inventory they're drawn in, as Path of Exile 2's (§10, zone H; §11, Grid). *(Items in hand and lying on the floor wait for slice 11 and loot on the ground, `BACKLOG.md` #6.)*
11. **Shapes:** heads, shoulders and the thing in hand for people and the dead, still primitives, each kind passing the silhouette test.
12. **Regions:** a palette shift with each new region.

---

## 17. Sources

The research behind §9, §10 and §13. Where a source is a blog or a fan page, it's a lead; the rules above lean on the primary sources: Microsoft's XAG, the Game Accessibility Guidelines, Godot's documentation and WCAG.

- **Microsoft, [Xbox Accessibility Guideline 101: Text display](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/101):**
  - minimum text of 18 px at 1080p on PC (26 px on console), scalable to 200%;
  - a sans-serif option;
  - lines of 80 characters at most;
  - sentence case and left alignment.
- **Microsoft, [Xbox Accessibility Guideline 102: Contrast](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/102).**
- **[Game Accessibility Guidelines, basic](https://gameaccessibilityguidelines.com/basic/):**
  - no information by colour alone;
  - high contrast;
  - readable default text;
  - remappable keys;
  - game speed;
  - no flicker;
  - settings saved.
- **[Game Accessibility Guidelines: avoid placing essential temporary information outside the player's eye-line](https://gameaccessibilityguidelines.com/avoid-placing-essential-temporary-information-outside-the-players-eye-line/).**
- **Unity, [Accessibility basics](https://unity.com/blog/video-game-accessibility-basics):** contrast of 4.5:1, 3:1 and 7:1.
- **Game Developer, [Perceiving without looking: designing HUDs for peripheral vision](https://gamedeveloper.com/design/perceiving-without-looking-designing-huds-for-peripheral-vision):**
  - large, bold, high-contrast elements;
  - brightness over hue;
  - neighbours that differ;
  - calibrated motion;
  - bars over strings of numbers.
- **Game Developer:**
  - [Key UX insights from Dreadnought's combat HUD](https://gamedeveloper.com/design/key-user-experience-insights-gained-during-the-creation-of-dreadnought-s-combat-hud): the corners get filtered out;
  - [My personal crusade against mini-maps and other corner-based HUD elements](https://gamedeveloper.com/design/my-personal-crusade-against-mini-maps-and-other-corner-based-hud-elements-in-immersive-games-).
- **Game UI and HUD design guides:**
  - [UX Collective, how not to overwhelm players](https://uxdesign.cc/how-to-not-overwhelm-players-c0e1d14c6b4c);
  - [UX Collective, 7 beginner mistakes with your HUD](https://bootcamp.uxdesign.cc/7-obvious-beginner-mistakes-with-your-games-hud-from-a-ui-ux-art-director-d852e255184a);
  - [Generalist Programmer, game UI/UX design](https://generalistprogrammer.com/game-ui-ux-design);
  - [Game Wisdom, focal points in UI design](https://game-wisdom.com/critical/focal-points-in-ui-design).
- **Peacocke, Teather, Carette, MacKenzie and McArthur, [An Empirical Comparison of First-person Shooter Information Displays: HUDs, Diegetic Displays, and Spatial Representations](https://www.yorku.ca/mack/ec2018.html)** (*Entertainment Computing* 26, 2018):
  - health worked best on a HUD;
  - ammunition worked best in the world;
  - a path drawn in the world was the best guide, with a HUD mini-map close behind.
- **Inventories:**
  - Tim Cain on grid against list inventories (summarised by [Stray Spark](https://www.strayspark.studio/blog/inventory-crafting-system-players-enjoy));
  - [gamedev.net discussion](https://gamedev.net/forums/topic/669150-inventory-management-grid-vs-list/5234550/).
- **Notifications:**
  - [RimWorld's user interface](https://rimworldwiki.com/wiki/User_interface): alerts top right;
  - [Ludeon forums on feedback messages](https://ludeon.com/forums/index.php?msg=457059): news versus why-not;
  - [Block Unwanted Minutiae](https://catalogue.smods.ru/archives/93900): notice spam.
- **Project Zomboid:**
  - [pzfans on Build 42's moodles](https://pzfans.com/ProjectZomboidB42Moodles/): four levels, and larger icons;
  - [pzfans on Build 42's lighting](https://pzfans.com/ProjectZomboidB42Lighting/): dark nights, interiors with few windows almost black, lamps that matter;
  - [Ludo.guide's UI overview](https://www.ludo.guide/guide/project-zomboid/getting-started/understanding-the-user-interface-ui);
  - [Steam, "Better inventory mod?"](https://steamcommunity.com/app/108600/discussions/0/3767857092599707677): players asking for faster transfers;
  - [Modern Status (B42)](https://catalogue.smods.ru/archives/342115): one of the mods that replace or hide parts of the HUD.

  The Zomboid wiki refused automated reading, so the description of Zomboid's moodles and inventory in §1 is from knowledge of the game.
- **Darkwood:** Epic Games Store, [Darkwood is a horror masterpiece viewed from an atypical perspective](https://store.epicgames.com/news/darkwood-is-a-horror-masterpiece-viewed-from-an-atypical-perspective). Sight as a cone; still things visible outside it; the unseen as the horror.
- **Mark of the Ninja:** Designing Sound, [The dynamics of Mark of the Ninja](https://designingsound.org/2013/06/the-dynamics-of-mark-of-the-ninja). The guards are louder than the ninja; sound drawn as rings.
- **Readability and palettes:**
  - [Bugnet, readable game art](https://bugnet.io/blog/readable-game-art-why-clarity-beats-detail): value and silhouette over detail;
  - [Wayline, the art of limited colour palettes](https://www.wayline.io/blog/art-of-limited-color-palettes-game-design);
  - [character readability in Team Fortress 2 and Overwatch](https://medium.com/@xavierck/character-readability-in-team-fortress-2-and-overwatch-68c41d454465).
- **Colour:** the Okabe–Ito palette, as given by [NYU's Siegal lab](https://siegal.bio.nyu.edu/color-palette/).
- **Godot:**
  - [Multiple resolutions](https://docs.godotengine.org/en/stable/tutorials/rendering/multiple_resolutions.html): `canvas_items` with `expand`, a 1920 × 1080 base, stretch scale for the player;
  - [GUI skinning](https://godot.readthedocs.io/en/4.2/tutorials/ui/gui_skinning.html): one Theme for every control.
- **[Game UI Database](https://www.gameuidatabase.com/):** a reference library of game interfaces.
