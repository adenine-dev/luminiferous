use crate::prelude::*;
use crate::{
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
};

use super::{BsdfFlags, BsdfSample, BsdfT};

/// Nothing interacts with this bsdf, should be used for media transitions
#[derive(Debug, Clone, Default)]
pub struct NullBsdf {}

impl NullBsdf {
    pub fn new() -> Self {
        STATS.bsdfs_created.inc();
        Self {}
    }
}

impl BsdfT for NullBsdf {
    fn eval(&self, _si: &SurfaceInteraction, _wi: Vector3, _wo: Vector3) -> Spectrum {
        Spectrum::zero()
    }

    fn flags(&self) -> BsdfFlags {
        BsdfFlags::Null
    }

    fn sample(&self, wi: Vector3, _si: &SurfaceInteraction, _u1: f32, _u2: Point2) -> BsdfSample {
        BsdfSample {
            wo: -wi,
            sampled: self.flags(),
            spectrum: Spectrum::from_rgb(1.0, 1.0, 1.0),
        }
    }
}
