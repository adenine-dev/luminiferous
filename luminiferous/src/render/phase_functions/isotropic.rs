use crate::{
    maths::{warp, Point2, Vector3},
    media::MediumInteraction,
};

use super::{PhaseFunctionSample, PhaseFunctionT};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IsotropicPhaseFunction {}

impl IsotropicPhaseFunction {
    pub fn new() -> Self {
        Self {}
    }
}

impl PhaseFunctionT for IsotropicPhaseFunction {
    fn sample(&self, u: Point2) -> PhaseFunctionSample {
        PhaseFunctionSample {
            wo: warp::square_to_uniform_sphere(u),
            // spectrum: Spectrum::splat(1.0),
        }
    }

    fn eval(&self, _mi: &MediumInteraction, _wi: Vector3, _wo: Vector3) -> f32 {
        core::f32::consts::FRAC_PI_4
    }
}
