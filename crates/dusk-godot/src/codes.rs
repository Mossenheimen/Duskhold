//! The numbers the bridge hands GDScript, named once. `client/codes.gd` is made from these: its
//! test (`codes_gd_is_current`) fails when the two differ, and with `DUSK_WRITE_CODES` set it
//! writes the file instead. So a code added here reaches the client by name, and can't drift.

use dusk_core::Moodle;

/// A group of codes: a constant each, and the group's table (`ALL`), for `codes.gd`.
macro_rules! codes {
    ($($name:ident = $value:expr,)*) => {
        $(pub const $name: i32 = $value;)*
        /// Every code in the group, by name.
        pub const ALL: &[(&str, i32)] = &[$((stringify!($name), $name),)*];
    };
}

/// An event's kind (`take_events`).
pub mod event {
    codes! {
        HEARD = 1,
        SAW = 2,
        REACHED = 3,
        NO_WAY = 4,
        STRUCK = 5,
        MISSED = 6,
        SHOVED = 7,
        DIED = 8,
        DOOR = 9,
        EXHAUSTED = 10,
        GRABBED = 11,
        FREED = 12,
        STRUGGLED = 13,
        THUMPED = 14,
        DOOR_BROKE = 15,
        DRAGGED_DOWN = 16,
        TOOK = 17,
        DROPPED = 18,
        WIELDED = 19,
        WEAPON_BROKE = 20,
        BANDAGED = 21,
        ATE = 22,
        DRANK = 23,
        DRANK_WATER = 24,
        FELL_ASLEEP = 25,
        WOKE = 26,
        LIT = 27,
        DOUSED = 28,
        BURNT_OUT = 29,
        NIGHTFALL = 30,
        DAYBREAK = 31,
        SKILL_UP = 32,
        SKILL_AT_CAP = 33,
        MASTERED = 34,
        BOARDED = 35,
        UNBOARDED = 36,
        BOARD_BROKE = 37,
        MADE_HOME = 38,
        STASHED = 39,
        PUT = 40,
        BROUGHT_HOME = 41,
        LEVEL_UP = 42,
        SMELLED = 43,
        FEVERED = 44,
        FEVER_WORSE = 45,
        FEVER_TOOK = 46,
        CURED = 47,
        GOT_UP = 48,
        KINDLED = 49,
        CAUGHT = 50,
        BURNT = 51,
        BOUGHT = 52,
        SOLD = 53,
        SUPPLIED = 54,
        MENDED = 55,
        TOLLING_TONIGHT = 56,
        TOLLING_BEGAN = 57,
        TOLLED = 58,
        TOLLING_ENDED = 59,
        RISEN = 60,
        CAME = 61,
        LIGHT_EATEN = 62,
        BOUNTY_CLAIMED = 63,
        IDENTIFIED = 64,
        CURSE_CLINGS = 65,
        CURSE_LIFTED = 66,
        RAIN_BEGAN = 67,
        RAIN_ENDED = 68,
        RAINED_OUT = 69,
        TALENT_CHOSEN = 70,
        SECOND_WIND = 71,
        LAID = 72,
        FIRE_LIT = 73,
        SMOTHERED = 74,
        FIRE_OUT = 75,
        SAW_FIRE = 76,
        DREAD = 77,
        BALKED = 78,
        RESISTED = 79,
        CRAFTED = 80,
        BROKE_DOWN = 81,
        SNEAKING = 82,
        SNEAK_ATTACK = 83,
        BRACING = 84,
        BLOCKED = 85,
        SHOT = 86,
        SPOILED = 87,
        STARTLED = 88,
        BUTCHERED = 89,
        CLASS_CHOSEN = 90,
        TURNED = 91,
        PRAYED = 92,
        CONSECRATED = 93,
        HEALED = 94,
        WARDED = 95,
        WARD_FADED = 96,
        PICKED = 97,
        REGREW = 98,
        GLANCED = 99,
        WORE = 100,
        TOOK_OFF = 101,
        CLIMBED = 102,
        WINDOW_BROKE = 103,
        FELLED = 104,
        GREW = 105,
        WET = 106,
        COLD = 107,
        CAST = 108,
        SCORCHED = 109,
        SPELL_BROKE = 110,
        SLOTS_BACK = 111,
    }
}

/// An item's kind, by definition (`item_kinds`).
pub mod item {
    codes! {
        OTHER = 0,
        WEAPON = 1,
        BANDAGE = 2,
        FOOD = 3,
        DRINK = 4,
        LIGHT = 5,
        CURE = 6,
        OIL = 7,
        SHIELD = 8,
        BOW = 9,
        HOLY = 10,
        ARMOUR = 11,
        BAG = 12,
    }
}

/// An actor's timed action (`actor_actions`). A blow at a shut door or window is `BEAT`; moving
/// a stack to or from the bag is a `TAKE`, and dropping the bag a `DROP`.
pub mod action {
    codes! {
        NONE = 0,
        SWING = 1,
        SHOVE = 2,
        DOOR = 3,
        BEAT = 4,
        STRUGGLE = 5,
        TAKE = 6,
        DROP = 7,
        BANDAGE = 8,
        WIELD = 9,
        PUT_AWAY = 10,
        SLEEP = 11,
        DRINK_WATER = 12,
        EAT = 13,
        DRINK = 14,
        LIGHT = 15,
        DOUSE = 16,
        BOARD = 17,
        UNBOARD = 18,
        STASH = 19,
        PUT = 20,
        CURE = 21,
        BURN = 22,
        BUY = 23,
        SELL = 24,
        MEND = 25,
        CLAIM = 26,
        IDENTIFY = 27,
        UNCURSE = 28,
        SECOND_WIND = 29,
        LAY_FIRE = 30,
        LIGHT_FIRE = 31,
        SMOTHER = 32,
        CRAFT = 33,
        BREAK_DOWN = 34,
        SHOOT = 36,
        BUTCHER = 37,
        TURN = 38,
        PRAY = 39,
        CONSECRATE = 40,
        HEAL = 41,
        WARD = 42,
        PICK = 43,
        WEAR = 44,
        TAKE_OFF = 45,
        HEAVY = 46,
        CAST = 47,
    }
}

/// A moodle (`character_moodles`): the core's own order (`Moodle`).
pub mod moodle {
    use super::Moodle;
    codes! {
        HUNGRY = Moodle::Hungry as i32,
        THIRSTY = Moodle::Thirsty as i32,
        TIRED = Moodle::Tired as i32,
        INJURED = Moodle::Injured as i32,
        BLEEDING = Moodle::Bleeding as i32,
        OUT_OF_BREATH = Moodle::OutOfBreath as i32,
        HEAVY_LOAD = Moodle::HeavyLoad as i32,
        HELD = Moodle::Held as i32,
        ASLEEP = Moodle::Asleep as i32,
        FEVER = Moodle::Fever as i32,
        PAIN = Moodle::Pain as i32,
        DREAD = Moodle::Dread as i32,
        ARM = Moodle::Arm as i32,
        LEG = Moodle::Leg as i32,
        HEAD = Moodle::Head as i32,
        SNEAKING = Moodle::Sneaking as i32,
        BRACING = Moodle::Bracing as i32,
        COLD = Moodle::Cold as i32,
        WET = Moodle::Wet as i32,
    }
}

/// An actor's state (`actor_states`).
pub mod state {
    codes! {
        STANDING = 0,
        DOWN = 1,
        DEAD = 2,
        ASLEEP = 3,
    }
}

/// What's become of a body (`actor_remains`): `GONE` is risen as a revenant, or butchered.
pub mod remains {
    codes! {
        NONE = 0,
        BURNING = 1,
        ASH = 2,
        GONE = 3,
        HALLOWED = 4,
    }
}

/// What a number key holds (`character_powers`: this, then a spell's number).
pub mod power {
    codes! {
        TURN = 0,
        SECOND_WIND = 1,
        MEND = 2,
        SPELL = 3,
    }
}

/// An edge's kind (`chunk_edges`), plus `BOARD` for each board nailed across it.
pub mod edge {
    codes! {
        OPEN = 0,
        WALL = 1,
        DOOR_SHUT = 2,
        DOOR_OPEN = 3,
        WINDOW_SHUT = 4,
        WINDOW_OPEN = 5,
        FENCE = 6,
        WINDOW_BROKEN = 7,
    }
    /// Added for each board on an edge: kind & 15 is the kind, kind >> 4 the boards.
    pub const BOARD: i32 = 16;
}

/// A group's codes, each a name and its number.
pub type Codes = &'static [(&'static str, i32)];

/// A group as GDScript gets it: its enum's name, what it numbers, and its codes.
pub type Group = (&'static str, &'static str, Codes);

/// Every group, in `codes.gd`'s order.
pub const GROUPS: &[Group] = &[
    (
        "Event",
        "An event's kind (DuskWorld.take_events(): six numbers an event).",
        event::ALL,
    ),
    (
        "Item",
        "An item's kind, by definition (DuskWorld.item_kinds()).",
        item::ALL,
    ),
    (
        "Action",
        "An actor's timed action (DuskWorld.actor_actions(): four numbers an actor).",
        action::ALL,
    ),
    (
        "Moodle",
        "A moodle (DuskWorld.character_moodles(): which, then its level).",
        moodle::ALL,
    ),
    (
        "State",
        "An actor's state (DuskWorld.actor_states()).",
        state::ALL,
    ),
    (
        "Remains",
        "What's become of a body (DuskWorld.actor_remains()).",
        remains::ALL,
    ),
    (
        "Edge",
        "An edge's kind (DuskWorld.chunk_edges(): & 15; >> 4 is its boards).",
        edge::ALL,
    ),
    (
        "Power",
        "What a number key holds (DuskWorld.character_powers(): this, then a spell's number).",
        power::ALL,
    ),
];

/// `client/codes.gd`: every group as a GDScript enum.
pub fn gdscript() -> String {
    let mut out = String::from(
        "extends RefCounted\n\
         ## The numbers DuskWorld hands GDScript, by name: made from crates/dusk-godot/src/codes.rs\n\
         ## by its test (set DUSK_WRITE_CODES, then cargo test -p dusk-godot codes). Don't edit it\n\
         ## by hand. A script names a group as const Event = Codes.Event, then Event.HEARD.\n",
    );
    for (name, what, codes) in GROUPS {
        out.push_str(&format!("\n## {what}\nenum {name} {{\n"));
        for (code, value) in *codes {
            out.push_str(&format!("\t{code} = {value},\n"));
        }
        out.push_str("}\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_gd_is_current() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../client/codes.gd");
        let made = gdscript();
        if std::env::var_os("DUSK_WRITE_CODES").is_some() {
            std::fs::write(path, &made).expect("client/codes.gd can't be written");
        }
        // Git may hand the file back with Windows line endings.
        let kept = std::fs::read_to_string(path)
            .unwrap_or_default()
            .replace("\r\n", "\n");
        assert!(
            kept == made,
            "client/codes.gd differs from codes.rs: set DUSK_WRITE_CODES and run \
             `cargo test -p dusk-godot codes` to write it again"
        );
    }

    #[test]
    fn no_two_names_in_a_group_share_a_number() {
        for (name, _, codes) in GROUPS {
            for (i, (a, x)) in codes.iter().enumerate() {
                for (b, y) in &codes[i + 1..] {
                    assert!(x != y, "{name}: {a} and {b} are both {x}");
                }
            }
        }
    }
}
