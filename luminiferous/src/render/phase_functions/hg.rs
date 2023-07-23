use crate::{media::MediumInteraction, prelude::*};

use super::{PhaseFunctionSample, PhaseFunctionT};

#[derive(Debug, Clone, PartialEq)]
pub struct HenyeyGreensteinPhaseFunction {
    g: f32,
}

impl HenyeyGreensteinPhaseFunction {
    pub fn new(g: f32) -> Self {
        if !(-1.0..=1.0).contains(&g) {
            warnln!("The Henyey Greenstein phase function g asymmetry value is not in the range -1.0 <= g <= 1.0. g = {g}");
        }
        if g == 0.0 {
            warnln!("Using the Henyey Greenstein phase function when g = 0.0 instead of the roughly equivalent and cheaper isotropic phase function.");
        }

        Self { g }
    }

    #[inline]
    fn hg(&self, cos_theta: f32) -> f32 {
        let denom = 1.0 + self.g * self.g + 2.0 * self.g * cos_theta;
        (core::f32::consts::PI * 4.0).recip() * (1.0 - self.g * self.g) / (denom * denom.sqrt())
    }
}

impl PhaseFunctionT for HenyeyGreensteinPhaseFunction {
    fn eval(&self, _mi: &MediumInteraction, wi: Vector3, wo: Vector3) -> f32 {
        self.hg(wi.dot(wo))
    }

    fn sample(&self, mi: &MediumInteraction, u: Point2) -> PhaseFunctionSample {
        let cos_theta = if self.g.abs() < 1e-3 {
            1.0 - 2.0 * u[0]
        } else {
            -(1.0 + self.g.powi(2)
                - ((1.0 - self.g.powi(2)) / (1.0 + self.g - 2.0 * self.g * u[0])).powi(2))
                / (2.0 * self.g)
        };

        let sin_theta = ((1.0 - cos_theta * cos_theta).max(0.0)).sqrt();
        let phi = core::f32::consts::TAU * u[1];

        let (v1, v2) = Frame3::coordinate_system(-mi.wi);

        PhaseFunctionSample {
            wo: spherical_direction_in(sin_theta, cos_theta, phi, v1, v2, -mi.wi),
        }
    }
}
