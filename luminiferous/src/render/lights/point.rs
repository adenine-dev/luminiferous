use crate::{
    maths::{Point2, Point3, Vector3},
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

    fn sample_li(&self, interaction: &SurfaceInteraction, _u: Point2) -> LightSample {
        let wi = (self.p - interaction.p).normalize();

        LightSample {
            wi,
            li: self.l_e(wi),
            visibility: Visibility {
                ray: interaction.spawn_ray(wi),
                end: self.p,
            },
        }
    }
}
