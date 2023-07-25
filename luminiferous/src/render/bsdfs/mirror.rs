use crate::prelude::*;
use crate::{
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
    textures::{SpectralTexture, TextureT},
};

use super::{util::reflect, BsdfFlags, BsdfSample, BsdfT};

/// A not strictly physically accurate mirror that perfectly reflects incoming rays.
#[derive(Debug, Clone)]
pub struct MirrorBsdf {
    reflectance: SpectralTexture,
}

impl MirrorBsdf {
    pub fn new(reflectance: SpectralTexture) -> Self {
        STATS.bsdfs_created.inc();

        Self { reflectance }
    }
}

impl BsdfT for MirrorBsdf {
    fn eval(&self, _si: &SurfaceInteraction, _wi: Vector3, _wo: Vector3) -> Spectrum {
        Spectrum::zero()
    }

    fn sample(&self, wi: Vector3, si: &SurfaceInteraction, _u1: f32, _u2: Point2) -> BsdfSample {
        let wo = reflect(wi);
        let reflectance = self.reflectance.eval(si);
        BsdfSample {
            wo,
            sampled: self.flags(),
            spectrum: reflectance,
        }
    }

    fn flags(&self) -> super::BsdfFlags {
        BsdfFlags::DeltaReflection
    }
}
