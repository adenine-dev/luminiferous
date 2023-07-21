use crate::prelude::*;
use crate::samplers::Sampler;
use crate::spectra::SpectrumT;
use crate::{phase_functions::PhaseFunction, spectra::Spectrum};

use super::{MediumInteraction, MediumT};

#[derive(Debug, Clone, PartialEq)]
pub struct HomogeneousMedium {
    phase_function: PhaseFunction,
    albedo: Spectrum,
    sigma_t: Spectrum,
    scale: f32,
}

//TODO: sigma, spatially varying albedo, scale, etc

impl HomogeneousMedium {
    pub fn new(
        phase_function: PhaseFunction,
        albedo: Spectrum,
        sigma_t: Spectrum,
        scale: f32,
    ) -> Self {
        Self {
            phase_function,
            albedo,
            sigma_t,
            scale,
        }
    }
}

impl MediumT for HomogeneousMedium {
    fn transmittance(&self, ray: Ray, t_max: f32) -> Spectrum {
        self.albedo * (-self.sigma_t * f32::MAX.min(t_max * ray.d.length())).exp()
    }

    fn sample(&self, ray: Ray, t_max: f32, u1: f32) -> Option<(MediumInteraction, Spectrum)> {
        let dist = -(1.0 - u1).ln() / self.scale;
        let t = (dist / ray.d.length()).min(t_max) / self.sigma_t.y();

        if t < t_max {
            let tr = self.transmittance(ray, t);
            Some((
                MediumInteraction {
                    p: ray.at(t),
                    wi: -ray.d,
                    medium: None,
                    phase_function: Some(self.phase_function.clone()),
                },
                tr,
            ))
        } else {
            None
        }
    }
}
