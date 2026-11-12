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
[burning]
time = 2.5
burn = 15
light = 3
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
get_up = { chance = 0.5, after = [6.0, 12.5] }
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
[pressure]
start = 100
least = 20
most = 250
per_day = 2.5
ceiling = 150
per_rise = 1
per_burn = 3
every = 45
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
    assert_eq!(
        (s.get_up.chance, s.get_up.after),
        (5000, (360, 750)),
        "a half, out of 10,000; game hours as game minutes"
    );
    assert_eq!(
        (r.burning.time, r.burning.burn, r.burning.light),
        (25, 15, 3),
        "seconds as ticks; game minutes and tiles as they are"
    );
    let p = data.region("test").unwrap().pressure.to_pressure();
    assert_eq!(
        (p.start, p.least, p.most, p.per_day, p.ceiling),
        (1000, 200, 2500, 25, 1500),
        "per cent as thousandths"
    );
    assert_eq!((p.per_rise, p.per_burn, p.every), (10, 30, 45));
    let hut = data.prefab("hut").unwrap();
    assert_eq!(hut.front, Side::South);
    assert_eq!(
        (hut.name.as_str(), hut.dead),
        ("hut", None),
        "no name: its id; no dead: the region's per_house"
    );
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

    let backwards = PREFABS.replace(
        "front = \"south\"",
        "front = \"south\"
dead = [3, 1]",
    );
    let err = load(&folder("backwards", &test_files(&backwards))).unwrap_err();
    assert!(err.message.contains("dead is [fewest, most]"), "{err}");
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
            time: 40,
            comfort: 0,
            spoils: 8
        },
        "40 points of hunger off, in 4 seconds, keeping 5 days"
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
    // Both get up again, unless they're burned; and there's oil to burn them with.
    assert!(g.get_up.chance > 0 && shambler.get_up.chance > 0);
    assert!(
        data.defs
            .items
            .iter()
            .any(|i| matches!(i.kind, dusk_core::ItemKind::Oil))
    );
    // And there's a cure to be found.
    assert!(
        data.defs
            .items
            .iter()
            .any(|i| matches!(i.kind, dusk_core::ItemKind::Cure { .. }))
    );
}

#[test]
fn the_abbey_sells_the_cure_for_what_a_trip_or_two_brings_home_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let abbey = d
        .holdouts
        .iter()
        .find(|h| h.id == "abbey")
        .expect("the abbey");
    let n = d.holdouts.iter().position(|h| h.id == "abbey").unwrap() as u16;
    assert!(d.counters.contains(&Some(n)), "a counter it trades over");
    assert_eq!(
        d.coin.and_then(|c| d.item(c)).map(|i| i.id.as_str()),
        Some("coins")
    );
    let cure = abbey
        .sells
        .iter()
        .find(|s| {
            matches!(
                d.item(s.item).unwrap().kind,
                dusk_core::ItemKind::Cure { .. }
            )
        })
        .expect("it sells the cure");
    // It pays less than it sells for, so nothing is bought and sold back at a profit.
    for s in &abbey.sells {
        let worth = d.item(s.item).unwrap().worth;
        assert!(u64::from(worth) * u64::from(abbey.buys) / 1_000_000 < u64::from(s.price));
    }
    // The leanest patch's finds (227 XP, `dusk-cli gen --seed 1`) fetch 113 coin there: the
    // cure is within half of that, but costs more than a trip or two brings.
    let leanest = 227 * abbey.buys / 1000;
    assert!(
        (20..=leanest / 2).contains(&cure.price),
        "the cure's price: {}",
        cure.price
    );
}

#[test]
fn a_body_rises_as_the_revenant_and_characters_have_names_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let (kind, hour) = d.revenant.expect("a body rises");
    assert_eq!(d.creature(kind).unwrap().id, "revenant");
    assert_eq!(hour, 0, "at midnight");
    // It never spawns in the region's mix, and put down it stays down.
    let region = data.region("hushlands").unwrap();
    assert!(!region.dead.kinds.iter().any(|k| k.creature == "revenant"));
    assert_eq!(d.creature(kind).unwrap().stats.get_up.chance, 0);
    assert!(d.given.len() >= 10 && d.family.len() >= 10);
}

#[test]
fn the_games_rares_have_affixes_names_and_a_bounty_on_them() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    assert!(d.affixes.len() >= 5 && d.rare_names.len() >= 10);
    let ids: Vec<&str> = d.affixes.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"lantern_eater") && ids.contains(&"thrice_risen"));
    let abbey = d.holdouts.iter().find(|h| h.id == "abbey").unwrap();
    assert!(abbey.bounty.coin > 0 && abbey.bounty.pressure > 0);
    let rares = data
        .region("hushlands")
        .unwrap()
        .rares
        .as_ref()
        .expect("rares");
    assert!(rares.count > 0);
}

#[test]
fn the_games_magic_has_properties_for_weapons_and_lights_and_curses_among_them() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let weapon = |curse: bool| {
        d.properties
            .iter()
            .filter(|p| !p.on_light && p.curse == curse)
            .count()
    };
    let light = |curse: bool| {
        d.properties
            .iter()
            .filter(|p| p.on_light && p.curse == curse)
            .count()
    };
    // A Rare weapon has two properties, so it needs at least two to choose from.
    assert!(weapon(false) >= 2 && light(false) >= 2);
    assert!(weapon(true) >= 1 && light(true) >= 1);
    assert!(d.properties.iter().any(|p| p.hallowed));
    assert!((1..=50).contains(&d.magic.learn));
    let abbey = d.holdouts.iter().find(|h| h.id == "abbey").unwrap();
    assert!(abbey.identify > 0 && abbey.uncurse > abbey.identify);
    // And the rares carry it: their weapons are magic.
    let rare = d.loot.iter().find(|t| t.id == "rare").unwrap();
    assert!(rare.entries.iter().any(|e| e.magic == 10_000));
}

#[test]
fn it_rains_in_the_hushlands_now_and_then_and_harder_rain_does_more() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let w = data
        .region("hushlands")
        .unwrap()
        .weather
        .as_ref()
        .expect("weather")
        .to_weather();
    assert!(
        w.spell > 0 && (1..10_000).contains(&w.rain),
        "rain some of the time, not all"
    );
    // Heavy rain does more than light: quieter, less scent, darker, and lights out sooner.
    assert!(w.sound[1] < w.sound[0] && w.scent[1] < w.scent[0] && w.sky[1] < w.sky[0]);
    assert!(w.douse[1] > w.douse[0]);
}

#[test]
fn the_fighter_has_its_talents_and_learns_its_weapons_faster_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let fighter = d.classes.first().expect("a class");
    assert_eq!(fighter.id, "fighter");
    assert!(fighter.learn > 1000 && fighter.endurance < 1000 && fighter.grabbed < 1000);
    use dusk_core::skills::Skill;
    for s in [Skill::Blade, Skill::Blunt, Skill::Axe] {
        assert!(fighter.skills.contains(&s));
    }
    // Enough talents to offer three at each of nine levels' picks, less what's taken: ten.
    assert!(fighter.talents.len() >= 9);
    let talent = |id: &str| d.talents.iter().any(|t| t.id == id);
    assert!(talent("cleave") && talent("second_wind"));
}

#[test]
fn the_market_towns_shops_have_their_own_wares_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let tavern = data.prefab("tavern").expect("a tavern");
    assert_eq!(
        (tavern.name.as_str(), tavern.dead),
        ("the tavern", Some((2, 5)))
    );
    assert_eq!(data.prefab("smithy").unwrap().name, "the smithy");
    // Each shop's containers fill from a table of its own.
    for (fixture, table) in [
        ("rack", "smithy"),
        ("till", "till"),
        ("shop_shelf", "store"),
        ("bar", "bar"),
        ("cask", "cellar"),
    ] {
        let n = data.fixtures.iter().position(|f| f.id == fixture).unwrap() + 1;
        let t = d.containers[n].unwrap_or_else(|| panic!("a {fixture} is a container"));
        assert_eq!(d.loot[usize::from(t)].id, table, "{fixture}");
    }
    // The smith's work: a weapon for each skill that had only a poor one.
    let item = |name: &str| d.items.iter().find(|i| i.id == name).unwrap();
    for (name, skill) in [
        ("sword", dusk_core::Skill::Blade),
        ("spear", dusk_core::Skill::Spear),
        ("mace", dusk_core::Skill::Blunt),
    ] {
        let dusk_core::ItemKind::Weapon { skill: s, wear, .. } = item(name).kind else {
            panic!("a {name} is a weapon");
        };
        assert_eq!(s, Some(skill), "{name}");
        let dusk_core::ItemKind::Weapon { wear: axe, .. } = item("hand_axe").kind else {
            panic!("a hand axe is a weapon");
        };
        assert!(
            wear < axe,
            "{name}: forged, it wears slower than a hand axe"
        );
    }
    assert!(matches!(
        item("ale").kind,
        dusk_core::ItemKind::Drink { .. }
    ));
    assert!(matches!(
        item("cheese").kind,
        dusk_core::ItemKind::Food { .. }
    ));
}

#[test]
fn the_farms_have_wheat_a_well_and_tools_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let region = data.region("hushlands").unwrap();
    let farms = region.farms.as_ref().expect("the Hushlands have farms");
    assert_eq!(
        (farms.tag.as_str(), farms.count, farms.off_road),
        ("farm", 3, (16, 48))
    );
    let farm = data
        .prefabs
        .iter()
        .find(|p| p.tags.contains(&farms.tag))
        .expect("a farmstead");
    assert_eq!(
        (farm.name.as_str(), farm.dead),
        ("a farmstead", Some((1, 3)))
    );
    let fixture = |id: &str| data.fixtures.iter().position(|f| f.id == id).unwrap() + 1;
    let wheat = &data.fixtures[fixture("wheat") - 1];
    assert!(
        !wheat.solid && wheat.opaque,
        "wheat: walked through, not seen through"
    );
    assert!(d.is_well(fixture("well") as u16) && !d.is_well(fixture("barrel") as u16));
    for (f, table) in [("sack", "harvest"), ("tools", "farm")] {
        let t = d.containers[fixture(f)].unwrap_or_else(|| panic!("a {f} is a container"));
        assert_eq!(d.loot[usize::from(t)].id, table, "{f}");
    }
    let scythe = d.items.iter().find(|i| i.id == "scythe").unwrap();
    assert!(matches!(
        scythe.kind,
        dusk_core::ItemKind::Weapon {
            skill: Some(dusk_core::Skill::Blade),
            ..
        }
    ));
    // The farmhouse and its barn are out on the farms now, not in the town.
    assert!(data.prefab("farmhouse").is_none() && data.prefab("barn").is_none());
}

#[test]
fn a_regions_farms_must_make_sense() {
    for (bad, says) in [
        (
            "tag = \"house\"\ncount = 1\noff_road = [5, 2]\ngap = 2",
            "farms:",
        ),
        (
            "tag = \"barn\"\ncount = 1\noff_road = [2, 5]\ngap = 2",
            "farms.tag",
        ),
    ] {
        let region = format!("{REGION}[farms]\n{bad}\n");
        let mut files = test_files(PREFABS);
        files.retain(|(path, _)| *path != "regions/test.toml");
        files.push(("regions/test.toml", &region));
        let err = load(&folder("farms", &files)).unwrap_err();
        assert!(err.message.contains(says), "{err}");
    }
}

#[test]
fn hearths_stoves_and_forges_take_a_fire_and_keep_logs_by_them_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let fixture = |id: &str| data.fixtures.iter().position(|f| f.id == id).unwrap() + 1;
    for (f, reach) in [("stove", 5), ("hearth", 6), ("forge", 6)] {
        assert_eq!(d.hearth(fixture(f) as u16), reach, "{f}");
        let t = d.containers[fixture(f)].unwrap_or_else(|| panic!("a {f} keeps logs by it"));
        assert_eq!(d.loot[usize::from(t)].id, "logs", "{f}");
    }
    assert_eq!(d.hearth(fixture("bed") as u16), 0, "a bed takes no fire");
    // Logs go on before planks, two game hours to one; a fire holds a night's worth.
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    assert_eq!(d.fuel, vec![(item("firewood"), 120), (item("plank"), 60)]);
    assert_eq!(data.rules.fire.most, 8 * 60);
    assert_eq!(
        d.items[usize::from(item("firewood"))].worth,
        0,
        "firewood is no find"
    );
}

#[test]
fn a_fires_numbers_become_the_cores_and_nonsense_is_refused() {
    let rules = format!("{RULES}[fire]\nmost = 6.0\nlay = 2.0\nlight = 3.0\nsmother = 1.5\n");
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "rules.toml");
    files.push(("rules.toml", &rules));
    let data = load(&folder("fire", &files)).unwrap_or_else(|e| panic!("{e}"));
    let f = data.rules.fire;
    assert_eq!((f.most, f.lay, f.light, f.smother), (360, 20, 30, 15));

    let fixtures = format!("{FIXTURES}[[fixture]]\nid = \"pyre\"\nsolid = true\nfire = 13\n");
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "fixtures.toml");
    files.push(("fixtures.toml", &fixtures));
    let err = load(&folder("fire-reach", &files)).unwrap_err();
    assert!(err.message.contains("a fire lights 1 to 12 tiles"), "{err}");

    let mut files = test_files(PREFABS);
    files.push((
        "items/wood.toml",
        "[[item]]\nid = \"log\"\nname = \"log\"\nweight = 1.0\nfuel = 0.0\n",
    ));
    let err = load(&folder("fire-fuel", &files)).unwrap_err();
    assert!(err.message.contains("fuel burns above 0"), "{err}");
}

#[test]
fn pain_and_dread_load_in_the_cores_units_and_nonsense_is_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let (p, d) = (data.rules.pain, data.rules.dread);
    assert_eq!(p.wound, [10_000, 20_000, 30_000]);
    assert_eq!(
        (p.dressed, p.levels, p.cost),
        (500, [10_000, 30_000, 60_000], [1250, 1500])
    );
    assert_eq!(
        (d.dark, d.dark_below, d.per_dead, d.near, d.most),
        (30_000, 25, 20_000, 8, 3)
    );
    assert_eq!((d.rise, d.fall, d.calm), (6_000, 1_000, 3_000));
    assert_eq!(
        (d.levels, d.aim),
        ([25_000, 50_000, 75_000], [900, 800, 700])
    );
    // Alone in the dark you're uneasy and no worse; one of the dead near makes you afraid, and
    // it takes three, or two and your pain, to frighten you.
    assert!(d.levels[0] <= d.dark && d.dark < d.levels[1]);
    assert!(d.dark + d.per_dead >= d.levels[1]);
    assert!(d.dark + 2 * d.per_dead < d.levels[2] && d.dark + 3 * d.per_dead >= d.levels[2]);

    let pain = "[pain]\nscratch = 10\nlaceration = 20\nbite = 30\ndressed = 0.5\n\
                levels = [30, 10, 60]\ncost = [1.25, 1.5]\n";
    let dread = "[dread]\ndark = 30\ndark_below = 25\nper_dead = 20\nnear = 8\nmost = 3\n\
                 per_pain = 10\nhungry = 10\nrise = 0.0\nfall = 1.0\ncalm = 3.0\n\
                 levels = [25, 50, 75]\naim = [0.9, 0.8, 0.7]\n";
    for (bad, says) in [(pain, "pain.levels"), (dread, "rise, fall and calm")] {
        let rules = format!("{RULES}{bad}");
        let mut files = test_files(PREFABS);
        files.retain(|(path, _)| *path != "rules.toml");
        files.push(("rules.toml", &rules));
        let err = load(&folder("dread", &files)).unwrap_err();
        assert!(err.message.contains(says), "{err}");
    }
}

#[test]
fn wounds_by_where_they_land_load_in_the_cores_units_and_nonsense_is_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let i = data.rules.injury;
    assert_eq!(i.levels, [10_000, 25_000]);
    assert_eq!((i.swing, i.arm_hit), ([1150, 1300], [900, 800]));
    assert_eq!((i.pace, i.run), ([850, 600], [1250, 1500]));
    assert_eq!(
        (i.head_hit, i.sight, i.torso),
        ([900, 800], [850, 700], 1500)
    );
    // A fresh laceration hurts its part, and a fresh bite hurts it badly ([pain]).
    let p = data.rules.pain;
    assert!(p.wound[1] >= i.levels[0] && p.wound[1] < i.levels[1]);
    assert!(p.wound[2] >= i.levels[1]);
    // Badly hurt legs can't outrun a ghoul.
    let ghoul = data
        .defs
        .creatures
        .iter()
        .find(|c| c.id == "ghoul")
        .unwrap();
    let run = data.rules.character_speed.run * i.pace[1] / 1000;
    assert!(
        run < ghoul.stats.speed.run,
        "{run} against {}",
        ghoul.stats.speed.run
    );

    for (bad, says) in [
        ("levels = [25, 10]", "injury.levels"),
        ("levels = [10, 25]\npace = [0.7, 0.85]", "injury.pace"),
    ] {
        let mut section = String::from(
            "[injury]\nswing = [1.15, 1.3]\narm_hit = [0.9, 0.8]\nrun = [1.25, 1.5]\n\
             head_hit = [0.9, 0.8]\nsight = [0.85, 0.7]\ntorso = 1.5\n",
        );
        section.push_str(bad);
        if !bad.contains("pace") {
            section.push_str("\npace = [0.85, 0.7]");
        }
        let rules = format!("{RULES}{section}\n");
        let mut files = test_files(PREFABS);
        files.retain(|(path, _)| *path != "rules.toml");
        files.push(("rules.toml", &rules));
        let err = load(&folder("injury", &files)).unwrap_err();
        assert!(err.message.contains(says), "{err}");
    }
}

#[test]
fn the_graveyard_holds_skeletons_that_blades_glance_off_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let skeleton = d.creatures.iter().find(|c| c.id == "skeleton").unwrap();
    let s = skeleton.stats;
    assert_eq!(s.resist, [400, 1500, 700, 300], "blade, blunt, axe, spear");
    assert_eq!(s.get_up.chance, 0, "shattered, it stays down");
    assert_eq!((s.wounds[2], s.fever), (0, [0; 3]), "no bite, no fever");
    // A mace puts one down in two blows at worst; a sword takes at least three.
    let damage = |id: &str| match &d.items.iter().find(|i| i.id == id).unwrap().kind {
        dusk_core::ItemKind::Weapon { swing, .. } => swing.damage,
        _ => panic!("{id} isn't a weapon"),
    };
    let (mace, sword) = (damage("mace"), damage("sword"));
    assert!(2 * mace.0 * 1500 / 1000 >= s.health);
    assert!(2 * sword.1 * 400 / 1000 < s.health);
    // Only in graveyards: not in the region's mix.
    let region = data.region("hushlands").unwrap();
    assert!(!region.dead.kinds.iter().any(|k| k.creature == "skeleton"));
    assert!(region.hamlet.lots.iter().any(|t| t == "graveyard"));
    let yard = data.prefab("graveyard").expect("a graveyard");
    assert_eq!(
        (yard.kind.as_deref(), yard.dead),
        (Some("skeleton"), Some((3, 6)))
    );
    let fixture = |id: &str| data.fixtures.iter().position(|f| f.id == id).unwrap() + 1;
    let tomb = d.containers[fixture("tomb")].expect("a tomb holds grave goods");
    assert_eq!(d.loot[usize::from(tomb)].id, "tomb");
}

#[test]
fn a_buildings_own_dead_must_be_a_creature_and_a_resistance_must_make_sense() {
    let prefabs = PREFABS.replace("front = \"south\"", "front = \"south\"\nkind = \"lich\"");
    let err = load(&folder("kind", &test_files(&prefabs))).unwrap_err();
    assert!(err.message.contains("kind \"lich\""), "{err}");

    let creature = format!("{SHAMBLER}resist = {{ blade = 5.0 }}\n");
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "creatures/dead.toml");
    files.push(("creatures/dead.toml", &creature));
    let err = load(&folder("resist", &files)).unwrap_err();
    assert!(err.message.contains("resist"), "{err}");
}

#[test]
fn the_games_recipes_and_furniture_that_breaks_down_load() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let recipe = |id: &str| d.recipes.iter().find(|r| r.id == id).unwrap();
    let torch = recipe("torch");
    assert_eq!(torch.makes, (item("torch"), 1));
    assert!(torch.uses.contains(&(item("firewood"), 1)) && torch.tools.is_empty());
    let club = recipe("spiked_club");
    assert!(club.uses.contains(&(item("cudgel"), 1)) && club.tools == vec![item("hammer")]);
    assert_eq!(club.skill, Some(dusk_core::Skill::Carpentry));
    // The spiked club is blunt, and better than the cudgel it was.
    let swing = |id: &str| match &d.items[usize::from(item(id))].kind {
        dusk_core::ItemKind::Weapon { swing, skill, .. } => (swing.damage, *skill),
        _ => panic!("{id} isn't a weapon"),
    };
    let ((club_dmg, club_skill), (cudgel_dmg, _)) = (swing("spiked_club"), swing("cudgel"));
    assert_eq!(club_skill, Some(dusk_core::Skill::Blunt));
    assert!(club_dmg.0 > cudgel_dmg.0 && club_dmg.1 > cudgel_dmg.1);
    // Tables and chairs break down; a cupboard (a container) and a stove don't.
    let fixture = |id: &str| data.fixtures.iter().position(|f| f.id == id).unwrap() + 1;
    assert_eq!(
        (d.planks[fixture("table")], d.planks[fixture("chair")]),
        (2, 1)
    );
    assert_eq!(
        (d.planks[fixture("cupboard")], d.planks[fixture("stove")]),
        (0, 0)
    );
    let b = d.breaking.as_ref().expect("[crafting]");
    assert_eq!(b.plank, item("plank"));
    assert!(b.tools.contains(&item("hammer")) && b.tools.contains(&item("wood_axe")));
}

#[test]
fn a_recipe_naming_no_item_and_a_container_that_breaks_down_are_refused() {
    let mut files = test_files(PREFABS);
    files.push((
        "recipes.toml",
        "[[recipe]]\nid = \"x\"\nmakes = \"gold\"\nuses = { lead = 1 }\ntime = 1.0\n",
    ));
    let err = load(&folder("recipe", &files)).unwrap_err();
    assert!(err.message.contains("no item called"), "{err}");

    let fixtures = format!("{FIXTURES}[[fixture]]\nid = \"chest\"\nloot = \"chest\"\nplanks = 2\n");
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "fixtures.toml");
    files.push(("fixtures.toml", &fixtures));
    files.push((
        "items/rags.toml",
        "[[item]]\nid = \"rag\"\nname = \"rag\"\nweight = 0.1\n",
    ));
    files.push((
        "loot/chest.toml",
        "[[loot]]\nid = \"chest\"\nrolls = [1, 1]\nitems = [{ item = \"rag\", weight = 1 }]\n",
    ));
    let err = load(&folder("planks", &files)).unwrap_err();
    assert!(err.message.contains("doesn't break down"), "{err}");
}

#[test]
fn the_wight_walks_only_by_night_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let wight = data
        .defs
        .creatures
        .iter()
        .find(|c| c.id == "wight")
        .unwrap();
    let s = wight.stats;
    assert!(s.nocturnal);
    assert_eq!((s.douses, s.dread, s.lit_harm), (4, 25_000, 2000));
    assert_eq!(s.get_up.chance, 0, "put down, it stays down");
    // Faster than your walk, slower than your run.
    let you = data.rules.character_speed;
    assert!(you.walk < s.speed.run && s.speed.run < you.run);
    // Only in barrows: not in the region's mix.
    let region = data.region("hushlands").unwrap();
    assert!(!region.dead.kinds.iter().any(|k| k.creature == "wight"));
    let barrows = region.barrows.as_ref().expect("[barrows]");
    assert_eq!((barrows.tag.as_str(), barrows.count), ("barrow", 2));
    let barrow = data.prefab("barrow").unwrap();
    assert_eq!(
        (barrow.kind.as_deref(), barrow.dead),
        (Some("wight"), Some((1, 1)))
    );
}

#[test]
fn sneaking_and_stealth_load_in_the_cores_units() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let s = data.rules.sneak;
    assert_eq!(
        (s.pace, s.noise, s.seen, s.harm, s.near),
        (500, 400, 500, 3000, 8)
    );
    let k = data.rules.skills;
    assert_eq!(
        (k.sneak, k.sneak_attack, k.sneak_pace, k.sneak_harm),
        (300, 3000, 30, 100)
    );
    assert_eq!(
        dusk_core::Skill::from_id("stealth"),
        Some(dusk_core::Skill::Stealth)
    );
    // Sneaking, a shambler sees you from no farther than half its sight.
    let shambler = data
        .defs
        .creatures
        .iter()
        .find(|c| c.id == "shambler")
        .unwrap();
    assert_eq!(shambler.stats.sight * s.seen as i32 / 1000, 4);
}

#[test]
fn shields_load_and_a_buckler_can_be_made_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let guard = |id: &str| match d.items.iter().find(|i| i.id == id).unwrap().kind {
        dusk_core::ItemKind::Shield(g) => g,
        _ => panic!("{id} isn't a shield"),
    };
    let (buckler, heater) = (guard("buckler"), guard("heater_shield"));
    assert_eq!(
        (buckler.block, buckler.endurance, buckler.wear, buckler.bash),
        (5000, 60, 40, 1500)
    );
    assert!(
        heater.block > buckler.block && heater.wear < buckler.wear,
        "the old shield's better"
    );
    let recipe = d.recipes.iter().find(|r| r.id == "buckler").unwrap();
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    assert!(recipe.uses.contains(&(item("plank"), 2)) && recipe.tools == vec![item("hammer")]);
    assert_eq!(data.rules.shield_pace, 600);
    assert_eq!(
        (data.rules.skills.shield, data.rules.skills.shield_block),
        (1500, 300)
    );
    assert_eq!(
        dusk_core::Skill::from_id("shield"),
        Some(dusk_core::Skill::Shield)
    );
}

#[test]
fn the_hunting_bow_shoots_arrows_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let dusk_core::ItemKind::Bow(bow) = d.items[usize::from(item("hunting_bow"))].kind else {
        panic!("the hunting bow isn't a bow");
    };
    assert_eq!((bow.ammo, bow.range), (item("arrow"), 12));
    assert!(bow.swing.noise < 4, "quieter than a blow");
    let arrows = d.recipes.iter().find(|r| r.id == "arrows").unwrap();
    assert_eq!(arrows.makes, (item("arrow"), 4));
    assert_eq!(
        dusk_core::Skill::from_id("archery"),
        Some(dusk_core::Skill::Archery)
    );
}

#[test]
fn meals_are_cooked_at_a_fire_and_meat_goes_off_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let food = |id: &str| match d.items[usize::from(item(id))].kind {
        dusk_core::ItemKind::Food {
            nutrition,
            comfort,
            spoils,
            ..
        } => (nutrition, comfort, spoils),
        _ => panic!("{id} isn't food"),
    };
    let (raw, roast) = (food("raw_mutton"), food("roast_mutton"));
    assert!(roast.0 > raw.0, "cooked fills you more");
    assert!(raw.1 < 0 && roast.1 > 0, "raw sits ill, cooked comforts");
    assert!(raw.2 > roast.2 && roast.2 > 0, "both go off, raw sooner");
    assert_eq!(food("jerky").2, 0, "dried meat keeps");
    for id in ["roast_mutton", "roast_turnip", "stew"] {
        let r = d.recipes.iter().find(|r| r.id == id).unwrap();
        assert!(r.fire, "{id} is cooked at a fire");
        assert_eq!(r.skill, Some(dusk_core::Skill::Cooking));
    }
    let stew = d.recipes.iter().find(|r| r.id == "stew").unwrap();
    assert_eq!(stew.tools, vec![item("cooking_pot")]);
    assert_eq!(data.rules.skills.cook_time, 50, "Cooking quickens it");
}

#[test]
fn the_deer_flees_and_gives_venison_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let deer = d.creatures.iter().find(|c| c.id == "deer").unwrap();
    assert!(deer.stats.flees > 0, "a beast");
    assert_eq!(deer.stats.attack.hit, 0, "it never strikes");
    assert_eq!(deer.stats.get_up.chance, 0, "it stays down");
    let cut = deer.stats.butcher.expect("it's butchered");
    assert_eq!((cut.item, cut.count), (item("raw_venison"), 3));
    let roast = d.recipes.iter().find(|r| r.id == "roast_venison").unwrap();
    assert!(roast.fire);
    assert_eq!(roast.uses, vec![(item("raw_venison"), 1)]);
    let region = data.region("hushlands").unwrap();
    let beasts = region.beasts.as_ref().expect("the Hushlands have wildlife");
    assert_eq!(beasts.kinds[0].creature, "deer");
    assert!(region.dead.kinds.iter().all(|k| k.creature != "deer"));
}
