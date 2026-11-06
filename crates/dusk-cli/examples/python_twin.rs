//! `research/python_tick_bench.py`, line for line in Rust, to measure the language difference on
//! identical work. (The real core does more per tick: walls on edges, paths, hearing every tick.)
//! Run: `cargo run --release -p dusk-cli --example python_twin`

use std::time::Instant;

use dusk_core::hash::{Purpose, below, roll};

const W: i32 = 96;
const H: i32 = 96;
const TICK_HZ: f64 = 10.0;

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: u32) -> u32 {
        self.0 += 1;
        below(roll(7, self.0, 0, Purpose::Map), n)
    }
}

fn run(n_dead: usize, ticks: u32, seed: u64) -> f64 {
    let mut rnd = Rng(seed * 1_000_003);
    let mut blocked = vec![vec![false; W as usize]; H as usize];
    for row in blocked.iter_mut() {
        for b in row.iter_mut() {
            *b = rnd.below(100) < 12;
        }
    }
    let mut occ = vec![vec![-1i32; W as usize]; H as usize];
    let (mut px, mut py) = (W / 2, H / 2);
    blocked[py as usize][px as usize] = false;
    // x, y, target x, target y, progress
    let mut dead: Vec<(i32, i32, i32, i32, f64)> = Vec::new();
    while dead.len() < n_dead {
        let (x, y) = (rnd.below(W as u32) as i32, rnd.below(H as u32) as i32);
        if !blocked[y as usize][x as usize] && occ[y as usize][x as usize] < 0 && (x, y) != (px, py)
        {
            occ[y as usize][x as usize] = dead.len() as i32;
            dead.push((x, y, -1, -1, 0.0));
        }
    }
    let sees = |blocked: &Vec<Vec<bool>>, mut x0: i32, mut y0: i32, x1: i32, y1: i32| {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let mut err = dx + dy;
        while (x0, y0) != (x1, y1) {
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
            if (x0, y0) != (x1, y1) && blocked[y0 as usize][x0 as usize] {
                return false;
            }
        }
        true
    };
    let start = Instant::now();
    for tick in 0..ticks {
        if tick % 5 == 0 {
            let (nx, ny) = (px + rnd.below(3) as i32 - 1, py + rnd.below(3) as i32 - 1);
            if (0..W).contains(&nx)
                && (0..H).contains(&ny)
                && !blocked[ny as usize][nx as usize]
                && occ[ny as usize][nx as usize] < 0
            {
                (px, py) = (nx, ny);
            }
        }
        let noise = tick % 10 == 0;
        for (i, z) in dead.iter_mut().enumerate() {
            let (x, y) = (z.0, z.1);
            let (dx, dy) = (px - x, py - y);
            let d2 = dx * dx + dy * dy;
            if (d2 <= 225 && sees(&blocked, x, y, px, py)) || (noise && d2 <= 400) {
                z.2 = px;
                z.3 = py;
            }
            if z.2 < 0 {
                continue;
            }
            z.4 += 0.08;
            if z.4 < 1.0 {
                continue;
            }
            z.4 -= 1.0;
            let sx = (z.2 > x) as i32 - (z.2 < x) as i32;
            let sy = (z.3 > y) as i32 - (z.3 < y) as i32;
            for (mx, my) in [(sx, sy), (sx, 0), (0, sy)] {
                let (nx, ny) = (x + mx, y + my);
                let free = (0..W).contains(&nx)
                    && (0..H).contains(&ny)
                    && !blocked[ny as usize][nx as usize]
                    && occ[ny as usize][nx as usize] < 0
                    && (nx, ny) != (px, py);
                let corner = mx != 0
                    && my != 0
                    && (blocked[y as usize][nx as usize] || blocked[ny as usize][x as usize]);
                if (mx != 0 || my != 0) && free && !corner {
                    occ[y as usize][x as usize] = -1;
                    z.0 = nx;
                    z.1 = ny;
                    occ[ny as usize][nx as usize] = i as i32;
                    break;
                }
            }
        }
    }
    f64::from(ticks) / start.elapsed().as_secs_f64()
}

fn main() {
    for n in [100, 500] {
        run(n, 200, 9);
        let tps = (1..=3).map(|s| run(n, 3000, s)).fold(0.0, f64::max);
        println!(
            "{n:4} dead: {tps:9.0} ticks/s = {:7.0}x real time at 10 ticks per game second",
            tps / TICK_HZ
        );
    }
}
