//! Smooth noise for the land, and plain rolls for scattering things: all a function of the seed
//! and the place, so any tile can be worked out on its own, in any order.

use dusk_core::hash::mix64;

/// What a number is for, so the layers of the land don't echo each other.
#[derive(Clone, Copy, Debug)]
#[repr(u64)]
pub enum Layer {
    Height = 1,
    Damp = 2,
    Tree = 3,
    Road = 4,
    Hamlet = 5,
    Dead = 6,
    DeadKind = 7,
}

/// A hash of (seed, layer, a, b): every input bit stirs every output bit.
pub fn hash(seed: u64, layer: Layer, a: i64, b: i64) -> u64 {
    mix64(seed ^ mix64((layer as u64) ^ mix64((a as u64) ^ mix64(b as u64))))
}

/// A number from 0 (included) to 1 (not), for (seed, layer, a, b).
pub fn unit(seed: u64, layer: Layer, a: i64, b: i64) -> f32 {
    (hash(seed, layer, a, b) >> 40) as f32 / (1u64 << 24) as f32
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Value noise: random heights on a lattice, eased between. From 0 to 1, varying over a unit.
fn value(seed: u64, layer: Layer, octave: i64, x: f32, y: f32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (sx, sy) = (smooth(x - x0), smooth(y - y0));
    let (ix, iy) = (x0 as i64, y0 as i64);
    let at = |dx: i64, dy: i64| {
        unit(
            seed ^ octave.wrapping_mul(0x9e37_79b9) as u64,
            layer,
            ix + dx,
            iy + dy,
        )
    };
    lerp(
        lerp(at(0, 0), at(1, 0), sx),
        lerp(at(0, 1), at(1, 1), sx),
        sy,
    )
}

/// Four octaves of value noise, from 0 to 1 (mostly 0.25 to 0.75), with features about `size`
/// tiles across.
pub fn fractal(seed: u64, layer: Layer, x: f32, y: f32, size: f32) -> f32 {
    let (mut sum, mut weight, mut amplitude, mut frequency) = (0.0, 0.0, 1.0, 1.0 / size);
    for octave in 0..4 {
        sum += amplitude * value(seed, layer, octave, x * frequency, y * frequency);
        weight += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    sum / weight
}
