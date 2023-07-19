use crate::prelude::*;
use crate::{
    primitive::{Primitive, SurfaceInteraction},
    spectra::Spectrum,
};

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

    fn sample_li(&self, si: &SurfaceInteraction, u: Point2) -> LightSample {
        self.sample(si.p, si.n, u)
    }

    fn sample(&self, p: Point3, n: Normal3, u: Point2) -> LightSample {
        let shape_sample = self.primitive.sample(u);
        let wo = face_forward((shape_sample.p - p).normalize(), n);

        LightSample {
            li: self.l_e(wo),
            wo,
            visibility: Visibility {
                ray: Ray::new(p + face_forward(n, wo) * 1e-6, wo),
                end: shape_sample.p,
            },
        }
    }
}
