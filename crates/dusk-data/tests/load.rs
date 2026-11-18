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
    // 24 health since the second playtest (30 before): `dusk-cli fight`.
    assert_eq!((s.health, s.down, s.attack.wind_up), (240, 40, 12));
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
fn in_the_games_data_the_dead_never_take_hold_and_a_blow_slows_you_a_moment() {
    // The second playtest (2026-11-17): "no grapple/freeze", only a brief slow.
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let c = data.rules.combat;
    assert_eq!(
        (c.grab, c.grab_per_extra, c.grab_tired, c.parting_grab),
        (0, 0, 0, 0)
    );
    assert_eq!(
        (c.stagger, c.stagger_pace),
        (5, 500),
        "half a second at half pace"
    );
    let shrug = data.defs.talents.iter().find(|t| t.id == "shrug").unwrap();
    assert_eq!(shrug.staggered, 0, "Shrug: blows no longer slow you");
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
            dusk_core::Item::new(id("torch"), 1),
            dusk_core::Item::new(id("life_potion"), 1)
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
fn quality_is_read_in_the_cores_units_and_odds_past_one_are_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let q = &data.defs.quality;
    // The game's: crude, plain, fine, masterwork, worst first.
    assert_eq!(q.names[1], "", "plain has no word");
    assert!(
        q.damage.windows(2).all(|d| d[0] < d[1]),
        "better made, harder blows"
    );
    assert!(q.wear.windows(2).all(|w| w[0] > w[1]), "and slower to wear");
    assert!(
        q.chance[0] + q.chance[2] + q.chance[3] < 10_000,
        "plain is left some"
    );

    let grades = "\n[quality]\n\
        crude = { name = \"crude\", chance = 0.25, damage = 0.8, wear = 1.5, worth = 0.5 }\n\
        plain = { damage = 1.0, wear = 1.0, worth = 1.0 }\n\
        fine = { name = \"fine\", chance = 0.15, damage = 1.1, wear = 0.85, worth = 1.5 }\n\
        masterwork = { name = \"masterwork\", chance = 0.05, damage = 1.25, wear = 0.6, worth = 3.0 }\n";
    let rules = format!("{RULES}{grades}");
    let mut files = test_files(PREFABS);
    files[0].1 = &rules;
    let q = load(&folder("quality", &files))
        .unwrap()
        .defs
        .quality
        .clone();
    assert_eq!(q.chance, [2500, 0, 1500, 500]);
    assert_eq!(q.damage, [800, 1000, 1100, 1250]);
    assert_eq!(q.names[3], "masterwork");

    let greedy = rules.replace("chance = 0.25", "chance = 0.9");
    files[0].1 = &greedy;
    let err = load(&folder("quality-odds", &files)).unwrap_err();
    assert!(err.message.contains("more than 1"), "{err}");
    let plain = rules.replace("plain = { damage", "plain = { chance = 0.1, damage");
    files[0].1 = &plain;
    let err = load(&folder("quality-plain", &files)).unwrap_err();
    assert!(err.message.contains("quality.plain"), "{err}");
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
            .filter(|p| p.fits == dusk_core::Fits::Weapon && p.curse == curse)
            .count()
    };
    let light = |curse: bool| {
        d.properties
            .iter()
            .filter(|p| p.fits == dusk_core::Fits::Light && p.curse == curse)
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
    assert!(fighter.learn > 1000 && fighter.endurance < 1000 && fighter.staggered < 1000);
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

#[test]
fn wolves_hunt_in_packs_and_are_skinned_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let wolf = d.creatures.iter().find(|c| c.id == "wolf").unwrap();
    assert!(wolf.stats.wild && wolf.stats.flees == 0, "a wild hunter");
    assert!(wolf.stats.groan > 0, "it howls");
    assert_eq!(wolf.stats.get_up.chance, 0);
    let pelt = d.items.iter().position(|i| i.id == "wolf_pelt").unwrap() as u16;
    assert_eq!(wolf.stats.butcher.map(|b| b.item), Some(pelt));
    assert!(d.items[usize::from(pelt)].worth > 0, "worth bringing home");
    let region = data.region("hushlands").unwrap();
    let wild = &region.beasts.as_ref().unwrap().kinds;
    let pack = wild.iter().find(|k| k.creature == "wolf").unwrap().pack;
    assert!(pack.0 >= 2, "wolves go in packs");
    assert!(region.dead.kinds.iter().all(|k| k.creature != "wolf"));
}

#[test]
fn the_cleric_turns_the_dead_with_faith_and_prays_it_back_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let cleric = d.classes.iter().find(|c| c.id == "cleric").unwrap();
    assert!(cleric.turns && cleric.faith == 100_000);
    assert!(!cleric.line.is_empty(), "a line for whoever chooses");
    assert!(cleric.skills.contains(&dusk_core::Skill::Devotion));
    assert_eq!(cleric.talents.len(), 10);
    let kit = cleric.kit.as_ref().expect("its own kit");
    assert_eq!(kit.wield, Some(item("cudgel")));
    assert!(
        kit.pack
            .iter()
            .any(|i| i.def == item("holy_water") && i.count == 2)
    );
    assert!(
        d.classes[0].kit.is_none(),
        "the Fighter keeps everyone's kit"
    );
    assert!(matches!(
        d.items[usize::from(item("holy_water"))].kind,
        dusk_core::ItemKind::Holy
    ));
    // The altar is a shrine; the chapel's fonts hold holy water, and the abbey sells it.
    let fixture = |id: &str| data.fixtures.iter().position(|f| f.id == id).unwrap() + 1;
    assert!(d.shrines[fixture("altar")]);
    let font = d.containers[fixture("font")].expect("a container");
    assert!(
        d.loot[usize::from(font)]
            .entries
            .iter()
            .any(|e| e.item == item("holy_water"))
    );
    assert!(
        d.holdouts[0]
            .sells
            .iter()
            .any(|s| s.item == item("holy_water"))
    );
    let f = data.rules.faith;
    assert_eq!(
        (f.turn, f.reach, f.turned, f.consecrate),
        (30_000, 8, 200, 15_000)
    );
    // Old dead things are turned for less time; the wild aren't turned at all (a rule, not data).
    let turned = |id: &str| {
        d.creatures
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .stats
            .turned
    };
    assert_eq!(
        (turned("shambler"), turned("wight"), turned("revenant")),
        (1000, 500, 500)
    );
}

/// rules.toml's [faith], as the game has it.
const FAITH: &str = "[faith]\npray = 10.0\ncomfort = 20\nturn = 30\nturn_time = 1.0\n\
turn_recovery = 0.5\nreach = 8\nturned = 20.0\nnoise = 6\nconsecrate = 15\nconsecrate_time = 3.0\n\
mend = 20\nmend_health = 15\nmend_time = 2.0\nward = 40\nward_time = 4.0\nward_hours = 8.0\n";

#[test]
fn faith_loads_in_the_cores_units_and_nonsense_is_refused() {
    let rules = format!("{RULES}{FAITH}");
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "rules.toml");
    files.push(("rules.toml", &rules));
    let data = load(&folder("faith", &files)).unwrap_or_else(|e| panic!("{e}"));
    let f = data.rules.faith;
    assert_eq!((f.pray, f.comfort, f.turn), (100, 20_000, 30_000));
    assert_eq!((f.turn_wind_up, f.turn_recovery, f.turned), (10, 5, 200));
    assert_eq!((f.mend_health, f.ward), (150, 40_000));
    let minute = u64::from(data.rules.clock.ticks_per_minute);
    assert_eq!(f.ward_lasts, 8 * 60 * minute);

    let rules = format!("{RULES}{}", FAITH.replace("reach = 8", "reach = 0"));
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "rules.toml");
    files.push(("rules.toml", &rules));
    let err = load(&folder("faith-reach", &files)).unwrap_err();
    assert!(err.message.contains("reach is 1 to 40"), "{err}");
}

#[test]
fn a_class_must_make_sense_of_its_faith_and_its_kit() {
    let talents = "[[talent]]\nid = \"a\"\nname = \"a\"\nline = \"a\"\n\n\
[[talent]]\nid = \"b\"\nname = \"b\"\nline = \"b\"\n\n\
[[talent]]\nid = \"mend\"\nname = \"m\"\nline = \"m\"\nmending = true\n";
    let refused = |name: &str, class: &str, said: &str| {
        let classes = format!(
            "[[class]]\nid = \"c\"\nname = \"C\"\nskills = []\nlearn = 1.0\nendurance = 1.0\n\
             staggered = 1.0\ntalents = [\"a\", \"b\", \"mend\"]\n{class}"
        );
        let mut files = test_files(PREFABS);
        files.push(("talents.toml", talents));
        files.push(("classes.toml", &classes));
        files.push((
            "items/rags.toml",
            "[[item]]\nid = \"rag\"\nname = \"rag\"\nweight = 0.1\n",
        ));
        let err = load(&folder(name, &files)).unwrap_err();
        assert!(err.message.contains(said), "{err}");
    };
    refused("class-turns", "turns = true\n", "needs some");
    refused("class-faith", "", "spends faith, and this class has none");
    refused(
        "class-wield",
        "faith = 50\nwield = \"rag\"\n",
        "isn't a weapon",
    );
    refused(
        "class-kit",
        "faith = 50\nkit = [\"gold\"]\n",
        "no item called",
    );
}

#[test]
fn herbs_grow_wild_and_make_remedies_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let fixture = |id: &str| data.fixtures.iter().position(|f| f.id == id).unwrap() + 1;
    // Each plant gives its herb, and grows back in days.
    let herbs = [
        ("yarrow", "yarrow"),
        ("chamomile", "chamomile"),
        ("bramble", "blackberries"),
        ("feverfew", "feverfew"),
    ];
    for (plant, herb) in herbs {
        let p = d.plants[fixture(plant)].unwrap_or_else(|| panic!("{plant} is a plant"));
        assert_eq!(p.item, item(herb), "{plant}");
        assert!(p.regrow > 0 && p.count.1 > 0, "{plant}");
    }
    assert_eq!(usize::from(d.bare), fixture("picked_plant"));
    assert!(
        d.plants[usize::from(d.bare)].is_none(),
        "nothing to pick on one picked bare"
    );
    // The remedies: a poultice heals as it binds, a tea calms, feverfew brews into the cure.
    let kind = |id: &str| &d.items[usize::from(item(id))].kind;
    assert!(
        matches!(kind("yarrow_poultice"), dusk_core::ItemKind::Bandage { salve, .. } if *salve < 1000)
    );
    assert!(
        matches!(kind("chamomile_tea"), dusk_core::ItemKind::Drink { comfort, .. } if *comfort > 0)
    );
    assert!(matches!(kind("tincture"), dusk_core::ItemKind::Cure { .. }));
    let recipe = |id: &str| d.recipes.iter().find(|r| r.id == id).unwrap();
    let poultice = recipe("yarrow_poultice");
    assert!(
        poultice.uses.contains(&(item("yarrow"), 2)) && poultice.uses.contains(&(item("rag"), 1))
    );
    assert!(recipe("chamomile_tea").fire);
    let tincture = recipe("feverfew_tincture");
    assert!(tincture.fire && tincture.uses.contains(&(item("feverfew"), 3)));
    assert_eq!(tincture.makes.0, item("tincture"));
    // Feverfew only where the dead lie: the region doesn't grow it wild; the barrows and the
    // graveyard grow it round them.
    let region = data.region("hushlands").unwrap();
    let wild = &region
        .plants
        .as_ref()
        .expect("the Hushlands grow wild plants")
        .kinds;
    assert!(wild.iter().all(|k| k.plant != "feverfew"));
    for prefab in ["barrow", "graveyard"] {
        let p = data.prefabs.iter().find(|p| p.id == prefab).unwrap();
        let (plant, _) = p.grows.expect("feverfew grows round it");
        assert_eq!(usize::from(plant.fixture), fixture("feverfew"), "{prefab}");
    }
    assert!(data.rules.skills.pick > 0, "picking teaches Foraging");
}

#[test]
fn plants_load_in_the_cores_units_and_nonsense_is_refused() {
    const HERB: &str =
        "[[item]]\nid = \"herb\"\nname = \"herb\"\nweight = 0.1\nworth = 0\nstack = 5\n";
    let plants = |extra: &str| {
        format!(
            "{FIXTURES}[[fixture]]\nid = \"herb_plant\"\n\
             forage = {{ item = \"herb\", count = [1, 2], regrow = 2.0 }}\n{extra}\
             [[fixture]]\nid = \"stalks\"\n"
        )
    };
    let foraging = format!("{RULES}\n[foraging]\npick = 2.0\nbare = \"stalks\"\n");
    let run = |name: &str, fixtures: &str, rules: &str| {
        let mut files = test_files(PREFABS);
        files.retain(|(path, _)| *path != "rules.toml" && *path != "fixtures.toml");
        files.push(("rules.toml", rules));
        files.push(("fixtures.toml", fixtures));
        files.push(("items/herbs.toml", HERB));
        load(&folder(name, &files))
    };
    let data = run("plants", &plants(""), &foraging).unwrap_or_else(|e| panic!("{e}"));
    let n = data
        .fixtures
        .iter()
        .position(|f| f.id == "herb_plant")
        .unwrap()
        + 1;
    let p = data.defs.plants[n].expect("a plant");
    let day = 24 * 60 * u64::from(data.rules.clock.ticks_per_minute);
    assert_eq!((p.item, p.count, p.regrow), (0, (1, 2), 2 * day));
    assert_eq!(usize::from(data.defs.bare), n + 1, "the stalks");
    assert_eq!(data.rules.forage.pick, 20, "two seconds");

    let err = run("plants-no-rules", &plants(""), RULES).unwrap_err();
    assert!(err.message.contains("needs [foraging]"), "{err}");
    let err = run("plants-solid", &plants("solid = true\n"), &foraging).unwrap_err();
    assert!(err.message.contains("walked through"), "{err}");
}

#[test]
fn armour_loads_and_the_wight_and_the_fighter_come_in_it_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let item = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let armour = |id: &str| match d.items[usize::from(item(id))].kind {
        dusk_core::ItemKind::Armour(a) => a,
        _ => panic!("{id} isn't armour"),
    };
    let mail = armour("mail_shirt");
    assert_eq!(mail.covers, 2 | 4, "the torso and the arms");
    assert_eq!(mail.turn, [7000, 5500, 4500]);
    assert_eq!(
        (mail.wear, mail.noise, mail.endurance, mail.sight),
        (10, 2, 150, 1000)
    );
    assert_eq!(mail.time, 80, "eight seconds");
    assert_eq!(armour("iron_helm").sight, 850);
    assert_eq!(armour("barrow_mail").covers, 2 | 4 | 8);
    let wight = d.creatures.iter().find(|c| c.id == "wight").unwrap();
    assert_eq!(wight.stats.wears, Some(item("barrow_mail")));
    let fighter = d.classes.iter().find(|c| c.id == "fighter").unwrap();
    assert_eq!(
        fighter.wear,
        vec![item("padded_jerkin"), item("leather_cap")]
    );
    let r = data.rules.armour;
    assert_eq!((r.bruise, r.worn_weight, r.off), (500, 500, 500));
    assert_eq!(r.pierce, [1000, 600, 800, 400]);
    // The smithy's anvil holds the smith's last piece.
    let anvil = data.fixtures.iter().position(|f| f.id == "anvil").unwrap() + 1;
    let table = d.container(anvil as u16).flatten().expect("a container");
    assert_eq!(d.loot[usize::from(table)].id, "anvil");
}

/// The share of a shambler's blows a piece turns: where they land times what it turns of each
/// kind of wound.
fn turned_share(data: &dusk_data::Data, id: &str) -> f64 {
    let d = &data.defs;
    let piece = d.items.iter().find(|i| i.id == id).unwrap();
    let dusk_core::ItemKind::Armour(a) = piece.kind else {
        panic!("{id} isn't armour");
    };
    let shambler = &d.creatures[0].stats;
    let per_blow: f64 = (0..3)
        .map(|k| f64::from(shambler.wounds[k]) * f64::from(a.turn[k]) / 1e8)
        .sum();
    let covered: u32 = dusk_core::Part::ODDS
        .iter()
        .filter(|(p, _)| a.covers & p.group().bit() != 0)
        .map(|(_, odds)| odds)
        .sum();
    f64::from(covered) / 100.0 * per_blow
}

#[test]
fn armour_is_a_trade_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let (padded, mail, barrow) = (
        turned_share(&data, "padded_jerkin"),
        turned_share(&data, "mail_shirt"),
        turned_share(&data, "barrow_mail"),
    );
    // Better armour turns more of what the dead do to you...
    assert!(
        padded > 0.15 && padded < mail && mail < barrow,
        "{padded} {mail} {barrow}"
    );
    assert!(
        mail > 0.4,
        "mail turns nearly half a shambler's blows: {mail}"
    );
    assert!(
        barrow < 0.75,
        "but never all of them: crowds stay deadly ({barrow})"
    );
    // ...and costs you for it: mail is heard, and tires you.
    let d = &data.defs;
    let armour = |id: &str| match d.items.iter().find(|i| i.id == id).unwrap().kind {
        dusk_core::ItemKind::Armour(a) => a,
        _ => panic!(),
    };
    let r = &data.rules;
    for id in ["mail_shirt", "barrow_mail"] {
        let a = armour(id);
        assert!(a.noise > 0 && a.endurance >= 100, "{id} is heavy armour");
        // Still, sneaking in it is quieter than walking bare: you can creep in mail.
        let sneaked = (r.walk_noise + a.noise) * r.sneak.noise as i32 / 1000;
        assert!(sneaked < r.walk_noise, "{id}: a sneak carries {sneaked}");
    }
    // Worn, even barrow mail leaves you more than half of what you can carry.
    let barrow_kg = d
        .items
        .iter()
        .find(|i| i.id == "barrow_mail")
        .unwrap()
        .weight;
    assert!(barrow_kg * r.armour.worn_weight / 1000 < d.carry / 2);
    // And a point goes through a wight's mail less than half as often as a blade is turned.
    assert!(r.armour.pierce[3] * 2 < r.armour.pierce[0]);
}

#[test]
fn armour_that_makes_no_sense_is_refused() {
    let run = |name: &str, armour: &str, creature: &str| {
        let mut files = test_files(PREFABS);
        let creature = format!("{SHAMBLER}{creature}");
        files.retain(|(path, _)| *path != "creatures/dead.toml");
        let item = format!("[[item]]\nid = \"coat\"\nname = \"coat\"\nweight = 1.0\n{armour}\n");
        let mut owned = files
            .into_iter()
            .map(|(p, t)| (p, t.to_string()))
            .collect::<Vec<_>>();
        owned.push(("creatures/dead.toml", creature));
        owned.push(("items/coat.toml", item));
        let borrowed: Vec<(&str, &str)> = owned.iter().map(|(p, t)| (*p, t.as_str())).collect();
        load(&folder(name, &borrowed))
    };
    let good = "armour = { covers = [\"torso\"], turn = { scratch = 0.5, laceration = 0.3, bite = 0.2 }, wear = 2.0, time = 4.0 }";
    let data = run("armour-good", good, "wears = \"coat\"\n").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(data.defs.creatures[0].stats.wears, Some(0));
    let err = run("armour-tail", &good.replace("torso", "tail"), "").unwrap_err();
    assert!(err.message.contains("covers"), "{err}");
    let err = run(
        "armour-twice",
        &good.replace("[\"torso\"]", "[\"torso\", \"torso\"]"),
        "",
    )
    .unwrap_err();
    assert!(err.message.contains("twice"), "{err}");
    let err = run(
        "armour-sight",
        &good.replace("time", "sight = 0.0, time"),
        "",
    )
    .unwrap_err();
    assert!(err.message.contains("sight"), "{err}");
    let err = run("armour-none", "", "wears = \"coat\"\n").unwrap_err();
    assert!(err.message.contains("no armour called"), "{err}");
}

#[test]
fn climbing_loads_and_the_dead_climb_by_kind_in_the_games_data() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let c = data.rules.climb;
    assert_eq!((c.window, c.fence, c.vault), (16, 10, 600));
    assert_eq!((c.noise, c.heavy), (4, 2000));
    assert_eq!(
        (c.glass, c.smash, c.cut),
        (80, 15, 2500),
        "glass gives in a few blows"
    );
    let kind = |id: &str| {
        let s = data
            .defs
            .creatures
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .stats;
        (s.climb, s.tumble)
    };
    assert_eq!(kind("shambler"), (2500, true), "slow, and it falls over");
    assert_eq!(kind("skeleton"), (2000, true));
    assert_eq!(kind("ghoul"), (1000, false), "as quick as you, on its feet");
    assert!(!kind("wight").1, "the wight lands on its feet");
    assert_eq!(kind("wolf"), (0, false), "wolves and deer don't climb");
    assert_eq!(kind("deer").0, 0);
}

#[test]
fn climbing_that_makes_no_sense_is_refused() {
    let run = |name: &str, creature: &str| {
        let files = test_files(PREFABS);
        let creature = format!("{SHAMBLER}{creature}");
        let mut owned = files
            .into_iter()
            .filter(|(path, _)| *path != "creatures/dead.toml")
            .map(|(p, t)| (p, t.to_string()))
            .collect::<Vec<_>>();
        owned.push(("creatures/dead.toml", creature));
        let borrowed: Vec<(&str, &str)> = owned.iter().map(|(p, t)| (*p, t.as_str())).collect();
        load(&folder(name, &borrowed))
    };
    let data = run("climb-good", "climb = 2.0\ntumble = true\n").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(data.defs.creatures[0].stats.climb, 2000);
    let err = run("climb-tumble", "tumble = true\n").unwrap_err();
    assert!(err.message.contains("climb"), "{err}");
    let err = run("climb-zero", "climb = 0.0\n").unwrap_err();
    assert!(err.message.contains("climb"), "{err}");
}

#[test]
fn heavy_swings_load_and_nonsense_is_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let h = data.rules.heavy;
    assert_eq!((h.wind_up, h.recovery, h.endurance), (2000, 1500, 2000));
    assert_eq!((h.harm, h.knock_down, h.wear), (1750, 5000, 2));
    let quick = "[heavy]\nwind_up = 0.5\nrecovery = 1.5\nendurance = 2.0\nharm = 1.75\n\
                 knock_down = 0.5\nwear = 2\n";
    let rules = format!("{RULES}{quick}");
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "rules.toml");
    files.push(("rules.toml", &rules));
    let err = load(&folder("heavy", &files)).unwrap_err();
    assert!(err.message.contains("heavy"), "{err}");
}

#[test]
fn each_class_of_weapons_edge_loads_and_shows_and_nonsense_is_refused() {
    use dusk_core::{Item, WeaponEdge, World};
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let e = data.rules.edges;
    let none = WeaponEdge::default();
    assert_eq!(
        [e.blade, e.blunt, e.axe, e.spear],
        [
            WeaponEdge { down: 500, ..none },
            WeaponEdge { fell: 1200, ..none },
            WeaponEdge { bleed: 80, ..none },
            WeaponEdge { push: 3000, ..none },
        ],
        "each class its own, and only that"
    );
    // Bone has no blood; flesh has.
    let bleeds = |id: &str| {
        let c = data.defs.creatures.iter().find(|c| c.id == id).unwrap();
        c.stats.bleeds
    };
    assert!(!bleeds("skeleton") && bleeds("shambler") && bleeds("wolf"));
    // Each on its weapons' cards, the axe's bleeding at the game's own rate.
    let w = World::with_defs(1, data.rules, data.defs.clone());
    let edge_line = |id: &str| {
        let n = data.defs.items.iter().position(|i| i.id == id).unwrap() as u16;
        let card = w.card(&Item::new(n, 1), None);
        let l = card.iter().find(|l| l.style == dusk_core::CardStyle::Mod);
        l.map(|l| (l.text.clone(), l.value.clone()))
    };
    let pair = |a: &str, b: &str| Some((a.to_string(), b.to_string()));
    assert_eq!(
        edge_line("kitchen_knife"),
        pair("+50% damage to one that's down", "Blade")
    );
    assert_eq!(
        edge_line("mace"),
        pair("12% chance a blow knocks down", "Blunt")
    );
    assert_eq!(
        edge_line("wood_axe"),
        pair("A blow bleeds 4 damage over 8 s", "Axe")
    );
    assert_eq!(
        edge_line("pitchfork"),
        pair("30% chance a blow drives it back a tile", "Spear")
    );
    assert_eq!(edge_line("hunting_bow"), None, "a bow has none");
    // More than twice again on one that's down is refused, as is a chance over 1.
    for (name, edges) in [
        ("edge-down", "[edges]\nblade = { down = 3.0 }\n"),
        ("edge-push", "[edges]\nspear = { push = 1.5 }\n"),
    ] {
        let rules = format!("{RULES}{edges}");
        let mut files = test_files(PREFABS);
        files.retain(|(path, _)| *path != "rules.toml");
        files.push(("rules.toml", &rules));
        let err = load(&folder(name, &files)).unwrap_err();
        assert!(err.message.contains("edges."), "{err}");
    }
}

#[test]
fn strength_and_fitness_load_and_nonsense_is_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let (s, f) = (data.rules.strength, data.rules.fitness);
    assert_eq!(
        (f.growth.most, f.growth.half),
        (600, 15_000),
        "1500 points of breath"
    );
    assert_eq!((f.cost, f.recover), (30, 50));
    assert_eq!((s.growth.most, s.growth.half), (600, 100_000));
    assert_eq!((s.blow, s.shove, s.haul), (1000, 1000, 500));
    assert_eq!((s.carry, s.harm, s.knock_down), (1000, 30, 200));
    let greedy = "[fitness]\nmost = 5.0\nhalf = 1500\ncost = 0.03\nrecover = 0.05\n";
    let rules = format!("{RULES}{greedy}");
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "rules.toml");
    files.push(("rules.toml", &rules));
    let err = load(&folder("fitness", &files)).unwrap_err();
    assert!(err.message.contains("fitness"), "{err}");
}

#[test]
fn bags_and_pockets_load_as_grids_and_a_bag_with_no_grid_is_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let defs = &data.defs;
    assert_eq!(defs.pockets, (4, 5), "four cells across, five down");
    assert_eq!(
        (defs.from_bag, defs.bag_on, defs.shift),
        (10, 15, 5),
        "1, 1.5 and 0.5 s"
    );
    let knapsack = defs.items.iter().position(|i| i.id == "knapsack").unwrap() as u16;
    let bag = defs.bag(knapsack).expect("a bag");
    assert_eq!((bag.grid, bag.reduce), ((6, 4), 400));
    assert_eq!(
        defs.size(knapsack),
        (2, 3),
        "carried, it takes up six cells"
    );
    let empty = r#"
[[item]]
id = "pouch"
name = "pouch"
weight = 0.1
bag = { grid = [0, 2], lighter = 0.1 }
"#;
    let mut files = test_files(PREFABS);
    files.push(("items/bags.toml", empty));
    let err = load(&folder("bag", &files)).unwrap_err();
    assert!(
        err.file.ends_with("bags.toml") && err.message.contains("\"pouch\""),
        "{err}"
    );
}

#[test]
fn items_take_up_their_size_and_containers_their_grids() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let defs = &data.defs;
    let item = |id: &str| defs.items.iter().position(|i| i.id == id).unwrap() as u16;
    assert_eq!(defs.size(item("bandage")), (1, 1), "left out: one cell");
    assert_eq!(defs.size(item("sword")), (1, 3));
    assert_eq!(defs.size(item("hunting_bow")), (2, 4));
    let fixture = |id: &str| defs.fixtures.iter().position(|f| f == id).unwrap() as u16;
    assert_eq!(defs.container_grid(fixture("rack")), (8, 5));
    assert_eq!(defs.container_grid(fixture("font")), (3, 3));
    // Too big an item, a grid on what's no container, and a container too small for its loot.
    let huge = r#"
[[item]]
id = "boulder"
name = "boulder"
weight = 50.0
size = [5, 5]
"#;
    let mut files = test_files(PREFABS);
    files.push(("items/boulder.toml", huge));
    let err = load(&folder("boulder", &files)).unwrap_err();
    assert!(err.message.contains("\"boulder\""), "{err}");
    // A chest's grid holds what its loot gives at the most; one that can't, or a grid on what's
    // no container, is refused.
    let rags = "[[item]]\nid = \"rag\"\nname = \"rag\"\nweight = 0.1\nsize = [1, 2]\n";
    let chest =
        "[[loot]]\nid = \"chest\"\nrolls = [1, 2]\nitems = [{ item = \"rag\", weight = 1 }]\n";
    let with = |name: &str, extra: &str| {
        let fixtures = format!("{FIXTURES}{extra}");
        let mut files = test_files(PREFABS);
        files.retain(|(path, _)| *path != "fixtures.toml");
        files.push(("fixtures.toml", &fixtures));
        files.push(("items/rags.toml", rags));
        files.push(("loot/chest.toml", chest));
        load(&folder(name, &files))
    };
    let chest_with = |grid: &str| format!("[[fixture]]\nid = \"chest\"\nloot = \"chest\"\n{grid}");
    let d = with("chest_grid", &chest_with("grid = [2, 2]\n"))
        .unwrap_or_else(|e| panic!("{e}"))
        .defs;
    let at = d.fixtures.iter().position(|f| f == "chest").unwrap() as u16;
    assert_eq!(d.container_grid(at), (2, 2));
    for (name, extra) in [
        ("chest_too_small", chest_with("grid = [1, 1]\n")),
        ("chest_too_full", chest_with("grid = [1, 2]\n")),
        (
            "gridded_tree",
            "[[fixture]]\nid = \"stump\"\ngrid = [2, 2]\n".to_string(),
        ),
    ] {
        let err = with(name, &extra).unwrap_err();
        assert!(err.file.ends_with("fixtures.toml"), "{name}: {err}");
    }
}

#[test]
fn the_clatterbox_loads_as_a_decoy_made_and_found_and_one_heard_too_far_is_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let defs = &data.defs;
    let n = defs
        .items
        .iter()
        .position(|i| i.id == "clatterbox")
        .unwrap() as u16;
    assert_eq!(
        defs.items[usize::from(n)].kind,
        dusk_core::ItemKind::Decoy(dusk_core::Decoy {
            wind: 20,
            delay: 100,
            every: 20,
            lasts: 900,
            noise: 14,
        }),
        "2 s to wind, 10 before it starts, every 2 for a minute and a half, heard 14 tiles off"
    );
    let recipe = defs.recipes.iter().find(|r| r.id == "clatterbox").unwrap();
    assert_eq!(recipe.makes, (n, 1), "made with a hammer");
    assert!(
        defs.loot
            .iter()
            .any(|t| t.entries.iter().any(|e| e.item == n)),
        "and found"
    );
    let loud = r#"
[[item]]
id = "foghorn"
name = "foghorn"
weight = 1.0
decoy = { wind = 1.0, delay = 0.0, every = 1.0, lasts = 10.0, noise = 60 }
"#;
    let mut files = test_files(PREFABS);
    files.push(("items/horns.toml", loud));
    let err = load(&folder("decoy", &files)).unwrap_err();
    assert!(err.message.contains("\"foghorn\""), "{err}");
}

#[test]
fn doors_and_walls_of_your_own_load_made_by_hand_and_a_piece_of_nothing_is_refused() {
    use dusk_core::{ItemKind, Piece};
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let defs = &data.defs;
    for (id, piece) in [("door", Piece::Door), ("plank_wall", Piece::Wall)] {
        let n = defs.items.iter().position(|i| i.id == id).unwrap() as u16;
        assert_eq!(defs.items[usize::from(n)].kind, ItemKind::Piece(piece));
        let recipe = defs.recipes.iter().find(|r| r.id == id).unwrap();
        assert_eq!(recipe.makes, (n, 1), "{id} is made by hand");
    }
    let b = data.rules.building;
    assert_eq!(
        (b.time, b.wall),
        (100, 800),
        "10 s to put up; a wall takes 80"
    );
    let odd = r#"
[[item]]
id = "window_frame"
name = "window frame"
weight = 5.0
piece = "window"
"#;
    let mut files = test_files(PREFABS);
    files.push(("items/frames.toml", odd));
    let err = load(&folder("piece", &files)).unwrap_err();
    assert!(err.message.contains("\"window_frame\""), "{err}");
}

#[test]
fn wet_and_cold_load_and_a_shiver_that_saves_breath_is_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let c = data.rules.cold;
    assert_eq!(
        (c.soak, c.wade),
        ([2000, 5000], 10_000),
        "points a game minute"
    );
    assert_eq!((c.night, c.wet, c.rain), (30_000, 40_000, [10_000, 20_000]));
    assert_eq!((c.aim, c.breath, c.harm), ([850, 700], [1150, 1300], 30));
    let jerkin = data
        .defs
        .items
        .iter()
        .find(|i| i.id == "padded_jerkin")
        .unwrap();
    match &jerkin.kind {
        dusk_core::ItemKind::Armour(a) => assert_eq!(a.warmth, 15_000),
        other => panic!("{other:?}"),
    }
    // Shivering that costs less breath than being warm.
    let warm = "[cold]\nsoak = [2.0, 5.0]\nwade = 10.0\ndry = 0.5\ndry_roof = 1.0\n\
        dry_fire = 5.0\nnight = 30\nwet = 40\nrain = [10, 20]\nroof = 20\nbed = 20\n\
        soaked = 0.5\nfire = 3\nrise = 0.5\nfall = 1.0\nfire_fall = 3.0\n\
        levels = [25, 50, 75]\naim = [0.85, 0.7]\nbreath = [0.5, 1.3]\nharm = 3.0\n";
    let mut files = test_files(PREFABS);
    files.retain(|(path, _)| *path != "rules.toml");
    let cold = format!("{RULES}{warm}");
    files.push(("rules.toml", &cold));
    let err = load(&folder("cold", &files)).unwrap_err();
    assert!(err.message.contains("cold"), "{err}");
}

#[test]
fn the_wizard_and_its_spells_load_and_a_spell_that_does_nothing_is_refused() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let defs = &data.defs;
    let wizard = defs.classes.iter().find(|c| c.id == "wizard").unwrap();
    // Mana, as Diablo's: 40 to hold, 0.4 back a second (in thousandths of a point).
    assert_eq!((wizard.mana, wizard.mana_regen), (40_000, 400));
    let known: Vec<&str> = wizard
        .spells
        .iter()
        .map(|&s| defs.spells[usize::from(s)].id.as_str())
        .collect();
    // Since the second playtest, the fight's spells first (the bar's four), none wanting a reagent.
    assert_eq!(
        known,
        [
            "firebolt",
            "fireball",
            "fire_nova",
            "light",
            "silence",
            "fog"
        ]
    );
    let fireball = defs.spells.iter().find(|s| s.id == "fireball").unwrap();
    assert_eq!(
        (
            fireball.mana,
            fireball.cast,
            fireball.radius,
            fireball.damage
        ),
        (16_000, 15, 2, (250, 350)),
        "16 mana, 1.5 s of words, 25 to 35 points"
    );
    assert!(
        wizard
            .spells
            .iter()
            .all(|&s| defs.spells[usize::from(s)].component.is_none()),
        "no reagents at the first level"
    );
    let bolt = defs.spells.iter().find(|s| s.id == "firebolt").unwrap();
    assert_eq!(
        (bolt.kind, bolt.mana, bolt.cast, bolt.range, bolt.damage),
        (dusk_core::SpellKind::Bolt, 6_000, 5, 10, (100, 150))
    );
    let nova = defs.spells.iter().find(|s| s.id == "fire_nova").unwrap();
    assert_eq!(
        (nova.kind, nova.radius, nova.cast),
        (dusk_core::SpellKind::Nova, 1, 4)
    );
    assert_eq!(
        data.rules.spell_setback, 3,
        "a blow sets the words back 0.3 s"
    );
    // Arcana: quicker words, a fireball's harm, a spell lasting longer, a level at a time.
    let s = &data.rules.skills;
    assert_eq!(
        (s.cast, s.arcana_cast, s.arcana_harm, s.arcana_lasts),
        (2000, 50, 50, 100)
    );
    assert_eq!(
        s.spell_harm(fireball.damage, 7),
        (337, 472),
        "a third more at the free cap"
    );
    // A fog that reaches nowhere.
    let mist = "[[spell]]\nid = \"mist\"\nname = \"Mist\"\nline = \"\"\nkind = \"fog\"\n\
        mana = 10\ncast = 1.0\nrange = 5\nlasts = 10\n";
    let mut files = test_files(PREFABS);
    files.push(("spells.toml", mist));
    let err = load(&folder("spell", &files)).unwrap_err();
    assert!(
        err.file.ends_with("spells.toml") && err.message.contains("\"mist\""),
        "{err}"
    );
}

#[test]
fn mana_and_its_coming_back_are_stats_arcana_and_gear_raise() {
    use dusk_core::magic::Magic;
    use dusk_core::{CardStyle, Item, World};
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let defs = &data.defs;
    assert_eq!(
        data.rules.skills.arcana_regen, 50,
        "a twentieth faster a level"
    );
    let reserve = defs
        .talents
        .iter()
        .find(|t| t.id == "arcane_reserve")
        .unwrap();
    assert_eq!(reserve.mana, 15_000);
    // A staff that's Brimming and Flowing: 20 more mana, and 40% faster back; on its card, as
    // numbers.
    let prop = |id: &str| defs.properties.iter().position(|p| p.id == id).unwrap() as u8;
    let staff = defs
        .items
        .iter()
        .position(|i| i.id == "quarterstaff")
        .unwrap() as u16;
    let mut magic = Magic {
        rarity: 1,
        known: true,
        curse_shown: true,
        ..Magic::MUNDANE
    };
    (magic.props[0], magic.props[1]) = (prop("brimming"), prop("flowing"));
    let item = Item {
        magic,
        ..Item::new(staff, 1)
    };
    let boosts = defs.boosts_of(&item);
    assert_eq!((boosts.mana, boosts.mana_regen), (20_000, 400));
    let w = World::with_defs(1, data.rules, defs.clone());
    let mods: Vec<String> = w
        .card(&item, None)
        .into_iter()
        .filter(|l| l.style == CardStyle::Mod)
        .map(|l| l.text)
        .collect();
    assert!(
        mods.contains(&"+20(16\u{2013}24) to maximum mana".to_string())
            && mods.contains(&"+40(32\u{2013}48)% mana regeneration".to_string()),
        "{mods:?}"
    );
    // Armour brings mana back faster, but holds none.
    let serene = &defs.properties[usize::from(prop("serene"))];
    assert_eq!((serene.mana, serene.mana_regen), (0, 150));
}

#[test]
fn the_games_uniques_are_weapons_armour_bows_and_shields_and_rares_and_tombs_carry_them() {
    use dusk_core::Fits;
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    // Six weapons, and two each of armour, bows and shields: each its kind's properties, its
    // effect its kind's, and a line of its story.
    let kinds = [Fits::Weapon, Fits::Armour, Fits::Bow, Fits::Shield];
    let made_on = |f: Fits| {
        d.uniques
            .iter()
            .filter(|u| d.fits(u.item) == Some(f))
            .count()
    };
    assert_eq!(kinds.map(made_on), [6, 2, 2, 2]);
    for u in &d.uniques {
        let kind = d.fits(u.item).unwrap();
        assert_eq!(u.effect.fits, kind, "{}", u.id);
        for &p in &u.props {
            assert_eq!(d.properties[usize::from(p)].fits, kind, "{}", u.id);
        }
        assert!(!u.lore.is_empty(), "{}: a line of its story", u.id);
    }
    // Every effect is some unique's.
    let any = |f: fn(&dusk_core::Property) -> bool| d.uniques.iter().any(|u| f(&u.effect));
    assert!(any(|e| e.cleave) && any(|e| e.pyre) && any(|e| e.drain > 0));
    assert!(any(|e| e.terror > 0) && any(|e| e.fell > 0) && any(|e| e.hearten > 0));
    assert!(any(|e| e.thorns.1 > 0) && any(|e| e.calm < 1000));
    // The rares carry each kind of them, and the old tombs hold them; and a unique's worth the
    // most.
    let rare = d.loot.iter().find(|t| t.id == "rare").unwrap();
    for kind in kinds {
        assert!(
            rare.entries
                .iter()
                .any(|e| e.unique > 0 && d.fits(e.item) == Some(kind)),
            "{kind:?}"
        );
    }
    let tomb = d.loot.iter().find(|t| t.id == "tomb").unwrap();
    assert!(tomb.entries.iter().any(|e| e.unique > 0));
    assert!(d.magic.worth[2] > d.magic.worth[1]);
    // Each one's card, once known: its own name, its effect in its own words, its story.
    use dusk_core::{CardStyle, World};
    let w = World::with_defs(1, data.rules, data.defs.clone());
    for (n, u) in d.uniques.iter().enumerate() {
        let mut item = d.unique_item(n as u8).unwrap();
        item.magic.known = true;
        let card = w.card(&item, None);
        let text = |style: CardStyle| -> Vec<&str> {
            card.iter()
                .filter(|l| l.style == style)
                .map(|l| l.text.as_str())
                .collect()
        };
        assert_eq!(text(CardStyle::Title), [u.name.as_str()]);
        let effect = format!("{}: {}", u.effect.name, u.effect.line);
        assert_eq!(text(CardStyle::Effect), [effect.as_str()]);
        assert_eq!(text(CardStyle::Lore), [format!("\"{}\"", u.lore).as_str()]);
    }
}

#[test]
fn a_unique_on_no_weapon_or_a_chance_where_none_is_made_is_refused() {
    let items = r#"
[[item]]
id = "club"
name = "club"
weight = 1.0
weapon = { wind_up = 0.4, recovery = 0.6, endurance = 9, hit = 0.85, damage = [8, 12], noise = 4, wear = 1.0, skill = "blunt" }
[[item]]
id = "bread"
name = "bread"
weight = 0.5
food = { nutrition = 40, time = 4.0 }
[[item]]
id = "buckler"
name = "buckler"
weight = 2.5
shield = { block = 0.5, endurance = 6, wear = 4.0, bash = 0.15 }
"#;
    let properties = r#"
[[property]]
id = "keen"
name = "Keen"
line = "harder"
on = "weapon"
damage = 1.5
[[property]]
id = "steadfast"
name = "Steadfast"
on = "shield"
block = 0.1
"#;
    let uniques = r#"
[[unique]]
id = "knotwood"
name = "Knotwood"
item = "club"
grade = "fine"
properties = ["keen"]
effect = { name = "Thirst", line = "it drinks", drain = 0.25 }
lore = "Old."
"#;
    let loot = r#"
[[loot]]
id = "chest"
rolls = [1, 1]
items = [{ item = "club", weight = 1, unique = 0.1 }]
"#;
    let try_with = |name: &str, uniques: &str, loot: &str| {
        let mut files = test_files(PREFABS);
        files.push(("items/things.toml", items));
        files.push(("properties.toml", properties));
        files.push(("uniques.toml", uniques));
        files.push(("loot/chest.toml", loot));
        load(&folder(name, &files))
    };
    let d = try_with("uniques", uniques, loot).unwrap().defs;
    let u = &d.uniques[0];
    assert_eq!(
        (u.name.as_str(), u.quality, &u.props),
        ("Knotwood", 2, &vec![0])
    );
    assert_eq!((u.effect.drain, u.effect.name.as_str()), (250, "Thirst"));
    assert_eq!(d.loot[0].entries[0].unique, 1000);

    let bread = uniques.replace("item = \"club\"", "item = \"bread\"");
    let err = try_with("unique-bread", &bread, loot).unwrap_err();
    assert!(err.message.contains("no weapon"), "{err}");
    let typo = uniques.replace("[\"keen\"]", "[\"kean\"]");
    let err = try_with("unique-typo", &typo, loot).unwrap_err();
    assert!(err.message.contains("\"kean\""), "{err}");
    let greedy = uniques.replace("drain = 0.25", "drain = 2.0");
    let err = try_with("unique-greedy", &greedy, loot).unwrap_err();
    assert!(err.message.contains("drain"), "{err}");
    let none = loot.replace("item = \"club\"", "item = \"bread\"");
    let err = try_with("unique-none", uniques, &none).unwrap_err();
    assert!(err.message.contains("makes none"), "{err}");

    // On a shield: a shield's properties, and an effect a shield's (a block's: harm back).
    let shield = r#"
[[unique]]
id = "wasp"
name = "The Wasp"
item = "buckler"
properties = ["steadfast"]
effect = { name = "Sting", line = "it stings", on = "shield", thorns = [3, 5] }
lore = "Small."
"#;
    let on_arm = loot.replace("\"club\"", "\"buckler\"");
    let d = try_with("unique-shield", shield, &on_arm).unwrap().defs;
    assert_eq!(d.uniques[0].effect.thorns, (30, 50));
    let keen = shield.replace("[\"steadfast\"]", "[\"keen\"]");
    let err = try_with("unique-shield-keen", &keen, &on_arm).unwrap_err();
    assert!(
        err.message.contains("\"keen\" is no property of a shield"),
        "{err}"
    );
    let weapons = shield.replace("on = \"shield\", ", "on = \"weapon\", ");
    let err = try_with("unique-shield-on", &weapons, &on_arm).unwrap_err();
    assert!(err.message.contains("its effect goes on a shield"), "{err}");
    // Thorns are a shield's or armour's, not a weapon's; and a calm is armour's.
    let thorny = uniques.replace("drain = 0.25", "thorns = [1, 2]");
    let err = try_with("unique-thorny", &thorny, loot).unwrap_err();
    assert!(
        err.message
            .contains("thorns goes on armour or a shield, not a weapon"),
        "{err}"
    );
    let calm = shield.replace("thorns = [3, 5]", "calm = 0.5");
    let err = try_with("unique-shield-calm", &calm, &on_arm).unwrap_err();
    assert!(
        err.message.contains("calm goes on armour, not a shield"),
        "{err}"
    );
}

#[test]
fn the_games_properties_come_in_families_of_three_tiers_and_rares_have_names() {
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    // A family is a kind's: armour's "might" is its own, as a weapon's is.
    let key = |p: &dusk_core::Property| (p.fits as u8, p.family_of().to_string());
    let mut families: Vec<(u8, String)> = d
        .properties
        .iter()
        .filter(|p| p.tier > 0)
        .map(key)
        .collect();
    families.sort_unstable();
    // A family of one has a tier of its own (Hallowed: the second); the rest have three.
    families.dedup();
    let many: Vec<(u8, String)> = families
        .into_iter()
        .filter(|f| d.properties.iter().filter(|p| key(p) == *f).count() > 1)
        .collect();
    assert!(many.len() >= 13, "{many:?}");
    for f in &many {
        let mut tiers: Vec<u8> = d
            .properties
            .iter()
            .filter(|p| key(p) == *f)
            .map(|p| p.tier)
            .collect();
        tiers.sort_unstable();
        assert_eq!(tiers, vec![1, 2, 3], "{f:?}");
    }
    // A Rare's three or four need as many families of a weapon's at the first tier.
    let first: Vec<&str> = d
        .properties
        .iter()
        .filter(|p| p.fits == dusk_core::Fits::Weapon && !p.curse && p.tier <= 1)
        .map(|p| p.family_of())
        .collect();
    assert!(first.len() >= 4);
    assert_eq!(d.magic.count, [(1, 2), (3, 4)]);
    assert!(d.magic.far[0] < d.magic.far[1]);
    let names = &d.thing_names;
    assert!(!names.first.is_empty() && names.second.iter().all(|w| !w.is_empty()));
    for table in ["rare", "tomb"] {
        let t = d.loot.iter().find(|t| t.id == table).unwrap();
        assert_eq!(t.level, 1, "{table} reaches a tier further");
    }
}

#[test]
fn a_fourth_tier_counts_out_of_order_and_an_empty_word_list_are_refused() {
    let magic = "\n[magic]\nrare = 0.3\ncursed = 0.2\nworth = [20, 50, 150]\nlearn = 15\n\
        count = [[1, 2], [3, 4]]\nfar = [60, 140]\n";
    let properties = "[[property]]\nid = \"keen\"\nname = \"Keen\"\nline = \"harder\"\n\
        on = \"weapon\"\nfamily = \"harm\"\ntier = 2\ndamage = 1.5\n";
    let names = "given = [\"Edda\"]\nfamily = [\"Marsh\"]\n[things]\nfirst = [\"Grim\"]\n\
        blade = [\"Edge\"]\nblunt = [\"Knell\"]\naxe = [\"Bite\"]\nspear = [\"Thorn\"]\n\
        weapon = [\"Brand\"]\nlight = [\"Glim\"]\narmour = [\"Hide\"]\nbow = [\"Arc\"]\n\
        shield = [\"Wall\"]\n";
    let try_with = |name: &str, magic: &str, properties: &str, names: &str| {
        let rules = format!("{RULES}{magic}");
        let mut files = test_files(PREFABS);
        files[0].1 = &rules;
        files.push(("properties.toml", properties));
        files.push(("names.toml", names));
        load(&folder(name, &files))
    };
    let d = try_with("tiers", magic, properties, names).unwrap().defs;
    assert_eq!(
        (d.properties[0].tier, d.properties[0].family_of()),
        (2, "harm")
    );
    assert_eq!(d.thing_names.second[3], vec!["Thorn".to_string()]);
    assert_eq!(d.magic.far, [60, 140]);

    let fourth = properties.replace("tier = 2", "tier = 4");
    let err = try_with("tier-four", magic, &fourth, names).unwrap_err();
    assert!(err.message.contains("tier"), "{err}");
    let backwards = magic.replace("[[1, 2], [3, 4]]", "[[1, 2], [4, 3]]");
    let err = try_with("count-backwards", &backwards, properties, names).unwrap_err();
    assert!(err.message.contains("magic.count"), "{err}");
    let wordless = names.replace("axe = [\"Bite\"]", "axe = []");
    let err = try_with("names-empty", magic, properties, &wordless).unwrap_err();
    assert!(err.message.contains("things"), "{err}");
}

#[test]
fn the_games_cards_are_numbers_as_diablos_and_path_of_exiles_are() {
    use dusk_core::magic::Magic;
    use dusk_core::quality::{MASTERWORK, PLAIN};
    use dusk_core::{ActorKind, CardLine, CardStyle, Intent, Item, Tile, TilePos, World};
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = data.defs.clone();
    let mut w = World::with_defs(1, data.rules, d.clone());
    w.grid.set_tile(TilePos::new(0, 0, 0), Tile::GRASS);
    let me = w
        .spawn(ActorKind::Character, TilePos::new(0, 0, 0))
        .unwrap();
    let def = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let known = |id: &str, quality: u8, props: &[&str]| {
        let mut magic = Magic {
            rarity: if props.len() > 2 { 2 } else { 1 },
            known: true,
            curse_shown: true,
            ..Magic::MUNDANE
        };
        for (k, p) in props.iter().enumerate() {
            magic.props[k] = d.properties.iter().position(|q| q.id == *p).unwrap() as u8;
        }
        Item {
            quality,
            magic,
            ..Item::new(def(id), 1)
        }
    };
    let line = |card: &[CardLine], name: &str| {
        let l = card.iter().find(|l| l.text == name).unwrap();
        (l.style, l.value.clone())
    };
    let mods = |card: &[CardLine]| -> Vec<(String, String)> {
        card.iter()
            .filter(|l| l.style == CardStyle::Mod)
            .map(|l| (l.text.clone(), l.value.clone()))
            .collect()
    };
    let pairs = |p: &[(&str, &str)]| -> Vec<(String, String)> {
        p.iter().map(|&(a, b)| (a.into(), b.into())).collect()
    };
    // A plain cudgel: its numbers as its data has them, none marked.
    let cudgel = w.card(&Item::new(def("cudgel"), 1), None);
    for (name, value) in [
        ("Damage per second", "10.0"),
        ("Damage", "8\u{2013}12"),
        ("Attacks per second", "1.00"),
        ("Chance to hit", "85%"),
        ("Stamina a blow", "9"),
    ] {
        assert_eq!(
            line(&cudgel, name),
            (CardStyle::Stat, value.into()),
            "{name}"
        );
    }
    // A masterwork Rare sword: its numbers changed and marked, its making and its properties
    // as numbers, each with its word and tier.
    let sword = known(
        "sword",
        MASTERWORK,
        &["cruel", "darting", "unerring", "poised"],
    );
    let card = w.card(&sword, None);
    assert_eq!(
        line(&card, "Attacks per second"),
        (CardStyle::Changed, "2.00".into())
    );
    // First its class's edge, as Path of Exile's implicits (`edge.rs`): every blade's.
    assert_eq!(
        mods(&card),
        pairs(&[
            ("+50% damage to one that's down", "Blade"),
            ("+25% damage", "Masterwork"),
            ("-40% wear", "Masterwork"),
            ("+80(64\u{2013}96)% damage", "Cruel III"),
            ("+67(47\u{2013}92)% attack speed", "Darting III"),
            ("+25(20\u{2013}30)% chance to hit", "Unerring III"),
            ("-35(28\u{2013}42)% stamina a blow", "Poised II"),
        ])
    );
    // The kinds Diablo has: a body made stronger or fitter, levels in skills (the staff's own,
    // the Wizard's and the Cleric's), and fire and holy harm, each a line of its own too.
    let staff = known(
        "quarterstaff",
        PLAIN,
        &["mighty", "seasoned", "sorcerous", "burning"],
    );
    let card = w.card(&staff, None);
    assert_eq!(
        mods(&card),
        pairs(&[
            ("12% chance a blow knocks down", "Blunt"),
            (
                "Adds 2(1.6\u{2013}2.4) to 4(3.2\u{2013}4.8) fire damage",
                "Burning II",
            ),
            ("+2 to Strength", "Mighty II"),
            ("+2 to Blunt", "Seasoned II"),
            ("+3 to Arcana", "Sorcerous III"),
        ])
    );
    assert_eq!(
        line(&card, "Fire damage"),
        (CardStyle::Fire, "2\u{2013}4".into())
    );
    let mace = known("mace", PLAIN, &["righteous", "hale", "blessed"]);
    assert_eq!(
        mods(&w.card(&mace, None)),
        pairs(&[
            ("12% chance a blow knocks down", "Blunt"),
            (
                "Adds 4(3.2\u{2013}4.8) to 7(5.6\u{2013}8.4) holy damage on the dead",
                "Righteous III",
            ),
            ("+1 to Fitness", "Hale I"),
            ("+1 to Devotion", "Blessed I"),
        ])
    );
    // A unique: its effect in words, as more than numbers.
    let red = d.uniques.iter().position(|u| u.id == "redhand").unwrap() as u8;
    let mut redhand = d.unique_item(red).unwrap();
    redhand.magic.known = true;
    let effect: Vec<String> = w
        .card(&redhand, None)
        .into_iter()
        .filter(|l| l.style == CardStyle::Effect)
        .map(|l| l.text)
        .collect();
    assert_eq!(
        effect,
        ["Thirst: a quarter of the harm it does comes back to you as health"]
    );
    // Armour: where it goes, what it turns, what it costs.
    let mail = w.card(&Item::new(def("mail_shirt"), 1), None);
    for (name, value) in [
        ("Covers", "torso, arms"),
        ("Turns lacerations", "55%"),
        ("Weight worn", "4.5 kg"),
        ("Stamina", "+15% on every cost"),
        ("Footsteps", "+2 tiles"),
    ] {
        assert_eq!(line(&mail, name), (CardStyle::Stat, value.into()), "{name}");
    }
    // A light, Bright and Lasting: farther, and burning twice as long.
    let torch = w.card(&known("torch", PLAIN, &["bright", "lasting"]), None);
    assert_eq!(
        line(&torch, "Light"),
        (CardStyle::Changed, "10 tiles".into())
    );
    assert_eq!(
        mods(&torch),
        pairs(&[
            ("+3 tiles of light", "Bright II"),
            ("+100(67\u{2013}150)% burn time", "Lasting II")
        ])
    );
    // In the pack, set against the cudgel in hand.
    let cudgel = Item::new(def("cudgel"), 1);
    w.give(me, cudgel);
    let slot = w
        .actor(me)
        .unwrap()
        .items
        .iter()
        .position(|i| i.def == cudgel.def);
    w.intent(
        me,
        Intent::Wield {
            slot: slot.unwrap() as u16,
            def: cudgel.def,
        },
    );
    for _ in 0..30 {
        w.step(1);
    }
    let notes: Vec<String> = w
        .card(&sword, Some(me))
        .into_iter()
        .filter(|l| l.style == CardStyle::Note)
        .map(|l| l.text)
        .collect();
    assert_eq!(
        notes,
        [
            "Against your cudgel: +50.6 damage per second, +1.00 attacks a second, +15% chance to hit"
        ]
    );
}

#[test]
fn a_property_raising_no_such_skill_harm_out_of_order_or_doing_nothing_is_refused() {
    use dusk_core::{Raises, Skill};
    let magic = "\n[magic]\nrare = 0.3\ncursed = 0.2\nworth = [20, 50, 150]\nlearn = 15\n\
        count = [[1, 2], [3, 4]]\nfar = [60, 140]\n";
    let try_with = |name: &str, properties: &str| {
        let rules = format!("{RULES}{magic}");
        let mut files = test_files(PREFABS);
        files[0].1 = &rules;
        files.push(("properties.toml", properties));
        load(&folder(name, &files))
    };
    let good = "[[property]]\nid = \"storied\"\nname = \"Storied\"\non = \"weapon\"\n\
        skill = \"own\"\nlevels = 3\n\n[[property]]\nid = \"runed\"\nname = \"Runed\"\n\
        on = \"weapon\"\nskill = \"arcana\"\nlevels = 1\nstrength = 2\nfire = [1, 2]\n";
    let d = try_with("kinds", good).unwrap().defs;
    let (own, runed) = (&d.properties[0], &d.properties[1]);
    assert_eq!((own.raises, own.levels), (Raises::Own, 3));
    assert_eq!(runed.raises, Raises::Skill(Skill::Arcana));
    assert_eq!((runed.strength, runed.fire), (2000, (10, 20)));
    assert!(own.line.is_empty(), "no words: its card says its numbers");
    let light = good.replace(
        "on = \"weapon\"\nskill = \"arcana\"",
        "on = \"light\"\nskill = \"arcana\"",
    );
    let idle = "[[property]]\nid = \"idle\"\nname = \"Idle\"\non = \"weapon\"\n".to_string();
    for (n, (bad, why)) in [
        (good.replace("\"arcana\"", "\"juggling\""), "juggling"),
        (good.replace("skill = \"own\"\n", ""), "levels"),
        (good.replace("[1, 2]", "[2, 1]"), "fire"),
        (
            light,
            "goes on a weapon, armour, a bow or a shield, not a light",
        ),
        (idle, "does something"),
    ]
    .into_iter()
    .enumerate()
    {
        let err = try_with(&format!("bad-property-{n}"), &bad).unwrap_err();
        assert!(err.message.contains(why), "{err}");
    }
}

#[test]
fn the_games_armour_is_made_well_or_badly_and_its_magic_is_numbers_on_its_card() {
    use dusk_core::magic::Magic;
    use dusk_core::quality::MASTERWORK;
    use dusk_core::{CardStyle, Fits, Item, World};
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = data.defs.clone();
    let w = World::with_defs(1, data.rules, d.clone());
    let def = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let prop = |id: &str| d.properties.iter().position(|p| p.id == id).unwrap() as u8;
    // Armour has families of its own, and curses.
    let armour = |curse: bool| {
        d.properties
            .iter()
            .filter(|p| p.fits == Fits::Armour && p.curse == curse)
            .count()
    };
    assert!(armour(false) >= 4 * 3 && armour(true) >= 1);
    // A masterwork Rare iron helm, known: its turning changed (never past 95%), its making and
    // its properties as numbers, and a made-up name for armour.
    let helm = def("iron_helm");
    let mut magic = Magic {
        rarity: 2,
        known: true,
        curse_shown: true,
        ..Magic::MUNDANE
    };
    for (k, id) in ["warded", "fleeced", "powerful", "fleet"]
        .iter()
        .enumerate()
    {
        magic.props[k] = prop(id);
    }
    magic.name = d.roll_name(helm, |k| k);
    let item = Item {
        quality: MASTERWORK,
        magic,
        ..Item::new(helm, 1)
    };
    let card = w.card(&item, None);
    let line = |name: &str| {
        let l = card.iter().find(|l| l.text == name).unwrap();
        (l.style, l.value.as_str())
    };
    assert_eq!(line("Turns scratches"), (CardStyle::Changed, "95%"));
    assert_eq!(line("Turns bites"), (CardStyle::Changed, "80%"));
    assert_eq!(line("Warmth"), (CardStyle::Changed, "8"));
    let mods: Vec<(&str, &str)> = card
        .iter()
        .filter(|l| l.style == CardStyle::Mod)
        .map(|l| (l.text.as_str(), l.value.as_str()))
        .collect();
    assert_eq!(
        mods,
        [
            ("+10% chance to turn blows", "Masterwork"),
            ("-40% wear", "Masterwork"),
            ("+10(8\u{2013}12)% chance to turn blows", "Warded II"),
            ("+6(4.8\u{2013}7.2) warmth", "Fleeced II"),
            ("+2 to Strength", "Powerful II"),
            ("+2 to Athletics", "Fleet II"),
        ]
    );
    let title = &card[0].text;
    let kind = &card[1].text;
    assert!(
        d.thing_names.second[6]
            .iter()
            .any(|w| title.ends_with(w.as_str())),
        "{title}"
    );
    assert_eq!(kind, "Masterwork iron helm");
}

#[test]
fn armour_taking_a_weapons_property_or_a_weapon_armours_is_refused() {
    let magic = "\n[magic]\nrare = 0.3\ncursed = 0.2\nworth = [20, 50, 150]\nlearn = 15\n\
        count = [[1, 2], [3, 4]]\nfar = [60, 140]\n";
    let try_with = |name: &str, properties: &str| {
        let rules = format!("{RULES}{magic}");
        let mut files = test_files(PREFABS);
        files[0].1 = &rules;
        files.push(("properties.toml", properties));
        load(&folder(name, &files))
    };
    let good = "[[property]]\nid = \"warded\"\nname = \"Warded\"\non = \"armour\"\n\
        ward = 0.1\nwarmth = 3\nskill = \"stealth\"\nlevels = 1\n";
    let d = try_with("armour-kinds", good).unwrap().defs;
    let p = &d.properties[0];
    assert_eq!(
        (p.fits, p.ward, p.warmth),
        (dusk_core::Fits::Armour, 1000, 3000)
    );
    for (n, (bad, why)) in [
        (
            good.replace("\"stealth\"", "\"own\""),
            "no skill of its own",
        ),
        (
            good.replace("ward = 0.1", "damage = 1.5"),
            "damage goes on a weapon or a bow, not armour",
        ),
        (
            good.replace("on = \"armour\"", "on = \"weapon\""),
            "goes on armour, not a weapon",
        ),
        (good.replace("ward = 0.1", "ward = 0.9"), "ward"),
    ]
    .into_iter()
    .enumerate()
    {
        let err = try_with(&format!("bad-armour-{n}"), &bad).unwrap_err();
        assert!(err.message.contains(why), "{err}");
    }
}

#[test]
fn the_games_bows_and_shields_have_their_own_magic_and_cards_in_numbers() {
    use dusk_core::magic::Magic;
    use dusk_core::quality::{FINE, MASTERWORK};
    use dusk_core::{CardLine, CardStyle, Fits, Item, World};
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = data.defs.clone();
    let w = World::with_defs(1, data.rules, d.clone());
    let def = |id: &str| d.items.iter().position(|i| i.id == id).unwrap() as u16;
    let prop = |id: &str| d.properties.iter().position(|p| p.id == id).unwrap() as u8;
    // Each has families of its own, and curses.
    let count = |fits: Fits, curse: bool| {
        d.properties
            .iter()
            .filter(|p| p.fits == fits && p.curse == curse)
            .count()
    };
    assert!(count(Fits::Bow, false) >= 6 * 3 && count(Fits::Bow, true) >= 1);
    assert!(count(Fits::Shield, false) >= 4 * 3 && count(Fits::Shield, true) >= 1);
    // A Rare of four properties, known, made as `quality` says, with a made-up name.
    let rare = |item: u16, quality: u8, ids: [&str; 4]| {
        let mut magic = Magic {
            rarity: 2,
            known: true,
            curse_shown: true,
            ..Magic::MUNDANE
        };
        for (k, id) in ids.iter().enumerate() {
            magic.props[k] = prop(id);
        }
        magic.name = d.roll_name(item, |k| k);
        Item {
            quality,
            magic,
            ..Item::new(item, 1)
        }
    };
    let mods = |card: &[CardLine]| -> Vec<(String, String)> {
        card.iter()
            .filter(|l| l.style == CardStyle::Mod)
            .map(|l| (l.text.clone(), l.value.clone()))
            .collect()
    };
    let line = |card: &[CardLine], name: &str| {
        let l = card.iter().find(|l| l.text == name).unwrap();
        (l.style, l.value.clone())
    };
    let pairs = |list: &[(&str, &str)]| -> Vec<(String, String)> {
        list.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    };
    // A masterwork Rare hunting bow: farther, burning, its making and its magic as numbers, and a
    // bow's made-up name.
    let bow = def("hunting_bow");
    let item = rare(
        bow,
        MASTERWORK,
        ["flaming", "farflung", "hunters", "frugal"],
    );
    let card = w.card(&item, None);
    assert_eq!(
        line(&card, "Range"),
        (CardStyle::Changed, "16 tiles".into())
    );
    assert_eq!(
        line(&card, "Fire damage"),
        (CardStyle::Fire, "2\u{2013}4".into())
    );
    assert_eq!(
        mods(&card),
        pairs(&[
            ("+25% damage", "Masterwork"),
            (
                "Adds 2(1.6\u{2013}2.4) to 4(3.2\u{2013}4.8) fire damage",
                "Flaming II",
            ),
            ("+4 tiles of range", "Far-flung II"),
            ("+2 to Archery", "Hunter's II"),
            ("-55(44\u{2013}66)% arrows broken", "Frugal II"),
        ])
    );
    assert!(
        d.thing_names.second[7]
            .iter()
            .any(|word| card[0].text.ends_with(word.as_str())),
        "{}",
        card[0].text
    );
    assert_eq!(card[1].text, "Masterwork hunting bow");
    // A fine Rare buckler: it blocks and bashes more, a block costs less breath.
    let buckler = def("buckler");
    let item = rare(
        buckler,
        FINE,
        ["steadfast", "spiked", "featherweight", "sentinels"],
    );
    let card = w.card(&item, None);
    assert_eq!(
        line(&card, "Chance to block"),
        (CardStyle::Changed, "65% braced".into())
    );
    assert_eq!(
        line(&card, "Bash"),
        (CardStyle::Changed, "+25% to knock down".into())
    );
    assert_eq!(
        line(&card, "Stamina a block"),
        (CardStyle::Changed, "3.9".into())
    );
    assert_eq!(
        mods(&card),
        pairs(&[
            ("+5% chance to block", "Fine"),
            ("-15% wear", "Fine"),
            ("+10(8\u{2013}12)% chance to block", "Steadfast II"),
            ("+10(8\u{2013}12)% chance a bash knocks down", "Spiked II",),
            ("+2 to Shield", "Sentinel's II"),
            ("-35(28\u{2013}42)% stamina a block", "Featherweight II"),
        ])
    );
    assert!(
        d.thing_names.second[8]
            .iter()
            .any(|word| card[0].text.ends_with(word.as_str())),
        "{}",
        card[0].text
    );
}

#[test]
fn a_bows_property_on_a_shield_or_a_shields_on_a_weapon_is_refused() {
    let magic = "\n[magic]\nrare = 0.3\ncursed = 0.2\nworth = [20, 50, 150]\nlearn = 15\n\
        count = [[1, 2], [3, 4]]\nfar = [60, 140]\n";
    let try_with = |name: &str, properties: &str| {
        let rules = format!("{RULES}{magic}");
        let mut files = test_files(PREFABS);
        files[0].1 = &rules;
        files.push(("properties.toml", properties));
        load(&folder(name, &files))
    };
    let bow = "[[property]]\nid = \"far\"\nname = \"Far\"\non = \"bow\"\nrange = 2\n\
        arrows = 0.5\nskill = \"own\"\nlevels = 1\nfire = [1, 2]\n";
    let shield = "[[property]]\nid = \"stalwart\"\nname = \"Stalwart\"\non = \"shield\"\n\
        block = 0.05\nbash = 0.1\nendurance = 0.8\nwear = 0.5\n";
    let d = try_with("bow-kinds", bow).unwrap().defs;
    let p = &d.properties[0];
    assert_eq!(
        (p.fits, p.range, p.arrows, p.fire),
        (dusk_core::Fits::Bow, 2, 500, (10, 20))
    );
    let d = try_with("shield-kinds", shield).unwrap().defs;
    let p = &d.properties[0];
    assert_eq!(
        (p.fits, p.block, p.bash, p.endurance),
        (dusk_core::Fits::Shield, 500, 1000, 800)
    );
    for (n, (bad, why)) in [
        (
            bow.replace("on = \"bow\"", "on = \"shield\"")
                .replace("fire = [1, 2]\n", ""),
            "range goes on a bow, not a shield",
        ),
        (
            shield.replace("on = \"shield\"", "on = \"weapon\""),
            "block goes on a shield, not a weapon",
        ),
        (
            shield.replace("wear = 0.5", "damage = 1.5"),
            "damage goes on a weapon or a bow, not a shield",
        ),
        (bow.replace("range = 2", "range = 20"), "range"),
        (shield.replace("block = 0.05", "block = 0.9"), "block"),
        (bow.replace("on = \"bow\"", "on = \"quiver\""), "\"shield\""),
    ]
    .into_iter()
    .enumerate()
    {
        let err = try_with(&format!("bad-gear-{n}"), &bad).unwrap_err();
        assert!(err.message.contains(why), "{err}");
    }
}

#[test]
fn the_potions_load_in_the_cores_units_with_a_belt_and_one_doing_two_things_is_refused() {
    use dusk_core::{Belt, Effect, ItemKind};
    let data = load(&default_dir()).unwrap_or_else(|e| panic!("{e}"));
    let d = &data.defs;
    let id = |name: &str| d.items.iter().position(|i| i.id == name).unwrap() as u16;
    let potion = |name: &str| d.potion(id(name)).unwrap();
    let life = potion("life_potion");
    assert_eq!(
        (life.effect, life.amount, life.lasts, life.time),
        (Effect::Life, 350, 40, 10),
        "35 points of health over 4 s, a second to drink"
    );
    assert_eq!(potion("mana_potion").amount, 25_000, "mana in thousandths");
    assert_eq!(potion("haste_potion").amount, 300, "per mille");
    assert_eq!(potion("courage_potion").amount, 1000);
    for effect in Effect::ALL {
        assert!(
            d.items
                .iter()
                .any(|i| matches!(i.kind, ItemKind::Potion(p) if p.effect == effect)),
            "{effect:?}"
        );
    }
    assert_eq!(
        data.rules.belt,
        Belt {
            loops: 4,
            holds: 3,
            rummage: 10
        }
    );
    // Found, and sold at the abbey; everyone starts with one of life, a Wizard one of mana too.
    let found = |def: u16| {
        d.loot
            .iter()
            .any(|t| t.entries.iter().any(|e| e.item == def))
    };
    for name in [
        "life_potion",
        "haste_potion",
        "iron_skin_potion",
        "courage_potion",
    ] {
        assert!(found(id(name)), "{name} is found");
    }
    let abbey = d.holdouts.iter().find(|h| h.id == "abbey").unwrap();
    assert!(abbey.sells.iter().any(|s| s.item == id("life_potion")));
    assert!(d.kit.contains(&dusk_core::Item::new(id("life_potion"), 1)));
    let wizard = d.classes.iter().find(|c| c.id == "wizard").unwrap();
    let kit = &wizard.kit.as_ref().expect("its own kit").pack;
    assert!(kit.iter().any(|i| i.def == id("mana_potion")));
    let both = r#"
[[item]]
id = "muddle"
name = "muddled potion"
weight = 0.3
potion = { life = 10, haste = 10, lasts = 5.0, time = 1.0 }
"#;
    let mut files = test_files(PREFABS);
    files.push(("items/muddle.toml", both));
    let err = load(&folder("potion", &files)).unwrap_err();
    assert!(err.message.contains("\"muddle\""), "{err}");
}
