# Duskhold — concept

*Draft 4, 2026-11-09: a design review against what's built (see "The 2026-11-09 review" under Decided). The survival rules are playable in a patch of the Hushlands (`ARCHITECTURE.md`, build step 5); the first playable is next.*

> The realm has fallen. Night is coming. Hold until dawn.
> This is how you died.

## Decided (2026-11-05 and 2026-11-06)

- **Regions, each with its own creatures.** Fantasy's variety of monsters, but each region's family keeps what Zomboid's zombies do well: it shapes how you move, sneak, loot, make noise and survive. Each family hunts by a different sense and bends a different rule. The first region is the dead: Zomboid's own.
- **An open world with base-building from the start,** with holdout towns and camps spread across it. Dungeons come later, as the end-game, where power keeps growing.
- **Zomboid's day and night cycle;** dungeons differ in the light they need.
- **One character.**
- **Real time with pause.**
- **The name: Duskhold.** "Barrowlight" was dropped. A web search on 2026-11-05 found no game by this exact name; Emberhold, Dreadmarch and "The Last Hearth Defense" were already taken.
- **The world is generated from a seed,** out of hand-made pieces.
- **The six regions** below, with three more on the bench.
- **Every Default in this draft** was accepted ("everything you listed sounds great").
- **The design research's 14 recommendations** (2026-11-06, `research/game_design.md`), with its four defaults ("defaults on all four, fold them into the concept"), each folded into its section below:
  - a grab has a way out;
  - three skills can be mastered;
  - no class bias in the world's containers;
  - sieges on a calendar, as well as the Director's events.

- **The 2026-11-09 review** (the user: "make any changes you see fit to ensure it's fun and mechanics/systems mesh well"). It checked each pillar against the numbers in `data/`, and found five places where a mechanic this concept leans on did nothing in play. All five are changed, with tests:
  - **light wasn't a trade.** At night you saw 4 tiles without a torch and 5 with one, while the torch showed you to the dead from 8. Now the night shows you 2 tiles and a torch about 6;
  - **weapon noise did nothing.** Every weapon's noise (2 to 6 tiles) was shorter than the dead's sight (8) and no louder than a wall muffles (4), so a quiet knife and a loud axe drew the same crowd: none. Now a knife stays in the room, a cudgel is heard next door and a wood axe two rooms off;
  - **the dead's tell couldn't be answered by moving.** A step takes 0.7 s and the tell was 0.8 s. It's 1.2 s now: shove, strike first, or step back;
  - **struggling was pointless,** because a shove always broke a hold, for half the endurance. A shove from a hold now needs a free hand, as "Combat and magic" already said: with a weapon and a torch, you struggle. It's one more cost of carrying light;
  - **one shambler never became a crowd,** though numbers are the Hushlands' whole rule. One that catches sight of you now groans, and the others in earshot come to it.

  It also made finishing a downed enemy spare the weapon (this concept said so; the game didn't), and settled four things that the first playable needs and that were vague: what "home" is, the Hushlands family's roles, how the Long Dusk's pressure rises and falls, and saves. Each is in its section below, marked *(review)*. What it checked and left alone: needs, healing, carry weight, endurance, and the skill curve.

**Default** marks a choice Claude made. All of them were accepted (2026-11-05, 2026-11-06) and stand unless revisited. The review's choices are Defaults too, and not yet confirmed by the user.

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
| Endurance, shoves and stomps | The same economy: every swing, shove and sprint draws on one endurance bar, and finishing a downed enemy spares the weapon |
| Skills that level by use | Skills that level by real use under real risk (three can be mastered), plus character levels from what you bring home |
| Loot in every container, where it would be | The same, plus a few magic items: rarity, properties that change how you play, identification, curses, attunement |
| Carry weight, bags, item condition | The same; plate is heavy and loud |
| Barricades, carpentry, a base | The same, and each region needs a different base: wards against ghosts, stone against fire, locks against goblins |
| Towns full of loot and no one alive | Towns full of loot, and holdouts full of people: trade, quests, a bunk for the night |
| The helicopter, alarms, the power going off | Each region's events (the Tolling, raids, full moons, ash storms, fog nights), and a realm that gets worse until you push back |
| Die bitten and you rise in your own clothes | Die and you rise as a **revenant** in your own gear: a named rare your next character can hunt down |
| Sandbox settings | The same, from day one (they double as training presets) |
| Cut-away walls, the vision cone, real darkness at night | The same |

## Pillars

1. **Every region hunts you differently.** Each family of creatures senses something else (noise, light, movement, scent, heat, fear), bends one survival rule, and has a weakness worth crafting for. Moving to a new region means learning to survive again, not just meeting bigger numbers.
2. **What hunts you, you can see.**
   - Every sense is drawn in the playfield: noise rings, your light's reach, scent trails, heat, tremors, dread, sight cones.
   - Every attack has a tell before it lands.
   - Players accept harsh rules they can see and reject fair rules they can't (Mark of the Ninja's lesson). A game whose regions hunt by different senses lives on drawing them.
3. **Fragile, not weak.** Progress gives you options, not immunity: a level-10 fighter caught in the open by a crowd still dies. D&D 5e's *bounded accuracy* works this way: weak monsters stay dangerous in numbers.
4. **Night is the enemy, and light is bait.** In most regions night is when they come. A light lets you see, and lets everything see you. Every action has a time, a noise and a light cost.
5. **Bring it home.**
   - Most XP comes from what you bring back to your base or a holdout (old-school D&D's 1 gold = 1 XP), and each find counts once.
   - Places found, people helped, quests, rare monsters, bosses and days survived pay too.
   - Ordinary kills pay little, so monsters are obstacles, not piñatas.
   - *(review)* **Home is your stash.** A base is a building where you've slept a night through, with one container you've marked as your stash. A find pays when it goes into the stash, or is sold at a holdout, by its worth. You carry 15 kg, so worth per kilo is the choice on every trip: that's the loop's main decision, and it needs no new system, only containers, sleep and weight.
6. **Loot tells the story.** Items lie where they would be: smithies hold tools and blades, chapels holy water, kitchens food, goblin warrens whatever goblins took (yours too). Wear, blood and curses say what happened.
7. **The world goes on without you.** Corpses stay, doors stay open, creatures wander toward what they sense, holdouts grow or fall, the Long Dusk deepens, and your dead get up.
8. **One set of rules for everyone.** The simulation is the game; the graphics are a view of it. The player, NPCs and creatures all act through the same commands and perceive only what they could. Later, a trained bot can drive any of them.

## The realm

- **Regions** lie around the old heartland, harder and stranger the farther out they are. You can walk anywhere from the start (in Zomboid, Louisville is open on day one); you won't last long in a region without its tools.
- **Borders mix:** where two regions meet, their creatures meet too, and families fight each other. Lead a horde of the dead into a goblin camp (Zomboid players lead hordes on purpose).
- **Holdouts** (the duskholds): walled towns, camps, a monastery, a mill, a lighthouse; 1–3 per region (Default). People live there: trade, repairs, healing, training, quests, a bunk for the night. Each knows its region's trick (the Wolfmark's silver, the moor's salt). They have stores, defenders and morale: they grow if you help them, and they can fall to a raid (then they're a ruin to loot). Caravans travel between them.
  - *(built 2026-11-10)* The abbey on the hill stands north of the road where it starts, and you start just inside its gate.
    - **Its walls are hallowed ground:** the dead never set foot inside, and pay no heed to anything done there, so you can sleep there with them about. Its beds are a guest's, though: sleeping there doesn't make a home.
    - **At its counter, Sister Agnes:**
      - sells the cure (one a day, 30 coin), bandages, oil, bread, water, torches and candles, from a stock that comes back each day;
      - buys anything with a worth for half of it in coin. A find sold there pays its XP too, as it would in your stash, so pillar 5's "or sold at a holdout" is built;
      - mends the weapon in your hand for coin, by how worn it is.
    - **Selling it what it's short of** (food, water, oil, candles) pushes the pressure back by a quarter of a per cent for each point of worth: "a holdout supplied".
    - **Not yet:** the sisters as people (actors with a day of their own), defenders, morale and raids, bounties, and training.
- **Your base:** any building you can make hold, or one you build (see Base-building). Later, survivors you rescue can move in, and your base becomes a holdout of its own.
- **Day and night** are Zomboid's: a game day is an hour of real time at normal speed, with the speed dial to skip the safe hours.
- **Weather and seasons** matter: rain puts torches out, masks sound and washes scent away; fog shortens sight (and the moor's ghosts come out in it); wind carries scent and sound; snow keeps footprints, yours and theirs; summer heat and winter cold.
- **The Long Dusk deepens:** over weeks, regions spread, roads close and holdouts fall, unless you push back. Each region's deepest dungeon holds its wardstone; relight it and the region weakens. That's the end-game.
  - *(review)* **It's one number per region, the pressure, and you can move it before any dungeon exists.** Otherwise the realm only ever gets worse until the end-game is built, which is a slow death, not tension.
    - It rises with time, to a ceiling (Zomboid's population peaks and stops), and with the region's own rule. In the Hushlands that's the unburned dead: every body left lying gets up.
    - It falls with deeds: bodies burned, bounties taken, a holdout supplied. A wardstone relit drops the ceiling for good.
    - It sets how many creatures a waking chunk holds, and how often the Director reaches for the region's event. Nothing else: no creature gets tougher (no level scaling).
    - *(built 2026-11-10)* It's measured against the dead a patch began with: at 100% the patch holds as many as at the start. The Hushlands start at 100%, and time adds 5% a game day up to 150%. Each body that gets up adds 2%, past 150% if it comes to that, up to 200%. Each body burned takes 2% off, down to 25%. While the patch holds fewer of the dead than that, one walks in every half hour, into a chunk away from you, so the dead you put down are made up for out of sight, never in front of you. The Director's event waits for the Tolling (slice 6).
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

*(review)* **Each kind in a family stresses a different system,** or it's only a bigger number. The Hushlands' four:

| Kind | What it's for | The system it stresses |
|---|---|---|
| Shambler | the wall: slow, many, drawn by sound, and its groan calls more | noise, doorways, endurance over a long fight |
| Ghoul | the reason to run: faster than your walk, slower than your run; goes for blood and the wounded; its bite carries ghoul fever | endurance shared between fighting and fleeing; wounds; the cure |
| Skeleton | the wrong-weapon check: edges and points do little, blunt breaks it; it doesn't groan | what you carry (weight), weapon kinds |
| Wight | the night's own: only after dark, puts out lights near it, dread | light, dread, being home by dusk |

You out-walk every shambler (1.5 tiles a second against 0.8, or 1.0 at night), so against shamblers alone, running and the endurance it costs never matter. The ghoul is what makes that bar a decision, so it comes first in the first playable. A shambler's bite is only a bad wound; the fever is the ghoul's.

*(built 2026-11-10)* The ghoul runs 2.0 tiles a second, or 2.5 at night: a full bar of endurance lasts 25 seconds of running at 3.0. It makes no groan, so it comes alone and quietly. It smells a bleeding character through walls, 12 tiles off for each wound that bleeds, so while you bleed it can't be lost: **bandage first, run second.** The fever's first day makes you thirsty and stops healing, its second leaves you short of breath, and its third kills. The cure is a rare tincture, one to four in a patch, until the abbey sells it.

## The loop

- **A day:** go out by daylight, scavenge, trade at a holdout, build; get home before dark.
- **A night:** hold. Bar the doors, keep the fire small (or none, in the Cinder Reach), listen.
- **A life** (days to weeks): level up, learn spells, find better gear, make a base that holds, craft the next region's tools, push farther out.
- **The end-game** (later): dungeons, the deepest in each region holding its wardstone; bosses; the most powerful loot; relighting a wardstone weakens its region.
- **A legacy** (across characters): your base stays where it is, the holdouts remember you, your map stays drawn, and your revenants wait where you fell.

## Character

### Classes (Default: four to start)

Each class plays a different part of the system:

| Class | Plays like | Its part of the system | Resource | Signature |
|---|---|---|---|---|
| Fighter | holds doorways; heavy armour (loud); shields | engagement: the lines to adjacent enemies, and grabs | endurance, more of it than anyone | brace, shield bash, cleave, second wind; shrugs off grabs |
| Rogue | dark and quiet; backstabs; locks and traps | darkness and the signature | the dark itself | sneak attacks from darkness or behind; reads tracks; picks locks, disarms traps |
| Cleric | healer, and the dead's natural enemy | dread and the dead | faith, restored by prayer at shrines | turn undead, wards, cures, steadying light |
| Wizard | fragile, versatile, loud when it counts | the sense fields themselves | spell slots, restored by rest | Silence, Fog, Chill, Light, Knock; fireball (a huge noise and light); identify |

Later: Ranger (bows, tracking, reading wind and tracks), Bard (songs ease dread, and carry), Paladin, Warlock (power at the price of corruption).

A class is:

- a starting kit;
- faster learning in its own skills (×1.5, like a Zomboid occupation);
- a list of talents to pick from (signatures, not trees);
- a resource;
- one signature mechanic.

Anyone can learn any skill, more slowly. A region's tools (silver, salt, fire oil) work for every class.

### Ancestry and traits (Default)

- **Ancestries are trait bundles:** human (one more trait), dwarf (darkvision 3 tiles, hardy, slow), elf (keen hearing, darkvision 2 tiles), halfling (small: quieter, lucky). Darkvision matters a great deal in a game about night.
- **Traits** are Zomboid's point-buy in fantasy dress: Brave / Afraid of the Dark, Light-footed / Clumsy, Keen Ears / Hard of Hearing, Strong Back / Weak Back, Iron Stomach / Weak Stomach, Quick Learner, Fast Healer / Slow Healer, Lucky / Unlucky, Cold-blooded (harder for heat-hunters to find).

### Levelling (two tracks)

- **Skills by use** (Zomboid): Blade, Blunt, Axe, Spear, Bow, Shield, Sneak, Lockpicking, Traps, First Aid, Lore (identifying), Alchemy, Smithing, Carpentry, Masonry, Cooking, Farming, Tracking, Athletics, and one per school of magic.
  - **Only real use under real risk pays:**
    - combat skills, for hits on things that can hurt you;
    - Sneak, while hostiles who could notice you are near and don't (Skyrim's rule);
    - Athletics, for running while chased or while carrying;
    - crafting, for useful products, the first of each kind paying most.
  - **Repeats pay less:** the same act on the same target, or in the same spot, earns less each time. Oblivion's players farmed Acrobatics by jumping under a well; here there's nothing to farm.
  - **Books and trainers:** books are loot that multiply a skill's XP (Zomboid's way), and holdout trainers sell lessons up to a set level.
  - **Three mastery slots** (decided):
    - every skill rises to 7 freely;
    - three can go on to 10. A skill that has earned level 8 waits at 7 until you choose it as a mastery;
    - characters stay distinct (Ultima Online's 700-point cap did that with bookkeeping), and the next one after a death plays differently.
- **Character level** (D&D) comes from deeds:
  - what you bring home, which is most of it. Each find counts once: an item is marked as found in the world and pays the first time it reaches a holdout or your base, so selling and buying back earns nothing;
  - places found, people helped, quests, rare monsters and bosses, days survived;
  - ordinary kills, a little.
- **Each level offers three curated talents** from the class list, plus:
  - a little health, bounded: *(built 2026-11-10)* 5 points a level, so level 10 has 145 against 100. The 2–3× first written here was D&D's; wounds here are fixed amounts, so 250 health would shrug off a crowd;
  - more spell slots or faith.

  There are no trees of one-point wonders (Diablo II's lesson), and a holdout trainer re-picks your talents for a price, so a bad pick is never a trap. Levels run 1–10 in the open world; dungeons take it further.
- **No enemy level scaling** (Oblivion's lesson). The danger comes from:
  - the region, and how far it lies from the heartland;
  - the Long Dusk's spread over time;
  - later, dungeon depth.

  Getting stronger shows as the Hushlands becoming manageable, never as goblins in glass armour.

## Body, moodles and afflictions

- **The health panel** is Zomboid's: head, torso, arms, hands, legs and feet; scratches, lacerations, deep wounds, fractures, burns and bites; bleeding and pain; bandages, splints and stitches. Armour protects per body part, as Zomboid's clothing does.
- **Each kind of injury has one clear effect and one treatment,** and heals over days (Stoneshard's lesson):
  - arm: accuracy and swing speed;
  - leg: speed, and endurance per step;
  - head: aim and sight;
  - torso: pain and endurance.
- **Moodles** (Default): Zomboid's set, cut down to the ones that make you decide something, and merged with D&D's conditions:
  - needs and the body: hungry, thirsty, tired, exhausted, heavy load, bleeding, pain;
  - the weather: wet, cold, hot;
  - afflictions: poisoned, diseased, cursed;
  - the fight: prone, grappled;
  - good ones: blessed, inspired, well fed.
- **Dread is the one stat for the mind** (Zomboid's panic). Its thresholds are clear: shaky aim first, then frightened (D&D's condition). Light, company, food and rest ease it.
- **Needs make plans, not chores** (Default). When hunger and thirst fall too fast, survival turns into errands.
  - Eat two or three times a game day, drink about as often, and sleep once.
  - Each need feeds another system: cooking gives buffs, food spoils (a reason to go out), water sources are places on the map, and sleep needs a safe bed.
  - Consequences come on gradually.
- **Afflictions** are Zomboid's bite with a way out, one per region:
  - **ghoul fever** (Hushlands; the first): a ghoul's bite. A fever on the first day, weakness on the second, and on the third you die and rise as a revenant;
  - **venom** (Webwood): paralysis at once, sickness after;
  - **lycanthropy** (Wolfmark): stronger at night, and the full moon takes over;
  - **festering burns** (Cinder Reach);
  - **spirit-touch** (Weeping Moor): dread that won't fall; at worst, a ghost takes you over for a while.

  Cures are quests: a holdout's chapel (at a price), a strong enough cleric, a rare herb from somewhere unpleasant.
- **The dead get up** (Hushlands): unburned human dead rise after a while. Burn them (oil) or consecrate them (holy water, a cleric), or leave them and maybe meet them again. Zomboid players burn corpse piles against corpse sickness; here it's the corpses themselves that come back.
  - *(built 2026-11-10)* Every shambler and ghoul you put down gets up again 12 to 36 game hours later, once: put down a second time, it stays down. So there are two answers, a second fight or the oil. It gets up from its back, so there's a window to finish it. Burning takes a flask of lamp oil and a lit torch or candle held up. The fire goes from body to body, a tile a game minute, but not through walls, so a pile at your door burns from one flask. A burning body lights the dark round it for 20 game minutes, then it's ash. A patch has 6 to 24 flasks (eight seeds measured). Holy water and consecration come with the cleric.

## Senses: how they find you

- **Your signature:** at every moment the character gives off noise, light, movement (how fast and how heavy), scent (a trail the wind bends), heat and dread. Each family reads one or two of them. A HUD shows them (Thief's light gem, for every sense), and the bot gets them as inputs.
- **Sight:** the character sees in a cone (Zomboid's), and only what's lit or within darkvision. Outside it you see what you remember (fog of war), never a live monster.
- **Light:** every tile has a light level, from day, night, weather and being indoors. Dungeons range from lit to unnaturally dark (see Dungeons).
- **Light sources:**
  - a torch: burns about an hour, held in a hand (so no shield, and no hand free to shove off a hold), put out by rain and water, can be dropped or thrown to light a room;
  - *(review)* the trade has to be real in the numbers: by night you make out 2 tiles without a light and about 6 by a torch, and the dead see a lit you from 8 tiles instead of 1. A test holds the game's data to it. Still to do: a light's brightness as well as its reach, so a candle shows you less than a torch does (today any light shows you fully);
  - a lantern: burns oil, hangs from the belt, dimmer, survives a splash;
  - candles, braziers, a fire, and spells (a light cantrip; a fireball's flash).
- **Noise:** every action has a radius (walking in plate, a door, a fight, a shout, a Knock spell), like Zomboid's world sounds.
- **Movement:** running and heavy armour shake the ground; webs carry every touch.
- **Scent:** you leave a trail; blood makes it stronger; rain, rivers and wind change it; food stores smell too.
- **Heat:** your body, your clothes (wet means cold: hidden from heat-hunters, and on the way to hypothermia), your fires.
- **Dread:** rises in the dark, near the dead, alone and wounded; falls with light, company, music, food and rest.
- **Stealth:** be what they can't sense, and get behind them: a sneak attack.
- **Every sense is drawn** (a pillar; Mark of the Ninja drew every noise for the same reason: players couldn't make sense of the game without it). In the playfield:
  - noise rings, for each footstep, swing, door and spell;
  - your light's reach;
  - your scent trail, drifting with the wind;
  - a heat shimmer;
  - tremors running along webs;
  - your dread, darkening the edges of the screen;
  - each creature's sight cone and alert state: unaware, curious (?), hunting (!).

  Paused, a queued action shows its noise and light before you commit.

## Athletics and endurance

- **Two kinds of tiredness:**
  - endurance: seconds to minutes (fights, sprints, climbs);
  - fatigue: hours (sleep).

  Both are shown, and kept apart.
- **Endurance pays for everything active:** swings, shoves, running, climbing, casting. Heavy armour and what you carry raise the cost. Fighting and fleeing draw on the same bar, so what you spend on a fight you can't spend running away.
- **Exhaustion is dangerous but recoverable.** An empty bar means no running, slow weak swings and an easier grab, until you stop and get your breath back. It is never an instant death: when Breath of the Wild's stamina wheel ran out on a cliff, the fall felt arbitrary.
- **Movement:**
  - walk, run, sneak;
  - climb fences, windows and low walls (a timed action that costs endurance);
  - vault;
  - later, swim.
- **The ground matters:**
  - snow slows you and keeps your tracks;
  - a web shakes when you touch it;
  - what you carry raises every step's endurance, noise and tremor.

  Mud and deep water come with the Drowned Mire.
- **Fitness and Strength grow over days of real exertion,** as Zomboid's do: running while carrying, fighting, hauling. Returns diminish within a day, so a week of real use beats an hour of jumping on the spot.

## The world's rules

One table of how things act on each other, written before the content, so the game stays consistent and players can invent (Breath of the Wild's "chemistry engine"):

- **fire** burns wood, cloth and webs; it spreads with the wind, gives light, crackles and gives off heat;
- **water** puts out fire and torches, wets clothes (making you cold and washing away scent), and slows you;
- **wind** carries scent, sound and fire;
- **snow** keeps tracks;
- **salt and iron** stop ghosts;
- **silver** bites lycanthropes;
- **blood** draws wolves and ghouls;
- **the families** fight each other.

## Combat and magic

- **Fights are an economy, not a damage race** (Zomboid's lesson). Every swing costs endurance, makes noise, risks a wound and wears the weapon. Dark Souls makes each action a commitment for the same reason.
- **The verbs.** Each has a wind-up, a strike tick and a recovery, shown while paused (like Pillars of Eternity's recovery time):
  - attack;
  - heavy attack: a longer wind-up, and it knocks down;
  - shove: cheap; knocks back, and down with enough Strength;
  - finish: a blow on someone on the ground always lands, does double harm and doesn't wear the weapon (Zomboid's stomp). That's what a shove buys: 6 endurance now for a weapon that lasts;
  - brace, with a shield;
  - step back.
- **Weapons differ in kind, not just in numbers:**
  - reach, arc (one target or a sweep), wind-up, knockback, noise and wear;
  - blunt for skeletons, edges for flesh, points for armour, silver for werewolves and ghosts;
  - a spear holds a doorway; a mace is loud.
- **Engagement you can see.** While paused, a line runs from each adjacent enemy to you. Stepping away from an engaged enemy gives it a free swing or a chance to grab. Doorways and corridors matter because they cap the number of lines. (Pillars of Eternity's disengagement attack was invisible, and players couldn't tell why they'd been hit.)
- **Every attack has a tell,** set per family in the data as a wind-up, an arc, a reach, a pose and a ground mark:
  - the dead lean in and reach;
  - goblin archers draw with a visible aim line;
  - spiders rear up;
  - wolves crouch before a rush.

  No hit lands without one, and two lethal moves never land on the same tick, unless you let a crowd form.

  *(review)* **A tell must leave time for each answer,** or it's decoration. The answers are a shove (0.2 s), a strike that lands first (0.2 to 0.8 s by weapon) and a step out of reach (0.7 s, risking a parting grab). So no family's tell is shorter than about a second unless stepping back is meant to fail against it (a wolf's rush).
- **Light is a trade** (Darkest Dungeon's lesson). A torch shows threats early and steadies dread, but everything sees you. The dark gives you surprise (sneak attacks, crits), and gives it to them too.
- **Crowds are the danger.**
  - Hit chances and armour stay in a narrow band (bounded accuracy), so weak enemies stay dangerous in numbers.
  - Each adjacent attacker adds a chance to grab.
  - **A grab has a way out** (decided): struggle free (endurance) or shove (with a hand free). Each extra grabber makes it harder, and three grabbing at once, or an empty endurance bar, is the drag-down. Deaths stay readable ("I let three get round me") without becoming rare.
- **Spells are timed actions, stated by their limits** (Sanderson: limits are more interesting than powers). Shown while paused:
  - cast time, noise and light;
  - a cost: a slot, faith, a component;
  - line of sight;
  - concentration: a hit can break the cast.
- **Spells work on the sense fields:**
  - Silence kills sound in a zone;
  - Fog blocks sight;
  - Warmth and Chill fool the heat-hunters;
  - Scent-mask;
  - Light, to see by or to lure with;
  - Knock opens any lock, and (as in the SRD) the knock can be heard 300 feet away;
  - Ward: salt and words;
  - Fire burns webs, scares wolves, and spreads;
  - Frost, against the Cinder Reach.

  Fireball lights up the street, and everything on it hears it. Few spells, each a tool for some region's puzzle rather than one more damage number.
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
- **Sieges you can see coming give a base its purpose** (see Quests and events). Attackers path to the weakest door, and damage stays until you repair it.

## Monsters beyond the regions

- **Rare monsters** (Diablo's champions): an ordinary creature with 1–3 affixes and a name ("Grimtooth the Lantern-Eater"). Shared affixes: Frenzied, Thrice-Risen (gets up twice), Lantern-Eater (puts out lights near it), Bellwether (calls its whole family when it sees you), Ironhide, Spectral (walks through doors), Plaguebearer (afflicts on a hit), Hunter (tracks you by any sense). Family ones: a goblin Thief-Lord who robs your base, a pack alpha that sends the others around you, a spider that spins over doorways, a Gravecaller that raises corpses. They roam; holdouts post bounties on them. They drop better loot.
- **Revenants:** your dead characters, risen in their gear, with their class abilities: a named rare haunting the place where they fell. Kill it to get your things back. Stretch goal: it fights the way you did, from a bot trained on your own play.
- **Bosses:** one per region, in its deepest dungeon, each fought in the region's own terms (the tables above). Bosses drop at least one Very Rare item.

## Loot

- **Mundane items are most of the loot** (as in Zomboid): food, tools, bandages, materials, in qualities from crude to standard, fine and masterwork.
- **Magic items are few, and each one is better** (Diablo III's Loot 2.0). A magic item is an event, not noise.
  - A starting point for tuning: one early on, then one every day or two of play, with the rarer tiers farther out and deeper down.
  - They have D&D's rarities: Uncommon, Rare, Very Rare, Legendary, and Artifacts (boss uniques with a story).
  - Any of them can be **cursed**.
- **A short list of properties, each changing how you play** (Diablo IV's Season 4 cut the rest):
  - properties that work on the senses:
    - Muffled: your swings make less noise;
    - Shadowed: your light reaches less far;
    - Scentless;
    - Coldforged: you give off less heat;
    - Moonsilver: it bites werewolves;
    - Sure-footed: you send no tremors along webs;
  - trade-offs, such as Blazing: it lights the room, and you;
  - a few plain numbers.

  Every property reads in one line. They're rolled by rarity and by region.
- **No class bias in the world's containers** (decided). A cupboard holds what it would, whoever opens it. The holdouts make every find worth something instead: trade it, sell it, swap it. A class bias on magic items can be a sandbox option.
- **Identification:** a magic item is unknown until identified (the Lore skill, a scroll, a holdout's sage) or attuned, which may reveal a curse the hard way.
- **Attunement:** at most 3 attuned items (D&D), which caps power.
- **Pity for named items:** each boss kill or deep vault that doesn't drop its unique raises the chance next time, so a long drought can't happen.
- **Coin has sinks** (Path of Exile's lesson: money needs a use), so gold doesn't pile up with nothing to buy:
  - repairs, cures, training;
  - rebuilding holdouts;
  - building materials;
  - bribes.
- **Region tools and reagents:**
  - the tools: silvered weapons, salt, iron nails, holy water, fire oil, frost charms;
  - the reagents: the Wolfmark's silver, web silk, the moor's salt, ember cores, grave dust.

  They come from the holdouts that know them and from the regions themselves (see Crafting).
- **Drops:** creatures drop what they carry, plus their own table; rares and bosses roll higher.
- **Wear:** weight and condition on everything, repaired with Smithing, with blood, dirt and scorch marks on gear.
  - Mundane weapons wear and break, as in Zomboid: they're plentiful.
  - **Magic items never vanish.** At 0 condition one is broken, unusable until repaired, at a cost that grows with its rarity. Breath of the Wild's weapons broke so fast that players hoarded their best ones unused.
- **Containers:** cupboards, chests, barrels, bookshelves, sarcophagi, bodies; locks and traps on the good ones; now and then, a mimic.
- **Inventory** as in Zomboid: lists with weights, not a grid; bags with capacities; both hands (a torch takes one). Weight is noise, endurance and speed here, so what to carry is a real choice.

## Crafting

- **Crafting covers:**
  - building the base: carpentry, masonry, wards;
  - keeping you going: torches, bandages, splints, cooking, farming;
  - fixing and improving gear: Smithing, silvering a blade;
  - the region tools: salt lines, holy water, fire oil, frost charms.

  Alchemy makes potions, wizards scribe scrolls, and rogues set traps.
- **Crafting with reagents is the counterweight to luck** (Diablo IV's tempering, Path of Exile's orbs).
  - Holdout smiths and enchanters spend a region's reagents to add a property you choose: silvering, a salt-lined hem, cold-iron studs, a felt-wrapped grip for quiet.
  - The reagents are trade goods and crafting materials at once, so progress doesn't hang on luck alone.

## Quests and events

- **Quests** (Default: generated from the simulation first, a written arc later), mostly from holdouts:
  - bounties on rare monsters (they roam; the board says where they were seen last);
  - rescues: bring survivors home, to a holdout or to your base;
  - recoveries: an heirloom, a relic, a map, someone's body;
  - cures for an affliction (the quest starts when you're bitten);
  - supplies for a holdout running short, escorts for a caravan;
  - the arc: the wardstones.
- **Events:** each region's (the tables above), plus the realm's: blood moons, caravans, a holdout under siege (help it, or loot the ruin), rival survivors (bots: trade, avoid, or fight), the Long Dusk deepening.
- **A Dusk Director paces each region** (Left 4 Dead's AI Director, RimWorld's storytellers):
  - it reads the character (wounds, dread, supplies, recent fights) and the world (the Long Dusk, the holdouts);
  - it schedules events through build-up, peak and relax;
  - it eases off with something good (a caravan, a quiet night) when you're spent;
  - presets as sandbox options: Classic, Gentle, Random.
- **Sieges on a calendar as well** (decided; 7 Days to Die's blood moon):
  - the Tolling, heard from far off;
  - a raid's horns the night before;
  - a hunter's moon you can count down to.

  You know a siege is coming, so you build for it.

  *(built 2026-11-10)* **The Tolling** is on day 3, then every 5 days. At dawn on the day, word comes that the old chapel's bell will toll at 22:00, and the clock line counts down to it. At 22:00 it tolls for an hour, and every dead thing within 96 tiles walks to the chapel, beating down shut doors on the way. Until dawn they don't give up on anything they go after: be seen going into a house that night, and you're besieged in it until the door gives or the sun comes up. The abbey is the safe place to sit it out; your hamlet base is where it's tested.

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
- **A fair death has a readable cause:** tells before hits, grabs with a way out, no unseen instant kills. The recap names the moment it went wrong: "grabbed by a third shambler in the hall".
- **The world keeps it:** your body and gear lie where you fell, and you rise there as a revenant. Your base stays where it is.
- **The next character** starts at a holdout and can walk back to the old base. What carries over: the base and what's in it, the holdouts' memory of you, your map, the places found, what you've learned about the creatures.
- *(built 2026-11-10)* Every character has a name, and the death screen tells who it was, how long they lasted, how they died and what they carried.
  - **The body stays where it fell, in its gear.** Unless it's burned first, at midnight it rises as a revenant, with your name: twice a shambler's health, faster than your walk, and it doesn't wander from where it fell. Put it down and your things fall where it falls. Burn the body before midnight and it never rises; the gear falls where it lay.
  - **R brings the next survivor** to the abbey's gate, in the same world. Your home and stash carry over, and so does the map you remember; the levels and skills don't.
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
- **How it's made** (decided, `ARCHITECTURE.md`): 3D low-poly models under a fixed orthographic camera at Zomboid's angle (Zomboid draws 3D characters over 2D tiles), with free CC0 kits first.

## The core that has to be fun first

Before classes, spells, quests and other regions: is surviving the Hushlands tense and fun?

*(review)* **The order matters more than the list.** What's built is a set of rules with no reason to go out and nothing that lasts. So the first playable starts with the two things that turn rules into a game, and leaves the decoration for last (`ARCHITECTURE.md`, build step 6, has the slices):

1. saves: a game day is an hour, and a base means nothing in a world that's gone when you quit;
2. the stash and bring-it-home XP: the reason to go out;
3. the ghoul and its fever: the reason to run, and the first clock on a life;
4. the dead that get up, and burning them: the pressure you can push back;
5. the abbey, the Tolling, death and the revenant;
6. last, and only if the loop is already fun without them: magic items and identification, rare affixes, rain.

The first playable:

- the Hushlands as a small open map: a market town, farms, a stretch of forest, a river, the abbey holdout;
- Zomboid's day and night, and rain;
- base-building basics: board the windows, barricade the doors, a bed, storage, a fire;
- the dead: shamblers and ghouls, the Tolling (the first siege on the calendar), a handful of rare affixes, revenants;
- one class (Fighter), perhaps two (+ Cleric);
- combat: endurance shared with running, the verbs, tells, engagement lines, grabs with a way out;
- the senses drawn: noise rings, sight cones and alert states, your light's reach;
- loot: mostly mundane, a few magic items, identification, weight, containers;
- wounds with clear effects, bandages, a few moodles (hungry, thirsty, tired, bleeding, pain, dread, heavy load), needs that make plans, ghoul fever;
- skills that grow by real use; scavenge by day, hold by night, bring it home for XP; death and the revenant.

Then the Warren Hills, to prove the first pillar (enemies that see and think instead of hear), then classes and spells, holdout quests and events, more regions, dungeons, co-op and PvP.

## What the plan must meet

Worked out in `ARCHITECTURE.md`.

- The simulation is separate from the graphics; headless is first-class; a fixed tick (say 10 per game second); deterministic from a seed.
- Callable from Python as a Gymnasium environment, in-process and vectorisable: the RL stack is Python.
- An open world in chunks: full simulation near the character, a summary elsewhere, saved as it changes (bases, corpses, opened doors).
- A world generated from a seed out of hand-made pieces (decided: towns, buildings, holdouts and camps made by hand, placed on generated land).
- A renderer that can do isometric cut-away walls, day and night lighting and the vision cone (decided: Godot 4).
- A tile grid (Zomboid's world is tiles too) with 8-direction movement drawn smoothly: fast to simulate, and the 8 moves the bot already has. (Zomboid's own movement is free; this is a Default.)
- Creatures and their affixes, items and their properties, spells, buildings and regions in data files (as Zomboid's scripts are), so content is cheap and moddable.
- Licensing: the D&D 5e SRD is CC-BY 4.0: its monsters, spells and rules can be used with attribution. Monsters and names outside it (beholders, mind flayers, the Forgotten Realms) can't. Zomboid's art and code are off-limits; it's inspiration only.

## Open questions

None (2026-11-06). The engine and architecture questions are in `ARCHITECTURE.md`.

## References

The design research behind many of these choices, with its sources, is in `research/game_design.md`.

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
- Mark of the Ninja: every noise drawn, for a stealth game the player can read.
- Pillars of Eternity: real-time with pause, recovery times, and engagement (made visible here).
- Dark Souls: stamina as commitment.
- Diablo III and IV: fewer, better drops; a short list of properties; crafted properties beside random ones.
- Path of Exile: money that's also crafting.
- Breath of the Wild: stamina, durability's limits, and the chemistry engine.
- Kenshi, Ultima Online, the Elder Scrolls: learning by doing, specialisation, and the exploits to avoid.
- Left 4 Dead and RimWorld: a director that paces tension.
