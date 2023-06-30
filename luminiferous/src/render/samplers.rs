use enum_dispatch::enum_dispatch;

mod random;
pub use random::*;

mod stratified;
pub use stratified::*;

use crate::maths::{Point2, UPoint2};

#[enum_dispatch]
pub trait SamplerT {
    fn begin_pixel(&mut self, p: UPoint2);

    fn advance(&mut self) -> bool;

    fn next_1d(&mut self) -> f32;

    fn next_2d(&mut self) -> Point2;

    fn fork(&self, seed: u64) -> Self;

    fn get_spp(&self) -> u32;
}

#[enum_dispatch(SamplerT)]
pub enum Sampler {
    Random(RandomSampler),
    Stratified(StratifiedSampler),
}
