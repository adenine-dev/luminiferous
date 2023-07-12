use crate::prelude::*;
use crate::{phase_functions::PhaseFunction, spectra::Spectrum};

use super::{MediumInteraction, MediumT};

#[derive(Debug, Clone, PartialEq)]
pub struct HomogeneousMedium {
    phase_function: PhaseFunction,
    albedo: Spectrum,
}

//TODO: sigma, spatially varying albedo, scale, etc

impl HomogeneousMedium {
    pub fn new(phase_function: PhaseFunction, albedo: Spectrum) -> Self {
        Self {
            phase_function,
            albedo,
        }
    }
}

impl MediumT for HomogeneousMedium {
    fn sample(&self, ray: Ray, t_max: f32, u1: f32) -> Option<(MediumInteraction, Spectrum)> {
        let dist = -(1.0 - u1).ln();
        let t = dist;

        if t < t_max {
            Some((
                MediumInteraction {
                    p: ray.at(t),
                    wi: -ray.d,
                    phase_function: Some(self.phase_function.clone()),
                },
                self.albedo,
            ))
        } else {
            None
        }
    }
}
