//! Bodies: health, endurance and wounds.
//!
//! Health and endurance are out of [`FULL`] (thousandths, so a designer's 100 points is 1000).
//! Endurance pays for everything active: running, swings, shoves. Fighting and fleeing draw on
//! the same bar (`CONCEPT.md`, athletics and endurance). Empty, you're exhausted until you've
//! got your breath back: no running, and slow, weak swings. It's never a death on its own.

/// A full bar of health or endurance.
pub const FULL: i32 = 1000;

/// Where a blow lands: Zomboid's health panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum Part {
    Head,
    Torso,
    LeftArm,
    RightArm,
    LeftHand,
    RightHand,
    LeftLeg,
    RightLeg,
    LeftFoot,
    RightFoot,
}

impl Part {
    /// Every part, with how often a blow lands there (out of 100): mostly the arms and hands
    /// that fend the dead off, least the head and feet.
    pub const ODDS: [(Part, u32); 10] = [
        (Part::Head, 6),
        (Part::Torso, 18),
        (Part::LeftArm, 15),
        (Part::RightArm, 15),
        (Part::LeftHand, 12),
        (Part::RightHand, 12),
        (Part::LeftLeg, 8),
        (Part::RightLeg, 8),
        (Part::LeftFoot, 3),
        (Part::RightFoot, 3),
    ];

    /// The part a roll from 0 to 99 lands on.
    pub fn from_roll(roll: u32) -> Part {
        let mut left = roll;
        for (part, odds) in Part::ODDS {
            if left < odds {
                return part;
            }
            left -= odds;
        }
        Part::Torso
    }

    pub fn name(self) -> &'static str {
        match self {
            Part::Head => "head",
            Part::Torso => "torso",
            Part::LeftArm => "left arm",
            Part::RightArm => "right arm",
            Part::LeftHand => "left hand",
            Part::RightHand => "right hand",
            Part::LeftLeg => "left leg",
            Part::RightLeg => "right leg",
            Part::LeftFoot => "left foot",
            Part::RightFoot => "right foot",
        }
    }
}

/// What the dead do to you. (Burns, fractures and the rest come with the regions that deal them.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub enum WoundKind {
    Scratch,
    Laceration,
    Bite,
}

impl WoundKind {
    pub const ALL: [WoundKind; 3] = [WoundKind::Scratch, WoundKind::Laceration, WoundKind::Bite];

    pub fn name(self) -> &'static str {
        match self {
            WoundKind::Scratch => "scratch",
            WoundKind::Laceration => "laceration",
            WoundKind::Bite => "bite",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Wound {
    pub part: Part,
    pub kind: WoundKind,
    /// Ticks it goes on bleeding (bandages come with items).
    pub bleeding: u32,
    /// A bandage stopped it.
    pub bandaged: bool,
    /// Game minutes left before it closes, counted once it's stopped bleeding.
    pub heal_left: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "save", derive(serde::Serialize, serde::Deserialize))]
pub struct Body {
    pub health: i32,
    pub endurance: i32,
    /// Out of breath: set when endurance runs out, cleared once it's back to the rules' level.
    pub exhausted: bool,
    pub wounds: Vec<Wound>,
    /// Needs, out of 100 points in thousandths (0 is fine; `needs.rs`).
    pub hunger: i32,
    pub thirst: i32,
    pub fatigue: i32,
    /// Asleep, and whether in a bed (where sleep rests you twice as fast).
    pub asleep: bool,
    pub in_bed: bool,
    /// Ghoul fever: the game minute it was caught (`fever.rs`).
    pub fever: Option<u64>,
}

impl Body {
    pub fn new(health: i32) -> Body {
        Body {
            health,
            endurance: FULL,
            exhausted: false,
            wounds: Vec::new(),
            hunger: 0,
            thirst: 0,
            fatigue: 0,
            asleep: false,
            in_bed: false,
            fever: None,
        }
    }

    pub fn bleeding(&self) -> bool {
        self.wounds.iter().any(|w| w.bleeding > 0)
    }

    /// Pays endurance for something; running out leaves you exhausted.
    pub fn spend(&mut self, endurance: i32) {
        self.endurance = (self.endurance - endurance).max(0);
        if self.endurance == 0 {
            self.exhausted = true;
        }
    }
}
