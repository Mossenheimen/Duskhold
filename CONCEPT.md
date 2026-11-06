# Duskhold — concept

*Draft 2, 2026-11-05: the concept's questions are settled. Nothing is built yet. How it's built: `ARCHITECTURE.md`.*

> The realm has fallen. Night is coming. Hold until dawn.
> This is how you died.

## Decided (2026-11-05)

- **Regions, each with its own creatures.** Fantasy's variety of monsters, but each region's family keeps what Zomboid's zombies do well: it shapes how you move, sneak, loot, make noise and survive. Each family hunts by a different sense and bends a different rule. The first region is the dead: Zomboid's own.
- **An open world with base-building from the start,** with holdout towns and camps spread across it. Dungeons come later, as the end-game, where power keeps growing.
- **Zomboid's day and night cycle;** dungeons differ in the light they need.
- **One character.**
- **Real time with pause.**
- **The name: Duskhold.** "Barrowlight" was dropped. A web search on 2026-11-05 found no game by this exact name; Emberhold, Dreadmarch and "The Last Hearth Defense" were already taken.
- **The world is generated from a seed,** out of hand-made pieces.
- **The six regions** below, with three more on the bench.
- **Every Default in this draft** was accepted ("everything you listed sounds great").

**Default** marks a choice Claude made. All of them were accepted on 2026-11-05 and stand unless revisited.

## The pitch

An open-world survival RPG in Project Zomboid's style: wounds on each body part, everything a timed action, every sound a lure, barricades and bases, and a day and night that decide when you go out. It's set in a fallen fantasy realm where each region woke a different kind of monster, with D&D's classes and spells and Diablo's loot.

For a thousand years the wardstones kept the realm's wild things in their places. One autumn night they all went dark. The dead got up in the heartland, goblins came down out of the hills, the old wood's spiders crossed the river, the wolves came south, the cinder country caught fire and the moor began to weep. People call what followed the Long Dusk, and the places that held through the first night the duskholds. You're one of the survivors. Find a place, make it hold, and go out by day for what you need. One night it won't hold.

Why build it:

1. **The game, which is the point:** Zomboid's mechanics and look, plus the RPG layer Zomboid doesn't have: loot rarity and drops, classes, levels, spells, rare monsters, bosses, quests, events, maybe PvP.
2. **Later, a world for the PPO agent** (not a priority, the user said on 2026-11-05):
   - to train in, far faster than ZomboidRL's 3–6 decisions a second;
   - maybe to drive the game's NPCs.

## What comes from Zomboid, and what it becomes

| Zomboid | Duskhold |
|---|---|
| Knox County, sealed off by the military | A fallen realm, its regions cut off from each other |
| One kind of enemy: zombies, led by sound | A family of creatures per region, each led by its own sense; the first region's are the dead, led by sound |
| A bite is the Knox infection: dead in days | Each region has its affliction; the dead's is ghoul fever (D&D 3.5's rule: die of it and you rise at midnight), with a cure if you reach it in time |
| Moodles | Moodles merged with D&D's conditions (frightened, poisoned, prone, grappled, exhaustion) |
| Health panel: wounds per body part, bandages | The same, plus burns, frostbite, venom and necrotic wounds |
| Timed actions (eat, bandage, loot, build) | The same, plus casting, reading scrolls, picking locks, praying; a hit can break a spell (D&D's concentration) |
| Noise draws zombies | Noise, light, movement, scent, heat or fear draws the creatures, depending on the region |
| Day and night, weather, seasons | The same: night is when most regions are deadliest; rain puts out torches and washes scent away; snow keeps tracks |
| Occupations and traits | Classes, ancestries and traits |
| Skills that level by use | Skills that level by use, plus character levels |
| Loot in every container, where it would be | The same, plus magic items with rarity, affixes, identification, curses and attunement |
| Carry weight, bags, item condition | The same; plate is heavy and loud |
| Barricades, carpentry, a base | The same, and each region needs a different base: wards against ghosts, stone against fire, locks against goblins |
| Towns full of loot and no one alive | Towns full of loot, and holdouts full of people: trade, quests, a bunk for the night |
| The helicopter, alarms, the power going off | Each region's events (the Tolling, raids, full moons, ash storms, fog nights), and a realm that gets worse until you push back |
| Die bitten and you rise in your own clothes | Die and you rise as a **revenant** in your own gear: a named rare your next character can hunt down |
| Sandbox settings | The same, from day one (they double as training presets) |
| Cut-away walls, the vision cone, real darkness at night | The same |

## Pillars

1. **Every region hunts you differently.** Each family of creatures senses something else (noise, light, movement, scent, heat, fear), bends one survival rule, and has a weakness worth crafting for. Moving to a new region means learning to survive again, not just meeting bigger numbers.
2. **Fragile, not weak.** Progress gives you options, not immunity: a level-10 fighter caught in the open by a crowd still dies. D&D 5e's *bounded accuracy* works this way: weak monsters stay dangerous in numbers.
3. **Night is the enemy, and light is bait.** In most regions night is when they come. A light lets you see, and lets everything see you. Every action has a time, a noise and a light cost.
4. **Bring it home.** Most XP comes from what you bring back to your base or a holdout (old-school D&D's 1 gold = 1 XP), plus places found, people helped, quests, rare monsters, bosses and days survived. Ordinary kills pay little.
5. **Loot tells the story.** Items lie where they would be: smithies hold tools and blades, chapels holy water, kitchens food, goblin warrens whatever goblins took (yours too). Wear, blood and curses say what happened.
6. **The world goes on without you.** Corpses stay, doors stay open, creatures wander toward what they sense, holdouts grow or fall, the Long Dusk deepens, and your dead get up.
7. **One set of rules for everyone.** The simulation is the game; the graphics are a view of it. The player, NPCs and creatures all act through the same commands and perceive only what they could. Later, a trained bot can drive any of them.

## The realm

- **Regions** lie around the old heartland, harder and stranger the farther out they are. You can walk anywhere from the start (in Zomboid, Louisville is open on day one); you won't last long in a region without its tools.
- **Borders mix:** where two regions meet, their creatures meet too, and families fight each other. Lead a horde of the dead into a goblin camp (Zomboid players lead hordes on purpose).
- **Holdouts** (the duskholds): walled towns, camps, a monastery, a mill, a lighthouse; 1–3 per region (Default). People live there: trade, repairs, healing, training, quests, a bunk for the night. Each knows its region's trick (the Wolfmark's silver, the moor's salt). They have stores, defenders and morale: they grow if you help them, and they can fall to a raid (then they're a ruin to loot). Caravans travel between them.
- **Your base:** any building you can make hold, or one you build (see Base-building). Later, survivors you rescue can move in, and your base becomes a holdout of its own.
- **Day and night** are Zomboid's: a game day is an hour of real time at normal speed, with the speed dial to skip the safe hours.
- **Weather and seasons** matter: rain puts torches out, masks sound and washes scent away; fog shortens sight (and the moor's ghosts come out in it); wind carries scent and sound; snow keeps footprints, yours and theirs; summer heat and winter cold.
- **The Long Dusk deepens:** over weeks, regions spread, roads close and holdouts fall, unless you push back. Each region's deepest dungeon holds its wardstone; relight it and the region weakens. That's the end-game.
- **No required ending,** as in Zomboid. The arc for those who want it: relight the wardstones.

## Regions (Default: six to plan for, one to build first)

| Region | Creatures | They hunt by | What changes | Weakness | Day and night |
|---|---|---|---|---|---|
| **The Hushlands** (first) | the restless dead: shamblers, ghouls, skeletons, wights | sound | Zomboid's rules: numbers, doors thumped down, being dragged down, ghoul fever; the unburned dead get up | fire, blessed weapons; blunt for skeletons | always about; more and faster at night |
| **The Warren Hills** | goblins and kobolds | sight; they carry torches at night | they think: they loot the houses before you, trap the loot, steal from your base, raid at night, blow horns to call the warren, run and come back with friends | their leaders (kill the chief and they scatter); kobolds hate sunlight | sleep by day, raid by night |
| **The Webwood** | giant spiders, centipedes, carrion beetles | movement: footsteps, and anything touching a web | running is loud and standing still hides you; a touched web tells every spider on it where you are; cocoons hold loot, and sometimes someone alive; venom paralyses | fire (webs burn, and so does the wood) | they hunt the ground at night; dawn dew shows the webs |
| **The Wolfmark** | wolves, dire wolves, werewolves | scent, carried by the wind | stay downwind; blood draws them from far off; rain and rivers wash your trail; packs circle; the cold; full moons bring werewolves, and lycanthropy | silver; fire keeps wolves back | dusk and dawn; full-moon nights |
| **The Cinder Reach** | salamanders, fire elementals, hell hounds | heat: your body, your fire, your torch | warmth gives you away, so wet and cold hide you; a cooking fire is a beacon; fire spreads through wooden houses, so build in stone; smoke and ash storms | water and frost | they glow at night; the day's heat is a danger of its own |
| **The Weeping Moor** | ghosts, wraiths, banshees, will-o'-wisps | fear: your dread | walls don't stop them, so a base needs wards (salt lines, iron, holy symbols); calm hides you (light, company, music, the Brave trait); wisps lead you into the bog; fog | silver, blessed weapons, salt and iron, daylight | night and fog only; days are safe |

| Region | Holdout | Dungeon (later) | Boss | Event |
|---|---|---|---|---|
| Hushlands | the abbey on the hill | crypts and catacombs | the Barrow King | the Tolling: a bell rings and every dead thing in earshot walks to it (Zomboid's helicopter) |
| Warren Hills | a fortified mill | the warrens under the hills | the Hob-King | raid nights |
| Webwood | a woodcutters' stockade on cleared ground | the brood hollows | the Brood Mother (her eggs hatch at noise) | the hatching |
| Wolfmark | a silver-mining town | the old silver mine | the Old Wolf | the hunter's moon |
| Cinder Reach | a quarry town built of stone | the forge-deeps | the Cinder Wyrm | ash storms |
| Weeping Moor | a lighthouse inside a salt ring | the drowned abbey | the Weeping Lady | fog nights |

On the bench: the Fey Wood (glamours: nothing is what it looks like, fey gold turns to leaves at dawn, cold iron), the Drowned Mire (water everywhere: sinking under weight, wet and cold, disease), the Troll Fells (trolls regrow unless burned, and sunlight turns them to stone: lure them into the dawn).

Within a region the variety comes from the family's kinds, rare monsters with affixes, wanderers from next door, and the region's own events.

## The loop

- **A day:** go out by daylight, scavenge, trade at a holdout, build; get home before dark.
- **A night:** hold. Bar the doors, keep the fire small (or none, in the Cinder Reach), listen.
- **A life** (days to weeks): level up, learn spells, find better gear, make a base that holds, craft the next region's tools, push farther out.
- **The end-game** (later): dungeons, the deepest in each region holding its wardstone; bosses; the most powerful loot; relighting a wardstone weakens its region.
- **A legacy** (across characters): your base stays where it is, the holdouts remember you, your map stays drawn, and your revenants wait where you fell.

## Character

### Classes (Default: four to start)

| Class | Plays like | Resource | Signature |
|---|---|---|---|
| Fighter | holds doorways; heavy armour (loud); shields | stamina | shield bash, second wind, cleave |
| Rogue | dark and quiet; backstabs; locks and traps | none: the dark is its resource | sneak attack from darkness or behind, picking locks, disarming traps |
| Cleric | healer, and the dead's natural enemy | faith, restored by prayer at shrines | turn undead, cure wounds, cure afflictions, sacred light, wards |
| Wizard | fragile, versatile, loud when it counts | spell slots, restored by rest | fireball (a huge noise and light), sleep, silence, identify |

Later: Ranger (bows, tracking, reading wind and tracks), Bard (songs ease dread, and carry), Paladin, Warlock (power at the price of corruption).

A class is a starting kit, starting skills, a talent tree and a resource. Anyone can learn any skill by using it; a class starts ahead in its own. A region's tools (silver, salt, fire oil) work for every class.

### Ancestry and traits (Default)

- **Ancestries are trait bundles:** human (one more trait), dwarf (darkvision 3 tiles, hardy, slow), elf (keen hearing, darkvision 2 tiles), halfling (small: quieter, lucky). Darkvision matters a great deal in a game about night.
- **Traits** are Zomboid's point-buy in fantasy dress: Brave / Afraid of the Dark, Light-footed / Clumsy, Keen Ears / Hard of Hearing, Strong Back / Weak Back, Iron Stomach / Weak Stomach, Quick Learner, Fast Healer / Slow Healer, Lucky / Unlucky, Cold-blooded (harder for heat-hunters to find).

### Levelling (Default: two tracks)

- **Skills by use** (Zomboid): Blade, Blunt, Axe, Spear, Bow, Shield, Sneak, Lockpicking, Traps, First Aid, Lore (identifying), Alchemy, Smithing, Carpentry, Masonry, Cooking, Farming, Tracking, and one per school of magic.
- **Character level** (D&D) from XP: what you bring home (most of it), places found, people helped, quests, rare monsters and bosses, days survived; ordinary kills a little. Each level: a little more health, a talent, more spell slots or faith. Levels 1–10 in the open world; dungeons take it further. Bounded: level 10 has 2–3× level 1's health, not 10×.

## Body, moodles and afflictions

- **The health panel** is Zomboid's: head, torso, arms, hands, legs and feet; scratches, lacerations, deep wounds, fractures, burns and bites; bleeding and pain; bandages, splints and stitches. Armour protects per body part, as Zomboid's clothing does.
- **Moodles** (Default: Zomboid's set, trimmed, merged with D&D's conditions): hungry, thirsty, tired, exhausted, dread (Zomboid's panic: worse aim; at its worst, frightened, as D&D's condition), wet, cold, hot, heavy load, bleeding, pain, poisoned, diseased, cursed, prone, grappled; and good ones: blessed, inspired, well fed.
- **Needs** are Zomboid's (hunger, thirst, sleep), since the open world is lived in day after day.
- **Afflictions** are Zomboid's bite with a way out, one per region:
  - **ghoul fever** (Hushlands; the first): a ghoul's bite. A fever on the first day, weakness on the second, and on the third you die and rise as a revenant;
  - **venom** (Webwood): paralysis at once, sickness after;
  - **lycanthropy** (Wolfmark): stronger at night, and the full moon takes over;
  - **festering burns** (Cinder Reach);
  - **spirit-touch** (Weeping Moor): dread that won't fall; at worst, a ghost takes you over for a while.

  Cures are quests: a holdout's chapel (at a price), a strong enough cleric, a rare herb from somewhere unpleasant.
- **The dead get up** (Hushlands): unburned human dead rise after a while. Burn them (oil) or consecrate them (holy water, a cleric), or leave them and maybe meet them again. Zomboid players burn corpse piles against corpse sickness; here it's the corpses themselves that come back.

## Senses: how they find you

- **Your signature:** at every moment the character gives off noise, light, movement (how fast and how heavy), scent (a trail the wind bends), heat and dread. Each family reads one or two of them. A HUD shows them (Thief's light gem, for every sense), and the bot gets them as inputs.
- **Sight:** the character sees in a cone (Zomboid's), and only what's lit or within darkvision. Outside it you see what you remember (fog of war), never a live monster.
- **Light:** every tile has a light level, from day, night, weather and being indoors. Dungeons range from lit to unnaturally dark (see Dungeons).
- **Light sources:**
  - a torch: burns about an hour, held in a hand (so no shield), put out by rain and water, can be dropped or thrown to light a room;
  - a lantern: burns oil, hangs from the belt, dimmer, survives a splash;
  - candles, braziers, a fire, and spells (a light cantrip; a fireball's flash).
- **Noise:** every action has a radius (walking in plate, a door, a fight, a shout, a Knock spell), like Zomboid's world sounds.
- **Movement:** running and heavy armour shake the ground; webs carry every touch.
- **Scent:** you leave a trail; blood makes it stronger; rain, rivers and wind change it; food stores smell too.
- **Heat:** your body, your clothes (wet means cold: hidden from heat-hunters, and on the way to hypothermia), your fires.
- **Dread:** rises in the dark, near the dead, alone and wounded; falls with light, company, music, food and rest.
- **Stealth:** be what they can't sense, and get behind them: a sneak attack.

## Combat and magic

- **Real-time and deliberate** (Zomboid): stamina, facing, reach, shoves and knockdowns, crits, weapons that wear and break, crowds that grab and drag you down. Doorways and corridors are your friends.
- **Weapons** differ in reach, speed, damage and knockback: blunt for skeletons, edges for flesh, points for armour, silver for werewolves and ghosts.
- **Spells are timed actions:** a cast time, a noise and a light radius, a cost (a slot, faith, a component); a hit can break the cast. Fireball lights up the street and everything on it hears it. Silence is a stealth tool. Knock opens any lock, and (as in the SRD) the knock can be heard 300 feet away.
- **Components are loot** (bat guano and sulphur for a fireball, as in D&D), so wizards scavenge too.
- **Resting** restores slots and health, and means sleeping somewhere that holds: Zomboid's night in a house with zombies outside.
- Default: about 8 spells per caster class at first; each school levels by use.

## Base-building (decided: from the start)

- **Zomboid's way:** claim a building and make it hold (board the windows, barricade and bar the doors, block the stairs), or build one (carpentry, then masonry: walls, doors, floors, roofs, palisades).
- **Inside:** a bed (sleep is how you rest; sleeping somewhere unsafe is how you die), storage, a fire or stove, workbenches (smithing, alchemy, scribing), a rain barrel or a well, a garden (food over weeks), traps and noise-makers.
- **Each region needs a different base:**
  - Hushlands: barricades and strong doors (they thump), decoy noise-makers to draw them off, a pyre for the dead;
  - Warren Hills: locks, hidden stashes and lookouts (they steal, and they scout), lights along the walls;
  - Webwood: cleared ground around it and firebreaks; webs grow on the walls overnight;
  - Wolfmark: food sealed away (they smell it), fire, stone, warmth;
  - Cinder Reach: stone, not wood; water barrels; a cool cellar to sleep in;
  - Weeping Moor: wards (salt lines, iron, holy symbols), because walls don't stop them; light and company against dread.
- **Wards are the wardstones' small cousins:** the same magic at the scale of a house.

## Monsters beyond the regions

- **Rare monsters** (Diablo's champions): an ordinary creature with 1–3 affixes and a name ("Grimtooth the Lantern-Eater"). Shared affixes: Frenzied, Thrice-Risen (gets up twice), Lantern-Eater (puts out lights near it), Bellwether (calls its whole family when it sees you), Ironhide, Spectral (walks through doors), Plaguebearer (afflicts on a hit), Hunter (tracks you by any sense). Family ones: a goblin Thief-Lord who robs your base, a pack alpha that sends the others around you, a spider that spins over doorways, a Gravecaller that raises corpses. They roam; holdouts post bounties on them. They drop better loot.
- **Revenants:** your dead characters, risen in their gear, with their class abilities: a named rare haunting the place where they fell. Kill it to get your things back. Stretch goal: it fights the way you did, from a bot trained on your own play.
- **Bosses:** one per region, in its deepest dungeon, each fought in the region's own terms (the tables above). Bosses drop at least one Very Rare item.

## Loot

- **Mundane items** (most of the loot, as in Zomboid) come in qualities: crude, standard, fine, masterwork.
- **Magic items** have D&D's rarities: Uncommon, Rare, Very Rare, Legendary, and Artifacts (boss uniques with a story). Any of them can be **cursed**.
- **Affixes** (prefixes and suffixes, rolled by rarity and by region) are often trade-offs: a Blazing sword lights the room, and you.
- **Identification:** a magic item is unknown until identified (the Lore skill, a scroll, a holdout's sage) or attuned, which may reveal a curse the hard way.
- **Attunement:** at most 3 attuned items (D&D), which caps power.
- **Region tools:** silvered weapons, salt, iron nails, holy water, fire oil, frost charms; from the holdouts that know them (silver from the Wolfmark's mining town) and from crafting.
- **Drops:** creatures drop what they carry, plus their own table; rares and bosses roll higher.
- **Wear:** weight and condition on everything; repair with Smithing; blood, dirt and scorch marks on gear.
- **Containers:** cupboards, chests, barrels, bookshelves, sarcophagi, bodies; locks and traps on the good ones; now and then, a mimic.
- **Inventory** as in Zomboid: lists with weights, bags with capacities, and both hands (a torch takes one).

## Crafting

Crafting builds the base (carpentry, masonry, wards), keeps you going (torches, bandages, splints, cooking, farming), fixes and improves gear (Smithing, silvering a blade), and makes the region tools (salt lines, holy water, fire oil, frost charms). Alchemy makes potions; wizards scribe scrolls; rogues set traps.

## Quests and events

- **Quests** (Default: generated from the simulation first, a written arc later), mostly from holdouts:
  - bounties on rare monsters (they roam; the board says where they were seen last);
  - rescues: bring survivors home, to a holdout or to your base;
  - recoveries: an heirloom, a relic, a map, someone's body;
  - cures for an affliction (the quest starts when you're bitten);
  - supplies for a holdout running short, escorts for a caravan;
  - the arc: the wardstones.
- **Events:** each region's (the tables above), plus the realm's: blood moons, caravans, a holdout under siege (help it, or loot the ruin), rival survivors (bots: trade, avoid, or fight), the Long Dusk deepening.

## Dungeons (decided: later, as the end-game)

- Each region has dungeons of its own family, from a one-room barrow to the wardstone vault. Deeper is harder and richer, and that's where power keeps growing: levels past 10, Very Rare and Legendary items, bosses.
- **The light differs by dungeon:**

  | Kind | Light | What it means |
  |---|---|---|
  | lived-in (goblin warrens, the forge-deeps) | lit by its owners | no torch needed, but their lights show you to them; put the lights out |
  | dark (crypts, mines) | none | bring light, and be seen |
  | light-hostile (the brood hollows) | none, and light wakes the brood | darkvision, or slow and blind |
  | unnaturally dark (the drowned abbey) | mundane flames die | only magic light or darkvision |
  | glowing (fungus caves) | dim and natural | enough to move by, not enough to read a scroll |
  | open to the sky (ruins) | daylight | safe-ish by day; at night they belong to the region |

- **A delve has its own pressures:** what you carry out is what you keep; light, rations, wounds and weight decide when to turn back; your revenant waits down there if you don't make it.

## Death and legacy (Default: permadeath in a persistent world)

- **Death ends the character,** as in Zomboid, with a recap: "You survived 23 days, 4 hours. Slain by Grimtooth the Lantern-Eater in the cellar of the old tannery, carrying a Rare longsword you never identified."
- **The world keeps it:** your body and gear lie where you fell, and you rise there as a revenant. Your base stays where it is.
- **The next character** starts at a holdout and can walk back to the old base. What carries over: the base and what's in it, the holdouts' memory of you, your map, the places found, what you've learned about the creatures.
- A sandbox option for a softer game: keep the character, lose what you carried.

## Multiplayer and PvP (Default: single-player first, built to allow more)

- **The first "PvP" is rival survivors played by trained bots:** after the same loot, with bases of their own.
- **Co-op later:** 2–4 players in one realm, like a Zomboid server.
- **PvP later, optional:** raiding other players' bases, an extraction mode in the dungeons (get out with the loot or lose it, like Dark and Darker), or arenas.
- **Why not first:** netcode multiplies the work on everything. The simulation will be multi-agent and authoritative from the start (the bots need that), so the door stays open.

## Built so a bot can play it later

The game comes first (2026-11-05). These rules cost the game little and help it anyway: any bug can be replayed, and the player's greyed-out buttons come from the same list the bot's masks would. They also keep the bot possible later. What ZomboidRL taught us, and the rule Duskhold starts with:

| ZomboidRL | Duskhold |
|---|---|
| 3–6 decisions a second; the game caps at 5× | No graphics in the loop: target 1,000× real time or more per core, many worlds in parallel |
| Resets of up to 20 s (teleports, streaming, god mode, death guards) | A reset is a function call; any scenario on demand (seed, region, place, time, kit, creatures) |
| Held actions, interrupts and "carry on when chosen again", built action by action | One timed-action rule in the engine: an action runs until it's done, interrupted or replaced, for players and bots alike |
| Masks rebuilt in Python from the state | The game says what's possible every tick (the same list greys out the player's buttons) |
| The network grown five times (112 → 248 inputs) as senses were added | Every sense the realm will use is in the observation from the start, by name, so a new region adds no inputs |
| Real deaths tracked down by decompiling the game's jar | Deterministic: a seed and the actions replay any episode exactly, in the dashboard |
| A reward bolted on from outside, and it learned to fight, not survive | The game's own XP is the reward, and bring-it-home XP pays for what we want |
| Spawn areas drained over a day | The world is generated from a seed, with repopulation rules |

- **Episodes in an open world:** the bot plays slices of the realm generated from a seed: a region, a place (a farmhouse, a town street, a forest camp), a time of day and a season, a kit and the creatures, until it dies, runs out of time or reaches a goal. The whole open world is for players, and for long bot runs.
- **Simulate near, summarise far:** full detail in the chunks around the character (Zomboid does the same with its cells), a coarse version everywhere else (migrations, holdout stores, raids). That's also what makes a bot's world fast.
- **Regions as tests:** train in the Hushlands, test in the Wolfmark: does it stay downwind without being told?
- **Perception, not omniscience:** the bot's observation is what the character perceives (its cone, darkvision, what it hears, its signature, its body, moodles and kit), never the god view, which is for debugging. That keeps bot rivals fair, and it means the bot needs memory (the remembered map, or a recurrent network): a plan item.
- **Long horizons later:** base-building and multi-day plans are hard for PPO. The bot starts on surviving, looting and fighting, and learns building later (perhaps with a planner on top).
- **Bots inside the game:** rival survivors, holdout scavengers and guards, and playtesters: thousands of runs per class and region overnight, to measure difficulty and catch exploits (a turret, say) before a player does.
- **What carries over from `11_PPO-Zombie`:** `ppo.py` (the action + kind policy, masking, the reward scale, per-network clipping, the KL limit), the telemetry and the dashboard, name-based checkpoints, and the lessons. Default: the bot's actions mirror ZomboidRL's where they overlap (8 moves, attack, the item verbs with kinds, interact, bandage, shove, sneak), plus cast, light and build, so `ppo.py` takes them as they are.
- **What doesn't:** the trained Zomboid weights. Different world, different inputs; naming shared inputs the same lets the loader warm-start them, but expect little. It works the other way round: Duskhold is where RL ideas get tried in minutes before going back to ZomboidRL.

## Look and feel

- **Isometric 2.5D like Zomboid:** tiles, walls cut away in front of the character, several floors (the one above cut away on the stairs).
- **Each region has its own colour key:** the Hushlands' grey-green fields, the Warren Hills' ochre, the Webwood's sickly white, the Wolfmark's snow and pine, the Cinder Reach's black and orange, the Weeping Moor's blue fog. Everywhere: warm firelight against blue-black night, and the holdouts' lit windows, the only lights for miles. The other saturated colours are magic (rarity), fire and blood.
- **Zomboid's UI language:** the moodle column, the health panel, inventory windows with weight bars, progress bars over the head, the speed dial (pause, 1×, faster when it's safe), a map that holds only what you've seen and takes your notes, and the signature HUD (how loud, bright, warm and scented you are).
- **Sound is gameplay:** footsteps, groans in the dark, a bell across the fields, wolves in the hills. A directional sound indicator, for accessibility and as a bot input.
- **Wear shows:** blood, dirt and scorch marks build up on gear and on the character; bases show their damage.
- How to make it (2D isometric sprites, or 3D models under a fixed isometric camera, the way Zomboid draws its 3D characters over 2D tiles) is a plan question; free CC0 kits exist for both.

## The core that has to be fun first

Before classes, spells, quests and other regions: is surviving the Hushlands tense and fun? The first playable:

- the Hushlands as a small open map: a market town, farms, a stretch of forest, a river, the abbey holdout;
- Zomboid's day and night, and rain;
- base-building basics: board the windows, barricade the doors, a bed, storage, a fire;
- the dead: shamblers and ghouls, the Tolling, a handful of rare affixes, revenants;
- one class (Fighter), perhaps two (+ Cleric);
- loot rarity and identification, weight, containers;
- wounds, bandages, a few moodles (hungry, thirsty, tired, bleeding, pain, dread, heavy load), ghoul fever;
- scavenge by day, hold by night, bring it home for XP; death and the revenant.

Then the Warren Hills, to prove the first pillar (enemies that see and think instead of hear), then classes and spells, holdout quests and events, more regions, dungeons, co-op and PvP.

## What the plan must meet

Worked out in `ARCHITECTURE.md`.

- The simulation is separate from the graphics; headless is first-class; a fixed tick (say 10 per game second); deterministic from a seed.
- Callable from Python as a Gymnasium environment, in-process and vectorisable: the RL stack is Python.
- An open world in chunks: full simulation near the character, a summary elsewhere, saved as it changes (bases, corpses, opened doors).
- A world generated from a seed out of hand-made pieces (decided: towns, buildings, holdouts and camps made by hand, placed on generated land).
- A renderer that can do isometric cut-away walls, day and night lighting and the vision cone (engine choice: plan).
- A tile grid (Zomboid's world is tiles too) with 8-direction movement drawn smoothly: fast to simulate, and the 8 moves the bot already has. (Zomboid's own movement is free; this is a Default.)
- Creatures, items, affixes, spells, buildings and regions in data files (as Zomboid's scripts are), so content is cheap and moddable.
- Licensing: the D&D 5e SRD is CC-BY 4.0: its monsters, spells and rules can be used with attribution. Monsters and names outside it (beholders, mind flayers, the Forgotten Realms) can't. Zomboid's art and code are off-limits; it's inspiration only.

## Open questions

None left for the concept (2026-11-05). The engine and architecture questions are in `ARCHITECTURE.md`.

## References

- Project Zomboid: the model for the simulation, the look and the tone.
- Valheim: biomes with their own threats, each asking for gear from the last.
- Stoneshard: a Zomboid-like fantasy RPG (wounds, moodles, turn-based).
- Dark and Darker: fantasy extraction PvPvE with classes and rarity.
- Diablo I and II: rarity, affixes, champions.
- Darkest Dungeon: stress, torchlight, a hamlet that grows.
- Baldur's Gate 1 and 2: isometric D&D, real-time with pause.
- 7 Days to Die: horde nights against the base you built.
- Old-school D&D: treasure XP; light and rations as the delve's clock.
- Thief: stealth by light and sound, and the light gem.
- Cataclysm: Dark Days Ahead (and its Magiclysm mod), Caves of Qud, Dwarf Fortress: simulation first.
