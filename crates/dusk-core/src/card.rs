//! A thing's card (`VISUALS.md` §11, the tooltip), laid out as Path of Exile's and Diablo's are
//! (the user, 2026-11-14: "more numerical, except for special effects, which can be more text"):
//!
//! - **Its name,** in its rarity's colour (a Rare's made-up name, or a unique's own, with its kind
//!   under it), then what it is: "Blade weapon · Rare".
//! - **Its numbers,** as it was made and as far as you know its magic: a weapon's damage per
//!   second first, as Diablo's, then each of the rest. A number its quality or its magic changes
//!   is marked (Path of Exile's blue), so what's better about it shows at a glance. A weapon's
//!   fire and holy harm are lines of their own, each in its own colour.
//! - **How it was made** (Diablo II's superior and low-quality weapons): a fine, masterwork or
//!   crude thing's quality, as numbers: a weapon's or a bow's harm, armour's turning, a shield's
//!   block, and the wear of what wears.
//! - **Its magic:** each property as numbers ("+80% damage", "+2 to Strength", "Adds 3 to 6 fire
//!   damage"), with its word and tier to the right ("Cruel III"), and a curse marked as one. Each
//!   number that rolled in its tier gives the tier's range after it, as Path of Exile 2's does:
//!   "+56(40–60)% damage". A property that's more than numbers says so in words (Hallowed,
//!   Blazing), as a unique's effect does; then a unique's story.
//! - **Notes:** what you don't know of it yet, how it stands against the weapon or bow in your
//!   hand (the shield on your arm, the piece you wear where it goes), and what it pays as a find.
//!
//! A card is the thing on its own, before your skill and your Strength: the skills panel shows
//! what a weapon in hand gives you.

use crate::actor::ActorId;
use crate::archery::Ranged;
use crate::armour::Armour;
use crate::body::Group;
use crate::items::{Defs, Item, ItemKind, NEW};
use crate::magic::{Fits, MOST, Property, Raises};
use crate::quality::PLAIN;
use crate::shield::Guard;
use crate::world::{Swing, TICKS_PER_SECOND, World};

/// How a line of a card is drawn (the client's `hud.gd`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardStyle {
    /// What it's called, as far as you know it: in its rarity's colour.
    Title,
    /// What kind of thing it is: "Masterwork arming sword", "Blade weapon · Rare".
    Kind,
    /// A rule between the card's parts.
    Rule,
    /// One of its numbers as its kind has it: the number's name, and its value to the right.
    Stat,
    /// One its quality or its magic changed (Path of Exile's blue).
    Changed,
    /// A weapon's fire harm, and its holy harm, each in its own colour.
    Fire,
    Holy,
    /// A property, or how well it was made, as numbers ("+80% damage"): its word and tier to the
    /// right ("Cruel III").
    Mod,
    /// A curse, as a property is, marked as one.
    Curse,
    /// What a unique's effect does, in words.
    Effect,
    /// A unique's story.
    Lore,
    /// What you don't know of it yet, how it stands against yours, what it pays as a find.
    Note,
}

impl CardStyle {
    /// Its word, as the client gets it.
    pub fn word(self) -> &'static str {
        match self {
            CardStyle::Title => "title",
            CardStyle::Kind => "kind",
            CardStyle::Rule => "rule",
            CardStyle::Stat => "stat",
            CardStyle::Changed => "changed",
            CardStyle::Fire => "fire",
            CardStyle::Holy => "holy",
            CardStyle::Mod => "mod",
            CardStyle::Curse => "curse",
            CardStyle::Effect => "effect",
            CardStyle::Lore => "lore",
            CardStyle::Note => "note",
        }
    }
}

/// A line of a card: how it's drawn, its words, and what goes to the right (a number's value, a
/// property's word and tier; empty for none).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardLine {
    pub style: CardStyle,
    pub text: String,
    pub value: String,
}

/// A card as it's written, a line at a time.
#[derive(Default)]
struct Lines(Vec<CardLine>);

impl Lines {
    fn add(&mut self, style: CardStyle, text: impl Into<String>, value: impl Into<String>) {
        self.0.push(CardLine {
            style,
            text: text.into(),
            value: value.into(),
        });
    }

    /// One of its numbers, marked if its quality or magic changed it.
    fn stat(&mut self, name: &str, value: String, changed: bool) {
        let style = if changed {
            CardStyle::Changed
        } else {
            CardStyle::Stat
        };
        self.add(style, name, value);
    }

    /// A rule between parts: never two together.
    fn rule(&mut self) {
        if self.0.last().is_some_and(|l| l.style != CardStyle::Rule) {
            self.add(CardStyle::Rule, "", "");
        }
    }

    /// The card, with no rule left at its end.
    fn done(mut self) -> Vec<CardLine> {
        while self.0.last().is_some_and(|l| l.style == CardStyle::Rule) {
            self.0.pop();
        }
        self.0
    }
}

/// Its first letter raised: "Fine sword".
fn capital(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// A number with at most `places` decimals, and none that are only zeros: "1.5", "2".
fn number(x: f64, places: usize) -> String {
    let s = format!("{x:.places$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Signed, with `places` decimals, as the number it's set against shows them: "+5.2", "-0.25".
fn signed(x: f64, places: usize) -> String {
    format!("{x:+.places$}")
}

/// Tenths of a point, to read: "12", "12.5".
fn points(tenths: i32) -> String {
    number(f64::from(tenths) / 10.0, 1)
}

/// Least to most, in tenths of a point: "11–16", or one number when they're the same.
fn span((lo, hi): (i32, i32)) -> String {
    if lo == hi {
        points(lo)
    } else {
        format!("{}\u{2013}{}", points(lo), points(hi))
    }
}

/// Least to most, in tenths of a point, as a property's line says it: "3 to 6", or one number
/// when they're the same (so a range after each can't be read as part of it).
fn to((lo, hi): (i32, i32)) -> String {
    if lo == hi {
        points(lo)
    } else {
        format!("{} to {}", points(lo), points(hi))
    }
}

/// A property's line as it rolled, each number that rolled followed by its tier's range, as Path
/// of Exile 2's: "+56(40–60)% damage". `lo` and `hi` are the same line at its tier's least and
/// most; a line they don't match (a number gone to nothing at one end) is given as it is.
fn ranged(now: &str, lo: &str, hi: &str) -> String {
    let (n, l, h) = (pieces(now), pieces(lo), pieces(hi));
    let words = |p: &[(bool, &str)]| -> Vec<String> {
        p.iter()
            .filter(|(number, _)| !number)
            .map(|(_, w)| w.to_string())
            .collect()
    };
    if n.len() != l.len() || n.len() != h.len() || words(&n) != words(&l) || words(&n) != words(&h)
    {
        return now.to_string();
    }
    let mut out = String::new();
    for ((&(number, text), &(_, a)), &(_, b)) in n.iter().zip(&l).zip(&h) {
        out.push_str(text);
        if number && a != b {
            let value = |x: &str| x.parse::<f64>().unwrap_or(0.0);
            let (a, b) = if value(a) <= value(b) { (a, b) } else { (b, a) };
            out.push_str(&format!("({a}\u{2013}{b})"));
        }
    }
    out
}

/// A line in its pieces, in order: each a number (digits, a point between them) or not.
fn pieces(s: &str) -> Vec<(bool, &str)> {
    let b = s.as_bytes();
    let digit = |i: usize| {
        b[i].is_ascii_digit()
            || (b[i] == b'.'
                && i > 0
                && b[i - 1].is_ascii_digit()
                && b.get(i + 1).is_some_and(u8::is_ascii_digit))
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let number = digit(i);
        let mut j = i + 1;
        while j < b.len() && digit(j) == number {
            j += 1;
        }
        // Every edge between pieces is beside a digit, so on a character's edge.
        out.push((number, &s[i..j]));
        i = j;
    }
    out
}

/// Out of 10,000, as a share: "85%", "12.5%".
fn share(n: u32) -> String {
    format!("{}%", number(f64::from(n) / 100.0, 1))
}

/// Out of 10,000, signed: "+10%", "-5%".
fn signed_share(n: i32) -> String {
    let sign = if n < 0 { "-" } else { "+" };
    format!("{sign}{}", share(n.unsigned_abs()))
}

/// Per mille, as a share: "15%", "0.5%".
fn mille(n: u32) -> String {
    format!("{}%", number(f64::from(n) / 10.0, 1))
}

/// A change, per mille, in whole hundredths and signed: "+25%", "-20%".
fn change(per_mille: i64) -> String {
    let pct = (per_mille as f64 / 10.0).round() as i64;
    if pct < 0 {
        format!("{pct}%")
    } else {
        format!("+{pct}%")
    }
}

/// A change to a time, per mille, as the change in how often it comes round: a swing at 750 of
/// its time is "+33%" attack speed.
fn quicker(time: u32) -> String {
    change(1_000_000 / i64::from(time.max(1)) - 1000)
}

/// "1 tile", "6 tiles".
fn tiles(n: i32) -> String {
    if n == 1 {
        "1 tile".into()
    } else {
        format!("{n} tiles")
    }
}

/// Ticks, in seconds: "0.4 s", "8 s".
fn seconds(ticks: u32) -> String {
    format!(
        "{} s",
        number(f64::from(ticks) / f64::from(TICKS_PER_SECOND), 1)
    )
}

/// Game minutes, to read: "59 m", "4 h", "1 h 30 m".
fn hours(minutes: u32) -> String {
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m} m"),
        (h, 0) => format!("{h} h"),
        (h, m) => format!("{h} h {m} m"),
    }
}

/// Grams, in kilograms: "4.5 kg".
fn kg(grams: u64) -> String {
    format!("{} kg", number(grams as f64 / 1000.0, 1))
}

/// Condition out of `items::NEW`, in whole hundredths: "87%".
fn condition(item: &Item) -> String {
    format!("{}%", (f64::from(item.condition) / 10.0).round())
}

/// Blows a second (or shots): a swing's wind-up and recovery, once round.
fn rate(s: &Swing) -> f64 {
    f64::from(TICKS_PER_SECOND) / f64::from((s.wind_up + s.recovery).max(1))
}

/// Damage per second, as Diablo's: the middle of its harm, its fire's and its holy harm, as often
/// as it swings (before its chance to land).
fn per_second(s: &Swing, fire: (i32, i32), holy: (i32, i32)) -> f64 {
    let mid = |(lo, hi): (i32, i32)| f64::from(lo + hi) / 20.0;
    (mid(s.damage) + mid(fire) + mid(holy)) * rate(s)
}

/// The wear a blow takes off it as a share of new: `wear` out of `items::NEW` a blow, at `made`
/// per mille of that ("1.3%", "0.05%": two places under one).
fn wear_share(wear: u16, made: u32) -> String {
    let pct = (u64::from(wear) * u64::from(made)) as f64 / 10_000.0;
    format!("{}%", number(pct, if pct < 1.0 { 2 } else { 1 }))
}

/// "Against your cudgel: +5.2 damage per second, ..." or, all the same, "much the same".
fn against_your(name: &str, said: &[String]) -> String {
    if said.is_empty() {
        format!("Against your {name}: much the same")
    } else {
        format!("Against your {name}: {}", said.join(", "))
    }
}

/// A property's word and tier, as the card shows them to the right of its lines: "Cruel III",
/// "Hallowed", "Clamorous (curse)".
fn tag(p: &Property) -> String {
    match p.tier {
        _ if p.curse => format!("{} (curse)", p.name),
        t @ 1..=3 => format!("{} {}", p.name, ["I", "II", "III"][usize::from(t) - 1]),
        _ => p.name.clone(),
    }
}

/// Where a property's lines go among the others: a weapon's or a bow's harm first, then its pace,
/// aim and reach, or a shield's block and bash, or armour's turning and warmth; then the body and
/// skills, then the rest (Path of Exile's order, roughly).
fn order(p: &Property) -> u8 {
    let does = [
        p.damage != 1000,
        p.fire.1 > 0,
        p.holy.1 > 0,
        p.swing != 1000,
        p.hit != 0,
        p.range != 0,
        p.block > 0,
        p.bash > 0,
        p.ward > 0,
        p.warmth != 0,
        p.strength > 0,
        p.fitness > 0,
        p.levels > 0,
        p.endurance != 1000,
        p.steps != 0,
        p.noise != 1000,
        p.wear != 1000,
        p.arrows != 1000,
        p.glow != 0,
        p.wane != 1000,
    ];
    does.iter().position(|&d| d).unwrap_or(does.len()) as u8
}

impl Defs {
    /// A property's lines on a card: its own line, if it's written in words, else what it does
    /// as numbers, a line each ("+80% damage", "+2 to Strength"). On `item`: "+2 to its skill"
    /// names the one it trains, and its breath is a blow's, a shot's or a block's.
    pub fn mod_lines(&self, p: &Property, item: &Item) -> Vec<String> {
        if !p.line.is_empty() {
            return vec![capital(&p.line)];
        }
        let pct = |m: u32| change(i64::from(m) - 1000);
        let sign = |n: i32| if n < 0 { "-" } else { "+" };
        let kind = self.fits(item.def);
        let a = match kind {
            Some(Fits::Bow) => "shot",
            Some(Fits::Shield) => "block",
            _ => "blow",
        };
        let mut out = Vec::new();
        if p.damage != 1000 {
            out.push(format!("{} damage", pct(p.damage)));
        }
        if p.fire.1 > 0 {
            out.push(format!("Adds {} fire damage", to(p.fire)));
        }
        if p.holy.1 > 0 {
            out.push(format!("Adds {} holy damage on the dead", to(p.holy)));
        }
        if p.swing != 1000 {
            out.push(format!("{} attack speed", quicker(p.swing)));
        }
        if p.hit != 0 {
            let by = share(p.hit.unsigned_abs());
            out.push(format!("{}{by} chance to hit", sign(p.hit)));
        }
        if p.range != 0 {
            let by = tiles(p.range.abs());
            out.push(format!("{}{by} of range", sign(p.range)));
        }
        if p.block > 0 {
            out.push(format!("+{} chance to block", share(p.block)));
        }
        if p.bash > 0 {
            out.push(format!("+{} chance a bash knocks down", share(p.bash)));
        }
        let level = |n: u32| number(f64::from(n) / 1000.0, 1);
        if p.strength > 0 {
            out.push(format!("+{} to Strength", level(p.strength)));
        }
        if p.fitness > 0 {
            out.push(format!("+{} to Fitness", level(p.fitness)));
        }
        let skill = match p.raises {
            Raises::Nothing => None,
            Raises::Skill(s) => Some(s.name()),
            Raises::Own => Some(self.own_skill(item.def).map_or("its skill", |s| s.name())),
        };
        if let Some(skill) = skill.filter(|_| p.levels > 0) {
            out.push(format!("+{} to {skill}", p.levels));
        }
        if p.endurance != 1000 {
            out.push(format!("{} breath a {a}", pct(p.endurance)));
        }
        if p.noise != 1000 {
            out.push(format!("{} noise", pct(p.noise)));
        }
        if p.wear != 1000 {
            out.push(format!("{} wear", pct(p.wear)));
        }
        if p.arrows != 1000 {
            out.push(format!("{} arrows broken", pct(p.arrows)));
        }
        if p.ward > 0 {
            out.push(format!("+{} chance to turn blows", share(p.ward)));
        }
        if p.warmth != 0 {
            let warmer = number(f64::from(p.warmth.unsigned_abs()) / 1000.0, 1);
            out.push(format!("{}{warmer} warmth", sign(p.warmth)));
        }
        if p.steps != 0 {
            let by = tiles(p.steps.abs());
            out.push(format!("{}{by} to every footstep", sign(p.steps)));
        }
        if p.glow != 0 {
            out.push(if p.fits == Fits::Light {
                format!("+{} of light", tiles(p.glow))
            } else {
                format!("Lights {} round you", tiles(p.glow))
            });
        }
        if p.wane != 1000 {
            out.push(format!("{} burn time", quicker(p.wane)));
        }
        if p.hallowed {
            out.push("What it puts down stays down".into());
        }
        // A unique's effect, if its data gives it no words: a weapon's blow's, a bow's shot's, a
        // shield's block's, or a blow's on what's worn.
        if p.drain > 0 {
            let drunk = mille(p.drain);
            out.push(format!("{drunk} of the harm it does comes back as health"));
        }
        if p.terror > 0 {
            let chance = share(p.terror);
            out.push(format!("{chance} chance a {a} sends the dead fleeing"));
        }
        if p.fell > 0 {
            out.push(format!("{} chance a {a} knocks down", share(p.fell)));
        }
        if p.hearten > 0 {
            let eased = number(f64::from(p.hearten) / 1000.0, 1);
            out.push(format!("Each of the dead it puts down eases {eased} dread"));
        }
        if p.cleave {
            out.push(if kind == Some(Fits::Bow) {
                "An arrow that puts one down flies on into the next".into()
            } else {
                "A blow that puts one down carries on into the next".into()
            });
        }
        if p.pyre {
            out.push("The dead it puts down catch fire".into());
        }
        if p.thorns.1 > 0 {
            let whatever = if kind == Some(Fits::Shield) {
                "whatever you block"
            } else {
                "whatever strikes you"
            };
            out.push(format!("{} harm back on {whatever}", to(p.thorns)));
        }
        if p.calm != 1000 {
            out.push(format!("{} dread taken", pct(p.calm)));
        }
        out
    }

    /// The fire's harm and the holy harm a weapon adds, as far as you know it.
    fn known_harm(&self, item: &Item) -> ((i32, i32), (i32, i32)) {
        self.known_of(item).fold(((0, 0), (0, 0)), |(f, h), p| {
            (
                (f.0 + p.fire.0, f.1 + p.fire.1),
                (h.0 + p.holy.0, h.1 + p.holy.1),
            )
        })
    }
}

impl World {
    /// A thing's card as far as you know it (above), set against what `mine` has in hand or wears
    /// where it goes (None: on its own, as for what you hold or wear yourself). Empty for a thing
    /// with none: anything mundane that's not a weapon, a shield, armour or a light.
    pub fn card(&self, item: &Item, mine: Option<ActorId>) -> Vec<CardLine> {
        let Some(def) = self.defs.item(item.def) else {
            return Vec::new();
        };
        let carded = matches!(
            def.kind,
            ItemKind::Weapon { .. }
                | ItemKind::Bow(_)
                | ItemKind::Shield(_)
                | ItemKind::Armour(_)
                | ItemKind::Light { .. }
        );
        if !carded && !item.magic.is_magic() {
            return Vec::new();
        }
        let mut c = Lines::default();
        self.card_name(&mut c, item);
        c.rule();
        match def.kind {
            ItemKind::Weapon { swing, wear, .. } => self.weapon_numbers(&mut c, item, swing, wear),
            ItemKind::Bow(r) => self.bow_numbers(&mut c, item, &r),
            ItemKind::Shield(g) => self.shield_numbers(&mut c, item, &g),
            ItemKind::Armour(a) => self.armour_numbers(&mut c, item, &a, def.weight),
            ItemKind::Light { radius, wane } => self.light_numbers(&mut c, item, radius, wane),
            _ => {}
        }
        c.rule();
        // What every weapon of its class has, as Path of Exile's implicits (`edge.rs`).
        if let ItemKind::Weapon { skill, .. } = def.kind {
            self.edge_lines(&mut c, skill);
            c.rule();
        }
        self.made_lines(&mut c, item);
        c.rule();
        self.magic_lines(&mut c, item);
        c.rule();
        self.notes(&mut c, item, mine);
        c.done()
    }

    /// Its name, then what it is: "Sorrow Fang", "Masterwork arming sword", "Blade weapon ·
    /// Rare".
    fn card_name(&self, c: &mut Lines, item: &Item) {
        let d = &self.defs;
        match d.own_name(item) {
            Some(own) => {
                c.add(CardStyle::Title, own, "");
                c.add(CardStyle::Kind, capital(&d.made_name(item)), "");
            }
            None => c.add(CardStyle::Title, capital(&d.label(item)), ""),
        }
        let what = match d.item(item.def).map(|d| &d.kind) {
            Some(ItemKind::Weapon { skill: Some(s), .. }) => format!("{} weapon", s.name()),
            Some(ItemKind::Weapon { .. }) => "Weapon".into(),
            Some(ItemKind::Bow(_)) => "Bow".into(),
            Some(ItemKind::Shield(_)) => "Shield".into(),
            Some(ItemKind::Armour(_)) => "Armour".into(),
            Some(ItemKind::Light { .. }) => "Light".into(),
            _ => String::new(),
        };
        let rarity = if item.magic.is_magic() {
            d.rarity_word(item)
        } else {
            ""
        };
        let line = match (what.is_empty(), rarity.is_empty()) {
            (_, true) => what,
            (true, false) => rarity.to_string(),
            (false, false) => format!("{what} \u{b7} {rarity}"),
        };
        if !line.is_empty() {
            c.add(CardStyle::Kind, line, "");
        }
    }

    /// A weapon's numbers, as made and as far as you know its magic, each marked where that
    /// changed it from its kind's: damage per second first, as Diablo's.
    fn weapon_numbers(&self, c: &mut Lines, item: &Item, base: Swing, wear: u16) {
        let d = &self.defs;
        let s = d.made_swing(item, base, true);
        let (fire, holy) = d.known_harm(item);
        let now = per_second(&s, fire, holy);
        let was = per_second(&base, (0, 0), (0, 0));
        c.stat(
            "Damage per second",
            format!("{now:.1}"),
            (now - was).abs() >= 0.05,
        );
        c.stat("Damage", span(s.damage), s.damage != base.damage);
        elements(c, fire, holy);
        c.stat(
            "Attacks per second",
            format!("{:.2}", rate(&s)),
            pace(&s) != pace(&base),
        );
        c.stat("Wind-up", seconds(s.wind_up), s.wind_up != base.wind_up);
        c.stat("Chance to hit", share(s.hit), s.hit != base.hit);
        c.stat(
            "Breath a blow",
            points(s.endurance),
            s.endurance != base.endurance,
        );
        c.stat("Noise", tiles(s.noise), s.noise != base.noise);
        let made = d.made_wear(item, true);
        let worn = format!("{} a blow", wear_share(wear, made));
        c.stat("Wear", worn, made != 1000);
        c.stat("Condition", condition(item), false);
    }

    /// A bow's numbers, as made and as far as you know its magic: what a shot does, how often,
    /// how far, each marked where that changed it from its kind's.
    fn bow_numbers(&self, c: &mut Lines, item: &Item, base: &Ranged) {
        let d = &self.defs;
        let r = d.made_bow(item, *base, true);
        let (s, b) = (r.swing, base.swing);
        let (fire, holy) = d.known_harm(item);
        let now = per_second(&s, fire, holy);
        let was = per_second(&b, (0, 0), (0, 0));
        c.stat(
            "Damage per second",
            format!("{now:.1}"),
            (now - was).abs() >= 0.05,
        );
        c.stat("Damage", span(s.damage), s.damage != b.damage);
        elements(c, fire, holy);
        c.stat(
            "Shots per second",
            format!("{:.2}", rate(&s)),
            pace(&s) != pace(&b),
        );
        c.stat("Draw", seconds(s.wind_up), s.wind_up != b.wind_up);
        let far = format!("{}, half at its range", share(s.hit));
        c.stat("Chance to hit", far, s.hit != b.hit);
        c.stat("Range", tiles(r.range), r.range != base.range);
        c.stat(
            "Breath a shot",
            points(s.endurance),
            s.endurance != b.endurance,
        );
        c.stat("Noise", tiles(s.noise), s.noise != b.noise);
        let ammo = d.item(r.ammo).map_or("arrow", |d| d.name.as_str());
        let breaks = format!("{}s break", capital(ammo));
        let often = format!("{} on a hit", share(r.breaks));
        c.stat(&breaks, often, r.breaks != base.breaks);
    }

    /// A shield's numbers, as made and as far as you know its magic: the block it gives braced,
    /// its bash, what a block costs you and it.
    fn shield_numbers(&self, c: &mut Lines, item: &Item, base: &Guard) {
        let g = self.defs.made_guard(item, *base, true);
        let block = format!("{} braced", share(g.block));
        c.stat("Chance to block", block, g.block != base.block);
        let breath = points(g.endurance);
        c.stat("Breath a block", breath, g.endurance != base.endurance);
        let bash = format!("+{} to knock down", share(g.bash));
        c.stat("Bash", bash, g.bash != base.bash);
        let worn = format!("{} a block", wear_share(g.wear, 1000));
        c.stat("Wear", worn, g.wear != base.wear);
        c.stat("Condition", condition(item), false);
    }

    /// A piece of armour's numbers: where it goes, what it turns, and what it costs you.
    fn armour_numbers(&self, c: &mut Lines, item: &Item, base: &Armour, grams: u32) {
        let a = self.defs.made_armour(item, *base, true);
        let places: Vec<&str> = Group::ALL
            .iter()
            .filter(|g| a.covers & g.bit() != 0)
            .map(|g| g.name())
            .collect();
        c.stat("Covers", places.join(", "), false);
        for (k, wound) in ["scratches", "lacerations", "bites"].iter().enumerate() {
            let changed = a.turn[k] != base.turn[k];
            c.stat(&format!("Turns {wound}"), share(a.turn[k]), changed);
        }
        let warmth = number(f64::from(a.warmth) / 1000.0, 1);
        c.stat("Warmth", warmth, a.warmth != base.warmth);
        let worn = u64::from(grams) * u64::from(self.rules.armour.worn_weight) / 1000;
        c.stat("Weight worn", kg(worn), false);
        if a.endurance > 0 {
            let more = change(i64::from(a.endurance));
            c.stat("Breath", format!("{more} on every cost"), false);
        }
        if a.noise > 0 {
            let louder = format!("+{}", tiles(a.noise));
            c.stat("Footsteps", louder, a.noise != base.noise);
        }
        if a.sight < 1000 {
            c.stat("Sight", format!("{} of yours", mille(a.sight)), false);
        }
        c.stat("Puts on in", seconds(a.time), false);
        let worn = format!("{} a blow turned", wear_share(a.wear, 1000));
        c.stat("Wear", worn, a.wear != base.wear);
        c.stat("Condition", condition(item), false);
    }

    /// A light's numbers, as far as you know its magic: how far it lights, how long it burns.
    fn light_numbers(&self, c: &mut Lines, item: &Item, radius: i32, wane: u16) {
        let d = &self.defs;
        let glow: i32 = d.known_of(item).map(|p| p.glow).sum();
        let pace = d.known_of(item).fold(1000u32, |w, p| w * p.wane / 1000);
        // Condition a game minute, as `light.rs` burns it down.
        let a_minute = (u32::from(wane) * pace / 1000).max(1);
        c.stat("Light", tiles(radius + glow), glow != 0);
        let lasts = hours(u32::from(NEW).div_ceil(a_minute));
        c.stat("Burns for", lasts, pace != 1000);
        if item.condition < NEW {
            let left = hours(u32::from(item.condition).div_ceil(a_minute));
            c.stat("Left", left, false);
        }
    }

    /// Its class's edge (`edge.rs`), as numbers, the class to the right: "+50% damage to one
    /// that's down", "Blade". A weapon of no class, or a class with none, says nothing.
    fn edge_lines(&self, c: &mut Lines, skill: Option<crate::skills::Skill>) {
        let Some(s) = skill else {
            return;
        };
        let (e, combat) = (self.rules.edges.of(skill), self.rules.combat);
        let mut lines = Vec::new();
        if e.down > 0 {
            lines.push(format!(
                "{} damage to one that's down",
                change(i64::from(e.down))
            ));
        }
        if e.fell > 0 {
            lines.push(format!("{} chance a blow knocks down", share(e.fell)));
        }
        if e.bleed > 0 {
            // A bleeding wound's harm comes every `bleed_every` ticks.
            let beats = (e.bleed / combat.bleed_every.max(1)) as i32;
            lines.push(format!(
                "A blow bleeds {} damage over {}",
                points(beats * combat.bleed_damage),
                seconds(e.bleed)
            ));
        }
        if e.push > 0 {
            lines.push(format!(
                "{} chance a blow drives it back a tile",
                share(e.push)
            ));
        }
        for line in lines {
            c.add(CardStyle::Mod, line, s.name());
        }
    }

    /// How well a thing was made, as numbers (Diablo II's superior and low-quality weapons): a
    /// weapon's or a bow's harm, armour's turning, a shield's block, and the wear of all but a bow
    /// (which doesn't wear). A plain one says nothing.
    fn made_lines(&self, c: &mut Lines, item: &Item) {
        let q = &self.defs.quality;
        if item.quality == PLAIN {
            return;
        }
        let word = capital(q.name_of(item.quality));
        let kind = self.defs.fits(item.def);
        let turn = q.turn_of(item.quality);
        let harm = i64::from(q.damage_of(item.quality)) - 1000;
        let wear = i64::from(q.wear_of(item.quality)) - 1000;
        let mut lines = Vec::new();
        match kind {
            Some(Fits::Armour) if turn != 0 => {
                lines.push(format!("{} chance to turn blows", signed_share(turn)));
            }
            Some(Fits::Shield) if turn != 0 => {
                lines.push(format!("{} chance to block", signed_share(turn)));
            }
            Some(Fits::Armour | Fits::Shield) => {}
            _ if harm != 0 => lines.push(format!("{} damage", change(harm))),
            _ => {}
        }
        if wear != 0 && kind != Some(Fits::Bow) {
            lines.push(format!("{} wear", change(wear)));
        }
        for line in lines {
            c.add(CardStyle::Mod, line, word.clone());
        }
    }

    /// Its magic as far as you know it: each property's lines, in order, curses last; then a
    /// unique's effect, in words, and its story.
    fn magic_lines(&self, c: &mut Lines, item: &Item) {
        let d = &self.defs;
        let m = item.magic;
        if !m.is_magic() {
            return;
        }
        // Each as it rolled, and, where it rolls, as its tier's least and most.
        let mut known: Vec<_> = d
            .rolls_of(item)
            .filter(|(p, _)| m.known || (p.curse && m.curse_shown))
            .map(|(p, roll)| {
                let ends = roll.map(|_| [p.rolled(d.magic.by(0)), p.rolled(d.magic.by(MOST))]);
                (d.as_rolled(p, roll), ends)
            })
            .collect();
        known.sort_by_key(|(p, _)| (p.curse, order(p)));
        for (p, ends) in &known {
            let style = if p.curse {
                CardStyle::Curse
            } else {
                CardStyle::Mod
            };
            let tag = tag(p);
            let mut lines = d.mod_lines(p, item);
            if let Some([least, most]) = ends {
                let (lo, hi) = (d.mod_lines(least, item), d.mod_lines(most, item));
                if lo.len() == lines.len() && hi.len() == lines.len() {
                    for (k, line) in lines.iter_mut().enumerate() {
                        *line = ranged(line, &lo[k], &hi[k]);
                    }
                }
            }
            for line in lines {
                c.add(style, line, tag.clone());
            }
        }
        if let Some(u) = d.unique_of(item).filter(|_| m.known) {
            c.rule();
            let e = &u.effect;
            let does = if e.line.is_empty() {
                d.mod_lines(e, item).join(", ")
            } else {
                e.line.clone()
            };
            c.add(CardStyle::Effect, format!("{}: {does}", e.name), "");
            if !u.lore.is_empty() {
                c.add(CardStyle::Lore, format!("\"{}\"", u.lore), "");
            }
        }
    }

    /// What you don't know of it yet, how it stands against yours, and what it pays as a find.
    fn notes(&self, c: &mut Lines, item: &Item, mine: Option<ActorId>) {
        let d = &self.defs;
        let m = item.magic;
        if m.is_magic() && !m.known {
            let how = match d.item(item.def).map(|d| &d.kind) {
                Some(ItemKind::Light { .. }) => "light it, or have a holdout read it".to_string(),
                Some(ItemKind::Armour(_)) => "put it on, or have a holdout read it".to_string(),
                Some(ItemKind::Shield(_)) => {
                    "take it up on your arm, or have a holdout read it".to_string()
                }
                Some(ItemKind::Bow(_)) => format!(
                    "have a holdout read it, or land {} shots with it",
                    d.magic.learn
                ),
                _ => format!(
                    "have a holdout read it, or strike {} blows with it",
                    d.magic.learn
                ),
            };
            c.add(CardStyle::Note, format!("Not known yet: {how}"), "");
        }
        if let Some(said) = mine.and_then(|id| self.against(item, id)) {
            c.add(CardStyle::Note, said, "");
        }
        let xp = (d.worth(item) + 500) / 1000;
        if item.found && xp > 0 {
            c.add(
                CardStyle::Note,
                format!("A find: {xp} XP in your stash"),
                "",
            );
        }
    }

    /// How a swing (or a shot) stands against another, as far as you know each thing: damage per
    /// second, how often (`often`: "attacks a second", "shots a second"), the chance to hit.
    fn swing_against(
        &self,
        (item, s): (&Item, &Swing),
        (mine, h): (&Item, &Swing),
        often: &str,
    ) -> Vec<String> {
        let d = &self.defs;
        let ((sf, sh), (hf, hh)) = (d.known_harm(item), d.known_harm(mine));
        let mut said = Vec::new();
        let more = per_second(s, sf, sh) - per_second(h, hf, hh);
        if more.abs() >= 0.05 {
            said.push(format!("{} damage per second", signed(more, 1)));
        }
        let quicker = rate(s) - rate(h);
        if quicker.abs() >= 0.005 {
            said.push(format!("{} {often}", signed(quicker, 2)));
        }
        let aim = s.hit as i32 - h.hit as i32;
        if aim != 0 {
            said.push(format!("{} chance to hit", signed_share(aim)));
        }
        said
    }

    /// How a weapon stands against the one in `id`'s hand, a bow against the bow there, a shield
    /// against the one on its arm, or a piece of armour against the one it wears where that goes,
    /// as far as you know each: "Against your cudgel: +5.2 damage per second, -0.25 attacks a
    /// second, +5% chance to hit". None for anything else, or with nothing there.
    fn against(&self, item: &Item, id: ActorId) -> Option<String> {
        let d = &self.defs;
        let a = self.actors.get(id)?;
        match d.item(item.def)?.kind {
            ItemKind::Weapon { swing, .. } => {
                let hand = a.hands?;
                let ItemKind::Weapon { swing: held, .. } = d.item(hand.def)?.kind else {
                    return None;
                };
                let (s, h) = (
                    d.made_swing(item, swing, true),
                    d.made_swing(&hand, held, true),
                );
                let said = self.swing_against((item, &s), (&hand, &h), "attacks a second");
                Some(against_your(&d.label(&hand), &said))
            }
            ItemKind::Bow(bow) => {
                let hand = a.hands?;
                let ItemKind::Bow(held) = d.item(hand.def)?.kind else {
                    return None;
                };
                let (r, h) = (d.made_bow(item, bow, true), d.made_bow(&hand, held, true));
                let mut said =
                    self.swing_against((item, &r.swing), (&hand, &h.swing), "shots a second");
                let farther = r.range - h.range;
                if farther != 0 {
                    let sign = if farther < 0 { "-" } else { "+" };
                    said.push(format!("{sign}{} of range", tiles(farther.abs())));
                }
                Some(against_your(&d.label(&hand), &said))
            }
            ItemKind::Shield(guard) => {
                let arm = a.shield?;
                let ItemKind::Shield(held) = d.item(arm.def)?.kind else {
                    return None;
                };
                let (g, h) = (
                    d.made_guard(item, guard, true),
                    d.made_guard(&arm, held, true),
                );
                let mut said = Vec::new();
                let block = g.block as i32 - h.block as i32;
                if block != 0 {
                    said.push(format!("{} chance to block", signed_share(block)));
                }
                let bash = g.bash as i32 - h.bash as i32;
                if bash != 0 {
                    said.push(format!("{} to knock down", signed_share(bash)));
                }
                let breath = g.endurance - h.endurance;
                if breath != 0 {
                    said.push(format!("{} breath a block", signed_points(breath)));
                }
                Some(against_your(&d.label(&arm), &said))
            }
            ItemKind::Armour(arm) => {
                let (piece, worn) = a.worn.iter().find_map(|w| match d.item(w.def)?.kind {
                    ItemKind::Armour(t) if t.covers & arm.covers != 0 => Some((w, t)),
                    _ => None,
                })?;
                let (arm, worn) = (
                    d.made_armour(item, arm, true),
                    d.made_armour(piece, worn, true),
                );
                let mean = |t: &Armour| t.turn.iter().map(|&n| i64::from(n)).sum::<i64>() / 3;
                let mut said = Vec::new();
                // Out of 10,000 a turn, so a tenth of it is per mille.
                let turned = (mean(&arm) - mean(&worn)) / 10;
                if turned != 0 {
                    said.push(format!("{} of blows turned", change(turned)));
                }
                let warmer = arm.warmth - worn.warmth;
                if warmer != 0 {
                    let sign = if warmer < 0 { "-" } else { "+" };
                    let by = number(f64::from(warmer.unsigned_abs()) / 1000.0, 1);
                    said.push(format!("{sign}{by} warmth"));
                }
                let breath = i64::from(arm.endurance) - i64::from(worn.endurance);
                if breath != 0 {
                    said.push(format!("{} breath", change(breath)));
                }
                Some(against_your(&d.label(piece), &said))
            }
            _ => None,
        }
    }
}

/// A swing's time, once round: its wind-up and recovery.
fn pace(s: &Swing) -> u32 {
    s.wind_up + s.recovery
}

/// A weapon's or a bow's fire and holy harm, each a line of its own in its own colour.
fn elements(c: &mut Lines, fire: (i32, i32), holy: (i32, i32)) {
    if fire.1 > 0 {
        c.add(CardStyle::Fire, "Fire damage", span(fire));
    }
    if holy.1 > 0 {
        let dead = format!("{} on the dead", span(holy));
        c.add(CardStyle::Holy, "Holy damage", dead);
    }
}

/// "+2", "-1.5": a change in tenths of a point.
fn signed_points(tenths: i32) -> String {
    let sign = if tenths < 0 { "-" } else { "+" };
    format!("{sign}{}", points(tenths.abs()))
}
