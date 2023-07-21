use crate::prelude::*;
use crate::primitive::Interaction;
use crate::{primitive::Primitive, spectra::Spectrum};

use super::{LightSample, LightT, Visibility};

pub struct AreaLight {
    radiance: Spectrum,
    area: f32,
    pub primitive: Primitive,
}

impl AreaLight {
    pub fn new(primitive: Primitive, radiance: Spectrum) -> Self {
        STATS.lights_created.inc();

        Self {
            area: primitive.area(),
            primitive,
            radiance,
        }
    }
}

impl LightT for AreaLight {
    fn is_environment(&self) -> bool {
        false
    }

    fn l_e(&self, _wi: Vector3) -> Spectrum {
        self.radiance
    }

    fn sample_li(&self, interaction: &Interaction, u: Point2) -> LightSample {
        let shape_sample = self.primitive.sample(u);
        let wo = (shape_sample.p - interaction.p).normalize();

        LightSample {
            li: self.l_e(wo),
            wo,
            visibility: Visibility {
                ray: interaction.spawn_ray(wo),
                end: shape_sample.p,
            },
        }
    }
}
