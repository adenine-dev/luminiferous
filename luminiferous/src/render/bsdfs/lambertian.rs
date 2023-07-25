use crate::prelude::*;
use crate::{
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    stats::STATS,
    textures::{SpectralTexture, TextureT},
};

use super::{BsdfFlags, BsdfSample, BsdfT};

#[derive(Debug, Clone)]
pub struct Lambertian {
    reflectance: SpectralTexture,
}

impl Lambertian {
    pub fn new(reflectance: SpectralTexture) -> Self {
        STATS.bsdfs_created.inc();

        Self { reflectance }
    }
}

impl BsdfT for Lambertian {
    fn sample(
        &self,
        _wi: Vector3,
        interaction: &SurfaceInteraction,
        _u1: f32,
        u2: Point2,
    ) -> BsdfSample {
        let wo = warp::square_to_cosine_hemisphere(u2);

        BsdfSample {
            wo,
            sampled: self.flags(),
            spectrum: self.reflectance.eval(interaction),
        }
    }

    fn eval(&self, si: &SurfaceInteraction, _wi: Vector3, _wo: Vector3) -> Spectrum {
        self.reflectance.eval(si) * core::f32::consts::FRAC_1_PI
    }

    fn flags(&self) -> BsdfFlags {
        BsdfFlags::DiffuseReflection
    }
}
