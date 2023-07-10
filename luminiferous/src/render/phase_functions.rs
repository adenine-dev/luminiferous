use crate::{
    maths::{Point2, Vector3},
    media::MediumInteraction,
};

use enum_dispatch::enum_dispatch;

mod isotropic;
pub use isotropic::*;

pub struct PhaseFunctionSample {
    pub wo: Vector3,
}

#[enum_dispatch]
pub trait PhaseFunctionT {
    fn sample(&self, u: Point2) -> PhaseFunctionSample;

    fn eval(&self, mi: &MediumInteraction, wi: Vector3, wo: Vector3) -> f32;
}

#[enum_dispatch(PhaseFunctionT)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhaseFunction {
    Isotropic(IsotropicPhaseFunction),
}
