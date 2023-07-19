use crate::prelude::*;
use crate::{primitive::SurfaceInteraction, spectra::Spectrum};

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

    fn sample_li(&self, si: &SurfaceInteraction, u: Point2) -> LightSample {
        self.sample(si.p, si.n, u)
    }

    fn sample(&self, p: Point3, n: Normal3, _u: Point2) -> LightSample {
        let wo = (self.p - p).normalize();

        LightSample {
            wo,
            li: self.l_e(wo),
            visibility: Visibility {
                ray: Ray::new(p + face_forward(n, wo) * 1e-6, wo),
                end: self.p,
            },
        }
    }
}
