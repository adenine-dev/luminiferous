use crate::prelude::*;
use crate::{
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
    textures::{Texture, TextureT},
};

use super::{util::reflect, BsdfFlags, BsdfSample, BsdfT};

#[derive(Debug, Clone)]
pub struct Conductor {
    reflectance: Texture,
}

impl Conductor {
    pub fn new(reflectance: Texture) -> Self {
        STATS.bsdfs_created.inc();

        Self { reflectance }
    }
}

//TODO: fresnel stuffs

impl BsdfT for Conductor {
    fn eval(&self, _si: &SurfaceInteraction, _wi: Vector3, _wo: Vector3) -> Spectrum {
        Spectrum::zero()
    }

    fn sample(&self, wi: Vector3, si: &SurfaceInteraction, _u: Point2) -> BsdfSample {
        let wo = reflect(wi);
        let reflectance = self.reflectance.eval(si);
        BsdfSample {
            wo,
            spectrum: reflectance,
        }
    }

    fn flags(&self) -> super::BsdfFlags {
        BsdfFlags::DeltaReflection
    }
}
