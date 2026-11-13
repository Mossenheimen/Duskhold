# The keys: what similar games do, and the layout that came of it

*2026-11-12. The user asked to "evaluate similar games and fix the complex key/button layout. It doesn't look practical." The layout below is built (`client/main.gd`'s `_add_input_actions`) and is described in `VISUALS.md` §10, "The keys". This file keeps the reasoning and the sources (at the end).*

## What was wrong

By the first playable, the game had 43 actions on keys, almost one per verb, and they used every one of the 26 letters:
- **Every world verb had its own letter.** Pick a plant was Y, pray P, burn or butcher G, consecrate O, tend a fire H, put it out J, board up B, pry off N, drink X. Each new feature took the next free letter, so letters stopped meaning anything: Y for "yarrow"?
- **Half of them were under the right hand,** which is on the mouse. Y U O P H J N M sit far from WASD, so using them meant letting go of the mouse or of the movement keys.
- **The number row was split three ways.** 1 to 3 were speed and 4 to 7 the Wizard's spells, while the Cleric's turning (U), Mending (M) and the Fighter's Second Wind (C) were scattered letters.
- **A ruinous key sat in the fighting hand.** R twice abandoned a saved world, beside E, F and Q.
- **The keys panel (F1) was a 4 × 10 grid** in no order, so finding a key meant reading all of it.

## What similar games do

| Game | Movement and fighting | The world's verbs | Panels | Time |
|---|---|---|---|---|
| **Project Zomboid** (B42) | WASD; Shift run; C sneak; Ctrl or right mouse aim; left mouse attack; Space shove | **E interact**, and everything else from the **right-click menu** (barricade, burn a corpse, forage, pray, cook) | I inventory, H health, L skills, B crafting, M map, J info | F2 pause, F3 normal, F4 to F6 faster |
| **Valheim** | WASD; left mouse attack, right mouse block; Q auto-run | **E use** | Tab inventory | |
| **Don't Starve** | WASD; F attack | **Space: the nearest action**; left and right mouse act on what's under the cursor | Tab map; 0 to 9 the inventory's slots | |
| Rust, 7 Days to Die | | **E, and hold E for a menu of the rest** (lock a door, pick it up) | Tab inventory | |
| ARPGs (Diablo, Path of Exile) | | | | the **number row is your skills** |

Zomboid has 84 bindings in all, but almost none are world verbs. Its keys are for moving, fighting, panels and time; whatever you do *to* something goes through E or a menu. That one design choice is what keeps it learnable with a large verb list, and it's what Duskhold was missing.

## The principles taken

1. **One key for the world: E.** Tapped, it does the likeliest thing beside you. Held, it lists everything there is to do, numbered, to pick with a number or a click (Rust's hold-E, Zomboid's right-click menu). The world's verbs need no letters, and a new verb never needs a new key.
2. **The left hand stays on WASD.** What's pressed in a fight or a hurry is within reach of it: Shift, C, F, Q, E, R, Z, the number row, Space.
3. **The number row is your powers,** as in an ARPG: a Cleric's turning and Mending, a Fighter's Second Wind, a Wizard's spells, in that order, 1 to 4.
4. **The right hand's letters only open panels,** which are pressed in calm moments: I (as well as Tab) the pack, as in Zomboid; B making, as in Zomboid; K skills; P spells, as in WoW's and Skyrim's spellbooks.
5. **The F keys are the game's own:** F1 the keys, F2 to F4 the speeds (Zomboid's F keys, moved off the number row), F5 save.
6. **The mouse does what the hand already holds:** left swings or walks; right shoots a bow, or with a shield on your arm, held, braces behind it (Valheim's block). The game knows which, since a bow needs both hands.
7. **What can't be undone takes two hands:** a new world is Ctrl+N, asked twice. R, dead, brings the next survivor; alive, it's the light.

## Before and after

| Before | After |
|---|---|
| E door, B board, N pry, G burn or butcher, O consecrate or ward, P pray, Y pick, H tend a fire, J put it out, X drink | **E** (and E held) |
| C Second Wind, U turn, M mend, 4 to 7 the spells | **1 to 4**, your powers |
| 1 to 3 speed | F2 to F4 |
| Ctrl sneak, Alt brace | C sneak, the right mouse button held |
| L light, T making, I spells | R light, B making, P spells; I the pack too |
| R new world (twice) | Ctrl+N (twice) |

43 actions on all 26 letters became 31 on 16 (N only with Ctrl), and ten of those letters are within reach of WASD. Every key cap on screen is read from the InputMap, so the remapping still to come (`VISUALS.md` §16, slice 7) will change them everywhere.

## Sources

- Project Zomboid's Build 42 bindings, all 84: PZ Guide, "Key bindings B42", https://pz-guide.com/en/keybind/ (read 2026-11-12).
- Zomboid's controller radial menus (held for a menu) and context menus: pzfans.com, "Zomboid Controller Survival", https://pzfans.com/zomboid_controller_survival_taming_the_apocalypse_on_gamepad/ (search result only).
- Don't Starve's controls: the Don't Starve wiki, "Controls", https://dontstarve.wiki.gg/wiki/Controls (read 2026-11-12).
- Valheim's controls: BisectHosting, "Valheim controls guide", https://bisecthosting.com/blog/valheim-controls-guide-pc-console-controller-xbox-keybinds, and Gameranx, "Valheim: 16 advanced keybinds", https://gameranx.com/features/id/218792/article/valheim-16-advanced-keybinds-you-need-to-know/ (search results only).
- Rust's and 7 Days to Die's hold-E menus, the ARPGs' skill rows and WoW's and Skyrim's spellbook keys are from general knowledge, not checked against a source this time.
