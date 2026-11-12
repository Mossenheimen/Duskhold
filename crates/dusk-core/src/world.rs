//! The world, and its tick.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::actor::{Actor, ActorId, ActorKind, Intent, Mind, Speeds, Step};
use crate::arena::Arena;
use crate::barricade::Barricade;
use crate::body::{Body, Part, WoundKind};
use crate::dread::{Dread, Pain};
use crate::fever::Fever;
use crate::fire::{Fire, Hearth};
use crate::geom::{ChunkPos, Dir8, TilePos};
use crate::grid::Grid;
use crate::hash::{FastMap, Fnv64, Purpose, below, roll};
use crate::home::Deeds;
use crate::injury::Injury;
use crate::items::{Defs, Item};
use crate::light::{Day, Sight, bright_enough};
use crate::needs::{Clock, Fatigue, Need, Needs, Woke};
use crate::path::find_path;
use crate::pressure::Region;
use crate::remains::Burning;
use crate::senses::{Cone, Muffling, hears, line_of_sight};
use crate::skills::{Skill, SkillRules};
use crate::stealth::Sneak;
use crate::text::{self, TextError, TextGrid, default_legend};

/// Ticks in a second of action time at 1×.
pub const TICKS_PER_SECOND: u32 = 10;

/// Thousandths of a tile in a straight step and in a diagonal one.
const STRAIGHT: u32 = 1000;
const DIAGONAL: u32 = 1414;

/// The tunable rules, in the core's units (thousandths of a tile, ticks, cosines). The game's
/// come from `data/rules.toml`, through `dusk-data`; `Rules::default()` is a fixed set for tests
/// and tools. What a kind of creature does is in its own definition (`creature.rs`), not here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// How far round a character the world runs in full, in chunks. Farther out it sleeps.
    pub active_radius: i32,
    pub character_speed: Speeds,
    /// How far footsteps carry, in tiles: walking, running.
    pub walk_noise: i32,
    pub run_noise: i32,
    pub muffling: Muffling,
    /// Ticks waiting on a blocked path before planning it again.
    pub repath_after: u32,
    /// The most tiles one A* search explores for a character. (A creature's own is its kind's.)
    pub path_budget: usize,
    pub combat: Combat,
    pub endurance: Endurance,
    pub clock: Clock,
    pub needs: Needs,
    pub day: Day,
    /// The character's eyes (`view.rs`).
    pub sight: Sight,
    /// Learning by doing (`skills.rs`).
    pub skills: SkillRules,
    /// Boarding up doors and windows (`barricade.rs`).
    pub barricade: Barricade,
    /// Bringing it home, and character levels (`home.rs`).
    pub deeds: Deeds,
    /// Ghoul fever (`fever.rs`).
    pub fever: Fever,
    /// Burning the dead (`remains.rs`).
    pub burning: Burning,
    /// Wading through shallows: how fast, and how loud.
    pub shallows: Shallows,
    /// A fire in a hearth, a stove or a forge (`fire.rs`).
    pub fire: Fire,
    /// Pain from wounds, and dread (`dread.rs`).
    pub pain: Pain,
    pub dread: Dread,
    /// What wounds do where they land (`injury.rs`).
    pub injury: Injury,
    /// Sneaking (`stealth.rs`).
    pub sneak: Sneak,
    /// Bracing behind a shield: what's left of your pace, per mille (`shield.rs`).
    pub shield_pace: u32,
    /// Faith: prayer, turning the dead, consecration, mending and wards (`faith.rs`).
    pub faith: crate::faith::Faith,
}

/// Wading (`Ground::Shallows`), per mille: the pace of a step into it, and its footsteps' noise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shallows {
    pub pace: u32,
    pub noise: u32,
}

impl Default for Shallows {
    fn default() -> Shallows {
        Shallows {
            pace: 500,
            noise: 1500,
        }
    }
}

/// A swing's numbers: its wind-up (the tell) and recovery, in ticks; the endurance it costs (out
/// of `body::FULL`); its chance to land, out of 10,000; the harm it does, a range out of
/// `body::FULL` health; and how far its noise carries, in tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Swing {
    pub wind_up: u32,
    pub recovery: u32,
    pub endurance: i32,
    pub hit: u32,
    pub damage: (i32, i32),
    pub noise: i32,
}

/// What a kind of wound costs: health at once, then ticks of bleeding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WoundRule {
    pub damage: i32,
    pub bleed: u32,
}

/// The fight's numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Combat {
    pub character_health: i32,
    /// The character's swing with nothing in hand (a weapon brings its own: `items.rs`).
    pub attack: Swing,
    /// A shove does no harm (its damage is unused); it knocks back a tile.
    pub shove: Swing,
    /// Out of 10,000: the chance a shove knocks down what it pushes.
    pub knock_down: u32,
    /// Opening or shutting a door: ticks, and how far it's heard.
    pub door_ticks: u32,
    pub door_noise: i32,
    /// What each kind of wound costs, by `WoundKind::ALL`.
    pub wounds: [WoundRule; 3],
    /// Each bleeding wound costs `bleed_damage` health every `bleed_every` ticks.
    pub bleed_every: u32,
    pub bleed_damage: i32,
    /// Grabs (`grab.rs`), out of 10,000: a hit's chance to take hold, more for each other one
    /// on the victim and when it's out of breath; the chance one on you grabs as you step out
    /// of its reach; and how often a hit by one that holds you is a bite.
    pub grab: u32,
    pub grab_per_extra: u32,
    pub grab_tired: u32,
    pub parting_grab: u32,
    pub held_bite: u32,
    /// This many holding you at once drag you down.
    pub drag_down: u32,
    /// Struggling free: its timing and cost, and its chance (`hit`), less for each other
    /// holder.
    pub struggle: Swing,
    pub struggle_per_extra: u32,
    /// A door's strength (out of `body::FULL`), and how far a blow on a door is heard. Beaten
    /// past its strength, a door gives way.
    pub door_strength: i32,
    pub thump_noise: i32,
}

/// Endurance a tick, out of `body::FULL`: spent running, got back walking and standing still.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endurance {
    pub run: i32,
    pub walk: i32,
    pub rest: i32,
    /// Exhausted until endurance is back up to this.
    pub recover_at: i32,
}

impl Default for Rules {
    fn default() -> Rules {
        Rules {
            active_radius: 3,
            character_speed: Speeds {
                walk: 1500,
                run: 3000,
            },
            walk_noise: 3,
            run_noise: 8,
            muffling: Muffling {
                wall: 4,
                shut_door: 2,
                shut_window: 1,
            },
            repath_after: 20,
            path_budget: 4096,
            combat: Combat {
                character_health: 1000,
                attack: Swing {
                    wind_up: 4,
                    recovery: 6,
                    endurance: 90,
                    hit: 8500,
                    damage: (80, 120),
                    noise: 4,
                },
                shove: Swing {
                    wind_up: 2,
                    recovery: 4,
                    endurance: 60,
                    hit: 10_000,
                    damage: (0, 0),
                    noise: 2,
                },
                knock_down: 4000,
                door_ticks: 5,
                door_noise: 3,
                wounds: [
                    WoundRule {
                        damage: 80,
                        bleed: 0,
                    },
                    WoundRule {
                        damage: 150,
                        bleed: 200,
                    },
                    WoundRule {
                        damage: 300,
                        bleed: 400,
                    },
                ],
                bleed_every: 20,
                bleed_damage: 10,
                grab: 1500,
                grab_per_extra: 1500,
                grab_tired: 2000,
                parting_grab: 2500,
                held_bite: 3500,
                drag_down: 3,
                struggle: Swing {
                    wind_up: 5,
                    recovery: 2,
                    endurance: 120,
                    hit: 7000,
                    damage: (0, 0),
                    noise: 3,
                },
                struggle_per_extra: 2000,
                door_strength: 600,
                thump_noise: 8,
            },
            endurance: Endurance {
                run: 4,
                walk: 3,
                rest: 10,
                recover_at: 300,
            },
            // 24× action time: a game minute is 25 ticks. Day 1 starts at 08:00.
            clock: Clock {
                ticks_per_minute: 25,
                start: 8 * 60,
            },
            // A game hour: hunger 6 points, thirst 8, fatigue 4 (12.5 off asleep in a bed);
            // starving costs 2 health an hour, dying of thirst 4; healing gives back 1 (health
            // counts tenths of a point).
            needs: Needs {
                hunger: Need {
                    rise: 100,
                    levels: [30_000, 55_000, 85_000],
                    recovery: 750,
                    harm: 20,
                },
                thirst: Need {
                    rise: 133,
                    levels: [30_000, 55_000, 85_000],
                    recovery: 500,
                    harm: 40,
                },
                fatigue: Fatigue {
                    rise: 67,
                    rest: 208,
                    levels: [45_000, 70_000, 90_000],
                    slow: [1250, 1500],
                },
                asleep: 500,
                heal: 10,
                sleep_heal: 3,
                wound_heal: [12 * 60, 48 * 60, 96 * 60],
                lie_down: 10,
                drink_water: 30,
                sleep_clear: 10,
            },
            // Full day from 07:00 to 19:00, dark from 21:00 to 05:00; the night sky gives a
            // fifth of the day's light, and a roof leaves 60% of the sky's.
            day: Day {
                dawn: 5 * 60,
                sunrise: 7 * 60,
                sunset: 19 * 60,
                dark: 21 * 60,
                night: 20,
                indoors: 600,
            },
            // 20 tiles in full light, 80° either side (a 160° cone), and the tiles beside you.
            sight: Sight {
                range: 20,
                cone: Cone { cos: 1736 },
                beside: 1,
            },
            skills: SkillRules::default(),
            barricade: Barricade::default(),
            deeds: Deeds::default(),
            fever: Fever::default(),
            burning: Burning::default(),
            shallows: Shallows::default(),
            fire: Fire::default(),
            pain: Pain::default(),
            dread: Dread::default(),
            injury: Injury::default(),
            sneak: Sneak::default(),
            shield_pace: 600,
            faith: crate::faith::Faith::default(),
        }
    }
}

/// Things that happened, for the client, logs and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    /// A creature that wasn't after anything heard something at `at`.
    Heard {
        who: ActorId,
        at: TilePos,
    },
    /// A creature that wasn't after anything saw `target`.
    Saw {
        who: ActorId,
        target: ActorId,
    },
    /// A creature got next to `target`, close enough to step onto its tile.
    Reached {
        who: ActorId,
        target: ActorId,
    },
    /// An actor asked to walk to `to` found no way there.
    NoWay {
        who: ActorId,
        to: TilePos,
    },
    /// A sound was made (footsteps, a shout), carrying `loudness` tiles in the open. The client
    /// draws it as a noise ring.
    Sound {
        at: TilePos,
        loudness: i32,
        by: Option<ActorId>,
    },
    /// `who`'s swing hit `target` for `damage` (out of `body::FULL` health). A creature's hit
    /// leaves a wound.
    Struck {
        who: ActorId,
        target: ActorId,
        damage: i32,
        wound: Option<(WoundKind, Part)>,
    },
    /// `who` swung or shoved at `at` and missed, or nobody was there.
    Missed {
        who: ActorId,
        at: TilePos,
    },
    /// `who` shoved `target` back; `down`: it fell.
    Shoved {
        who: ActorId,
        target: ActorId,
        down: bool,
    },
    Died {
        who: ActorId,
    },
    /// `who` opened (or shut) a door. Doors are named by the tile that owns the edge and its
    /// side, north or west (`grid::edge_owner`).
    Door {
        who: ActorId,
        at: TilePos,
        side: Dir8,
        open: bool,
    },
    /// `who` ran out of breath.
    Exhausted {
        who: ActorId,
    },
    /// `who` took hold of `target`.
    Grabbed {
        who: ActorId,
        target: ActorId,
    },
    /// `who` got free of `from` (it was shoved off, struggled loose, let go or went down).
    Freed {
        who: ActorId,
        from: ActorId,
    },
    /// `who` struggled, and didn't get free.
    StruggledInVain {
        who: ActorId,
    },
    /// `who` beat on a shut door.
    Thumped {
        who: ActorId,
        at: TilePos,
        side: Dir8,
    },
    /// A door gave way: the edge is open now, for good.
    DoorBroke {
        at: TilePos,
        side: Dir8,
    },
    /// `holders` of them held `who` at once, or held it with no breath left: it was dragged down.
    DraggedDown {
        who: ActorId,
        holders: u32,
    },
    /// `who` took `count` of an item (by definition number) from a container or the floor.
    Took {
        who: ActorId,
        item: u16,
        count: u16,
    },
    /// `who` dropped `count` of an item on its tile.
    Dropped {
        who: ActorId,
        item: u16,
        count: u16,
    },
    /// `who` took a weapon in hand.
    Wielded {
        who: ActorId,
        item: u16,
    },
    /// The weapon in `who`'s hand broke.
    WeaponBroke {
        who: ActorId,
        item: u16,
    },
    /// `who` bandaged the wound on a part, and it stopped bleeding.
    Bandaged {
        who: ActorId,
        part: Part,
    },
    /// `who` ate, or drank, an item.
    Ate {
        who: ActorId,
        item: u16,
    },
    Drank {
        who: ActorId,
        item: u16,
    },
    /// `who` drank its fill from the water beside it.
    DrankWater {
        who: ActorId,
    },
    /// `who` fell asleep, in a bed or on the ground.
    FellAsleep {
        who: ActorId,
        in_bed: bool,
    },
    /// `who` woke, and why.
    WokeUp {
        who: ActorId,
        why: Woke,
    },
    /// `who` lit a light and held it up, put it out, or it burnt out.
    Lit {
        who: ActorId,
        item: u16,
    },
    Doused {
        who: ActorId,
        item: u16,
    },
    BurntOut {
        who: ActorId,
        item: u16,
    },
    /// The sun is going down (sunset), or the sky is lightening (dawn).
    Nightfall,
    Daybreak,
    /// `who`'s skill rose to a level.
    SkillUp {
        who: ActorId,
        skill: Skill,
        level: u8,
    },
    /// `who`'s skill reached the free cap: it learns no more unless it's chosen as a mastery.
    SkillAtCap {
        who: ActorId,
        skill: Skill,
    },
    /// `who` chose a skill as a mastery: it's at `level` now, and goes on to 10.
    Mastered {
        who: ActorId,
        skill: Skill,
        level: u8,
    },
    /// `who` nailed a board across a door or window (named as doors are), which has `boards` now.
    Boarded {
        who: ActorId,
        at: TilePos,
        side: Dir8,
        boards: u8,
    },
    /// `who` pried a board off; `boards` are left.
    Unboarded {
        who: ActorId,
        at: TilePos,
        side: Dir8,
        boards: u8,
    },
    /// A board was beaten through; `boards` are left.
    BoardBroke {
        at: TilePos,
        side: Dir8,
        boards: u8,
    },
    /// `who` slept the night through: where it slept is home now.
    MadeHome {
        who: ActorId,
        at: TilePos,
    },
    /// `who` made the container on `at` its stash.
    Stashed {
        who: ActorId,
        at: TilePos,
    },
    /// `who` put `count` of an item into a container.
    Put {
        who: ActorId,
        item: u16,
        count: u16,
    },
    /// ...and they were finds, put into its stash: brought home, for `xp` (thousandths).
    BroughtHome {
        who: ActorId,
        item: u16,
        count: u16,
        xp: u32,
    },
    /// `who` reached a character level.
    LevelUp {
        who: ActorId,
        level: u8,
    },
    /// A creature that saw nothing smelled `target`'s blood, and comes for it.
    Smelled {
        who: ActorId,
        target: ActorId,
    },
    /// `who` caught ghoul fever from a wound (`fever.rs`).
    Fevered {
        who: ActorId,
    },
    /// `who`'s fever moved on to `stage`: 2, weakness; 3, dying.
    FeverWorse {
        who: ActorId,
        stage: u8,
    },
    /// The fever ran its course: `who` dies of it (a `Died` follows).
    FeverTook {
        who: ActorId,
    },
    /// `who` took the cure, and the fever's gone.
    Cured {
        who: ActorId,
    },
    /// A body got up again (`remains.rs`): `who` is on its feet, or getting there.
    GotUp {
        who: ActorId,
    },
    /// `who` poured oil on `body` and set it alight.
    Kindled {
        who: ActorId,
        body: ActorId,
    },
    /// `body` caught fire from a body burning beside it.
    Caught {
        body: ActorId,
    },
    /// `body` burnt down to ash.
    Burnt {
        body: ActorId,
    },
    /// `who` put a piece of fuel (`item`) on the fire on `at` (`fire.rs`).
    Laid {
        who: ActorId,
        at: TilePos,
        item: u16,
    },
    /// `who` lit the fire on `at`.
    FireLit {
        who: ActorId,
        at: TilePos,
    },
    /// `who` put out the fire on `at`: the one who tended it, or a Lantern-Eater.
    Smothered {
        who: ActorId,
        at: TilePos,
    },
    /// The fire on `at` burnt out.
    FireOut {
        at: TilePos,
    },
    /// `who`, one of the dead, saw the fire on `at` in the night and comes to it.
    SawFire {
        who: ActorId,
        at: TilePos,
    },
    /// `who`'s dread reached a new level (`dread.rs`): 0 calm, 1 uneasy, 2 afraid, 3 frightened.
    Dread {
        who: ActorId,
        level: u8,
    },
    /// `who` made what recipe `recipe` makes (`craft.rs`).
    Crafted {
        who: ActorId,
        recipe: u16,
    },
    /// A beast took fright, and runs (`hunting.rs`).
    Startled {
        who: ActorId,
    },
    /// `who` cut up the carcass `body`, for `count` of `item` (`hunting.rs`).
    Butchered {
        who: ActorId,
        body: ActorId,
        item: u16,
        count: u16,
    },
    /// Food in `who`'s pack went off, and is gone (`cooking.rs`).
    Spoiled {
        who: ActorId,
        item: u16,
    },
    /// `who` broke the furniture on `at` down into `planks` planks.
    BrokeDown {
        who: ActorId,
        at: TilePos,
        planks: u16,
    },
    /// `who` took up sneaking (`on`), or left it (`stealth.rs`).
    Sneaking {
        who: ActorId,
        on: bool,
    },
    /// `who` loosed an arrow from `from`, and it flew to `to` (`archery.rs`).
    Shot {
        who: ActorId,
        from: TilePos,
        to: TilePos,
    },
    /// `who` braced behind its shield (`on`), or stopped (`shield.rs`).
    Bracing {
        who: ActorId,
        on: bool,
    },
    /// `who` blocked a blow from `by` with its shield.
    Blocked {
        who: ActorId,
        by: ActorId,
    },
    /// `who` struck `target` unawares: a sneak attack.
    SneakAttack {
        who: ActorId,
        target: ActorId,
    },
    /// `who`, frightened, couldn't make itself step nearer the dead.
    Balked {
        who: ActorId,
    },
    /// `who`'s blow on `target` was turned by its kind (a blade on bone), or, `weak`, told
    /// all the more (a mace on bone: `creature.rs`).
    Resisted {
        who: ActorId,
        target: ActorId,
        weak: bool,
    },
    /// `who` bought one of an item at a holdout, for `price` coin (`holdout.rs`).
    Bought {
        who: ActorId,
        item: u16,
        price: u32,
    },
    /// `who` sold `count` of an item at a holdout, for `price` coin.
    Sold {
        who: ActorId,
        item: u16,
        count: u16,
        price: u32,
    },
    /// ...and it was what the holdout was short of: the pressure fell `by` (thousandths).
    Supplied {
        who: ActorId,
        by: u32,
    },
    /// `who` had the weapon in hand mended, for `price` coin.
    Mended {
        who: ActorId,
        item: u16,
        price: u32,
    },
    /// Dawn on a Tolling day: the bell tolls tonight (`tolling.rs`).
    TollingTonight,
    /// The Tolling began: the bell at `at` tolls, and the dead in reach walk to it.
    TollingBegan {
        at: TilePos,
    },
    /// A toll of the bell.
    Tolled {
        at: TilePos,
    },
    /// Dawn: the Tolling's over, and the dead give up as they did.
    TollingEnded,
    /// A Lantern-Eater put out the light `who` held up (`rare.rs`).
    LightEaten {
        by: ActorId,
        who: ActorId,
    },
    /// `who` learnt what a magic item is (`magic.rs`): by use, by lighting it, or at a holdout.
    Identified {
        who: ActorId,
        item: u16,
    },
    /// A cursed weapon `who` took in hand clings: its curse shows itself.
    CurseClings {
        who: ActorId,
        item: u16,
    },
    /// A holdout lifted the curse on the weapon in `who`'s hand.
    CurseLifted {
        who: ActorId,
        item: u16,
    },
    /// `who` claimed the bounty on a rare put down, for `coin` (`holdout.rs`).
    BountyClaimed {
        who: ActorId,
        rare: ActorId,
        coin: u32,
    },
    /// A character's body rose at midnight as a revenant, in its gear (`legacy.rs`).
    Risen {
        body: ActorId,
        revenant: ActorId,
    },
    /// The next character came into the world.
    Came {
        who: ActorId,
    },
    /// `who` chose a talent (`class.rs`).
    TalentChosen {
        who: ActorId,
        talent: u8,
    },
    /// `who` caught its second wind.
    SecondWind {
        who: ActorId,
    },
    /// It began to rain (`weather.rs`), or rain hard.
    RainBegan {
        heavy: bool,
    },
    /// The rain stopped.
    RainEnded,
    /// The rain put out the light `who` held up.
    RainedOut {
        who: ActorId,
        item: u16,
    },
    /// `who` took up a class before it was under way (`class.rs`).
    ClassChosen {
        who: ActorId,
        class: u8,
    },
    /// `who` held up the holy symbol (`faith.rs`): `turned` of the dead turned from it, and
    /// with Sunder, `sundered` fell.
    Turned {
        who: ActorId,
        turned: u16,
        sundered: u16,
    },
    /// `who` prayed at a shrine: its dread eased, and its faith back (`faith`: it has any).
    Prayed {
        who: ActorId,
        faith: bool,
    },
    /// `who` consecrated `body`, with holy water (`water`) or with faith: it never gets up.
    Consecrated {
        who: ActorId,
        body: ActorId,
        water: bool,
    },
    /// `who` mended a wound with faith (Mending).
    Healed {
        who: ActorId,
    },
    /// `who` warded the door on `at`'s `side` (named as doors are): the dead won't beat on it.
    Warded {
        who: ActorId,
        at: TilePos,
        side: Dir8,
    },
    /// The ward on a door faded.
    WardFaded {
        at: TilePos,
        side: Dir8,
    },
}

/// A sound made this tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Sound {
    pub at: TilePos,
    /// How many tiles it carries in the open.
    pub loudness: i32,
    pub by: Option<ActorId>,
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: u64,
    pub tick: u64,
    pub rules: Rules,
    pub grid: Grid,
    pub(crate) actors: Arena<Actor>,
    pub(crate) characters: Vec<ActorId>,
    /// Who's on each tile, or stepping onto it.
    pub(crate) occupied: FastMap<TilePos, ActorId>,
    sounds: Vec<Sound>,
    pub(crate) events: Vec<Event>,
    /// The harm each beaten door has taken, by (owner tile, side). Ordered, for the hash.
    pub(crate) door_damage: BTreeMap<(TilePos, Dir8), i32>,
    /// What each board nailed across a door or window has left, the last nailed last, by
    /// (owner tile, side) (`barricade.rs`). Ordered, for the hash.
    pub(crate) board_strength: BTreeMap<(TilePos, Dir8), Vec<i32>>,
    /// The data's definitions (items, loot tables, containers, the start kit), shared
    /// read-only by every world.
    pub defs: Arc<Defs>,
    /// What containers and floors hold, once something changed it (`items.rs`): an unopened
    /// container isn't here, its contents come from the seed. Ordered, for the hash.
    pub(crate) stores: BTreeMap<TilePos, Vec<Item>>,
    /// The region this world is a patch of (`pressure.rs`): from the data, not saved.
    pub region: Region,
    /// The region's pressure, in thousandths of the dead the patch began with.
    pub(crate) pressure: u32,
    /// The bodies burning now (`remains.rs`): worked out from the actors, kept for the light.
    pub(crate) fires: Vec<ActorId>,
    /// The fires laid in hearths, stoves and forges (`fire.rs`): only those with fuel in them.
    /// Ordered, for the hash.
    pub(crate) hearths: BTreeMap<TilePos, Hearth>,
    /// Hallowed ground (`holdout.rs`): inside the holdouts' walls. From the patch's making, not
    /// saved.
    pub hallowed: Vec<crate::holdout::Area>,
    /// What each holdout's line has sold, and on which game day, by (holdout, line). Ordered,
    /// for the hash.
    pub(crate) sold: BTreeMap<(u16, u16), (u64, u16)>,
    /// The bell's flow field while the Tolling's on (`tolling.rs`): worked out, not saved.
    pub(crate) flow: Option<crate::flow::Flow>,
    /// The rares whose bounties have been claimed (`holdout.rs`). Ordered, for the hash.
    pub(crate) claimed: std::collections::BTreeSet<ActorId>,
    /// A blow carrying on into another foe (Cleave, `class.rs`): only ever true inside a strike,
    /// so one cleave doesn't start another.
    pub(crate) cleaving: bool,
    /// The doors warded against the dead (`faith.rs`), by (owner tile, side), and the tick each
    /// ward fades. Ordered, for the hash.
    pub(crate) wards: BTreeMap<(TilePos, Dir8), u64>,
}

impl World {
    /// An empty world with the fixed test rules, and no items.
    pub fn new(seed: u64) -> World {
        World::with_rules(seed, Rules::default())
    }

    pub fn with_rules(seed: u64, rules: Rules) -> World {
        World::with_defs(seed, rules, Arc::new(Defs::default()))
    }

    pub fn with_defs(seed: u64, rules: Rules, defs: Arc<Defs>) -> World {
        World {
            seed,
            tick: 0,
            rules,
            grid: Grid::new(),
            actors: Arena::new(),
            characters: Vec::new(),
            occupied: FastMap::default(),
            sounds: Vec::new(),
            events: Vec::new(),
            door_damage: BTreeMap::new(),
            board_strength: BTreeMap::new(),
            defs,
            stores: BTreeMap::new(),
            region: Region::default(),
            pressure: 0,
            fires: Vec::new(),
            hearths: BTreeMap::new(),
            hallowed: Vec::new(),
            sold: BTreeMap::new(),
            flow: None,
            claimed: std::collections::BTreeSet::new(),
            cleaving: false,
            wards: BTreeMap::new(),
        }
    }

    /// A world drawn as a text grid (see [`text`]), its north-west cell at (0, 0) on the ground
    /// floor. `@` puts the character there, `z` the first creature in the definitions (the
    /// shambler). Returns the world and the actors placed, in reading order.
    pub fn from_text(seed: u64, map: &str) -> Result<(World, Vec<ActorId>), TextError> {
        let parsed = TextGrid::parse(map, default_legend)?;
        let mut world = World::new(seed);
        parsed.stamp(&mut world.grid, TilePos::new(0, 0, 0));
        let mut placed = Vec::new();
        for (marker, x, y) in parsed.markers() {
            let kind = match marker {
                '@' => ActorKind::Character,
                'z' | 'Z' => ActorKind::Creature(0),
                _ => continue,
            };
            placed.extend(world.spawn(kind, TilePos::new(x, y, 0)));
        }
        Ok((world, placed))
    }

    /// Puts an actor on a free, walkable tile. None if the tile isn't, or the definitions have
    /// no such creature.
    pub fn spawn(&mut self, kind: ActorKind, pos: TilePos) -> Option<ActorId> {
        if !self.grid.tile(pos).walkable() || self.occupied.contains_key(&pos) {
            return None;
        }
        let health = match kind {
            ActorKind::Character => self.rules.combat.character_health,
            ActorKind::Creature(_) => self.creature(kind)?.health,
        };
        let mut actor = Actor::new(kind, pos);
        actor.body = Body::new(health);
        // A beast is after nobody: it runs (`hunting.rs`).
        if self.creature(kind).is_some_and(|c| c.flees > 0) {
            actor.mind = Mind::Flee {
                from: None,
                since: 0,
            };
        }
        actor.born = self.clock_minutes();
        if kind == ActorKind::Character {
            actor.name = self.roll_name(self.actors.len() as u64);
            // The first class in the data, until another is chosen (`class.rs`).
            actor.class = (!self.defs.classes.is_empty()).then_some(0);
        }
        let id = self.actors.insert(actor);
        self.occupied.insert(pos, id);
        if kind == ActorKind::Character {
            self.characters.push(id);
            // It starts with its class's kit (or everyone's) and its faith.
            self.equip(id);
        }
        Some(id)
    }

    /// Sounds made since the last hearing pass, to be heard on the next tick.
    #[cfg(feature = "save")]
    pub(crate) fn pending_sounds(&self) -> &[Sound] {
        &self.sounds
    }

    /// Replaces every actor (a save's), and works out again who stands where.
    #[cfg(feature = "save")]
    pub(crate) fn set_actors(
        &mut self,
        actors: Arena<Actor>,
        characters: Vec<ActorId>,
        sounds: Vec<Sound>,
    ) {
        self.actors = actors;
        self.characters = characters;
        self.sounds = sounds;
        self.events.clear();
        self.occupied.clear();
        for (id, a) in self.actors.iter() {
            // The living hold their tile, and the one they're stepping onto.
            if !a.dead {
                self.occupied.insert(a.pos, id);
                if let Some(step) = a.step {
                    self.occupied.insert(step.to, id);
                }
            }
        }
        self.find_fires();
    }

    pub fn actor(&self, id: ActorId) -> Option<&Actor> {
        self.actors.get(id)
    }

    /// Every actor, in id order.
    pub fn actors(&self) -> impl Iterator<Item = (ActorId, &Actor)> {
        self.actors.iter()
    }

    /// The player's character: the last to come into the world (after a death, the next one).
    pub fn character(&self) -> Option<ActorId> {
        self.characters.last().copied()
    }

    /// Asks an actor to do something from now on.
    pub fn intent(&mut self, id: ActorId, intent: Intent) {
        if let Some(a) = self.actors.get_mut(id) {
            if a.intent != intent {
                a.path.clear();
                a.path_goal = None;
                a.blocked = 0;
            }
            a.intent = intent;
        }
    }

    /// How far a kind of creature sees in full light, and how wide: None for the character, whose
    /// field of view is another rule (`view.rs`).
    pub fn sight(&self, kind: ActorKind) -> Option<(i32, Cone)> {
        self.creature(kind).map(|c| (c.sight, c.cone))
    }

    /// Can a creature see a tile: in range, inside its cone, lit enough for how far off it is
    /// (or within its darkvision), and in a line of sight? The one rule for creatures' eyes.
    /// Their hunting uses it, and so does the client's drawing of what they see.
    pub fn sees(&self, who: ActorId, at: TilePos) -> bool {
        let Some(a) = self.actors.get(who) else {
            return false;
        };
        if a.dead || a.prone_until > 0 {
            return false; // the dead see nothing, and nor does one lying on its back
        }
        // Its kind's eyes, read in place: no affix changes them (`rare.rs`), and this runs for
        // every tile the client shades, so nothing is copied or worked out.
        let ActorKind::Creature(n) = a.kind else {
            return false;
        };
        let Some(c) = self.defs.creature(n).map(|d| &d.stats) else {
            return false;
        };
        let (range, cone) = (c.sight, c.cone);
        let d2 = a.pos.dist2(at);
        a.pos.z == at.z
            && d2 <= i64::from(range).pow(2)
            && cone.contains(a.facing, at.x - a.pos.x, at.y - a.pos.y)
            && (a.pos.steps_to(at) <= c.darkvision || bright_enough(d2, self.light_at(at), range))
            && line_of_sight(&self.grid, a.pos, at)
    }

    /// Makes a sound at a tile, heard on the next tick (what a shout or a dropped pan does).
    pub fn make_sound(&mut self, at: TilePos, loudness: i32, by: Option<ActorId>) {
        self.emit(Sound { at, loudness, by });
    }

    /// A sound made: heard by the creatures on this tick's hearing pass, and told to the client.
    pub(crate) fn emit(&mut self, s: Sound) {
        self.sounds.push(s);
        self.events.push(Event::Sound {
            at: s.at,
            loudness: s.loudness,
            by: s.by,
        });
    }

    /// Advances the world by some ticks.
    pub fn step(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.tick_once();
        }
    }

    /// What happened since the last call.
    pub fn events(&mut self) -> std::vec::Drain<'_, Event> {
        self.events.drain(..)
    }

    /// A hash of everything that makes the game go on the way it does. The same seed and the same
    /// intents give the same hashes, tick for tick.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv64::default();
        self.tick.hash(&mut h);
        self.grid.hash(&mut h);
        self.door_damage.hash(&mut h);
        self.board_strength.hash(&mut h);
        self.stores.hash(&mut h);
        self.pressure.hash(&mut h);
        self.sold.hash(&mut h);
        self.claimed.hash(&mut h);
        self.hearths.hash(&mut h);
        self.wards.hash(&mut h);
        for (id, a) in self.actors.iter() {
            id.hash(&mut h);
            a.hash(&mut h);
        }
        h.finish()
    }

    /// Draws `w × h` tiles from `origin` as a text grid: `@` the character, `z` a creature that
    /// isn't after anything, `Z` one that is.
    pub fn render(&self, origin: TilePos, w: i32, h: i32) -> String {
        let mut marks: FastMap<TilePos, char> = FastMap::default();
        for (_, a) in self.actors.iter() {
            let mark = match (a.kind, a.mind) {
                _ if a.dead => 'x',
                (ActorKind::Character, _) => '@',
                (
                    ActorKind::Creature(_),
                    Mind::Hunt {
                        target: Some(_), ..
                    },
                ) => 'Z',
                (ActorKind::Creature(_), _) => 'z',
            };
            marks.insert(a.pos, mark);
        }
        text::render(&self.grid, origin, w, h, |p| marks.get(&p).copied())
    }

    /// Whether the world runs in full at a tile: within `active_radius` chunks of a character.
    /// Elsewhere it sleeps, and creatures there stay as they are until someone comes near.
    pub fn is_awake(&self, pos: TilePos) -> bool {
        let r = self.rules.active_radius;
        self.anchors().any(|at| {
            let (a, b) = (pos.chunk(), at.chunk());
            at.z == pos.z && (a.x - b.x).abs() <= r && (a.y - b.y).abs() <= r
        })
    }

    /// What keeps the world awake round it: each character, and the bell while the Tolling's
    /// on (`tolling.rs`), so the dead walk to it even with you away.
    fn anchors(&self) -> impl Iterator<Item = TilePos> + '_ {
        let characters = self
            .characters
            .iter()
            .filter_map(|&id| self.actors.get(id))
            .filter(|c| !c.dead);
        let bell = self.region.bell.filter(|_| self.flow.is_some());
        characters.map(|c| c.pos).chain(bell)
    }

    /// The chunks that are awake, for the tools and the client.
    pub fn awake_chunks(&self) -> Vec<ChunkPos> {
        let r = self.rules.active_radius;
        let mut out: Vec<ChunkPos> = Vec::new();
        for anchor in self.anchors() {
            let at = anchor.chunk();
            for y in at.y - r..=at.y + r {
                for x in at.x - r..=at.x + r {
                    out.push(ChunkPos { x, y });
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }

    /// One tick, in a fixed order: timed actions or else movement, then hearing, then thinking
    /// (whose intents act on the next tick), then bodies. Only the living where the world is
    /// awake take part.
    fn tick_once(&mut self) {
        self.tick += 1;
        let mut ids = self.actors.ids();
        ids.retain(|&id| {
            self.actors
                .get(id)
                .is_some_and(|a| !a.dead && self.is_awake(a.pos))
        });
        // Each mover's stride this tick (Some(running)), for what its body pays.
        let mut strides = Vec::with_capacity(ids.len());
        let night = self.is_night();
        for &id in &ids {
            let stride = if self.act(id) {
                None
            } else {
                self.move_actor(id, night)
            };
            strides.push((id, stride));
        }
        self.hear(&ids);
        // Heard, this tick's sounds are done with. One made from here on (a groan, as a creature
        // thinks) is heard on the next tick.
        self.sounds.clear();
        for &id in &ids {
            self.think(id);
        }
        self.grips();
        self.bodies(&strides);
        self.needs();
        self.minds();
        self.remains();
        self.press();
        self.toll();
        self.weather();
        self.burn_lights();
        self.burn_fires();
        self.spoil();
        self.lapse_wards();
        self.drag_downs();
    }

    /// Starts a step if the actor's intent asks for one, then moves it along. The tile ahead is
    /// taken as the step starts, so two actors never share a tile. At night the dead are faster.
    fn move_actor(&mut self, id: ActorId, night: bool) -> Option<bool> {
        let rules = self.rules;
        // Its pace and wits: a creature's own (with its affixes), or the character's.
        let (speeds, path_budget, night_pace) = self.pace(id)?;
        // Held, a character can't step; and as it steps out of reach of the dead on it, each
        // may grab it as it goes (`grab.rs`). Only characters are held, which keeps this cheap.
        let starting = self
            .actors
            .get(id)
            .is_some_and(|a| a.kind == ActorKind::Character && a.step.is_none());
        let held = starting && self.is_held(id);
        let heavy = starting && self.weighed_down(id);
        // Frightened, a character won't step nearer what it's frightened of (`dread.rs`).
        let fear = if starting { self.fear(id) } else { None };
        let athletics = self
            .actors
            .get(id)
            .filter(|a| a.kind == ActorKind::Character)
            .map_or(0, |a| a.skills.level(Skill::Athletics, &rules.skills));
        let on_me: Vec<(ActorId, TilePos)> = if starting && !held {
            self.engaged(id)
                .into_iter()
                .filter_map(|e| {
                    self.actors
                        .get(e)
                        .filter(|o| o.holding.is_none())
                        .map(|o| (e, o.pos))
                })
                .collect()
        } else {
            Vec::new()
        };
        let a = self.actors.get_mut(id)?;
        if a.step.is_none() {
            // Holding on to its prey, or held, nobody steps.
            if held || a.holding.is_some() {
                a.carry = 0;
                return None;
            }
            let (dir, run) = match next_heading(a, &self.grid, rules.repath_after, path_budget) {
                // Out of breath or weighed down, nobody runs.
                // Sneaking, nobody runs either.
                Heading::Go(dir, run) => (
                    dir,
                    run && !a.body.exhausted && !heavy && !a.sneaking && !a.bracing,
                ),
                Heading::Stand => {
                    a.carry = 0;
                    return None;
                }
                Heading::NoWay(to) => {
                    a.carry = 0;
                    self.events.push(Event::NoWay { who: id, to });
                    return None;
                }
            };
            a.facing = dir;
            let to = a.pos.step(dir);
            // Only creatures have a night pace, and none of them steps onto hallowed ground.
            let barred = night_pace.is_some() && crate::holdout::in_hallowed(&self.hallowed, to);
            if fear.is_some_and(|f| to.dist2(f) < a.pos.dist2(f)) {
                a.blocked += 1;
                a.carry = 0;
                self.events.push(Event::Balked { who: id });
                return None;
            }
            if barred || !self.grid.can_step(a.pos, dir) || self.occupied.contains_key(&to) {
                a.blocked += 1;
                a.carry = 0;
                return None;
            }
            a.blocked = 0;
            let (seed, tick) = (self.seed, self.tick);
            let parting = on_me.iter().find(|&&(e, at)| {
                !reaches(&self.grid, at, to)
                    && below(roll(seed, tick, e.index() as u64, Purpose::Grab), 10_000)
                        < rules.combat.parting_grab
            });
            if let Some(&(e, _)) = parting {
                a.carry = 0;
                self.grab(e, id);
                return None;
            }
            if matches!(a.intent, Intent::WalkTo { .. }) {
                a.path.pop();
            }
            let length = if dir.is_diagonal() {
                DIAGONAL
            } else {
                STRAIGHT
            };
            let wade = self.grid.tile(to).ground == crate::grid::Ground::Shallows;
            a.step = Some(Step {
                from: a.pos,
                to,
                dir,
                progress: a.carry,
                length,
                run,
                wade,
            });
            a.carry = 0;
            self.occupied.insert(to, id);
        }
        let mut step = a.step?;
        let mut speed = if step.run {
            rules.skills.run_speed(speeds.run, athletics)
        } else {
            speeds.walk
        };
        if let (true, Some(pace)) = (night, night_pace) {
            speed = speed * pace / 1000;
        }
        // Wading is slow going, for everyone.
        let wading = step.wade;
        if wading {
            speed = speed * rules.shallows.pace / 1000;
        }
        step.progress += speed / TICKS_PER_SECOND;
        if step.progress < step.length {
            a.step = Some(step);
            return Some(step.run);
        }
        a.step = None;
        a.pos = step.to;
        a.carry = step.progress - step.length;
        let footsteps = (a.kind == ActorKind::Character).then_some(if step.run {
            rules.run_noise
        } else {
            rules.walk_noise
        });
        // Sneaking, a footstep carries a fraction as far (`stealth.rs`).
        let sneaking = a.sneaking;
        let footsteps = footsteps.map(|n| {
            if sneaking {
                (i64::from(n) * i64::from(rules.sneak.noise) / 1000).max(1) as i32
            } else {
                n
            }
        });
        // A splash carries farther than a footstep.
        let footsteps = footsteps.map(|n| {
            if wading {
                (i64::from(n) * i64::from(rules.shallows.noise) / 1000) as i32
            } else {
                n
            }
        });
        self.occupied.remove(&step.from);
        if let Some(loudness) = footsteps {
            self.emit(Sound {
                at: step.to,
                loudness,
                by: Some(id),
            });
            // A character's run, a tile at a time, may teach it something.
            if step.run {
                self.train_run(id);
            } else if sneaking {
                self.train_sneak(id);
            }
        }
        Some(step.run)
    }

    /// Creatures hear this tick's sounds: each goes after the nearest one it heard.
    fn hear(&mut self, ids: &[ActorId]) {
        if self.sounds.is_empty() {
            return;
        }
        let (rules, tick) = (self.rules, self.tick);
        for &id in ids {
            let Some(a) = self.actors.get(id) else {
                continue;
            };
            // A sleeper wakes to any noise that reaches it, but its own.
            if a.body.asleep {
                let pos = a.pos;
                let woken = self.sounds.iter().any(|s| {
                    let loud = self.carries(s.at, s.loudness);
                    s.by != Some(id) && hears(&self.grid, s.at, pos, loud, rules.muffling)
                });
                if woken {
                    self.wake(id, Woke::Noise);
                }
                continue;
            }
            // A beast runs from the nearest noise it hears (`hunting.rs`).
            if matches!(a.mind, Mind::Flee { .. }) {
                let pos = a.pos;
                let heard = self
                    .sounds
                    .iter()
                    .filter(|s| {
                        let loud = self.carries(s.at, s.loudness);
                        s.by != Some(id) && hears(&self.grid, s.at, pos, loud, rules.muffling)
                    })
                    .map(|s| s.at)
                    .min_by_key(|at| at.dist2(pos));
                if let Some(at) = heard {
                    self.startle(id, at);
                }
                continue;
            }
            let Mind::Hunt {
                target,
                touching,
                sees,
                ..
            } = a.mind
            else {
                continue;
            };
            let pos = a.pos;
            let mut nearest: Option<(i64, TilePos)> = None;
            for s in &self.sounds {
                // What's done on hallowed ground, the dead pay no heed; and the bell draws them
                // by its field, not as a noise to chase.
                if crate::holdout::in_hallowed(&self.hallowed, s.at) || self.is_bell(s) {
                    continue;
                }
                // Out in the rain, a sound carries less far.
                let loud = self.carries(s.at, s.loudness);
                if s.by != Some(id) && hears(&self.grid, s.at, pos, loud, rules.muffling) {
                    let d2 = s.at.dist2(pos);
                    if nearest.is_none_or(|(best, _)| d2 < best) {
                        nearest = Some((d2, s.at));
                    }
                }
            }
            let Some((_, at)) = nearest else { continue };
            if target.is_none() {
                self.events.push(Event::Heard { who: id, at });
            }
            if let Some(a) = self.actors.get_mut(id) {
                a.mind = Mind::Hunt {
                    target: Some(at),
                    since: tick,
                    touching,
                    sees,
                };
            }
        }
    }

    /// A creature's turn to think: look for prey, give up on old leads, decide where to go.
    fn think(&mut self, id: ActorId) {
        let (tick, seed) = (self.tick, self.seed);
        // Its beat first, from its kind alone (no affix changes it): most ticks a creature
        // doesn't think, and then nothing more is worked out.
        let Some(a) = self.actors.get(id) else { return };
        let ActorKind::Creature(n) = a.kind else {
            return; // a character's thoughts are its controller's
        };
        let every = self.defs.creature(n).map_or(1, |c| c.stats.think_every);
        if !(tick + id.index() as u64).is_multiple_of(every.max(1)) {
            return;
        }
        if a.prone_until > 0 || a.dead {
            return;
        }
        let Some(kind) = self.stats(id) else { return };
        // A beast thinks of nothing but getting away (`hunting.rs`).
        if kind.flees > 0 {
            self.think_beast(id, kind);
            return;
        }
        // Turned, one of the dead thinks only of getting away from whoever turned it (`faith.rs`).
        if self.think_turned(id) {
            return;
        }
        // Only after dark: by day it lies still where it is, after nothing (`creature.rs`).
        if kind.nocturnal && !self.is_night() {
            if let Some(a) = self.actors.get_mut(id) {
                a.intent = Intent::Idle;
                if let Mind::Hunt { target, .. } = &mut a.mind {
                    *target = None;
                }
            }
            return;
        }
        if kind.douses > 0 {
            self.eat_lights(id, kind.douses);
        }
        let Some(a) = self.actors.get(id) else { return };
        // Holding on to its prey, it keeps at it.
        if let Some(victim) = a.holding {
            let at = self.actors.get(victim).map_or(a.pos, |v| v.pos);
            let dir = Dir8::toward(at.x - a.pos.x, at.y - a.pos.y);
            if let Some(a) = self.actors.get_mut(id) {
                a.intent = Intent::Attack { dir };
            }
            return;
        }
        let Mind::Hunt {
            mut target,
            mut since,
            touching,
            ..
        } = a.mind
        else {
            return;
        };
        let pos = a.pos;

        let seen = self.nearest_visible_character(id);
        if let Some((prey, at)) = seen {
            if target.is_none() {
                self.events.push(Event::Saw {
                    who: id,
                    target: prey,
                });
                // Catching sight of prey, it groans, and the others in earshot come to it. Only
                // one that was after nothing groans, so a crowd doesn't call itself for ever.
                if kind.groan > 0 {
                    // (`emit`, written out: the actor is still borrowed here.)
                    let (at, loudness, by) = (pos, kind.groan, Some(id));
                    self.sounds.push(Sound { at, loudness, by });
                    self.events.push(Event::Sound { at, loudness, by });
                }
            }
            target = Some(at);
            since = tick;
        }
        // Seeing nothing, one with a nose for it follows blood: to where the prey is now, so
        // it can't be lost while the wound bleeds.
        if seen.is_none()
            && let Some((prey, at)) = self.smells(id, kind.blood)
        {
            if target.is_none() {
                self.events.push(Event::Smelled {
                    who: id,
                    target: prey,
                });
            }
            target = Some(at);
            since = tick;
        }
        // Gone past the forgetting, or onto hallowed ground, where it won't follow. On a Tolling
        // night, the dead the bell draws don't forget: what they're after, they keep after.
        let hallowed = target.is_some_and(|t| self.is_hallowed(t));
        // The bell draws the dead, not the wild (`hunting.rs`).
        let belled = !kind.wild && self.drawn_to_bell(pos);
        let forgot = tick.saturating_sub(since) > kind.forget_after && !belled;
        if forgot || hallowed {
            target = None;
        }
        let next_to = seen
            .filter(|&(_, at)| self.can_reach(pos, at))
            .map(|(prey, _)| prey);
        if let Some(prey) = next_to
            && !touching
        {
            self.events.push(Event::Reached {
                who: id,
                target: prey,
            });
        }
        let intent = match (next_to, target) {
            // Next to its prey: it swings at it. (Grabs come next.)
            (Some(_), Some(goal)) => Intent::Attack {
                dir: Dir8::toward(goal.x - pos.x, goal.y - pos.y),
            },
            (Some(_), None) => Intent::Idle,
            // Got to where the noise was, and nothing's there.
            (None, Some(goal)) if goal == pos => {
                target = None;
                Intent::Idle
            }
            (None, Some(goal)) => self.chase(a, goal),
            // After nothing, on a Tolling night: to the bell, by its field.
            (None, None) if belled => self.pulled(id, pos).unwrap_or_default(),
            (None, None) => {
                let who = id.index() as u64;
                if below(roll(seed, tick, who, Purpose::Wander), 10_000) < kind.wander {
                    let dir =
                        Dir8::ALL[below(roll(seed, tick, who, Purpose::WanderDir), 8) as usize];
                    Intent::Move { dir, run: false }
                } else {
                    Intent::Idle
                }
            }
        };
        if let Some(a) = self.actors.get_mut(id) {
            a.mind = Mind::Hunt {
                target,
                since,
                touching: next_to.is_some(),
                sees: seen.is_some(),
            };
            a.intent = intent;
        }
    }

    /// How a shambler goes after `goal`: straight at it while it's in sight and the walls allow,
    /// otherwise along a path, through doors and round walls. Where it found no way, it presses
    /// toward the goal anyway, as the dead do against a wall.
    fn chase(&self, a: &Actor, goal: TilePos) -> Intent {
        // Mid-step, it plans from the tile it's stepping onto: the heading takes effect there.
        // (From the tile it's leaving, its own reserved tile would look like someone in the way.)
        let pos = a.step.map_or(a.pos, |s| s.to);
        if line_of_sight(&self.grid, pos, goal)
            && let Some(dir) = self.open_step_toward(pos, goal)
        {
            return Intent::Move { dir, run: true };
        }
        if a.no_way.is_some_and(|tried| tried.steps_to(goal) <= 1) {
            // No way round: a shut door straight ahead gets beaten on; a wall, pressed against.
            if let Some(side) = Dir8::nearest(goal.x - pos.x, goal.y - pos.y)
                .and_then(|d| self.shut_door_toward(pos, d))
            {
                return Intent::Attack { dir: Some(side) };
            }
            return Intent::Move {
                dir: self.lurch_toward(pos, goal),
                run: true,
            };
        }
        // Keep the path already planned while the goal has only shifted by a tile.
        let to = match a.intent {
            Intent::WalkTo { to, .. } if to.steps_to(goal) <= 1 => to,
            _ => goal,
        };
        Intent::WalkTo { to, run: true }
    }

    /// The step straight at `goal`, when the walls allow it; None when they don't (then the way
    /// round needs a path: a step to one side would only look closer). Someone standing in that
    /// tile is gone round by an eighth of a turn if that's free, else waited behind. When that
    /// tile is the goal itself, the step stands: it turns to what's there (and so can see it).
    fn open_step_toward(&self, from: TilePos, goal: TilePos) -> Option<Dir8> {
        let dir = Dir8::nearest(goal.x - from.x, goal.y - from.y)?;
        if !self.grid.can_step(from, dir) {
            return None;
        }
        let ahead = from.step(dir);
        if ahead == goal || !self.occupied.contains_key(&ahead) {
            return Some(dir);
        }
        for d in [dir.turned(1), dir.turned(-1)] {
            if self.grid.can_step(from, d) && !self.occupied.contains_key(&from.step(d)) {
                return Some(d);
            }
        }
        Some(dir)
    }

    /// Could an actor at `from` step onto `to`'s tile: next to it, with no wall or corner in the
    /// way? (Whoever stands there doesn't count.)
    pub(crate) fn can_reach(&self, from: TilePos, to: TilePos) -> bool {
        reaches(&self.grid, from, to)
    }

    /// The side of `from` with a shut door the dead may beat on, going `dir` (for a diagonal,
    /// either of its sides): not one that's warded (`faith.rs`).
    fn shut_door_toward(&self, from: TilePos, dir: Dir8) -> Option<Dir8> {
        let sides = match dir.split() {
            Some((a, b)) => [Some(a), Some(b)],
            None => [Some(dir), None],
        };
        sides
            .into_iter()
            .flatten()
            .find(|&d| self.grid.edge(from, d).is_shut_door() && !self.is_warded(from, d))
    }

    /// The heading a shambler lurches in to reach `goal`: straight at it, or an eighth of a turn
    /// to either side when that way is blocked (by a wall or by another actor).
    fn lurch_toward(&self, from: TilePos, goal: TilePos) -> Dir8 {
        let Some(dir) = Dir8::nearest(goal.x - from.x, goal.y - from.y) else {
            return Dir8::S;
        };
        for d in [dir, dir.turned(1), dir.turned(-1)] {
            if self.grid.can_step(from, d) && !self.occupied.contains_key(&from.step(d)) {
                return d;
            }
        }
        dir
    }

    /// The nearest character that `who` sees.
    pub(crate) fn nearest_visible_character(&self, who: ActorId) -> Option<(ActorId, TilePos)> {
        let from = self.actors.get(who)?.pos;
        let mut best: Option<(i64, ActorId, TilePos)> = None;
        for &id in &self.characters {
            let Some(c) = self
                .actors
                .get(id)
                .filter(|c| !c.dead && !self.is_hallowed(c.pos))
            else {
                continue;
            };
            let d2 = from.dist2(c.pos);
            if best.is_some_and(|(b, ..)| d2 >= b) {
                continue;
            }
            // Sneaking, only from nearer (`stealth.rs`).
            if self.sees(who, c.pos) && self.spots(who, c) {
                best = Some((d2, id, c.pos));
            }
        }
        best.map(|(_, id, at)| (id, at))
    }

    /// The nearest character a creature smells the blood of: within `per_wound` tiles for each
    /// of its wounds that's bleeding, up to three. Walls and the dark don't stop a scent.
    pub fn smells(&self, who: ActorId, per_wound: i32) -> Option<(ActorId, TilePos)> {
        let a = self
            .actors
            .get(who)
            .filter(|a| !a.dead && a.prone_until == 0)?;
        if per_wound <= 0 {
            return None;
        }
        let from = a.pos;
        let mut best: Option<(i64, ActorId, TilePos)> = None;
        for &id in &self.characters {
            let Some(c) = self
                .actors
                .get(id)
                .filter(|c| !c.dead && c.pos.z == from.z && !self.is_hallowed(c.pos))
            else {
                continue;
            };
            let bleeding = c
                .body
                .wounds
                .iter()
                .filter(|w| w.bleeding > 0)
                .count()
                .min(3);
            // Out in the rain, blood washes away: smelled from less far.
            let range = i64::from(self.scent_reach(c.pos, per_wound)) * bleeding as i64;
            let d2 = from.dist2(c.pos);
            if d2 <= range * range && best.is_none_or(|(b, ..)| d2 < b) {
                best = Some((d2, id, c.pos));
            }
        }
        best.map(|(_, id, at)| (id, at))
    }
}

/// What an actor's intent asks of its legs right now.
enum Heading {
    Stand,
    Go(Dir8, bool),
    /// A `WalkTo` that found no way there.
    NoWay(TilePos),
}

/// The heading an actor's intent asks for now. Plans (and re-plans) the path of a `WalkTo`: when
/// the goal changed, when the next tile isn't next to it any more, or after waiting too long.
fn next_heading(a: &mut Actor, grid: &Grid, repath_after: u32, budget: usize) -> Heading {
    match a.intent {
        Intent::Move { dir, run } => Heading::Go(dir, run),
        Intent::WalkTo { to, run } => {
            if a.pos == to {
                a.intent = Intent::Idle;
                a.path.clear();
                a.path_goal = None;
                return Heading::Stand;
            }
            let stale = a.path_goal != Some(to)
                || a.path.last().is_none_or(|&next| a.pos.steps_to(next) != 1);
            // Waited long enough on the way ahead: plan again only if the map itself now bars
            // it (a door shut). Barred only by someone standing there, a search would find the
            // very same way (it doesn't count actors), so it waits on.
            let barred = a.path.last().is_none_or(|&next| {
                Dir8::toward(next.x - a.pos.x, next.y - a.pos.y)
                    .is_none_or(|d| !grid.can_step(a.pos, d))
            });
            if a.blocked >= repath_after && !barred && !stale {
                a.blocked = 0;
            }
            if stale || a.blocked >= repath_after {
                a.blocked = 0;
                match find_path(grid, a.pos, to, budget) {
                    Some(mut path) => {
                        path.reverse();
                        a.path = path;
                        a.path_goal = Some(to);
                        a.no_way = None;
                    }
                    None => {
                        a.intent = Intent::Idle;
                        a.path.clear();
                        a.path_goal = None;
                        a.no_way = Some(to);
                        return Heading::NoWay(to);
                    }
                }
            }
            match a
                .path
                .last()
                .and_then(|&next| Dir8::toward(next.x - a.pos.x, next.y - a.pos.y))
            {
                Some(dir) => Heading::Go(dir, run),
                None => Heading::Stand,
            }
        }
        // Idle, or an action waiting for the step under way to end: the legs stand.
        _ => Heading::Stand,
    }
}

/// Could an actor at `from` step onto `to`'s tile: next to it, with no wall or corner in the
/// way? (Whoever stands there doesn't count.) The reach of a swing, a grab and a shove.
pub(crate) fn reaches(grid: &Grid, from: TilePos, to: TilePos) -> bool {
    from.steps_to(to) == 1
        && Dir8::toward(to.x - from.x, to.y - from.y).is_some_and(|d| grid.can_step(from, d))
}
