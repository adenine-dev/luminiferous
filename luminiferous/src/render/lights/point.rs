use crate::{
    maths::{face_forward, Normal3, Point2, Point3, Ray, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    stats::STATS,
};

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
        let wi = (self.p - p).normalize();

        LightSample {
            wo: wi,
            li: self.l_e(wi),
            visibility: Visibility {
                ray: Ray::new(p + face_forward(n, wi) * 1e-6, wi),
                end: self.p,
            },
        }
    }
}
