use crate::prelude::*;
use crate::primitive::Interaction;
use crate::spectra::Spectrum;

use super::{LightSample, LightT, Visibility};

pub struct PointLight {
    p: Point3,
    radiance: Spectrum,
}

impl PointLight {
    pub fn new(p: Point3, radiance: Spectrum) -> Self {
        STATS.lights_created.inc();

        Self { p, radiance }
    }
}

impl LightT for PointLight {
    fn is_environment(&self) -> bool {
        false
    }

    fn l_e(&self, _wi: Vector3) -> Spectrum {
        self.radiance
    }

    fn sample_li(&self, interaction: &Interaction, _u: Point2) -> LightSample {
        let wo = (self.p - interaction.p).normalize();

        LightSample {
            wo,
            li: self.l_e(wo),
            visibility: Visibility {
                ray: interaction.spawn_ray(wo),
                end: self.p,
            },
        }
    }
}
