//! Actors (the character, NPCs, creatures) and what their controllers ask of them.

use crate::body::Body;
use crate::geom::{Dir8, TilePos};
use crate::items::Item;
use crate::skills::Skills;

pub type ActorId = crate::arena::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum ActorKind {
    Character,
    /// A creature, by its number in `Defs::creatures` (`creature.rs`).
    Creature(u16),
}

impl ActorKind {
    pub fn is_character(self) -> bool {
        self == ActorKind::Character
    }

    /// The living fight the creatures, and the creatures go for the living. (The families
    /// fighting each other comes with the second family.)
    pub fn hostile_to(self, other: ActorKind) -> bool {
        self.is_character() != other.is_character()
    }
}

/// What a controller (the player, a bot, an NPC's or a creature's AI) asks an actor to do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Intent {
    #[default]
    Idle,
    /// Keep walking (or running) this way.
    Move { dir: Dir8, run: bool },
    /// Walk (or run) to a tile, finding the way there.
    WalkTo { to: TilePos, run: bool },
    /// Swing at whatever is that way; None: the nearest enemy beside you, else straight ahead.
    /// One swing: the intent goes back to idle once it starts.
    Attack { dir: Option<Dir8> },
    /// Shove that way (None as for `Attack`): it knocks back, and sometimes down.
    Shove { dir: Option<Dir8> },
    /// Open or shut the door beside you: the one you face, else the first of north, east,
    /// south and west.
    Door,
    /// Fight free of what holds you.
    Struggle,
    /// Take the stack at `slot` from the container or floor on `at` (within reach). `def` is
    /// what the controller saw there, so a stale click takes nothing rather than the wrong item.
    Take { at: TilePos, slot: u16, def: u16 },
    /// Drop the stack at `slot` of your pack onto your tile.
    Drop { slot: u16, def: u16 },
    /// Use the item at `slot` of your pack: a bandage goes on the worst wound that's bleeding.
    Use { slot: u16, def: u16 },
    /// Take the weapon at `slot` of your pack in hand.
    Wield { slot: u16, def: u16 },
    /// Put what's in your hand into your pack.
    Unwield,
    /// Lie down and sleep (tired, and with none of the dead about): a bed beside you rests you
    /// twice as fast as the ground.
    Sleep,
    /// Get up. (Asleep, any intent wakes you; this one does nothing else.)
    Wake,
    /// Drink your fill from the water beside you.
    DrinkWater,
    /// Light the torch or candle at `slot` of your pack and hold it up in your other hand.
    Light { slot: u16, def: u16 },
    /// Put out the light you hold up.
    Douse,
    /// Nail a board across the door or window beside you (`barricade.rs`): the one you face,
    /// else the first of north, east, south and west.
    Board,
    /// Pry a board off the boarded door or window beside you.
    Unboard,
    /// Make the container on `at` your stash: within reach, under a roof, near home.
    Stash { at: TilePos },
    /// Put the stack at `slot` of your pack into the container on `at`.
    Put { at: TilePos, slot: u16, def: u16 },
    /// Buy one of a holdout's `line` over the counter on `at` (`holdout.rs`).
    Buy { at: TilePos, line: u16 },
    /// Sell the stack at `slot` of your pack over the counter on `at`.
    Sell { at: TilePos, slot: u16, def: u16 },
    /// Have the weapon in your hand mended at the counter on `at`.
    Mend { at: TilePos },
    /// Catch your second wind (a talent, `class.rs`): breath full, and a little health back.
    SecondWind,
    /// Claim the bounty on a rare put down, at the counter on `at`.
    Claim { at: TilePos, rare: ActorId },
    /// Have the item at `slot` of your pack (or in hand: `holdout::HANDS`) read at the counter.
    Identify { at: TilePos, slot: u16 },
    /// Have the curse on the weapon in your hand lifted at the counter on `at`.
    Uncurse { at: TilePos },
    /// Pour oil on a body beside you (or under you) and set it alight from the light you hold
    /// up (`remains.rs`): the one you face, else the one you stand on, else the first round you.
    Burn,
    /// Cut up the beast's carcass beside you (`hunting.rs`), with a blade or an axe.
    Butcher,
    /// Tend the hearth, stove or forge beside you (`fire.rs`): light it, if it's laid and you
    /// hold up a light, else put fuel on.
    Tend,
    /// Put out the fire beside you: what fuel's left stays in it.
    Smother,
    /// Put the shield on your arm into your pack (`shield.rs`).
    Unshield,
    /// Shoot the bow in your hand at `at` (`archery.rs`).
    Shoot { at: TilePos },
    /// Make what recipe `recipe` makes (`craft.rs`), from what you carry.
    Craft { recipe: u16 },
    /// Break the furniture beside you down for planks, with a hammer or an axe.
    BreakDown,
    /// Hold up the holy symbol and turn the dead (`faith.rs`): a class that turns them, with faith
    /// enough.
    Turn,
    /// Kneel and pray at the shrine beside you (an altar): anyone, for its comfort; the faithful
    /// get their faith back.
    Pray,
    /// Consecrate the body beside you (or under you) with holy water, or with faith: it never gets
    /// up. The one you face, else the one you stand on, else the first round you.
    Consecrate,
    /// Mend your worst bleeding wound with faith (a talent, Mending).
    Heal,
    /// Ward the shut door beside you with faith (a talent, Warding): the dead won't beat on it.
    Ward,
    /// Pick the wild plant beside you (`forage.rs`): the one you face, else the one you stand
    /// on, else the first round you.
    Pick,
    /// Put on the armour at `slot` of your pack (`armour.rs`): what covered the same places
    /// comes off, into the pack.
    Wear { slot: u16, def: u16 },
    /// Take off the `piece` of armour you wear (still `def`), into your pack.
    TakeOff { piece: u16, def: u16 },
}

impl Intent {
    /// Whether it asks for a timed action rather than for the legs.
    pub fn is_action(self) -> bool {
        !matches!(
            self,
            Intent::Idle | Intent::Move { .. } | Intent::WalkTo { .. }
        )
    }
}

/// The timed actions (`ARCHITECTURE.md`, one rule for every timed action).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionKind {
    Attack,
    Shove,
    Door,
    Struggle,
    Take {
        at: TilePos,
        slot: u16,
        def: u16,
    },
    Drop {
        slot: u16,
        def: u16,
    },
    Use {
        slot: u16,
        def: u16,
    },
    Wield {
        slot: u16,
        def: u16,
    },
    Unwield,
    Sleep,
    DrinkWater,
    Light {
        slot: u16,
        def: u16,
    },
    Douse,
    /// Boarding up, or prying a board off, the edge on the action's side.
    Board,
    Unboard,
    Stash {
        at: TilePos,
    },
    Put {
        at: TilePos,
        slot: u16,
        def: u16,
    },
    /// Setting a body alight.
    Burn {
        body: ActorId,
    },
    /// Cutting up a carcass.
    Butcher {
        body: ActorId,
    },
    /// Laying or lighting the fire on `at`, and putting it out.
    Tend {
        at: TilePos,
        tend: crate::fire::Tend,
    },
    Smother {
        at: TilePos,
    },
    /// Putting the shield away.
    Unshield,
    /// Drawing a bow and loosing at `at`.
    Shoot {
        at: TilePos,
    },
    /// Making something, and breaking the furniture on `at` down.
    Craft {
        recipe: u16,
    },
    BreakDown {
        at: TilePos,
    },
    /// Deals over a holdout's counter.
    Buy {
        at: TilePos,
        line: u16,
    },
    Sell {
        at: TilePos,
        slot: u16,
        def: u16,
    },
    Mend {
        at: TilePos,
    },
    Claim {
        at: TilePos,
        rare: ActorId,
    },
    Identify {
        at: TilePos,
        slot: u16,
    },
    Uncurse {
        at: TilePos,
    },
    SecondWind,
    /// The holy symbol held up: the dead turn on its strike tick (`faith.rs`).
    Turn,
    /// Praying at the shrine on `at`.
    Pray {
        at: TilePos,
    },
    /// Consecrating a body.
    Consecrate {
        body: ActorId,
    },
    /// Mending a wound with faith.
    Heal,
    /// Warding the door on the action's side.
    Ward,
    /// Picking the plant on `at`.
    Pick {
        at: TilePos,
    },
    /// Putting on armour from the pack, and taking a piece off (`armour.rs`).
    Wear {
        slot: u16,
        def: u16,
    },
    TakeOff {
        piece: u16,
        def: u16,
    },
}

/// A timed action under way: a wind-up of `strike` ticks (its tell), the tick it takes effect,
/// then a recovery until `length`. Nobody moves while acting, and a hit during the wind-up
/// breaks it off.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Action {
    pub kind: ActionKind,
    pub dir: Dir8,
    /// Ticks since it began.
    pub elapsed: u32,
    pub strike: u32,
    pub length: u32,
}

impl Action {
    /// Still winding up: it hasn't taken effect yet.
    pub fn winding_up(&self) -> bool {
        self.elapsed < self.strike
    }
}

/// A step under way from one tile to the next. `progress` and `length` are in thousandths of a
/// tile: 1000 for a straight step, 1414 for a diagonal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Step {
    pub from: TilePos,
    pub to: TilePos,
    pub dir: Dir8,
    pub progress: u32,
    pub length: u32,
    pub run: bool,
    /// Into shallows: slow going, and a splash (`Rules::shallows`). Settled as the step starts.
    pub wade: bool,
}

/// A creature's thoughts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Mind {
    #[default]
    None,
    /// Zomboid's zombie: goes after the last thing it heard or saw, and gives up after a while.
    /// How far it senses, how fast it goes and how long it remembers are its kind's
    /// (`creature.rs`).
    Hunt {
        target: Option<TilePos>,
        /// The tick it last heard or saw something.
        since: u64,
        /// Next to its prey right now.
        touching: bool,
        /// It saw its prey when it last thought.
        sees: bool,
    },
    /// A beast (`hunting.rs`): after nobody, it runs from where it was last startled, until
    /// it's forgotten (`since`, the tick of the fright).
    Flee { from: Option<TilePos>, since: u64 },
    /// One of the dead, turned (`faith.rs`): it flees from `by` (the cleric who turned it) until
    /// the tick `until`, then hunts again. It strikes nobody; a blow ends it.
    Turned { by: ActorId, until: u64 },
}

/// What a creature knows of its prey: the mark the client draws over its head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Alert {
    /// Not after anything.
    Unaware,
    /// After something it heard, or the place it last saw prey: "?".
    Curious,
    /// It sees its prey: "!".
    Hunting,
}

/// What's become of a body (`remains.rs`). The living are `Lying` too: it says nothing until
/// they fall.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Remains {
    /// A body that stays down.
    #[default]
    Lying,
    /// A body that gets up on this tick, unless it's burned first.
    GetsUp { at: u64 },
    /// Alight until this tick: then it's ash.
    Burning { until: u64 },
    /// Burnt: nothing left to get up.
    Ash,
    /// A character's body that rose as a revenant (`legacy.rs`): nothing's left here.
    Risen,
    /// A beast's carcass cut up (`hunting.rs`): nothing's left here.
    Butchered,
    /// Consecrated (`faith.rs`): holy water or a prayer on it, and it never gets up.
    Hallowed,
}

/// How fast a kind of actor moves, in thousandths of a tile a second (`Rules::speeds`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Speeds {
    pub walk: u32,
    pub run: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Actor {
    pub kind: ActorKind,
    pub pos: TilePos,
    pub facing: Dir8,
    pub intent: Intent,
    pub step: Option<Step>,
    /// The way to a `WalkTo` goal, the next tile last.
    pub path: Vec<TilePos>,
    /// The goal `path` leads to.
    pub path_goal: Option<TilePos>,
    /// The last `WalkTo` goal no way was found to.
    pub no_way: Option<TilePos>,
    /// Ticks spent waiting on the way ahead (a tile someone is in, a door that shut).
    pub blocked: u32,
    /// Progress left over from the last step, carried into the next, so speeds come out exact.
    pub carry: u32,
    pub mind: Mind,
    pub action: Option<Action>,
    pub body: Body,
    /// Knocked down until this tick (0: standing).
    pub prone_until: u64,
    /// A corpse: it stays where it fell and takes no part in the tick.
    pub dead: bool,
    /// What's become of its body, once it's dead: lying, getting up, burning or ash.
    pub remains: Remains,
    /// How many times it has got up again (`remains.rs`).
    pub risen: u8,
    /// Who it has hold of (`grab.rs`).
    pub holding: Option<ActorId>,
    /// What it has in hand (a weapon), and in its pack. Both count toward what it carries.
    pub hands: Option<Item>,
    pub items: Vec<Item>,
    /// The light it holds up in its other hand, lit (`light.rs`).
    pub light: Option<Item>,
    /// What it has learnt by doing (`skills.rs`).
    pub skills: Skills,
    /// Where it last slept the night through, and its stash (`home.rs`).
    pub home: Option<TilePos>,
    pub stash: Option<TilePos>,
    /// XP from deeds, in thousandths: what's brought home. Its level comes from this.
    pub deeds: u32,
    /// A character's name (`legacy.rs`), by its numbers in the data's lists; a revenant keeps
    /// the one it was. None for the nameless.
    pub name: Option<(u16, u16)>,
    /// The game minute it came into the world, and the one it died.
    pub born: u64,
    pub died: Option<u64>,
    /// A rare's affixes (`rare.rs`), by their numbers in the data: none for the ordinary.
    pub affixes: Vec<u8>,
    /// A character's class and the talents it has picked (`class.rs`), by their numbers in the
    /// data; and whether its second wind is spent until it sleeps.
    pub class: Option<u8>,
    pub talents: Vec<u8>,
    pub winded: bool,
    /// Sneaking: a stance (`stealth.rs`).
    pub sneaking: bool,
    /// The shield on the off arm, and bracing behind it: a stance (`shield.rs`).
    pub shield: Option<Item>,
    pub bracing: bool,
    /// A Cleric's faith (`faith.rs`), in thousandths of a point: spent on its acts, and back by
    /// prayer at a shrine. 0 for a class without it.
    pub faith: i32,
    /// The armour it wears, a piece to a place on the body at most (`armour.rs`).
    pub worn: Vec<Item>,
}

impl Actor {
    pub fn new(kind: ActorKind, pos: TilePos) -> Actor {
        let mind = match kind {
            ActorKind::Creature(_) => Mind::Hunt {
                target: None,
                since: 0,
                touching: false,
                sees: false,
            },
            ActorKind::Character => Mind::None,
        };
        Actor {
            kind,
            pos,
            facing: Dir8::S,
            intent: Intent::Idle,
            step: None,
            path: Vec::new(),
            path_goal: None,
            no_way: None,
            blocked: 0,
            carry: 0,
            mind,
            action: None,
            body: Body::new(crate::body::FULL),
            prone_until: 0,
            dead: false,
            remains: Remains::Lying,
            risen: 0,
            holding: None,
            hands: None,
            items: Vec::new(),
            light: None,
            skills: Skills::default(),
            home: None,
            stash: None,
            deeds: 0,
            name: None,
            born: 0,
            died: None,
            affixes: Vec::new(),
            class: None,
            talents: Vec::new(),
            winded: false,
            sneaking: false,
            shield: None,
            bracing: false,
            faith: 0,
            worn: Vec::new(),
        }
    }

    /// Where the actor is in tiles, part way between two while a step is under way. For drawing
    /// it smoothly; game logic uses `pos`.
    pub fn position(&self) -> (f32, f32) {
        match self.step {
            Some(s) => {
                let t = (s.progress as f32 / s.length as f32).min(1.0);
                (
                    s.from.x as f32 + (s.to.x - s.from.x) as f32 * t,
                    s.from.y as f32 + (s.to.y - s.from.y) as f32 * t,
                )
            }
            None => (self.pos.x as f32, self.pos.y as f32),
        }
    }

    /// What the actor knows of its prey (a creature's; the character is always `Unaware`).
    pub fn alert(&self) -> Alert {
        if self.dead {
            return Alert::Unaware;
        }
        match self.mind {
            Mind::Hunt { sees: true, .. } => Alert::Hunting,
            Mind::Hunt {
                target: Some(_), ..
            } => Alert::Curious,
            _ => Alert::Unaware,
        }
    }
}
