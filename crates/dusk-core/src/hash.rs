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
pub type FastMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<Fnv64>>;

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
