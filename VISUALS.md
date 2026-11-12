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
  - fire `#FF8F3D`;
  - embers and a burning body `#FF6B1A`, emissive.
- **Cold light** belongs to the night and the dead: moonlight, and the wight's dark.
- **Rain** darkens the sky and sends streaks past the camera; lights outdoors gutter.
- **Dread** darkens the screen's edges violet-black (`shaders/dread.gdshader`), pulsing when you're afraid. The pulse must stay slower than 3 Hz, and it stops when flashes are reduced (§13).

---

## 4. Colour

Colours are named tokens, and code uses the names (§14). In code each token is a constant in `client/look.gd`, named by its last word in capitals: `signal.calm` is `Look.CALM`, `moodle.mild` is `Look.MILD`, `ui.panel` is `Look.PANEL`. The ground is `Look.GROUND[code]`, and `moodle.good` is `Look.REST`.

**Today** (2026-11-12): every colour the client draws with is a token in `look.gd`, merged as the tables below say. The one exception is the tables of furniture and plants (`chunk_view.gd`'s `FURNITURE` and `PLANTS`), which move to data with the looks of things (§14, §16).

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
| `build.door` | `#614026` | doors |
| `build.board` | `#9E7A4D` | boards nailed up |
| `build.fence` | `#73614D` | fences and railings |
| `tree.trunk` | `#4D3B2B` | trunks |
| `tree.leaves` | `#3D5436` | crowns, lightened a little per tree |
| `world.litter` | `#9E855C` | a bundle dropped on a floor |
| `light.rain` | `#B3C7E6`, 35% | a streak of rain past the camera |
| `light.shot` | `#D9CCA6` | an arrow's flight |

**Should:** world colours keep a saturation under about 35% and stay in the middle values, so actors and signals stand forward (pillar 2).

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
| `signal.holy` | `#FFF2B3` | turned ("+"), consecrated, hallowed; the abbey's counter | `#D9E6FF` counter header |
| `signal.rest` | `#99B8FF` | asleep ("z"), restful moodles | |
| `signal.yours` | `#F2EBCC` | your own tell, and the bar of the act under way | `#D9D9BF` act bar |
| `signal.good` | `#BFE699` | can be made now, healed | |
| `signal.gold` | `#FFC74D` | XP, a level, a mastery, a find's worth, your stash | `#FFD98C` level, `#FFCC4D` flash, `#FFD173` stash |
| `signal.lore` | `#BFCCE6` | what a known magic item's properties do | |
| `signal.dread` | `#05000A` | the screen's edges darkening with dread | |
| `magic.uncommon` | `#8CBFFF` | an uncommon magic item's name | |
| `magic.rare` | `#FFCC59` | a rare magic item's name, and a rare monster's | |

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
| `ui.hurt_text` | `#FFD1C7` | your wounds, listed under the bars |

### The GUI

| Token | Value | Used for |
|---|---|---|
| `ui.panel` | `#0D0F14`, 72% | every panel's ground (95% in high contrast, §13) |
| `ui.edge` | white, 10% | a panel's 1 px edge (Default) |
| `ui.text` | `#FFFFFF` | text |
| `ui.text_dim` | white, 70% | hints, secondary lines, what can't be done yet |
| `ui.outline` | black, 85% | a 6 px outline on any text drawn straight on the world |
| `ui.track` | black, 50% | the empty part of a bar |
| `ui.veil` | `#050814`, 60% | sleep |
| `ui.dim` | black, 50% | the world behind a choice, death, or a menu |
| `ui.hurt` | `#B30000`, up to 35% | the flash when you're hurt |

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
  - danger vermillion `#D55E00`.

  The holds stay a heavier line, so they don't depend on hue.
- **A region shifts the world's colours, never the signals** (§15).

---

## 5. Shapes and models

- **Must: the silhouette test.** Every creature kind can be told from every other by shape alone at zoom 14.
  - Draw them all flat black on white in a row: height, stoop, width, legs and head carry it.
  - Run the test before a new kind gets a colour.
  - Today the kinds part by height (0.95 to 1.8 m), stoop (28° forward to leaning back) and legs.
- **Built in code from primitives**, which is the toy-diorama look:
  - people and the dead are capsules that lean into a swing (the tell), lie down when knocked down, and darken when dead;
  - beasts get four legs and a head at their own height (`actor_view.gd`, `_build_beast`);
  - furniture is boxes with a surface (§6);
  - trees are a trunk and a sphere crown, a little different from tile to tile;
  - plants are low clumps, the herbs with a disc of flowers on top.
- **Size bands**, so a scene reads at a glance:
  - knee height (to 0.6 m): plants, crates, sacks;
  - waist (0.6 to 1.1 m): tables, beds, counters, altars;
  - head (1.1 to 2 m): shelves, wardrobes, people, wheat;
  - over your head: walls (2.2 m) and trees (about 3 m).
- **Should: detail comes from parts, not texture.** A head, shoulders and the thing in hand carry it, never painted detail at a scale the camera can't show.
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

Surfaces are procedural, drawn by shaders in world space so tiles join without seams (`shaders/surfaces.gdshaderinc`: `sf_noise`, `sf_fbm`, `sf_blocks`, `sf_boards`).

| Code | Surface | Used on |
|---|---|---|
| 0 | plain | anything that needs no texture |
| 1 | plaster, over a stone plinth under 0.5 m | house walls |
| 2 | wood (boards) | furniture, fences, doors; anything not listed |
| 3 | stone (blocks) | the altar, forge, hearth, well, graves, tombs |
| 4 | foliage | tree crowns, plants |
| 5 | straw | wheat, hay |
| 6 | cloth | beds, sacks, the sister |
| 7 | iron | the stove, the anvil |
| 8 | bark | trunks |

- **Should: patterns stay quiet**, moving value by no more than about ±12%, so an actor standing on them still reads.
- **To add a surface:**
  1. Write a function in the include.
  2. Give it the next code.
  3. Map fixtures to it in `chunk_view.gd`'s `SURFACES` (moving to `data/looks.toml`, §14).
  4. Check it at noon and at midnight.
- The ground is one colour per kind (§4), with soft noise, and the kind travels in the texel's alpha (`ground.gdshader`).

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

**Should:** a new sense or state reuses a token above. A new colour needs a new row here first.

---

## 8. Type

- **Default typefaces:**
  - **Alegreya Sans** for everything in the GUI;
  - **Alegreya**, the serif from the same family, only for titles: the game's name, screen titles ("Who comes to the abbey?"), region names, and the death screen.

  Both are under the SIL Open Font License, so they ship in `client/fonts/` with their licence. If Alegreya Sans reads poorly at 18 px, swap it for Noto Sans.

  **Today:** Godot's built-in font.
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

  So the held banner says "Three of them have hold of you", in `type.banner` and `signal.danger_text`.
- **Left-aligned** in its box, even when the box sits on the right edge. Centred only for one-line banners and titles.
- **Lines of 80 characters at most** (news, tooltips, descriptions).
- **Numbers:** counts as "×2", weights as "0.6 kg", times as "3h 50m", and a percentage only for condition.
- **Words:** plain and short, in the game's voice (`CONCEPT.md`). An act and its key are said together: "Y to pick it".

---

## 9. The GUI: what the research says, and our rules

| What the research says | Our rule |
|---|---|
| Show only what the player needs now, and hide the rest. Too much on screen is the first HUD mistake | **Progressive disclosure:** the pack, skills, making and the keys each open on a key. Hints show only what's possible here and now |
| Critical, short-lived information belongs near the eye-line. The brain filters out the corners, and edges suit only what the player has learnt to expect | Nothing stays in the middle. The urgent (held, the act under way, what you can do here) sits **just below or above you**, and the habitual (status, news, moodles, panels) sits **at the edges** |
| Read in peripheral vision: large, bold, high-contrast; values as brightness or size, not as small text or hue; neighbours that look different; motion and flashes catch the eye, but too strong pulls it | Bars over numbers. A low bar pulses in **brightness**, not colour. A new or worsened moodle pops once. Health, endurance and faith get different colours and icons. One flash at a time |
| A study of shooter displays: players did best with health on a HUD, and with ammunition and the way to go drawn in the world | Body and needs are on the HUD. The senses and threats are drawn in the world (§7) |
| Lists are simpler than grids and work with keyboard and mouse, but get long. Grids suit games where packing is the play | **Lists** (Zomboid's), grouped and sortable, with weight on every row. Packing isn't our puzzle; weight is |
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
│    Health     ███████████░░░                             A howl: something...│
│    Endurance  █████████████░                                                 │
│    Faith      ██████░░░░░░░░                          E  Moodles             │
│    (body diagram: wounds by part)                        ▣ Hungry            │
│                   C   Three of them have hold of you     ▣ Uneasy            │
│                                                          ▣ Bleeding          │
│ J  Skills (K)                                                                │
│    or Making (T):                                                            │
│    one at a time            · · ·  the world  · · ·                          │
│                             (nothing stays here)                             │
│                                      @   you                                 │
│                          F   [Y] Pick chamomile                              │
│                              Picking it  ▓▓▓▓▓░░                             │
│                                                                              │
│ G  Within reach                                       H  Your pack (Tab)     │
│    Stove · alight, 3h 50m                                In hand: cudgel     │
│    Crate              [Take all]                         bandage ×2    [Use] │
│ I  F1: the keys                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

| Zone | What | Where, how big | Shows | Today |
|---|---|---|---|---|
| **A** Status | region, day and time, speed, the pressure, the Tolling, rain, where you are | top left; one line, `type.heading` | always | as here |
| **B** Body | level and XP; health, endurance and faith bars; out of breath; wounds | top left, under A; 380 wide | always (faith only for a class that has it) | no body diagram; wounds as text |
| **C** Banner | the one thing you must answer now: held, paused, asleep | top centre, 18% down; `type.banner` | while it holds | as here |
| **D** News | what just happened, newest at the bottom; 6 lines, each lives 6 s and fades over 1 s | top right; 620 wide, text left-aligned | when there's news | as here |
| **E** Moodles | needs and states, worst first | right edge, under D; 44 px tiles with names | while any is on | text only, no icons |
| **F** Here and now | what you can do beside you (at most two prompts), and the act under way with its bar | centred just under you, about 60% down | when there's a prompt or an act | the bar is here; prompts come only through the news |
| **G** Within reach | the world's things beside you: containers, water, a fire, a counter, plants | bottom left; 420 wide, at most 45% of the height, scrolling | when anything is in reach | as here |
| **H** Your pack | in hand, on your arm, held up, then the pack; weight and coin in its title | bottom right; 420 wide, at most 45% high | Tab | as here |
| **I** Keys | every key, with the legend of the drawn senses. For a world's first game hour, a line at the bottom centre instead: "F1: every key" | over the middle of the screen, in four columns, while it's open | on F1 | as here |
| **J** Book panels | skills and talents (K), making (T); later the character and the map | left middle; 460 wide; **one at a time** | on their key | as here, but Esc doesn't close panels yet |

**Rules:**

- **Yours on the right, the world's on the left:** the pack is H, what's within reach is G. Moving a thing between them always moves it across the screen.
- **The middle is the world's.** Only world-anchored marks, zone F and banner C may sit near you.
- **Panels don't stack.** Opening a book panel (J) closes the other one. A modal screen dims the world (`ui.dim`) and pauses the game.
- **Esc** closes the top panel first, and only then opens the menu.

### Screens

| Screen | What's on it | Rules |
|---|---|---|
| Title | the name in the serif, a dusk scene behind (the abbey, a slow camera); Carry on, New world, Settings, Quit | a key or a click in reach of everything; carrying on is one click, since the game saves itself |
| Who comes to the abbey? | each class, its line and its kit; a number or a click takes one | dims and pauses the world; today as text in the middle |
| Paused | the world stilled under a dim; "Paused" banner; Space to go on | nothing else changes |
| Menu (Esc) | Resume, Settings, Save and quit | centred; dims |
| Settings | the GUI, text, colour, flashes, keys, sound, the game (§13) | every setting saved; changes show at once |
| Death | "This is how you died": the cause, the days, the deeds, where the body lies; R for the next survivor | the serif title; the world behind, dimmed, the body in view |

---

## 11. The GUI: parts

- **Panel:**
  - Background `ui.panel` with a 1 px `ui.edge` and 4 px corners (Default).
  - Padding is 12 px across and 10 px down, with 6 px between rows.
  - Its title, in `type.heading`, says what it holds and what it weighs.
  - A panel never grows past 45% of the screen's height; past that it scrolls.
- **Bar:**
  - A label 110 px wide, then a 240 × 18 bar: the fill's token on a `ui.track` back.
  - Numbers show on hover, or always with "show numbers" on (§13).
  - Under 25%, the fill pulses in brightness once a second.
- **Button:**
  - Its word, then its key: "Take all [T]".
  - A click target is at least 32 × 32 px, with 8 px between targets.
  - It never relies on a hover to say what it does.
- **Key caps:** a key always appears in brackets in the same face as its line, such as "[Y] Pick chamomile". Remapping a key changes every place it's shown.
- **Item row:**
  - Name ×count, weight, then the actions on the right.
  - A find carries a small `signal.gold` dot until it's brought home.
  - A magic item's name is in its rarity's colour, with the rarity in words on hover.
  - Condition shows under 100%, and freshness for food.
  - The rows group by kind (weapons, food and drink, cures, light, materials), and sort by kind, weight or name.
- **Moodle:**
  - A 44 px tile in its severity's colour (`moodle.mild`, `moodle.bad`, `moodle.worst`, or `moodle.good`), with a white glyph of 32 px.
  - Its name sits beside it in `type.body`, and what it does and what eases it shows on hover.
  - A new or worsened moodle pops once (scale 1.25 back to 1 over 0.3 s).
- **Mark over a head:**
  - A glyph 0.45 m above the head, billboarded, in `type.banner` scaled to the zoom, with an outline.
  - It pops when it changes.
- **Prompt (zone F):**
  - A key cap, a verb and a thing: "[G] Butcher the deer", "[Y] Pick chamomile".
  - At most two, the likeliest first. A prompt that can't be done says why in `ui.text_dim`.
- **Tooltip:**
  - After 0.4 s of hovering, in a panel, at most 70 characters a line.
  - It never hides what it describes.
- **Icons:**
  - SVG on a 64 × 64 canvas: a solid white silhouette, no text inside, legible at 32 px.
  - Moodles get a glyph each, and items get a glyph by kind until they have models; then each item's icon is drawn from its own model (§16).
  - Icons scale with the text, up to 200%.
- **Body diagram:**
  - A front silhouette, 120 px high, its parts tinted by their worst wound: scratch, laceration or bite, plus bleeding and dressed.
  - Hovering a part lists its wounds.
  - It sits in zone B, beside the bars.

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
- **Must:** nothing flickers faster than 3 Hz, and any flash can be turned off (§13).
- **Should:** one flash at a time; a newer one replaces the last.

---

## 13. Settings for everyone

These come from the Game Accessibility Guidelines' basic set and Microsoft's Xbox Accessibility Guidelines. Every setting is saved.

- **GUI scale:** 75% to 200% (`content_scale_factor`), and **text size** up to 200% on its own.

  **Today** (2026-11-12): `-` and `=` step the GUI's size from 75% to 200%, and the player's own game keeps it in `user://settings.cfg`. Text size on its own, and the rest of this list, wait for a settings screen.
- **Colour-blind mode:** off, or the Okabe–Ito set for the alert states (§4).
- **High contrast:** panels at 95%, and every text outlined.
- **Reduce flashes and motion:** no screen flashes and no vignette breathing; marks and moodles change without popping.
- **Show numbers** on the bars.
- **Keys:** every key can be remapped, and the GUI says the remapped key everywhere.
- **Game speed:** already there (pause, 1×, 2×, 3×).
- **Sound:** separate volumes for effects, the world and music, once there's sound. Nothing is told by sound alone: every sound is also a ring (§7) and, when it matters, a line of news.

---

## 14. Working on the look: the AI and you

### One place for every value

- **Tokens:** `client/look.gd` holds every token in §4 and §8 as named constants, and the GUI's parts as functions:
  - `panel_style()`, a panel's ground;
  - `text()`, a label's size and colour, outlined when it's drawn on the world;
  - `bar()`, a bar's fill on its track.

  Each script that draws preloads it (`const Look = preload("res://look.gd")`), as the client's scripts are. **Must:** code uses a token, never a new inline colour. A new colour is a new token in this guide and in `look.gd`.
- **Looks of things:** `data/looks.toml` (Default) moves the creature, furniture and plant looks out of GDScript (`LOOKS`, `FURNITURE`, `PLANTS`, `SURFACES`). A person can then tune them without code, and a mod can add its own:

```toml
[[look]]
of = "creature:ghoul"      # or "fixture:table", "plant:yarrow"
colour = "#D1C7B3"
height = 1.3               # metres
stoop = -28                # degrees forward
surface = "plain"          # §6
```

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
2. **Type:** the fonts in. *(The sizes of §8, and the held banner in sentence case: done 2026-11-12.)*
3. **Moodles as tiles:** glyphs, severity tints, a pop on change, the detail on hover.
4. **Here and now:** prompts under you (zone F). *(The act's bar under you, and the keys behind F1: done 2026-11-12.)*
5. **Book panels:** Esc closes panels first. *(Making on the left, one book panel at a time: done 2026-11-12.)*
6. **The body diagram** in zone B.
7. **Settings:** colour-blind mode, reduce flashes, high contrast, show numbers, and key remapping.
8. **Looks as data:** `data/looks.toml`.
9. **Screens:** title, the class choice as a screen, and death as a story.
10. **Icons:** glyphs by kind, then each item's icon drawn from its own model.
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
