// broadly adapted from:
// https://graphics.pixar.com/library/MultiJitteredSampling/paper.pdf

use crate::prelude::*;

use super::SamplerT;

use oorandom::Rand32;

pub struct StratifiedSampler {
    spp: u32,
    sample_index: u32,
    dimension_index: u32,
    seed: u64,
    rng: Rand32,
    jitter: bool,
}

impl StratifiedSampler {
    pub fn new(mut spp: u32, seed: u64, jitter: bool) -> Self {
        if !spp.is_power_of_two() {
            let old = spp;
            spp = spp.next_power_of_two();
            warnln!("When using the stratified sampler, spp must be a power of 2, rounding from {old} to {spp}");
        }

        Self {
            spp,
            sample_index: 0,
            dimension_index: 0,
            seed,
            rng: Rand32::new(seed),
            jitter,
        }
    }
}

fn permute(mut index: u32, sample_count: u32, seed: u32) -> u32 {
    let mut w = sample_count - 1;
    w |= w >> 1;
    w |= w >> 2;
    w |= w >> 4;
    w |= w >> 8;
    w |= w >> 16;

    while {
        index ^= seed;
        index = index.wrapping_mul(0xe170893d);
        index ^= seed >> 16;
        index ^= (index & w) >> 4;
        index ^= seed >> 8;
        index = index.wrapping_mul(0x0929eb3f);
        index ^= seed >> 23;
        index ^= (index & w) >> 1;
        index = index.wrapping_mul(1 | seed >> 27);
        index = index.wrapping_mul(0x6935fa69);
        index ^= (index & w) >> 11;
        index = index.wrapping_mul(0x74dcb303);
        index ^= (index & w) >> 2;
        index = index.wrapping_mul(0x9e501cc3);
        index ^= (index & w) >> 2;
        index = index.wrapping_mul(0xc860a3df);
        index &= w;
        index ^= index >> 5;

        index >= sample_count
    } {}

    (index + seed) % sample_count
}

impl SamplerT for StratifiedSampler {
    fn begin_pixel(&mut self, _p: UPoint2) {
        self.dimension_index = 0;

        self.sample_index = 0;
    }

    fn advance(&mut self) -> bool {
        self.dimension_index = 0;

        self.sample_index += 1;
        self.sample_index <= self.spp
    }

    fn next_1d(&mut self) -> f32 {
        let s = permute(
            self.sample_index,
            self.spp,
            (self.seed as u32 + self.dimension_index).wrapping_mul(0xa511e9b3),
        );
        self.dimension_index += 1;

        let j = if self.jitter {
            self.rng.rand_float()
        } else {
            0.5
        };

        (s as f32 + j) / self.spp as f32
    }

    fn next_2d(&mut self) -> Point2 {
        let seed = self.seed as u32 + self.dimension_index;
        let s = permute(self.sample_index, self.spp, seed.wrapping_mul(0x51633e2d));

        let m = (self.spp as f32).sqrt() as u32;
        let n = (self.spp + m - 1) / m;

        let x = permute(s % m, m, (seed).wrapping_mul(0x68bc21eb)) as f32;
        let y = permute(s / m, n, (seed).wrapping_mul(0x02e5be93)) as f32;

        self.dimension_index += 1;

        let j = if self.jitter {
            Vector2::new(self.rng.rand_float(), self.rng.rand_float())
        } else {
            Vector2::splat(0.5)
        };

        Point2::new(
            (x + ((y + j.x) / n as f32)) / m as f32,
            (s as f32 + j.y) / self.spp as f32,
        )
    }

    fn fork(&self, seed: u64) -> Self {
        Self {
            spp: self.spp,
            sample_index: self.sample_index,
            dimension_index: 0,
            seed: self.seed + seed,
            rng: Rand32::new(self.seed + seed),
            jitter: self.jitter,
        }
    }

    fn get_spp(&self) -> u32 {
        self.spp
    }
}
