# Game design research: what works, and what fits Duskhold

*2026-11-06. The user asked to "research good game design; fight mechanics, loot mechanics, athletics, leveling/skills, etc.", then "evaluate what might be good for our game". All 14 recommendations were accepted the same day, with the defaults on the four questions ("defaults on all four, fold them into the concept"), and are now in `CONCEPT.md`, which is the source of truth. This file keeps the reasoning and the sources (at the end).*

## The recommendations in brief

| # | Recommendation | Learned from | In the first playable? |
|---|---|---|---|
| 1 | Fights are an economy (endurance, wear, noise, wounds), not a damage race | Zomboid, Dark Souls | yes |
| 2 | Every threat can be read before it lands: wind-ups, engagement lines, noise rings, every sense drawn | Mark of the Ninja, telegraph design, Pillars of Eternity's critics | yes |
| 3 | Crowds are the danger; a grab has a way out | Zomboid, D&D 5e's bounded accuracy | yes |
| 4 | Fewer, better magic items; a short list of properties that change how you play | Diablo III's Loot 2.0, Diablo IV's Season 4 | later |
| 5 | Crafting with each region's reagents, as the counterweight to luck | Diablo IV's tempering, Path of Exile's currency | later |
| 6 | Durability that never makes you hoard: magic items break, they don't vanish | Breath of the Wild | partly |
| 7 | Weight and containers, not a grid | Zomboid, Tarkov | yes |
| 8 | Skill XP only for real use under real risk, paying less for repeats | Oblivion's exploits, Skyrim's sneak rule, Kenshi | yes |
| 9 | No enemy level scaling: regions, the Long Dusk and depth set the danger | Oblivion | yes |
| 10 | Three mastery slots instead of a point total; levels offer picks, not trees | Ultima Online, Diablo II | later |
| 11 | A Dusk Director paces events (build-up, peak, relax), with sieges you see coming | Left 4 Dead, RimWorld, 7 Days to Die | the Tolling only |
| 12 | Magic defined by its limits, working on the sense fields | Sanderson's laws, Breath of the Wild | later |
| 13 | Needs that make plans, not chores | PC Gamer's critique, The Long Dark | yes |
| 14 | Injuries with one clear effect each | Stoneshard, Zomboid | yes |

## Fighting

### What the research says

- **Zomboid makes combat an economy.** Every swing costs endurance, makes noise, risks injury and wears the weapon.
  - Fan guides put a heavy swing at roughly 12–18% of endurance, a shove at 4–7%, a stomp at 2–3%.
  - Shove, then stomp: the efficient solo pattern, which saves the weapon and stays quiet.
  - With multi-hit on, one swing strikes several zombies, but each costs its own endurance.
- **Dark Souls makes every action a commitment.** Attacks, blocks, rolls and sprints draw on one stamina bar, so each press spends something limited.
- **Pillars of Eternity** is real-time with pause, like Duskhold:
  - after each action comes a recovery time, set by armour and Dexterity;
  - stepping away from a melee enemy you're engaged with draws a free "disengagement attack", so movement has a cost.
  - That attack was invisible and worked outside the normal combat rules, and players couldn't tell why they'd been hit.
- **Telegraphs make difficulty fair.** A wind-up the player can read in time turns a hit into a lesson instead of a surprise. Designers specify each attack's rhythm:
  - how often an enemy threatens;
  - how long it telegraphs, and from which side;
  - whether two lethal moves can overlap;
  - what answer is expected.
- **Darkest Dungeon's light meter trades safety for reward.** Bright light improves scouting and the chance to surprise enemies, and spares stress; low light makes fights deadlier and loot richer.

### For Duskhold

- **Endurance pays for everything active:** swings, shoves, running, climbing, casting.
  - Heavy armour raises the cost.
  - An empty bar means exhaustion: slow, weak swings and no running, until you stop and recover.
  - Fighting and fleeing draw on the same bar, which is Zomboid's tension: what you spend on a fight you can't spend running away.
- **A small set of distinct combat verbs:**
  - attack;
  - heavy attack: a longer wind-up, and it knocks down;
  - shove: cheap; knocks back, and down with enough Strength;
  - finish: on someone on the ground; cheap, quiet, no wear;
  - brace, with a shield;
  - step back.

  Each has a wind-up, a strike tick and a recovery in the data (Pillars' recovery time), shown while paused.
- **Weapons differ in kind, not just in numbers:**
  - reach, arc (one target or a sweep), wind-up, knockback, noise and wear;
  - blunt for skeletons, edges for flesh, points for armour, silver for werewolves;
  - a spear holds a doorway; a mace is loud.
- **Engagement you can see.** A line runs from each adjacent enemy to you, drawn while paused. Stepping away from an engaged enemy gives it a free swing or a chance to grab. Doorways and corridors matter because they cap the number of lines.
- **Each family has its own tells**, set in the data as a wind-up, an arc, a reach, a pose and a ground mark:
  - the dead lean in and reach: a long wind-up, easy to read alone, hard to read in a crowd;
  - goblin archers draw with a visible aim line;
  - spiders rear up;
  - wolves crouch before a rush.

  No hit lands without a tell, and two lethal moves never land on the same tick unless the player let a crowd form.
- **Light is a trade** (Darkest Dungeon's lesson). A torch shows threats early and steadies dread, but everything sees you. The dark gives you surprise (sneak attacks, crits) and gives it to them too.

## Crowds and grabs

- **Bounded accuracy keeps weak enemies dangerous.** D&D 5e keeps attack bonuses and armour in a narrow band (the proficiency bonus only runs from +2 to +6), so ten goblins still hit a veteran. That's Zomboid's horde in rule form: keep hit chances and armour in a narrow band.
- **The danger is the second and third attacker.** One shambler is a chore; three around you are a crisis. Each adjacent attacker adds to the chance of a grab.
- **A grab is a state with a way out, not an instant death** (Default; Zomboid's drag-down is instant):
  - grabbed, you struggle free (costing endurance) or shove (with a hand free);
  - each extra grabber makes it harder;
  - three grabbing at once, or an empty endurance bar, means the drag-down.

  Deaths stay readable ("I let three get round me") without becoming rare.

## Stealth, and being able to read the senses

- **Mark of the Ninja draws every noise the enemies will hear** as a spreading ring that shows exactly how far it carries. The designers added it because players couldn't make sense of the game without it. Their principle: stealth is about the player's choices, so the player needs complete information about the systems.
- **In Duskhold, readability is a pillar, not polish.** Every region hunts by a different sense, and you can only play around a sense you can see. In the playfield:
  - noise rings: each footstep, swing, door and spell;
  - your light's reach;
  - your scent trail, drifting with the wind (Wolfmark);
  - a heat shimmer (Cinder Reach);
  - tremors running along webs (Webwood);
  - your dread, darkening the screen's edges (Weeping Moor);
  - each creature's sight cone and alert state: unaware, curious (?), hunting (!).
- **Paused, a queued action shows its noise and light before you commit.** The signature HUD stays: Thief's light gem, for every sense.
- **The bot later reads the same numbers** (`ARCHITECTURE.md`), so the game and the bot share one truth.

## Loot

### What the research says

- **Diablo III's Loot 2.0** (2014) fixed a launch that filled bags with items nobody wanted:
  - fewer drops, each one better;
  - "smart loot", skewed to the class that found it;
  - the auction house removed.

  Progress became steadier, at the price of fewer jackpot moments.
- **Diablo IV's Season 4, "Loot Reborn"** (2024):
  - shrank the pool of item properties, cutting the narrow, conditional ones that made sorting a chore;
  - added "greater" properties, 1.5× as strong;
  - added a crafted path beside the random one: tempering adds chosen properties from recipes, and masterworking upgrades them.
- **Path of Exile has no gold.** Its currency items are crafting tools (orbs that reroll, add or upgrade an item's properties), so money always has a use and gets used up, which keeps inflation down.
- **Bad luck protection** caps the worst run of luck while keeping the thrill of a rare drop. A pity timer guarantees the drop after a set number of misses; Hearthstone's card packs guarantee a legendary within 40. Unpredictable rewards are the strongest hook there is, the same one a slot machine uses. That's worth using carefully in a single-player game with nothing to sell.
- **Extraction games** (Escape from Tarkov, Dark and Darker) give gear meaning through loss: dying costs what you carried. That "gear fear" makes loot feel owned. Take too much of the risk away and the hook goes with it.

### For Duskhold

- **Fewer, better magic items** (Loot 2.0's lesson).
  - Most loot stays mundane and useful, as in Zomboid: food, tools, bandages, materials.
  - A magic item is an event, not noise.
  - A starting point for tuning: one early on, then one every day or two of play, with the rarer tiers farther out and deeper down.
- **A short list of item properties, each changing how you play** (Season 4's lesson). Properties that work on the senses suit this game:
  - Muffled: your swings make less noise;
  - Shadowed: your light reaches less far;
  - Scentless;
  - Coldforged: you give off less heat;
  - Moonsilver: it bites werewolves;
  - Sure-footed: you send no tremors along webs.

  Add trade-offs (Blazing: it lights the room, and you) and a few plain numbers. Every property reads in one line; none applies only on Tuesdays.
- **No "smart loot" in the world's containers** (Default). A cupboard doesn't know your class, and Zomboid's realism is part of the game. The holdouts make every find worth something instead: trade it, sell it, swap it. A gentle class bias on magic items could be a sandbox option.
- **Crafting with the regions' reagents** (tempering and Path of Exile's orbs): the Wolfmark's silver, web silk, the moor's salt, ember cores, grave dust.
  - Holdout smiths and enchanters spend them to add a chosen property: silvering, a salt-lined hem, cold-iron studs, a felt-wrapped grip for quiet.
  - The reagents are trade goods and crafting materials at once, so money has a use and progress doesn't hang on luck alone.
- **Pity for named items.** Each boss kill or deep vault that doesn't drop its unique raises the chance next time. You never see it working, but a 30-run drought can't happen.
- **Coin needs sinks** (Path of Exile's lesson): repairs, cures, training, rebuilding holdouts, building materials, bribes. Gold piling up with nothing to buy would break bring-it-home XP.
- **Bring-it-home XP counts each find once.** An item is marked "found in the world", and XP is paid the first time it reaches a holdout or your base, so selling something and buying it back earns nothing.
- **The stakes of dying are already right.** Your revenant keeps your gear: the gear fear of an extraction game, with a way to win it back (Dark Souls' bloodstain, made harder).

## Inventory and durability

- **Breath of the Wild's weapons broke so often that players hoarded their best ones unused.** Tears of the Kingdom softened it with Fuse (sticking materials onto weapons) but kept the breaking. The lesson: durability must never make the best thing too precious to use.
- **For Duskhold:**
  - **mundane weapons** wear and break, as in Zomboid: they're plentiful, and Smithing and materials repair them;
  - **magic items never vanish.** At 0 condition an item is broken, unusable until repaired, at a cost that grows with its rarity;
  - **condition is shown** on the item and during the fight;
  - **finishing** someone on the ground spares the weapon (Zomboid's stomp).
- **Weight and containers, not a grid** (Default). A grid (Resident Evil 4's attaché case, Tarkov's rigs) is a packing puzzle. Weight is a choice about what to carry, and here weight is also noise, endurance and speed. Keep Zomboid's lists with weights, bags with capacities, and both hands.

## Athletics and endurance

- **Breath of the Wild's stamina wheel** pays for sprinting, climbing, swimming and gliding. When it empties, Link is exhausted: slow, and unable to climb or glide. Players praised climbing on a budget, but losing your grip on an empty wheel, often fatally, could feel arbitrary.
- **Zomboid makes Fitness and Strength slow on purpose.** They take about 487,500 XP to reach level 10, against about 32,775 for other skills. Exercise and exertion train them; no books exist for them, and the Fast Learner trait doesn't apply.
- **Kenshi trains each stat by its own use:**
  - athletics by running unencumbered;
  - strength by heavy loads and blunt weapons;
  - toughness by taking beatings.

  Players get their own characters beaten up on purpose.
- **For Duskhold:**
  - **two kinds of tiredness:** endurance (seconds to minutes: fights, sprints, climbs) and fatigue (hours: sleep). Both are in the concept already; keep them separate and both visible.
  - **exhaustion is dangerous but recoverable:** no running, slow swings, a higher chance of being grabbed. Never an instant death, Breath of the Wild's lesson.
  - **movement verbs:** walk, run, sneak, climb (fences, windows and low walls: a timed action that costs endurance), vault, and later swim.
  - **the ground matters:**
    - snow slows you and keeps your tracks;
    - a web you walk into shakes;
    - mud and water come with the Drowned Mire;
    - what you carry raises each step's endurance, noise and tremor.
  - **Fitness and Strength grow over days of exertion:** running while carrying, fighting, hauling. Returns diminish within a day (Zomboid's "regularity"), so a week of real use beats an hour of jumping on the spot.

## Levelling and skills

### What the research says

- **Learning by doing invites grinding.**
  - Oblivion players jumped with their heads under a well's corner, or in low-ceilinged rooms, to farm Acrobatics and Athletics.
  - Enemies scaled with the player's level, so levelling the "wrong" skills made the game harder.
  - In the Remaster, one player pushed both skills past 15,000 and leapt across the map in single bounds.
- **Skyrim ties Sneak XP to risk.** It comes mainly from staying unseen near hostiles, and from sneak attacks.
- **Zomboid multiplies XP with skill books** for its crafting and survival skills. Occupations and traits set the starting levels; Fast Learner is ×1.3, Slow Learner ×0.7.
- **Ultima Online capped each character's skills at a total of 700 points.** Specialising was the game: each skill is set to raise, lower or lock.
- **Diablo II's skill trees had flaws:**
  - skills worth one point and no more (Teleport, Static Field);
  - points spent on skills nobody wanted, just to reach the good ones;
  - synergies, added later, narrowed the builds further.
- **Vertical and horizontal progress:** bigger numbers (vertical) feel like growth; new options (horizontal) avoid power creep and add depth. Most good games use both.
- **Old-school D&D paid 1 XP per gold piece of treasure,** which made monsters obstacles rather than piñatas. In the 1981 Basic rules, three-quarters or more of a character's XP came from treasure.

### For Duskhold

- **Skill XP only for real use, under real risk** (Default):
  - combat skills for hits on things that can hurt you;
  - Sneak while hostiles who could notice you are near and don't (Skyrim's rule);
  - Athletics for running while chased or while carrying;
  - crafting for useful products, the first of each kind paying most.
- **Repeats pay less.** Doing the same thing to the same target or in the same spot earns less each time, so there's nothing to farm at a well.
- **Books and trainers** (Zomboid's way):
  - books are loot that multiply a skill's XP;
  - holdout trainers sell lessons up to a set level;
  - a class learns its own skills faster (×1.5, like an occupation).
- **Mastery slots instead of a point total** (Default). Every skill rises to 7 freely, and three can be mastered (levels 8–10), chosen as each first reaches 8.
  - Characters stay distinct, as Ultima Online's cap made them, without the raise/lower/lock bookkeeping.
  - The next character after a death plays differently.
- **Character levels come from deeds:**
  - bring-it-home XP, each find counted once;
  - places found and people helped;
  - quests, rares and bosses;
  - ordinary kills a little.

  This is old-school D&D's rule, and it removes the incentive to "fight until bitten", which we watched ZomboidRL learn.
- **A level is a choice, not only numbers.**
  - Each level offers three curated talents from the class list (horizontal), plus a little health (vertical, but bounded: 2–3× at level 10).
  - No trees of one-point wonders.
  - A holdout trainer re-picks talents for a price, so a bad pick is never a trap.
- **No enemy level scaling** (Oblivion's lesson). Danger comes from:
  - the region, and how far it lies from the heartland;
  - the Long Dusk spreading over time;
  - later, dungeon depth.

  Getting stronger shows as the Hushlands becoming manageable, never as goblins in glass armour.

## Classes

- **Each class plays a different part of the system:**
  - Fighter: engagement. Holds doorways, braces, taunts, shrugs off grabs.
  - Rogue: darkness and the signature. Quiet, unseen, sneak attacks, reads tracks.
  - Cleric: dread and the dead. Turns undead, wards, cures, steadying light.
  - Wizard: the fields themselves. Light, sound, heat, fog.
- **Signatures, not trees.** A class is a starting kit, faster learning in its own skills, its talent list and one signature mechanic. Anyone can learn any skill, more slowly.

## Magic

- **Brandon Sanderson's laws of magic:**
  1. magic can solve problems only as far as the audience understands its rules;
  2. its limits are more interesting than its powers;
  3. deepen what exists before adding something new.
- **Breath of the Wild's "multiplicative gameplay":** objects react to the player and to each other by universal rules. A "chemistry engine" governs fire, ice, wind and materials.
- **In Duskhold, spells work on the sense fields:**
  - Silence kills sound in a zone: walk past the dead, or smother a goblin's horn;
  - Fog blocks sight;
  - Warmth and Chill fool the heat-hunters;
  - Scent-mask;
  - Light, to see by or to lure with;
  - Knock, which is loud;
  - Ward: salt and words;
  - Fire burns webs, scares wolves, and spreads;
  - Frost, against the Cinder Reach.

  Few spells, each a tool for some region's puzzle rather than one more damage number.
- **Each spell is stated by its limits,** shown while paused: cast time, noise, light, component, slot or faith, line of sight, concentration.

## The world's rules: a chemistry table

The senses and the elements need one table of how things act on each other, written before the content, so the game stays consistent and players can invent:

- fire burns wood, cloth and webs; it spreads with the wind, gives light, crackles and gives off heat;
- water puts out fire and torches, wets clothes (making you cold and washing away scent), and slows you;
- wind carries scent, sound and fire;
- snow keeps tracks;
- salt and iron stop ghosts;
- silver bites lycanthropes;
- blood draws wolves and ghouls;
- the families fight each other.

Breath of the Wild built this as an engine; in Duskhold it's data plus a few systems.

## Pacing, events and difficulty

- **Left 4 Dead's AI Director** watches the players' stress and cycles through build-up, peak and relax, so every big fight has a run-up and an afterwards.
- **RimWorld's "storytellers"** choose events for the story they'll make:
  - Cassandra Classic: rising tension, with breaks;
  - Phoebe Chillax: more quiet time between events;
  - Randy Random: no arc at all.
- **7 Days to Die's blood moon** comes every seventh night, and everyone knows when, so they build for it. The horde takes the weakest way into a base and digs through. Players learn to fight it from a separate base.
- **For Duskhold, a Dusk Director per region:**
  - it reads the character (wounds, dread, supplies, recent fights) and the world (the Long Dusk, the holdouts);
  - it schedules events through build-up, peak and relax;
  - it eases off with something good (a caravan, a quiet night) when you're spent;
  - presets as sandbox options: Classic, Gentle, Random.
- **Sieges you can see coming give base-building its purpose:**
  - the Tolling, heard from far off;
  - a raid's horns the night before;
  - a hunter's moon on the calendar.

  Attackers path to the weakest door, as the core's shamblers already do (build step 2), and damage stays until you repair it.

## Survival needs

- **When hunger and thirst fall too fast, survival turns into chores** (PC Gamer). A hunger meter is really a countdown to starving. The decision it should drive is where to go today. The Long Dark builds most of its tension from cold, hunger and weather rather than from a horde at your back.
- **For Duskhold** (Default):
  - eat two or three times a game day, drink about as often, and sleep once;
  - every need feeds another system: cooking gives buffs, food spoils (a reason to go out), water sources are places on the map, and sleep needs a safe bed;
  - consequences come on gradually, shown as moodles.

## Injuries and the mind

- **Stoneshard injures body parts, each with its own effect:**
  - a head injury makes you dizzy;
  - a broken arm costs accuracy;
  - a leg injury slows you;
  - a torso wound adds pain.

  Each needs its own treatment (splint, bandage, cauterising), and pain comes from damage and from healing. Its psyche system (sanity and morale) can lead to breakdowns.
- **For Duskhold:**
  - **each injury type has one clear effect and one treatment,** and heals over days:
    - arm: accuracy and swing speed;
    - leg: speed, and endurance per step;
    - head: aim and sight;
    - torso: pain and endurance.
  - **dread is the one stat for the mind.** Its thresholds are clear (shaky aim first, then frightened), and light, company, food and rest ease it.
  - **few moodles, each one a decision.**

## Death and legacy

- **Keep what the concept has:** permadeath; your revenant keeps your gear; the base and the holdouts remember you.
- **A fair death has a readable cause:** tells before hits, grabs with a way out, no unseen instant kills.
- **The death recap names what killed you and the moment it went wrong:** "grabbed by a third shambler in the hall".

## What it changed in CONCEPT.md

All of the following was folded in on 2026-11-06:

- **Combat:** the verb set; engagement lines; grabs with a way out; tells per family.
- **A pillar:** every sense drawn in the playfield.
- **Loot and items:**
  - fewer, better magic items;
  - the property list, with sense properties;
  - no smart loot in containers;
  - crafting with region reagents;
  - pity for uniques;
  - coin sinks;
  - bring-it-home XP counted once per find;
  - magic items break rather than vanish.
- **Skills and levels:**
  - XP gated by risk, with repeats paying less;
  - three mastery slots;
  - talents as curated picks, with a paid re-pick;
  - no level scaling.
- **Classes:** each one tied to a part of the system.
- **Magic:** spells as tools on the fields, defined by their limits.
- **The world:** the chemistry table.
- **Pacing:** the Dusk Director with presets, and sieges you see coming.
- **The body:** need rates and injury effects as above.

## Questions for the user (answered 2026-11-06: the defaults on all four)

1. **Grabs:** a way out. Not Zomboid's instant drag-down.
2. **Skill caps:** three mastery slots. Not Ultima Online's point total, and not no cap.
3. **Loot bias:** no class bias in the world's containers. Not Diablo's smart loot.
4. **Sieges:** on the calendar as well as the Director's events. Not the Director only.

## Sources

- Zomboid: [combat basics](https://projectzomboid.wiki/guides/combat-basics/), [multi-hit](https://pzfans.com/multi-hit-mechanic-in-zomboid-benefits-and-drawbacks/), [shove tactics](https://pzfans.com/knock_tactics_breakdown_build_41_vs_42_zomboid_secrets/), [skills](https://pzfans.com/skills/), [fitness](https://pzfans.com/project-zomboid-fitness-guide/)
- Dark Souls: [stamina](https://darksouls.wiki.fextralife.com/Stamina); [Stamina in Death of a Wish](https://www.gamedeveloper.com/design/stamina-in-death-of-a-wish)
- Pillars of Eternity: [PC Gamer combat guide](https://www.pcgamer.com/au/pillars-of-eternity-a-beginners-guide-to-combat), [Josh Sawyer on attack recovery](https://forums.obsidian.net/topic/68171-josh-sawyer-visually-explains-attack-recovery/page/4), [RPG Codex interview](https://rpgcodex.net/content.php?id=9547)
- Telegraphs: [Designing for difficulty: readability in ARPGs](https://www.gamedeveloper.com/game-platforms/designing-for-difficulty-readability-in-arpgs), [how to design enemy attack telegraphs](https://bugnet.io/blog/how-to-design-enemy-attack-telegraphs)
- Darkest Dungeon: [light meter](https://darkestdungeon.wiki.gg/wiki/Light_Meter)
- Mark of the Ninja: [Reinventing stealth in 2D](https://gamedeveloper.com/design/reinventing-stealth-in-2d-with-i-mark-of-the-ninja-i-), [five stealth design rules](https://gamedeveloper.com/design/-i-mark-of-the-ninja-i-s-five-stealth-design-rules), [GDC talk](https://www.gdcvault.com/play/1017791/Of-Choice-and-Breaking-New), [Destructoid interview](https://www.destructoid.com/interview-defining-2d-stealth-in-mark-of-the-ninja/)
- Diablo: [Loot 2.0](https://di.diablowiki.net/Loot_2.0), [Shacknews on patch 2.0.1](https://www.shacknews.com/article/83266/diablo-3-patch-201-deploys-with-loot-20), [GameSpot on Reaper of Souls](https://www.gamespot.com/articles/breathing-life-back-into-diablo-3-with-reaper-of-souls/1100-6417912/); [Season 4 tempering and masterworking](https://www.pcgamer.com/games/action/diablo-4-tempering-manuals-masterworking), [why the rework mattered](https://www.icy-veins.com/d4/news/why-diablo-4s-tempering-masterworking-rework-is-game-changer/); [Diablo II synergies](https://diablo.fandom.com/wiki/Synergy), [a critique of Diablo II-style trees](https://mail.rpgcodex.net/forums/threads/a-critique-of-diablo-2-style-skills-trees.151048)
- Path of Exile: [currency guide](https://www.exitlag.com/blog/path-of-exile-currency/)
- Bad luck protection: [pity systems](https://track360.io/glossary/pity-system)
- Extraction games: [a history of extraction shooters](https://www.gfinityesports.com/article/history-of-extraction-shooters), [Tarkov review](https://www.gamesradar.com/games/fps/escape-from-tarkov-review)
- Breath of the Wild: [durability in Tears of the Kingdom](https://www.denofgeek.com/games/tears-of-the-kingdoms-weapon-durability-explained/), [AV Club on Fuse](https://www.avclub.com/weapons-still-break-constantly-in-tears-of-the-kingdom-but-fusing-makes-it-a-little-better), [stamina](https://www.zeldadungeon.net/wiki/Stamina), [GDC 2017 talk](https://www.thumbsticks.com/gdc-17-breaking-conventions-breath-of-the-wild), [Engadget on the talk](https://www.engadget.com/2017/03/12/breath-of-the-wild-gdc-talk/)
- Elder Scrolls: [Oblivion Acrobatics](https://content1.uesp.net/wiki/Oblivion:Acrobatics), [PC Gamer on Oblivion's levelling](https://pcgamer.com/games/the-elder-scrolls/oblivion-remastered-leveling-explained-skills-list), [GamesRadar on the 15,000 player](https://www.gamesradar.com/games/the-elder-scrolls/oblivion-remastered-player-pours-15-000-points-into-athletics-and-acrobatics-letting-them-leap-across-the-entire-map-in-a-single-bound); [Skyrim stealth](https://en.uesp.net/wiki/Skyrim:Stealth)
- Kenshi: [Athletics](https://kenshi.wiki/w/Athletics), [Strength](https://kenshi.wiki/w/Strength)
- Ultima Online: [skill cap](https://uoguide.com/Skill_Cap), [skills, stats and attributes](https://uo.com/wiki/ultima-online-wiki/player/stats/skills-stats-and-attributes/)
- Progression: [vertical and horizontal progression](https://www.engadget.com/2014/02/05/mmo-mechanics-comparing-vertical-and-horizontal-progression), [Dissecting progress](https://www.gamedeveloper.com/design/dissecting-progress)
- Old-school D&D: [XP for gold](https://kateplays.substack.com/p/xp-for-gold), [why D&D characters get tons of gold](https://dmdavid.com/tag/why-dd-characters-get-tons-of-gold-and-nowhere-to-spend-it/), [a primer on XP in the 1e DMG](https://www.enworld.org/goto/post?id=8354785)
- Sanderson: [the second law](https://www.brandonsanderson.com/sandersons-second-law/), [the three laws](https://coppermind.net/wiki/Sanderson%27s_Laws_of_Magic)
- Pacing: [Left 4 Dead](https://en.wikipedia.org/wiki/Left_4_Dead), [Left 4 Dead at GDC](https://www.shacknews.com/article/57892/left-4-dead-at-gdc); [RimWorld](https://en.wikipedia.org/wiki/RimWorld); [7 Days to Die structures](https://7daystodie.wiki.gg/wiki/Building_Structures), [horde bases](https://www.ibtimes.com/7-days-die-how-make-good-horde-base-3473781)
- Survival: [how survival games get hunger and thirst wrong](https://pcgamer.com/how-survival-games-get-hunger-and-thirst-wrong-and-how-to-fix-it)
- Stoneshard: [injuries and pain](https://stoneshard.com/wiki/Injuries_%26_Pain), [PC Gamer on Stoneshard](https://pcgamer.com/wounded-intoxicated-and-covered-in-leeches-a-normal-day-in-stoneshard), [the psyche system](https://www.gamebanshee.com/k4q3r)
