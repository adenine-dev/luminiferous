use rand::prelude::Distribution;

use crate::{
    maths::{random_in_hemisphere, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

use super::{BsdfSample, BsdfT};

pub struct Lambertian {
    reflectance: Spectrum,
}

impl Lambertian {
    pub fn new(reflectance: Spectrum) -> Self {
        Self { reflectance }
    }
}

impl BsdfT for Lambertian {
    fn sample(&self, _wi: Vector3, interaction: &SurfaceInteraction) -> BsdfSample {
        let wo = random_in_hemisphere(interaction.n).normalize();

        BsdfSample {
            wo,
            spectrum: self.reflectance,
        }
    }

    fn eval(&self, _wi: Vector3, _wo: Vector3) -> Spectrum {
        self.reflectance * core::f32::consts::FRAC_1_PI
    }
}
