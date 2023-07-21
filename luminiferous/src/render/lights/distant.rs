use crate::prelude::*;
use crate::primitive::Interaction;
use crate::spectra::Spectrum;

use super::{LightSample, LightT, Visibility};

pub struct DistantLight {
    w_light: Point3,
    radiance: Spectrum,
}

impl DistantLight {
    pub fn new(w_light: Vector3, radiance: Spectrum) -> Self {
        STATS.lights_created.inc();

        Self {
            w_light: w_light.normalize(),
            radiance,
        }
    }
}

impl LightT for DistantLight {
    fn is_environment(&self) -> bool {
        false
    }

    fn l_e(&self, _wi: Vector3) -> Spectrum {
        self.radiance
    }

    fn sample_li(&self, interaction: &Interaction, _u: Point2) -> LightSample {
        let wo = self.w_light;

        let visibility_ray = interaction.spawn_ray(wo);

        LightSample {
            wo,
            li: self.l_e(wo),
            visibility: Visibility {
                ray: visibility_ray,
                end: visibility_ray.at(1e7),
            },
        }
    }
}
