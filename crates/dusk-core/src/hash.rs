//! Deterministic hashing, and the game's random rolls.

use std::hash::{BuildHasherDefault, Hasher};

/// FNV-1a, 64 bits: quick for small keys, and the same on every run. (The standard library's
/// default hasher is seeded randomly per run.)
#[derive(Clone, Copy, Debug)]
pub struct Fnv64(u64);

impl Default for Fnv64 {
    fn default() -> Fnv64 {
        Fnv64(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for Fnv64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

/// A hash map for lookups. Never iterate one in game logic: its order isn't defined.
pub type FastMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<WordHasher>>;

/// The lookup maps' hasher: a word at a time (rotate, xor, multiply: rustc's own "FxHash"),
/// where FNV goes a byte at a time. A tile position is three writes instead of nine bytes'
/// worth of multiplies. Only for lookups: what it hashes to is never saved, compared or
/// iterated in order (`Fnv64` is for that).
#[derive(Clone, Copy, Debug, Default)]
pub struct WordHasher(u64);

impl WordHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
}

impl Hasher for WordHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(8) {
            let mut word = [0u8; 8];
            word[..chunk.len()].copy_from_slice(chunk);
            self.add(u64::from_le_bytes(word));
        }
    }

    #[inline]
    fn write_u8(&mut self, n: u8) {
        self.add(u64::from(n));
    }

    #[inline]
    fn write_u16(&mut self, n: u16) {
        self.add(u64::from(n));
    }

    #[inline]
    fn write_u32(&mut self, n: u32) {
        self.add(u64::from(n));
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.add(n);
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.add(n as u64);
    }

    #[inline]
    fn write_i8(&mut self, n: i8) {
        self.add(n as u8 as u64);
    }

    #[inline]
    fn write_i32(&mut self, n: i32) {
        self.add(n as u32 as u64);
    }

    #[inline]
    fn write_i64(&mut self, n: i64) {
        self.add(n as u64);
    }
}

/// SplitMix64's finaliser: every input bit affects every output bit.
pub fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// What a roll is for, so that two rolls by the same actor on the same tick differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Purpose {
    Wander = 1,
    WanderDir = 2,
    Map = 3,
    Place = 4,
    Hit = 5,
    Damage = 6,
    Wound = 7,
    Part = 8,
    KnockDown = 9,
    Grab = 10,
    Struggle = 11,
    Thump = 12,
    Loot = 13,
    LootCount = 14,
    Fever = 15,
    GetUp = 16,
    GetUpWhen = 17,
    Arrival = 18,
    ArrivalKind = 19,
    Name = 20,
    Magic = 21,
    Weather = 22,
    Talent = 23,
    Block = 24,
}

/// A gameplay roll: a function of (seed, tick, who, why) and nothing else. No roll shifts when
/// another system starts rolling too, and none depends on the order the systems run in.
pub fn roll(seed: u64, tick: u64, who: u64, why: Purpose) -> u64 {
    mix64(seed ^ mix64(tick.wrapping_add(mix64(who.wrapping_add(mix64(why as u64))))))
}

/// A roll mapped onto 0..n.
pub fn below(roll: u64, n: u32) -> u32 {
    (((roll >> 32) * u64::from(n)) >> 32) as u32
}
