use crate::maths::{Point2, UPoint2};

use super::SamplerT;

use oorandom::Rand32;

pub struct RandomSampler {
    spp: u32,
    samples: u32,
    seed: u64,
    rng: Rand32,
}

impl RandomSampler {
    pub fn new(spp: u32, seed: u64) -> Self {
        Self {
            spp,
            samples: 0,
            seed,
            rng: Rand32::new(seed),
        }
    }
}

impl SamplerT for RandomSampler {
    fn begin_pixel(&mut self, _p: UPoint2) {
        self.samples = 0;
    }

    fn advance(&mut self) -> bool {
        self.samples += 1;
        self.samples <= self.spp
    }

    fn next_1d(&mut self) -> f32 {
        self.rng.rand_float()
    }

    fn next_2d(&mut self) -> Point2 {
        Point2::new(self.next_1d(), self.next_1d())
    }

    fn fork(&self, seed: u64) -> Self {
        Self {
            spp: self.spp,
            samples: self.samples,
            seed: self.seed + seed,
            rng: Rand32::new(self.seed + seed),
        }
    }

    fn get_spp(&self) -> u32 {
        self.spp
    }
}
