//! Duskhold's data files (`data/`): their types, loading them, and checking them.
//!
//! The files are in a designer's units (tiles a second, seconds, degrees across); loading turns
//! them into the core's (thousandths of a tile, ticks, cosines). Everything a file names must
//! exist (fixtures in a legend, prefab tags in a region), and every number must make sense, or
//! loading fails with the file and what's wrong.

mod barricade;
mod class;
mod dread;
mod farm;
mod fever;
mod fire;
mod holdout;
mod home;
mod items;
mod legacy;
mod light;
mod magic;
mod needs;
mod rare;
mod remains;
mod river;
mod skills;
mod tolling;
mod weather;

pub use farm::FarmsFile;
pub use rare::RaresFile;
pub use remains::PressureFile;
pub use river::RiverFile;
pub use tolling::TollingFile;
pub use weather::WeatherFile;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dusk_core::Defs;
use dusk_core::actor::Speeds;
use dusk_core::senses::Muffling;
use dusk_core::text::TextGrid;
use dusk_core::world::{Combat, Endurance, Rules, Swing, TICKS_PER_SECOND, WoundRule};
use dusk_core::{Creature, CreatureDef, Edge, Ground, Tile};
use serde::Deserialize;
use serde::de::DeserializeOwned;

/// Everything the game reads from `data/`.
#[derive(Clone, Debug)]
pub struct Data {
    pub rules: Rules,
    /// The items, loot tables, containers and start kit, as the core uses them while it runs.
    pub defs: Arc<Defs>,
    /// A tile's fixture number n is `fixtures[n - 1]` (0 is no fixture).
    pub fixtures: Vec<Fixture>,
    pub prefabs: Vec<Prefab>,
    pub regions: Vec<Region>,
}

impl Data {
    pub fn region(&self, id: &str) -> Option<&Region> {
        self.regions.iter().find(|r| r.id == id)
    }

    pub fn prefab(&self, id: &str) -> Option<&Prefab> {
        self.prefabs.iter().find(|p| p.id == id)
    }

    /// The fixture a tile's number stands for.
    pub fn fixture(&self, number: u16) -> Option<&Fixture> {
        self.fixtures.get(usize::from(number).checked_sub(1)?)
    }
}

/// What can stand on a tile.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub id: String,
    /// Nobody walks through it.
    #[serde(default)]
    pub solid: bool,
    /// Nobody sees through it.
    #[serde(default)]
    pub opaque: bool,
    /// A container: the loot table it fills from.
    #[serde(default)]
    pub loot: Option<String>,
    /// A bed: you sleep best beside one.
    #[serde(default)]
    pub bed: bool,
    /// A counter: the holdout that trades over it (`holdouts.toml`).
    #[serde(default)]
    pub trade: Option<String>,
    /// Water to drink from, as a pond is: a well.
    #[serde(default)]
    pub water: bool,
    /// A hearth, a stove or a forge: it takes a fire, which lights this many tiles round it
    /// (0: it takes none).
    #[serde(default)]
    pub fire: i32,
}

/// A building, as a text grid. Its main door is on its `front` side.
#[derive(Clone, Debug)]
pub struct Prefab {
    pub id: String,
    /// What you're told when you're inside it: "the smithy". Its id, if the data gives none.
    pub name: String,
    pub tags: Vec<String>,
    /// How many of the dead are in it at the start, the fewest and the most, where it isn't
    /// the region's `per_house`.
    pub dead: Option<(u32, u32)>,
    pub front: Side,
    pub grid: TextGrid<PrefabCell>,
}

/// A prefab's cell: the tile it puts down, or None to leave the land as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefabCell(pub Option<Tile>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    North,
    East,
    South,
    West,
}

impl Side {
    /// The side it faces after a quarter turn clockwise.
    pub fn turned(self) -> Side {
        match self {
            Side::North => Side::East,
            Side::East => Side::South,
            Side::South => Side::West,
            Side::West => Side::North,
        }
    }

    /// The side it faces in a mirror (east and west swap).
    pub fn mirrored(self) -> Side {
        match self {
            Side::East => Side::West,
            Side::West => Side::East,
            side => side,
        }
    }
}

/// The doors on one side of a grid: their place along that side (x for north and south, y for
/// east and west).
pub fn doors_on<C: Copy>(grid: &TextGrid<C>, side: Side) -> Vec<i32> {
    let is_door = |e: Edge| matches!(e, Edge::Door { .. });
    match side {
        Side::North => (0..grid.width)
            .filter(|&x| is_door(grid.north(x, 0)))
            .collect(),
        Side::South => (0..grid.width)
            .filter(|&x| is_door(grid.north(x, grid.height)))
            .collect(),
        Side::West => (0..grid.height)
            .filter(|&y| is_door(grid.west(0, y)))
            .collect(),
        Side::East => (0..grid.height)
            .filter(|&y| is_door(grid.west(grid.width, y)))
            .collect(),
    }
}

/// A region: its land, its road, its hamlet and its dead, as the generator reads them.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub patch: Patch,
    pub terrain: Terrain,
    pub road: Road,
    pub hamlet: Hamlet,
    pub dead: Dead,
    /// Its pressure (`dusk_core::pressure`), in per cent of the dead a patch began with.
    pub pressure: remains::PressureFile,
    /// Its holdout, if a patch has one: where the road starts, and you with it.
    #[serde(default)]
    pub holdout: Option<HoldoutSite>,
    /// Its event on a calendar, if it has one: the Hushlands' Tolling.
    #[serde(default)]
    pub tolling: Option<TollingFile>,
    /// Its rare monsters, if a patch has any (`affixes.toml`).
    #[serde(default)]
    pub rares: Option<RaresFile>,
    /// Its rain, if it has any.
    #[serde(default)]
    pub weather: Option<WeatherFile>,
    /// Its river, if a patch has one.
    #[serde(default)]
    pub river: Option<RiverFile>,
    /// Its farms, if a patch has any.
    #[serde(default)]
    pub farms: Option<FarmsFile>,
}

/// A region's holdout in a patch: its walled grounds (a prefab, whose walls bound the hallowed
/// ground), north of the road where you start, its gate facing the road.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldoutSite {
    pub prefab: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    /// The patch's size, in chunks.
    pub chunks: (i32, i32),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terrain {
    pub hill_size: f32,
    pub pond_below: f32,
    pub woods_from: f32,
    pub woods_thick: f32,
    pub woods_trees: f32,
    pub field_trees: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Road {
    pub width: i32,
    pub bend_every: i32,
    pub sway: i32,
    pub start: i32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hamlet {
    /// Prefab tags, from the middle outward on alternate sides of the road.
    pub lots: Vec<String>,
    pub gap: i32,
    pub setback: i32,
    pub yard: i32,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dead {
    /// Which creatures they are (`creatures/*.toml`), and what share of them each kind is.
    pub kinds: Vec<DeadKind>,
    pub per_chunk: f32,
    pub per_house: (u32, u32),
    pub clear_start: i32,
}

/// One kind among a region's dead: a creature's id, and its share of them (the shares add up
/// to 1).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeadKind {
    pub creature: String,
    pub share: f32,
}

/// What's wrong, and in which file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataError {
    pub file: PathBuf,
    pub message: String,
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file.display(), self.message)
    }
}

impl std::error::Error for DataError {}

/// The `data/` folder this crate was built beside: the game's own data, for the tools and the
/// game in development.
pub fn default_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// Loads and checks everything in a data folder.
pub fn load(dir: &Path) -> Result<Data, DataError> {
    let rules_file = dir.join("rules.toml");
    let raw: RulesFile = read(&rules_file)?;
    let mut creatures = Vec::new();
    for file in files_in(&dir.join("creatures"))? {
        let parsed: CreaturesFile = read(&file)?;
        creatures.extend(parsed.creature.into_iter().map(|c| (file.clone(), c)));
    }
    let rules = to_rules(&rules_file, &raw)?;
    let creatures = to_creatures(&dir.join("creatures"), &creatures)?;

    let fixtures_file = dir.join("fixtures.toml");
    let fixtures = read::<FixturesFile>(&fixtures_file)?.fixture;
    check_unique(&fixtures_file, "fixture", fixtures.iter().map(|f| &f.id))?;
    if fixtures.len() >= usize::from(u16::MAX) {
        return Err(fail(&fixtures_file, "too many fixtures"));
    }
    if let Some(f) = fixtures.iter().find(|f| !(0..=12).contains(&f.fire)) {
        return Err(fail(
            &fixtures_file,
            format!(
                "fixture {:?}: a fire lights 1 to 12 tiles (0 or left out: it takes none)",
                f.id
            ),
        ));
    }
    let kit = items::Kit {
        carry: raw.character.carry,
        wield: raw.character.wield.as_deref(),
        kit: &raw.character.kit,
    };
    let mut defs = items::load_defs(dir, &rules_file, kit, &raw.items, &fixtures)?;
    if let Some(b) = &raw.barricade {
        defs.boarding = Some(barricade::to_boarding(&rules_file, b, &defs.items)?);
    }
    holdout::load(dir, &mut defs, &fixtures)?;
    defs.creatures = creatures;
    legacy::load(dir, &rules_file, raw.revenant.as_ref(), &mut defs)?;
    defs.affixes = rare::load(dir)?;
    defs.properties = magic::load(dir)?;
    (defs.classes, defs.talents) = class::load(dir)?;
    if let Some(m) = &raw.magic {
        defs.magic = magic::to_rules(&rules_file, m)?;
    }
    let defs = Arc::new(defs);

    let mut prefabs = Vec::new();
    for file in files_in(&dir.join("prefabs"))? {
        let parsed: PrefabsFile = read(&file)?;
        for p in &parsed.prefab {
            prefabs.push(to_prefab(&file, p, &parsed.legend, &fixtures)?);
        }
    }
    check_unique(
        &dir.join("prefabs"),
        "prefab",
        prefabs.iter().map(|p| &p.id),
    )?;

    let mut regions = Vec::new();
    for file in files_in(&dir.join("regions"))? {
        let region: Region = read(&file)?;
        check_region(&file, &region, &prefabs, &defs.creatures)?;
        let tables: Vec<String> = defs.loot.iter().map(|t| t.id.clone()).collect();
        if let Some(problem) = region.rares.as_ref().and_then(|r| r.problem(&tables)) {
            return Err(fail(&file, problem));
        }
        if region.rares.as_ref().is_some_and(|r| r.count > 0)
            && (defs.affixes.is_empty() || defs.rare_names.is_empty())
        {
            return Err(fail(
                &file,
                "rares: a patch with rares needs affixes.toml and names.toml's rare names",
            ));
        }
        regions.push(region);
    }
    check_unique(
        &dir.join("regions"),
        "region",
        regions.iter().map(|r| &r.id),
    )?;

    Ok(Data {
        rules,
        defs,
        fixtures,
        prefabs,
        regions,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesFile {
    world: WorldFile,
    sound: SoundFile,
    character: CharacterFile,
    items: items::ItemTimesFile,
    endurance: EnduranceFile,
    attack: SwingFile,
    shove: SwingFile,
    door: DoorFile,
    wounds: WoundsFile,
    grab: GrabFile,
    clock: needs::ClockFile,
    hunger: needs::NeedFile,
    thirst: needs::NeedFile,
    fatigue: needs::FatigueFile,
    sleep: needs::SleepFile,
    healing: needs::HealingFile,
    water: needs::WaterFile,
    day: light::DayFile,
    sight: light::SightFile,
    skills: skills::SkillsFile,
    deeds: home::DeedsFile,
    fever: fever::FeverFile,
    burning: remains::BurningFile,
    /// Optional: with no [shallows], wading uses the core's defaults.
    #[serde(default)]
    shallows: Option<river::ShallowsFile>,
    /// Optional: with no [magic], magic items use the core's defaults (and need properties.toml).
    #[serde(default)]
    magic: Option<magic::MagicFile>,
    /// Optional: with no [revenant], a character's body stays down.
    #[serde(default)]
    revenant: Option<legacy::RevenantFile>,
    /// Optional: a data set with no [barricade] can't board anything up.
    #[serde(default)]
    barricade: Option<barricade::BarricadeFile>,
    /// Optional: with no [fire], fires use the core's defaults.
    #[serde(default)]
    fire: Option<fire::FireFile>,
    /// Optional: with no [pain] or [dread], they use the core's defaults.
    #[serde(default)]
    pain: Option<dread::PainFile>,
    #[serde(default)]
    dread: Option<dread::DreadFile>,
}

/// Points out of 100 (health and endurance), as the core's thousandths.
fn points(p: f64) -> i32 {
    (p * 10.0).round() as i32
}

/// A chance from 0 to 1, out of 10,000.
fn chance(file: &Path, what: &str, p: f64) -> Result<u32, DataError> {
    if !(0.0..=1.0).contains(&p) {
        return Err(fail(file, format!("{what} is a chance, from 0 to 1")));
    }
    Ok((p * 10_000.0).round() as u32)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnduranceFile {
    run: f64,
    walk: f64,
    rest: f64,
    recover_at: f64,
}

/// A swing or shove: seconds of wind-up and recovery, endurance (points), the chance it lands,
/// its damage (points of health, a range), and its noise (tiles). A shove has a knock-down
/// chance and no damage; a creature's swing wounds rather than doing damage.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SwingFile {
    wind_up: f64,
    recovery: f64,
    #[serde(default)]
    endurance: f64,
    #[serde(default = "always")]
    hit: f64,
    #[serde(default)]
    damage: (f64, f64),
    noise: i32,
    #[serde(default)]
    knock_down: f64,
}

fn always() -> f64 {
    1.0
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DoorFile {
    time: f64,
    noise: i32,
    strength: f64,
    thump_noise: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GrabFile {
    chance: f64,
    per_extra: f64,
    tired: f64,
    parting: f64,
    bite: f64,
    drag_down: u32,
    struggle: SwingFile,
    struggle_per_extra: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WoundFile {
    damage: f64,
    bleed: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WoundsFile {
    scratch: WoundFile,
    laceration: WoundFile,
    bite: WoundFile,
    /// Health a second that each bleeding wound costs.
    bleeding: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WoundOddsFile {
    scratch: f64,
    laceration: f64,
    bite: f64,
}

impl SwingFile {
    fn to_swing(&self, file: &Path, what: &str) -> Result<Swing, DataError> {
        let t = |s: f64, part: &str| ticks(file, &format!("{what}.{part}"), s);
        let wind_up = t(self.wind_up, "wind_up")?;
        if wind_up == 0 || self.damage.0 > self.damage.1 || self.noise < 0 || self.endurance < 0.0 {
            return Err(fail(
                file,
                format!(
                    "{what}: the wind-up must be at least a tick (0.1 s), damage [least, most], \
                     and noise and endurance 0 or more"
                ),
            ));
        }
        Ok(Swing {
            wind_up: wind_up as u32,
            recovery: t(self.recovery, "recovery")? as u32,
            endurance: points(self.endurance),
            hit: chance(file, &format!("{what}.hit"), self.hit)?,
            damage: (points(self.damage.0), points(self.damage.1)),
            noise: self.noise,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorldFile {
    active_radius: i32,
    repath_after: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SoundFile {
    wall: i32,
    shut_door: i32,
    shut_window: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CharacterFile {
    walk: f64,
    run: f64,
    walk_noise: i32,
    run_noise: i32,
    path_budget: usize,
    health: f64,
    carry: f64,
    #[serde(default)]
    wield: Option<String>,
    #[serde(default)]
    kit: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreaturesFile {
    creature: Vec<CreatureFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureFile {
    id: String,
    /// What the game calls it, if not by its id.
    #[serde(default)]
    name: Option<String>,
    walk: f64,
    run: f64,
    sight: i32,
    cone: f64,
    think_every: u64,
    forget_after: f64,
    wander: f64,
    path_budget: usize,
    health: f64,
    /// Seconds it stays down when knocked down.
    down: f64,
    attack: SwingFile,
    wounds: WoundOddsFile,
    /// What its blow does to a shut door in its way, in points.
    thump: (f64, f64),
    /// Tiles it sees in any light (`sight` is in full light).
    darkvision: i32,
    /// At night it moves this many times as fast.
    night_speed: f64,
    /// Tiles its groan carries as it catches sight of prey (0, or left out: it makes none).
    #[serde(default)]
    groan: i32,
    /// Tiles it smells blood from, for each wound that's bleeding (0, or left out: it doesn't).
    #[serde(default)]
    blood: i32,
    /// The chance each kind of wound it leaves carries ghoul fever (left out: none does).
    #[serde(default)]
    fever: Option<WoundOddsFile>,
    /// Whether its body gets up again, and when (left out: it stays down).
    #[serde(default)]
    get_up: Option<remains::GetUpFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixturesFile {
    fixture: Vec<Fixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrefabsFile {
    legend: BTreeMap<String, LegendEntry>,
    prefab: Vec<PrefabFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegendEntry {
    ground: String,
    #[serde(default)]
    fixture: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrefabFile {
    id: String,
    #[serde(default)]
    name: Option<String>,
    tags: Vec<String>,
    front: Side,
    #[serde(default)]
    dead: Option<(u32, u32)>,
    grid: String,
}

fn fail(file: &Path, message: impl Into<String>) -> DataError {
    DataError {
        file: file.to_path_buf(),
        message: message.into(),
    }
}

fn read<T: DeserializeOwned>(file: &Path) -> Result<T, DataError> {
    let text = std::fs::read_to_string(file).map_err(|e| fail(file, e.to_string()))?;
    toml::from_str(&text).map_err(|e| fail(file, e.to_string()))
}

/// The `.toml` files in a folder, in name order (the order matters: it's the prefabs' order).
fn files_in(dir: &Path) -> Result<Vec<PathBuf>, DataError> {
    let entries = std::fs::read_dir(dir).map_err(|e| fail(dir, e.to_string()))?;
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    Ok(files)
}

fn check_unique<'a>(
    file: &Path,
    what: &str,
    ids: impl Iterator<Item = &'a String>,
) -> Result<(), DataError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(fail(file, format!("two {what}s are called {id:?}")));
        }
    }
    Ok(())
}

/// Tiles a second, as thousandths of a tile a second.
fn speed(file: &Path, what: &str, tiles_a_second: f64) -> Result<u32, DataError> {
    if !(tiles_a_second > 0.0 && tiles_a_second < 100.0) {
        return Err(fail(
            file,
            format!("{what} must be above 0 tiles a second (and sensible)"),
        ));
    }
    Ok((tiles_a_second * 1000.0).round() as u32)
}

/// Seconds, as ticks.
fn ticks(file: &Path, what: &str, seconds: f64) -> Result<u64, DataError> {
    if !(0.0..1e6).contains(&seconds) {
        return Err(fail(file, format!("{what} must be 0 seconds or more")));
    }
    Ok((seconds * f64::from(TICKS_PER_SECOND)).round() as u64)
}

/// Every creature in `data/creatures/`, as the core's definitions: in file order, then the order
/// they're listed in. A creature's number is its place here.
fn to_creatures(
    dir: &Path,
    creatures: &[(PathBuf, CreatureFile)],
) -> Result<Vec<CreatureDef>, DataError> {
    if creatures.is_empty() || creatures.len() >= usize::from(u16::MAX) {
        return Err(fail(
            dir,
            "the data needs at least one creature (and sensibly many)",
        ));
    }
    check_unique(dir, "creature", creatures.iter().map(|(_, c)| &c.id))?;
    let mut defs = Vec::new();
    for (cfile, c) in creatures {
        let bad = |message: &str| fail(cfile, format!("creature {:?}: {message}", c.id));
        if !(c.cone > 0.0 && c.cone <= 360.0) {
            return Err(bad("cone must be above 0 and at most 360 degrees"));
        }
        if c.sight < 0
            || c.think_every == 0
            || c.path_budget == 0
            || !(0.0..=1.0).contains(&c.wander)
        {
            return Err(bad(
                "sight can't be negative, think_every and path_budget must be above 0, and \
                 wander is a chance from 0 to 1",
            ));
        }
        if c.darkvision < 0 || c.groan < 0 || c.blood < 0 || !(0.5..=3.0).contains(&c.night_speed) {
            return Err(bad(
                "darkvision, groan and blood can't be negative, and night_speed is a pace from 0.5 to 3",
            ));
        }
        let get_up = match &c.get_up {
            Some(g) => remains::to_get_up(cfile, g).map_err(|m| bad(&m))?,
            None => dusk_core::creature::GetUp::default(),
        };
        let fever = match &c.fever {
            Some(f) => [
                chance(cfile, "fever.scratch", f.scratch)?,
                chance(cfile, "fever.laceration", f.laceration)?,
                chance(cfile, "fever.bite", f.bite)?,
            ],
            None => [0; 3],
        };
        if c.health <= 0.0 || c.thump.0 < 0.0 || c.thump.0 > c.thump.1 {
            return Err(bad("health must be above 0, and thump is [least, most]"));
        }
        let odds = &c.wounds;
        let wounds = [
            chance(cfile, "wounds.scratch", odds.scratch)?,
            chance(cfile, "wounds.laceration", odds.laceration)?,
            chance(cfile, "wounds.bite", odds.bite)?,
        ];
        if wounds.iter().sum::<u32>() != 10_000 {
            return Err(bad("wounds: the three chances must add up to 1"));
        }
        defs.push(CreatureDef {
            id: c.id.clone(),
            name: c.name.clone().unwrap_or_else(|| c.id.clone()),
            stats: Creature {
                speed: Speeds {
                    walk: speed(cfile, "walk", c.walk)?,
                    run: speed(cfile, "run", c.run)?,
                },
                night_speed: (c.night_speed * 1000.0).round() as u32,
                sight: c.sight,
                // The cone's half-angle as a cosine, in ten-thousandths: worked out once, here.
                cone: light::cone(c.cone),
                darkvision: c.darkvision,
                groan: c.groan,
                think_every: c.think_every,
                forget_after: ticks(cfile, "forget_after", c.forget_after)?,
                wander: (c.wander * 10_000.0).round() as u32,
                path_budget: c.path_budget,
                health: points(c.health),
                down: ticks(cfile, "down", c.down)? as u32,
                attack: c.attack.to_swing(cfile, "attack")?,
                wounds,
                thump: (points(c.thump.0), points(c.thump.1)),
                blood: c.blood,
                fever,
                get_up,
                douses: 0,
            },
        });
    }
    Ok(defs)
}

fn to_rules(file: &Path, r: &RulesFile) -> Result<Rules, DataError> {
    let mut rules = Rules {
        active_radius: r.world.active_radius,
        character_speed: Speeds {
            walk: speed(file, "character.walk", r.character.walk)?,
            run: speed(file, "character.run", r.character.run)?,
        },
        walk_noise: r.character.walk_noise,
        run_noise: r.character.run_noise,
        path_budget: r.character.path_budget,
        muffling: Muffling {
            wall: r.sound.wall,
            shut_door: r.sound.shut_door,
            shut_window: r.sound.shut_window,
        },
        repath_after: u32::try_from(ticks(file, "world.repath_after", r.world.repath_after)?)
            .map_err(|_| fail(file, "world.repath_after is too long"))?,
        ..Rules::default()
    };
    if rules.active_radius < 1 {
        return Err(fail(file, "world.active_radius must be 1 chunk or more"));
    }
    if rules.character_speed.run < rules.character_speed.walk {
        return Err(fail(file, "character.run must be at least character.walk"));
    }
    let numbers = [
        r.character.walk_noise,
        r.character.run_noise,
        r.sound.wall,
        r.sound.shut_door,
        r.sound.shut_window,
    ];
    if numbers.iter().any(|&n| n < 0) || rules.path_budget == 0 {
        return Err(fail(
            file,
            "noises and sound losses can't be negative, and path_budget must be above 0",
        ));
    }

    // The fight, and what it costs.
    let tick_rate = f64::from(TICKS_PER_SECOND);
    let per_tick = |p: f64| (p * 10.0 / tick_rate).round() as i32;
    let en = &r.endurance;
    if en.run < 0.0 || en.walk < 0.0 || en.rest < 0.0 || !(0.0..=100.0).contains(&en.recover_at) {
        return Err(fail(
            file,
            "endurance: rates can't be negative, and recover_at is out of 100",
        ));
    }
    rules.endurance = Endurance {
        run: per_tick(en.run),
        walk: per_tick(en.walk),
        rest: per_tick(en.rest),
        recover_at: points(en.recover_at),
    };
    let wound = |w: &WoundFile, what: &str| -> Result<WoundRule, DataError> {
        if w.damage < 0.0 {
            return Err(fail(
                file,
                format!("wounds.{what}: damage can't be negative"),
            ));
        }
        Ok(WoundRule {
            damage: points(w.damage),
            bleed: ticks(file, &format!("wounds.{what}.bleed"), w.bleed)? as u32,
        })
    };
    let w = &r.wounds;
    if r.character.health <= 0.0 || w.bleeding < 0.0 || r.door.noise < 0 {
        return Err(fail(
            file,
            "health must be above 0; bleeding and the door's noise can't be negative",
        ));
    }
    let (g, d) = (&r.grab, &r.door);
    if g.drag_down == 0 || d.strength <= 0.0 || d.thump_noise < 0 {
        return Err(fail(
            file,
            "grab.drag_down and door.strength must be above 0, and door.thump_noise can't be \
             negative",
        ));
    }
    // Bleeding is paid once a second.
    rules.combat = Combat {
        grab: chance(file, "grab.chance", g.chance)?,
        grab_per_extra: chance(file, "grab.per_extra", g.per_extra)?,
        grab_tired: chance(file, "grab.tired", g.tired)?,
        parting_grab: chance(file, "grab.parting", g.parting)?,
        held_bite: chance(file, "grab.bite", g.bite)?,
        drag_down: g.drag_down,
        struggle: g.struggle.to_swing(file, "grab.struggle")?,
        struggle_per_extra: chance(file, "grab.struggle_per_extra", g.struggle_per_extra)?,
        door_strength: points(d.strength),
        thump_noise: d.thump_noise,
        character_health: points(r.character.health),
        attack: r.attack.to_swing(file, "attack")?,
        shove: r.shove.to_swing(file, "shove")?,
        knock_down: chance(file, "shove.knock_down", r.shove.knock_down)?,
        door_ticks: (ticks(file, "door.time", r.door.time)? as u32).max(1),
        door_noise: r.door.noise,
        wounds: [
            wound(&w.scratch, "scratch")?,
            wound(&w.laceration, "laceration")?,
            wound(&w.bite, "bite")?,
        ],
        bleed_every: TICKS_PER_SECOND,
        bleed_damage: points(w.bleeding),
    };
    (rules.clock, rules.needs) = needs::to_needs(file, r)?;
    (rules.day, rules.sight) = light::to_light(file, r)?;
    rules.skills = skills::to_skills(file, r, rules.clock.ticks_per_minute)?;
    rules.deeds = home::to_deeds(file, &r.deeds)?;
    rules.fever = fever::to_fever(file, &r.fever)?;
    rules.burning = remains::to_burning(file, &r.burning)?;
    if let Some(s) = &r.shallows {
        rules.shallows = river::to_shallows(file, s)?;
    }
    if let Some(b) = &r.barricade {
        rules.barricade = barricade::to_barricade(file, b)?;
    }
    if let Some(f) = &r.fire {
        rules.fire = fire::to_fire(file, f)?;
    }
    if let Some(p) = &r.pain {
        rules.pain = dread::to_pain(file, p)?;
    }
    if let Some(d) = &r.dread {
        rules.dread = dread::to_dread(file, d)?;
    }
    Ok(rules)
}

fn to_prefab(
    file: &Path,
    p: &PrefabFile,
    legend: &BTreeMap<String, LegendEntry>,
    fixtures: &[Fixture],
) -> Result<Prefab, DataError> {
    let mut cells: BTreeMap<char, PrefabCell> = BTreeMap::new();
    cells.insert(' ', PrefabCell(None));
    for (key, entry) in legend {
        let mut chars = key.chars();
        let (Some(ch), None) = (chars.next(), chars.next()) else {
            return Err(fail(
                file,
                format!("legend key {key:?} must be a single character"),
            ));
        };
        let ground = match entry.ground.as_str() {
            "floor" => Ground::Floor,
            "grass" => Ground::Grass,
            "dirt" => Ground::Dirt,
            "road" => Ground::Road,
            "water" => Ground::Water,
            "shallows" => Ground::Shallows,
            "bridge" => Ground::Bridge,
            other => {
                return Err(fail(
                    file,
                    format!("legend {key:?}: no ground called {other:?}"),
                ));
            }
        };
        let mut tile = Tile {
            ground,
            roof: ground == Ground::Floor,
            ..Tile::VOID
        };
        if let Some(name) = &entry.fixture {
            let Some(i) = fixtures.iter().position(|f| &f.id == name) else {
                return Err(fail(
                    file,
                    format!("legend {key:?}: no fixture called {name:?} in fixtures.toml"),
                ));
            };
            tile.fixture = (i + 1) as u16;
            tile.solid = fixtures[i].solid;
            tile.opaque = fixtures[i].opaque;
        }
        cells.insert(ch, PrefabCell(Some(tile)));
    }
    let grid = TextGrid::parse(&p.grid, |ch| cells.get(&ch).copied())
        .map_err(|e| fail(file, format!("prefab {:?}: {e}", p.id)))?;
    if doors_on(&grid, p.front).is_empty() {
        return Err(fail(
            file,
            format!(
                "prefab {:?}: no door on its front ({:?}) side",
                p.id, p.front
            ),
        ));
    }
    if let Some((x, y)) = blocked_door(&grid) {
        return Err(fail(
            file,
            format!(
                "prefab {:?}: the door by cell ({x}, {y}) opens onto something solid",
                p.id
            ),
        ));
    }
    if p.tags.is_empty() {
        return Err(fail(file, format!("prefab {:?} has no tags", p.id)));
    }
    if p.dead.is_some_and(|(fewest, most)| fewest > most) {
        return Err(fail(
            file,
            format!("prefab {:?}: dead is [fewest, most]", p.id),
        ));
    }
    Ok(Prefab {
        id: p.id.clone(),
        name: p.name.clone().unwrap_or_else(|| p.id.clone()),
        tags: p.tags.clone(),
        dead: p.dead,
        front: p.front,
        grid,
    })
}

/// A cell beside a door that nobody can stand on (furniture in the doorway), if there is one.
/// Cells outside the building (a space in the grid) are the land's business, not the prefab's.
fn blocked_door(grid: &TextGrid<PrefabCell>) -> Option<(i32, i32)> {
    let blocked = |x: i32, y: i32| {
        (0..grid.width).contains(&x)
            && (0..grid.height).contains(&y)
            && grid.cell(x, y).0.is_some_and(|t| !t.walkable())
    };
    let is_door = |e: Edge| matches!(e, Edge::Door { .. });
    for y in 0..=grid.height {
        for x in 0..grid.width {
            if is_door(grid.north(x, y))
                && let Some(cell) = [(x, y - 1), (x, y)]
                    .into_iter()
                    .find(|&(x, y)| blocked(x, y))
            {
                return Some(cell);
            }
        }
    }
    for y in 0..grid.height {
        for x in 0..=grid.width {
            if is_door(grid.west(x, y))
                && let Some(cell) = [(x - 1, y), (x, y)]
                    .into_iter()
                    .find(|&(x, y)| blocked(x, y))
            {
                return Some(cell);
            }
        }
    }
    None
}

fn check_region(
    file: &Path,
    r: &Region,
    prefabs: &[Prefab],
    creatures: &[CreatureDef],
) -> Result<(), DataError> {
    let t = &r.terrain;
    let unit = |x: f32| (0.0..=1.0).contains(&x);
    let problem = if r.patch.chunks.0 < 1 || r.patch.chunks.1 < 1 {
        Some("patch.chunks must be at least [1, 1]".to_string())
    } else if !(t.hill_size > 0.0
        && unit(t.pond_below)
        && unit(t.woods_from)
        && unit(t.woods_thick)
        && t.woods_from < t.woods_thick
        && unit(t.woods_trees)
        && unit(t.field_trees))
    {
        Some(
            "terrain: hill_size must be above 0, the rest from 0 to 1, with woods_from below \
             woods_thick"
                .to_string(),
        )
    } else if r.road.width < 1 || r.road.bend_every < 1 || r.road.sway < 0 || r.road.start < 0 {
        Some("road: width and bend_every must be at least 1, sway and start 0 or more".to_string())
    } else if r.hamlet.gap < 0 || r.hamlet.setback < 0 || r.hamlet.yard < 0 {
        Some("hamlet: gap, setback and yard can't be negative".to_string())
    } else if r.dead.per_chunk < 0.0 || r.dead.per_house.0 > r.dead.per_house.1 {
        Some("dead: per_chunk can't be negative, and per_house is [fewest, most]".to_string())
    } else if let Some(k) = r
        .dead
        .kinds
        .iter()
        .find(|k| !creatures.iter().any(|c| c.id == k.creature))
    {
        Some(format!(
            "dead.kinds: no creature called {:?} in creatures/",
            k.creature
        ))
    } else if r
        .dead
        .kinds
        .iter()
        .any(|k| k.share.is_nan() || k.share <= 0.0)
        || (r.dead.kinds.iter().map(|k| k.share).sum::<f32>() - 1.0).abs() > 0.001
    {
        Some("dead.kinds: each share is above 0, and they add up to 1".to_string())
    } else if let Some(problem) = r.pressure.problem() {
        Some(problem)
    } else if let Some(problem) = r.tolling.as_ref().and_then(TollingFile::problem) {
        Some(problem)
    } else if let Some(problem) = r.weather.as_ref().and_then(WeatherFile::problem) {
        Some(problem)
    } else if let Some(problem) = r.river.as_ref().and_then(RiverFile::problem) {
        Some(problem)
    } else if let Some(problem) = r.farms.as_ref().and_then(FarmsFile::problem) {
        Some(problem)
    } else if let Some(f) = r
        .farms
        .as_ref()
        .filter(|f| !prefabs.iter().any(|p| p.tags.contains(&f.tag)))
    {
        Some(format!("farms.tag: no prefab has the tag {:?}", f.tag))
    } else if let Some(t) = r
        .tolling
        .as_ref()
        .filter(|t| !prefabs.iter().any(|p| p.id == t.bell))
    {
        Some(format!("tolling.bell: no prefab called {:?}", t.bell))
    } else if let Some(site) = r
        .holdout
        .as_ref()
        .filter(|s| !prefabs.iter().any(|p| p.id == s.prefab))
    {
        Some(format!(
            "holdout.prefab: no prefab called {:?}",
            site.prefab
        ))
    } else {
        r.hamlet
            .lots
            .iter()
            .find(|tag| !prefabs.iter().any(|p| p.tags.contains(tag)))
            .map(|tag| format!("hamlet.lots: no prefab has the tag {tag:?}"))
    };
    match problem {
        Some(message) => Err(fail(file, message)),
        None => Ok(()),
    }
}
