use crate::{
    maths::{random_in_hemisphere, Point3, Vector3, Ray},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

use super::{EmitterSample, EmitterT, Visibility};

pub struct PointLight {
    p: Point3,
    radiance: Spectrum,
}

impl PointLight {
    pub fn new(p: Point3, radiance: Spectrum) -> Self {
        Self { p, radiance }
    }
}

impl EmitterT for PointLight {
    fn is_environment(&self) -> bool {
        false
    }

    fn l_e(&self, _wi: Vector3) -> Spectrum {
        self.radiance
    }

    fn sample_li(&self, interaction: &SurfaceInteraction) -> EmitterSample {
        let wi = (self.p - interaction.p).normalize();

        EmitterSample {
            wi,
            li: self.l_e(wi),
            visibility: Visibility {
                ray: Ray::new(interaction.p, wi),
                end: self.p,
            },
        }
    }
}
