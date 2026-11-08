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
"#;

const FIXTURES: &str = r#"
[[fixture]]
id = "tree"
solid = true
opaque = true
[[fixture]]
id = "bed"
solid = true
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
    assert_eq!((r.shambler_speed.walk, r.shambler_speed.run), (500, 800));
    assert_eq!(r.shambler_cone.cos, 7071, "90° across is 45° either side");
    assert_eq!(r.forget_after, 300, "30 seconds in ticks");
    assert_eq!(r.repath_after, 15);
    assert_eq!(r.wander_chance, 1000, "a tenth, out of 10,000");
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
    assert_eq!(
        (c.shambler_health, c.prone, c.shambler_attack.wind_up),
        (300, 40, 8)
    );
    assert_eq!(c.shambler_wounds, [7000, 2500, 500]);
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
    assert_eq!(
        (c.door_strength, c.shambler_thump, c.thump_noise),
        (600, (20, 40), 8)
    );
}
