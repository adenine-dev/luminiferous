use crate::{
    maths::{Point2, Point3, Ray, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

use super::{LightSample, LightT, Visibility};

pub struct PointLight {
    p: Point3,
    radiance: Spectrum,
}

impl PointLight {
    pub fn new(p: Point3, radiance: Spectrum) -> Self {
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

    fn sample_li(&self, interaction: &SurfaceInteraction, _u: Point2) -> LightSample {
        let wi = (self.p - interaction.p).normalize();

        LightSample {
            wi,
            li: self.l_e(wi),
            visibility: Visibility {
                ray: Ray::new(interaction.p + interaction.n * 1.0e-6, wi),
                end: self.p,
            },
        }
    }
}
