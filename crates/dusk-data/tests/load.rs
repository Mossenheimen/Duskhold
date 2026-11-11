//! Loading the data: the game's own, the units it turns into the core's, and what a mistake
//! looks like.

use std::path::PathBuf;

use dusk_data::{Side, default_dir, doors_on, load};

const RULES: &str = r#"
[world]
active_radius = 2
repath_after = 1.5
[sound]
wall = 4
shut_door = 2
shut_window = 1
[character]
walk = 1.5
run = 3.25
walk_noise = 3
run_noise = 8
path_budget = 1000
health = 100
carry = 15.0
[items]
take = 0.4
per_kg = 0.1
drop = 0.2
wield = 0.5
unwield = 0.3
light = 1.5
douse = 0.5
[endurance]
run = 4.0
walk = 3.0
rest = 10.0
recover_at = 30
[attack]
wind_up = 0.4
recovery = 0.6
endurance = 9
hit = 0.85
damage = [8, 12]
noise = 4
[shove]
wind_up = 0.2
recovery = 0.4
endurance = 6
knock_down = 0.4
noise = 2
[door]
time = 0.5
noise = 3
strength = 60
thump_noise = 8
[wounds]
scratch = { damage = 8, bleed = 0.0 }
laceration = { damage = 15, bleed = 20.0 }
bite = { damage = 30, bleed = 40.0 }
bleeding = 0.5
[grab]
chance = 0.15
per_extra = 0.15
tired = 0.2
parting = 0.25
bite = 0.35
drag_down = 3
struggle = { wind_up = 0.5, recovery = 0.2, endurance = 12, hit = 0.7, noise = 3 }
struggle_per_extra = 0.2
[clock]
rate = 12
start = 6.5
[hunger]
rise = 6.0
levels = [30, 55, 85]
recovery = 0.75
harm = 2.0
[thirst]
rise = 9.0
levels = [25, 50, 80]
recovery = 0.5
harm = 4.0
[fatigue]
rise = 4.0
rest = 12.5
levels = [45, 70, 90]
slow = [1.25, 1.5]
[sleep]
lie_down = 1.0
needs = 0.5
clear = 10
[healing]
health = 1.0
asleep = 3
scratch = 12.0
laceration = 48.0
bite = 96.0
[water]
drink = 3.0
[day]
dawn = 5.0
sunrise = 7.0
sunset = 19.5
dark = 21.0
night = 25
indoors = 0.5
[sight]
range = 16
cone = 180
beside = 1
[skills]
levels = [5, 20, 50, 100, 200, 400, 700, 1100, 1600, 2500]
free_cap = 6
masteries = 2
repeat = 0.5
forget = 2.0
spot = 6
chased = 10
loaded = 0.4
[skills.xp]
hit = 1.5
down = 0.25
run = 0.1
bandage = 2.0
board = 1.5
[skills.level]
hit = 0.02
damage = 0.1
endurance = 0.05
wear = 0.1
run_speed = 0.03
run_endurance = 0.05
bandage = 0.08
board_time = 0.04
board_strength = 0.2
[deeds]
levels = [20, 50, 100, 200, 350, 550, 800, 1100, 1500]
health = 4
stash = 2.0
home = 6
[fever]
stage = 12.0
thirst = 2.0
recovery = 0.25
harm = 6.0
"#;

const SHAMBLER: &str = r#"
[[creature]]
id = "shambler"
walk = 0.5
run = 0.8
sight = 8
cone = 90
think_every = 5
forget_after = 30.0
wander = 0.1
path_budget = 400
health = 30
down = 4.0
attack = { wind_up = 0.8, recovery = 0.8, hit = 0.5, noise = 3 }
wounds = { scratch = 0.7, laceration = 0.25, bite = 0.05 }
thump = [2, 4]
darkvision = 2
night_speed = 1.5
"#;

const FIXTURES: &str = r#"
[[fixture]]
id = "tree"
solid = true
opaque = true
[[fixture]]
id = "bed"
solid = true
bed = true
"#;

const PREFABS: &str = r#"
[legend]
"." = { ground = "floor" }
"b" = { ground = "floor", fixture = "bed" }
[[prefab]]
id = "hut"
tags = ["house"]
front = "south"
grid = """
+-+-+
|b .|
+-+d+
"""
"#;

const REGION: &str = r#"
id = "test"
name = "Test"
[patch]
chunks = [2, 2]
[terrain]
hill_size = 40.0
pond_below = 0.2
woods_from = 0.6
woods_thick = 0.8
woods_trees = 0.5
field_trees = 0.01
[road]
width = 3
bend_every = 16
sway = 4
start = 4
[hamlet]
lots = ["house"]
gap = 2
setback = 2
yard = 1
[dead]
kinds = [{ creature = "shambler", share = 1.0 }]
per_chunk = 1.0
per_house = [0, 1]
clear_start = 8
"#;

/// A data folder in the temp directory, from (path, text) pairs.
fn folder(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dusk-data-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (path, text) in files {
        let file = dir.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }
    dir
}

fn test_files(prefabs: &str) -> Vec<(&'static str, &str)> {
    vec![
        ("rules.toml", RULES),
        ("creatures/dead.toml", SHAMBLER),
        ("fixtures.toml", FIXTURES),
        ("prefabs/huts.toml", prefabs),
        ("regions/test.toml", REGION),
    ]
}

#[test]
fn the_games_data_loads() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    // The shambler is the first creature, and the Hushlands' dead are shamblers.
    let shambler = &data.defs.creatures[0];
    let s = shambler.stats;
    assert_eq!(shambler.id, "shambler");
    assert_eq!((s.speed.walk, s.speed.run, s.night_speed), (500, 800, 1250));
    assert_eq!((s.sight, s.darkvision, s.groan), (8, 1, 10));
    assert_eq!(
        (s.think_every, s.forget_after, s.path_budget),
        (5, 300, 400)
    );
    assert_eq!((s.health, s.down, s.attack.wind_up), (300, 40, 12));
    let kinds = &data.region("hushlands").unwrap().dead.kinds;
    assert_eq!(
        data.defs.creature_kind(&kinds[0].creature),
        Some(dusk_core::ActorKind::Creature(0))
    );
    let region = data.region("hushlands").expect("the Hushlands");
    assert!(!data.prefabs.is_empty() && !data.fixtures.is_empty());
    for tag in &region.hamlet.lots {
        assert!(data.prefabs.iter().any(|p| p.tags.contains(tag)), "{tag}");
    }
    for p in &data.prefabs {
        assert!(
            !doors_on(&p.grid, p.front).is_empty(),
            "{} has a front door",
            p.id
        );
    }
}

#[test]
fn a_designers_units_become_the_cores() {
    let data = load(&folder("units", &test_files(PREFABS))).unwrap_or_else(|e| panic!("{e}"));
    let r = data.rules;
    assert_eq!(
        (r.character_speed.walk, r.character_speed.run),
        (1500, 3250)
    );
    let shambler = &data.defs.creatures[0];
    assert_eq!(
        (shambler.id.as_str(), data.defs.creatures.len()),
        ("shambler", 1)
    );
    let s = shambler.stats;
    assert_eq!((s.speed.walk, s.speed.run), (500, 800));
    assert_eq!(s.cone.cos, 7071, "90° across is 45° either side");
    assert_eq!(s.forget_after, 300, "30 seconds in ticks");
    assert_eq!(r.repath_after, 15);
    assert_eq!(s.wander, 1000, "a tenth, out of 10,000");
    let hut = data.prefab("hut").unwrap();
    assert_eq!(hut.front, Side::South);
    let bed = hut.grid.cell(0, 0).0.expect("a tile");
    assert_eq!(data.fixture(bed.fixture).unwrap().id, "bed");
    assert!(bed.solid && !bed.opaque && bed.roof);
}

#[test]
fn a_mistake_says_where_and_what() {
    let typo = PREFABS.replace("fixture = \"bed\"", "fixture = \"bedd\"");
    let err = load(&folder("typo", &test_files(&typo))).unwrap_err();
    assert!(err.file.ends_with("huts.toml"), "{err}");
    assert!(err.message.contains("\"bedd\""), "{err}");

    let no_door = PREFABS.replace("+-+d+", "+-+-+");
    let err = load(&folder("no-door", &test_files(&no_door))).unwrap_err();
    assert!(err.message.contains("no door on its front"), "{err}");
}

#[test]
fn a_door_onto_furniture_is_refused() {
    let blocked = PREFABS.replace("|b .|\n+-+d+", "|. b|\n+-+d+");
    let err = load(&folder("blocked", &test_files(&blocked))).unwrap_err();
    assert!(err.message.contains("opens onto something solid"), "{err}");
}

#[test]
fn the_fights_numbers_become_the_cores() {
    let data = load(&folder("fight", &test_files(PREFABS))).unwrap_or_else(|e| panic!("{e}"));
    let (c, e) = (data.rules.combat, data.rules.endurance);
    assert_eq!(c.character_health, 1000, "100 points");
    assert_eq!(
        (c.attack.wind_up, c.attack.recovery),
        (4, 6),
        "seconds in ticks"
    );
    assert_eq!((c.attack.damage, c.attack.hit), ((80, 120), 8500));
    assert_eq!(c.knock_down, 4000);
    let s = data.defs.creatures[0].stats;
    assert_eq!((s.health, s.down, s.attack.wind_up), (300, 40, 8));
    assert_eq!(s.wounds, [7000, 2500, 500]);
    assert_eq!((c.wounds[1].damage, c.wounds[1].bleed), (150, 200));
    assert_eq!(
        (c.bleed_every, c.bleed_damage),
        (10, 5),
        "half a point a second"
    );
    assert_eq!((e.run, e.walk, e.rest, e.recover_at), (4, 3, 10, 300));

    let lopsided = SHAMBLER.replace("bite = 0.05", "bite = 0.5");
    let mut files = test_files(PREFABS);
    files[1].1 = &lopsided;
    let err = load(&folder("odds", &files)).unwrap_err();
    assert!(err.message.contains("add up to 1"), "{err}");
}

#[test]
fn grabs_and_doors_become_the_cores() {
    let data = load(&folder("grab", &test_files(PREFABS))).unwrap_or_else(|e| panic!("{e}"));
    let c = data.rules.combat;
    assert_eq!((c.grab, c.grab_per_extra, c.grab_tired), (1500, 1500, 2000));
    assert_eq!((c.parting_grab, c.held_bite, c.drag_down), (2500, 3500, 3));
    assert_eq!(
        (c.struggle.wind_up, c.struggle.endurance, c.struggle.hit),
        (5, 120, 7000)
    );
    assert_eq!(c.struggle_per_extra, 2000);
    let thump = data.defs.creatures[0].stats.thump;
    assert_eq!((c.door_strength, thump, c.thump_noise), (600, (20, 40), 8));
}

#[test]
fn the_clock_and_needs_become_the_cores() {
    let data = load(&folder("needs", &test_files(PREFABS))).unwrap_or_else(|e| panic!("{e}"));
    let (c, n) = (data.rules.clock, data.rules.needs);
    assert_eq!(
        (c.ticks_per_minute, c.start),
        (50, 390),
        "12× action time: a game minute is 5 seconds; day 1 begins at 06:30"
    );
    assert_eq!(
        (n.hunger.rise, n.thirst.rise, n.fatigue.rise, n.fatigue.rest),
        (100, 150, 67, 208),
        "points a game hour, as thousandths a game minute"
    );
    assert_eq!(n.thirst.levels, [25_000, 50_000, 80_000]);
    assert_eq!((n.hunger.recovery, n.hunger.harm), (750, 20));
    assert_eq!((n.fatigue.slow, n.asleep), ([1250, 1500], 500));
    assert_eq!(
        (n.heal, n.sleep_heal),
        (10, 3),
        "a point of health a game hour"
    );
    assert_eq!(n.wound_heal, [720, 2880, 5760], "game hours, in minutes");
    assert_eq!((n.lie_down, n.drink_water, n.sleep_clear), (10, 30, 10));
    assert_eq!(
        data.defs.beds,
        vec![false, false, true],
        "no fixture, tree, bed"
    );

    let fast = RULES.replace("rate = 12", "rate = 1000");
    let mut files = test_files(PREFABS);
    files[0].1 = &fast;
    let err = load(&folder("clock", &files)).unwrap_err();
    assert!(err.message.contains("clock.rate"), "{err}");
}

#[test]
fn day_night_and_sight_become_the_cores() {
    let data = load(&folder("day", &test_files(PREFABS))).unwrap_or_else(|e| panic!("{e}"));
    let r = data.rules;
    let d = r.day;
    assert_eq!(
        (d.dawn, d.sunrise, d.sunset, d.dark),
        (300, 420, 1170, 1260),
        "hours, in game minutes"
    );
    assert_eq!((d.night, d.indoors), (25, 500));
    assert_eq!(
        (r.sight.range, r.sight.cone.cos, r.sight.beside),
        (16, 0, 1),
        "180° across is 90° either side"
    );
    let s = data.defs.creatures[0].stats;
    assert_eq!((s.darkvision, s.night_speed), (2, 1500));
    assert_eq!((data.defs.light_time, data.defs.douse_time), (15, 5));

    let backwards = RULES.replace("sunset = 19.5", "sunset = 6.0");
    let mut files = test_files(PREFABS);
    files[0].1 = &backwards;
    let err = load(&folder("night", &files)).unwrap_err();
    assert!(err.message.contains("in that order"), "{err}");
}

#[test]
fn the_games_items_and_loot_load() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let id = |name: &str| d.items.iter().position(|i| i.id == name).unwrap() as u16;
    let cudgel = &d.items[usize::from(id("cudgel"))];
    assert_eq!(cudgel.weight, 1500, "1.5 kg in grams");
    let dusk_core::ItemKind::Weapon { swing, wear, skill } = cudgel.kind else {
        panic!("a cudgel is a weapon");
    };
    assert_eq!((swing.wind_up, swing.damage, wear), (4, (80, 120), 10));
    assert_eq!(skill, Some(dusk_core::Skill::Blunt));
    let boarding = d.boarding.as_ref().expect("the game's data can board up");
    assert_eq!(boarding.hammers, vec![id("hammer")]);
    assert_eq!((boarding.plank, boarding.nails), (id("plank"), id("nails")));
    assert_eq!(d.wield, Some(id("cudgel")));
    assert_eq!(
        d.kit,
        vec![
            dusk_core::Item::new(id("bandage"), 1),
            dusk_core::Item::new(id("torch"), 1)
        ]
    );
    assert_eq!(d.carry, 15_000);
    assert_eq!(
        d.items[usize::from(id("bread"))].kind,
        dusk_core::ItemKind::Food {
            nutrition: 40_000,
            time: 40
        },
        "40 points of hunger off, in 4 seconds"
    );
    assert_eq!(
        d.items[usize::from(id("torch"))].kind,
        dusk_core::ItemKind::Light {
            radius: 7,
            wane: 17
        },
        "an hour's burn: 17 of its 1000 a game minute"
    );
    let bed = data.fixtures.iter().position(|f| f.id == "bed").unwrap() + 1;
    assert!(d.beds[bed] && !d.beds[1], "a bed is a bed, a tree isn't");
    // Every container's table is there, and the cupboard's is the kitchen's.
    let cupboard = data
        .fixtures
        .iter()
        .position(|f| f.id == "cupboard")
        .unwrap()
        + 1;
    let table = d.containers[cupboard].expect("a cupboard is a container");
    assert_eq!(d.loot[usize::from(table)].id, "kitchen");
    assert!(d.containers[1].is_none(), "a tree isn't a container");
}

#[test]
fn skills_load_in_the_cores_units() {
    let data = load(&folder("skills", &test_files(PREFABS))).unwrap_or_else(|e| panic!("{e}"));
    let s = data.rules.skills;
    assert_eq!(s.levels[0], 5_000, "XP in thousandths");
    assert_eq!(s.levels[9], 2_500_000);
    assert_eq!((s.free_cap, s.masteries), (6, 2));
    assert_eq!(
        (s.repeat, s.down, s.loaded),
        (500, 250, 400),
        "shares per mille"
    );
    // The clock runs 12× here: a game minute is 50 ticks, so two game hours are 6,000.
    assert_eq!(s.forget, 6_000);
    assert_eq!((s.hit, s.run, s.bandage), (1_500, 100, 2_000));
    assert_eq!(s.weapon_hit, 200, "+2% a level, out of 10,000");
    assert_eq!(
        (s.weapon_damage, s.run_speed, s.bandage_time),
        (100, 30, 80)
    );
}

#[test]
fn barricades_load_with_their_items() {
    let items = r#"
[[item]]
id = "plank"
name = "plank"
weight = 2.0
[[item]]
id = "nails"
name = "nails"
weight = 0.01
stack = 100
[[item]]
id = "hammer"
name = "hammer"
weight = 1.0
weapon = { wind_up = 0.35, recovery = 0.5, endurance = 8, hit = 0.85, damage = [7, 11], noise = 4, wear = 0.8, skill = "blunt" }
"#;
    let barricade = "[barricade]
plank = \"plank\"
nails = \"nails\"
nails_each = 2
                     hammers = [\"hammer\"]
most = 3
strength = 45
time = 2.5
noise = 10
                     remove = 1.0
remove_noise = 5
";
    let rules = format!("{RULES}{barricade}");
    let mut files = test_files(PREFABS);
    files[0].1 = &rules;
    files.push(("items/tools.toml", items));
    let data = load(&folder("barricade", &files)).unwrap_or_else(|e| panic!("{e}"));
    let b = data.rules.barricade;
    assert_eq!((b.most, b.strength, b.time, b.noise), (3, 450, 25, 10));
    assert_eq!(b.remove_time, 10);
    let boarding = data
        .defs
        .boarding
        .as_ref()
        .expect("boarding up is possible");
    assert_eq!(
        (boarding.plank, boarding.nails, boarding.nails_each),
        (0, 1, 2)
    );
    assert_eq!(boarding.hammers, vec![2]);
    assert_eq!(data.rules.skills.board_strength, 200);

    // A hammer that isn't an item is refused.
    let typo = rules.replace("hammers = [\"hammer\"]", "hammers = [\"hamer\"]");
    files[0].1 = &typo;
    let err = load(&folder("barricade-typo", &files)).unwrap_err();
    assert!(err.message.contains("\"hamer\""), "{err}");
}

#[test]
fn skill_levels_that_dont_rise_are_refused() {
    let flat = RULES.replace("levels = [5, 20, 50,", "levels = [5, 5, 50,");
    let mut files = test_files(PREFABS);
    files[0].1 = &flat;
    let err = load(&folder("skill-levels", &files)).unwrap_err();
    assert!(err.message.contains("skills.levels"), "{err}");
}

#[test]
fn a_weapon_naming_no_weapon_skill_is_refused() {
    let items = r#"
[[item]]
id = "club"
name = "club"
weight = 1.0
weapon = { wind_up = 0.4, recovery = 0.6, endurance = 9, hit = 0.85, damage = [8, 12], noise = 4, wear = 1.0, skill = "athletics" }
"#;
    let mut files = test_files(PREFABS);
    files.push(("items/clubs.toml", items));
    let err = load(&folder("weapon-skill", &files)).unwrap_err();
    assert!(
        err.file.ends_with("clubs.toml") && err.message.contains("\"athletics\""),
        "{err}"
    );
}

#[test]
fn a_loot_table_naming_no_item_is_refused() {
    let loot = r#"
[[loot]]
id = "kitchen"
rolls = [1, 2]
items = [{ item = "breadd", weight = 1 }]
"#;
    let items = r#"
[[item]]
id = "bread"
name = "bread"
weight = 0.5
food = { nutrition = 40, time = 4.0 }
"#;
    let mut files = test_files(PREFABS);
    files.push(("items/food.toml", items));
    files.push(("loot/kitchen.toml", loot));
    let err = load(&folder("loot-typo", &files)).unwrap_err();
    assert!(
        err.file.ends_with("kitchen.toml") && err.message.contains("\"breadd\""),
        "{err}"
    );
}

/// The game's own numbers must keep light a real trade (CONCEPT.md, pillar 4): at night a torch
/// shows you much more, and shows you to the dead from as far as they see.
#[test]
fn at_night_a_torch_is_worth_its_risk_in_the_games_data() {
    use dusk_core::{ActorKind, Intent, Tile, TilePos, View, World};
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let mut rules = data.rules;
    rules.clock.start = 0; // midnight
    let mut w = World::with_defs(1, rules, data.defs.clone());
    w.creature_mut(0).wander = 0;
    for y in 0..40 {
        for x in 0..5 {
            w.grid.set_tile(TilePos::new(x, y, 0), Tile::GRASS);
        }
    }
    // You face south down a strip of grass; a shambler 8 tiles behind you faces your back.
    let me = w
        .spawn(ActorKind::Character, TilePos::new(2, 10, 0))
        .unwrap();
    let z = w
        .spawn(ActorKind::Creature(0), TilePos::new(2, 2, 0))
        .unwrap();
    let ahead = |w: &World| {
        let mut view = View::new(5, 40);
        w.look(me, &mut view);
        (1..30)
            .take_while(|d| view.sees(TilePos::new(2, 10 + d, 0)))
            .count()
    };
    let you = TilePos::new(2, 10, 0);
    let dark = ahead(&w);
    assert!(
        !w.sees(z, you),
        "in the dark it doesn't see you 8 tiles off"
    );
    let torch = data
        .defs
        .items
        .iter()
        .position(|i| i.id == "torch")
        .unwrap() as u16;
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == torch)
        .expect("a torch in the kit") as u16;
    w.intent(me, Intent::Light { slot, def: torch });
    w.step(20);
    let lit = ahead(&w);
    assert!(dark <= 2, "{dark} tiles in the dark");
    assert!(
        lit >= 3 * dark.max(1),
        "{lit} tiles by torch, {dark} without"
    );
    assert!(w.sees(z, you), "and by your torch it does");
}

#[test]
fn the_ghoul_out_paces_your_walk_and_not_your_run_in_the_games_data() {
    // The ghoul's whole job (CONCEPT.md, the Hushlands' four): walking won't lose it, running
    // will, while your breath lasts. By day and by night.
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let you = data.rules.character_speed;
    let kind = data.defs.creature_kind("ghoul").expect("a ghoul");
    let dusk_core::ActorKind::Creature(n) = kind else {
        unreachable!()
    };
    let g = data.defs.creature(n).unwrap().stats;
    let night = g.speed.run * g.night_speed / 1000;
    assert!(g.speed.run > you.walk, "it catches you walking");
    assert!(night < you.run, "and you out-run it, even at night");
    let shambler = data.defs.creature(0).unwrap().stats;
    assert!(shambler.speed.run * shambler.night_speed / 1000 < you.walk);
    // It follows blood, and its bite is the fever's; a shambler's isn't.
    assert!(g.blood > 0 && g.fever[2] > g.fever[0]);
    assert_eq!((shambler.blood, shambler.fever), (0, [0; 3]));
    // And there's a cure to be found.
    assert!(
        data.defs
            .items
            .iter()
            .any(|i| matches!(i.kind, dusk_core::ItemKind::Cure { .. }))
    );
}
