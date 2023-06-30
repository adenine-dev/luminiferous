use crate::{
    maths::{Point2, Vector3},
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
    textures::{Texture, TextureT},
};

use super::{BsdfFlags, BsdfSample, BsdfT};

#[derive(Clone)]
pub struct Conductor {
    reflectance: Texture,
}

impl Conductor {
    pub fn new(reflectance: Texture) -> Self {
        STATS.bsdfs_created.inc();

        Self { reflectance }
    }
}

fn reflect(v: Vector3) -> Vector3 {
    v * Vector3::new(-1.0, -1.0, 1.0)
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
