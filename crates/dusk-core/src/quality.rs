//! A weapon's quality, or a bow's, a shield's or a piece of armour's (`CONCEPT.md`, loot: "in
//! qualities from crude to standard, fine and masterwork"; Diablo II's low-quality and superior
//! weapons, Path of Exile's quality): how well it was made, apart from any magic it holds. It's
//! rolled as it's found, and changes how hard a weapon's blows and a bow's shots land, how often
//! armour turns a blow and a shield blocks one, how fast any of them wears and what it's worth; its word goes before its name ("a fine sword"). What you start with, make or
//! buy is plain.

use crate::hash::below;

/// The grades, worst first: an item's `quality` is one of these.
pub const CRUDE: u8 = 0;
pub const PLAIN: u8 = 1;
pub const FINE: u8 = 2;
pub const MASTERWORK: u8 = 3;

/// The grades in the core's units (rules.toml's `[quality]`), each array by grade (crude, plain,
/// fine, masterwork): its word before a name ("" for plain); out of 10,000, how many weapons
/// found are of it (plain is what's left); per mille, the harm its blows do, how fast it wears
/// and what it's worth; and, out of 10,000, the chance armour of it turns a blow, added (a
/// shield's block too).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualityRules {
    pub names: [String; 4],
    pub chance: [u32; 4],
    pub damage: [u32; 4],
    pub wear: [u32; 4],
    pub worth: [u32; 4],
    pub turn: [i32; 4],
}

impl Default for QualityRules {
    fn default() -> QualityRules {
        QualityRules {
            names: [
                "crude".into(),
                String::new(),
                "fine".into(),
                "masterwork".into(),
            ],
            chance: [2500, 0, 1500, 500],
            damage: [800, 1000, 1100, 1250],
            wear: [1500, 1000, 850, 600],
            worth: [500, 1000, 1500, 3000],
            turn: [-500, 0, 500, 1000],
        }
    }
}

impl QualityRules {
    /// A found weapon's grade, by a roll: crude, fine or masterwork by their chances, else plain.
    pub fn roll(&self, r: u64) -> u8 {
        let mut pick = below(r, 10_000);
        for grade in [CRUDE, FINE, MASTERWORK] {
            let chance = self.chance[usize::from(grade)];
            if pick < chance {
                return grade;
            }
            pick -= chance;
        }
        PLAIN
    }

    fn at(quality: u8) -> usize {
        usize::from(quality.min(MASTERWORK))
    }

    /// Per mille: the harm its blows do, how fast it wears, what it's worth.
    pub fn damage_of(&self, quality: u8) -> u32 {
        self.damage[Self::at(quality)]
    }

    pub fn wear_of(&self, quality: u8) -> u32 {
        self.wear[Self::at(quality)]
    }

    pub fn worth_of(&self, quality: u8) -> u32 {
        self.worth[Self::at(quality)]
    }

    /// Out of 10,000: what armour of it adds to its chance to turn a blow, and a shield to its
    /// block.
    pub fn turn_of(&self, quality: u8) -> i32 {
        self.turn[Self::at(quality)]
    }

    /// Its word before a name: "" for plain.
    pub fn name_of(&self, quality: u8) -> &str {
        &self.names[Self::at(quality)]
    }
}
