use crate::prelude::*;

use crate::media::MediumInteraction;

use enum_dispatch::enum_dispatch;

mod isotropic;
pub use isotropic::*;

mod hg;
pub use hg::*;

pub struct PhaseFunctionSample {
    pub wo: Vector3,
}

#[enum_dispatch]
pub trait PhaseFunctionT {
    fn sample(&self, mi: &MediumInteraction, u: Point2) -> PhaseFunctionSample;

    fn eval(&self, mi: &MediumInteraction, wi: Vector3, wo: Vector3) -> f32;
}

#[enum_dispatch(PhaseFunctionT)]
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseFunction {
    Isotropic(IsotropicPhaseFunction),
    HenyeyGreenstein(HenyeyGreensteinPhaseFunction),
}
