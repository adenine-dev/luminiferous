use crate::prelude::*;

use crate::{
    bsdfs::{Bsdf, BsdfFlags, BsdfSample, BsdfT},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

use super::{make_frame, MaterialT};

#[derive(Debug, Clone)]
pub struct DirectMaterial {
    bsdf: Bsdf,
}

impl DirectMaterial {
    pub fn new(bsdf: Bsdf) -> Self {
        STATS.materials_created.inc();

        Self { bsdf }
    }
}

impl MaterialT for DirectMaterial {
    fn sample(&self, wi_world: Vector3, si: &SurfaceInteraction, u: Point2) -> BsdfSample {
        let frame = make_frame(si);
        let wi = frame.to_local(wi_world).normalize();

        let mut sample = self.bsdf.sample(wi, si, u);
        sample.wo = frame.to_world(sample.wo).normalize();
        sample
    }

    fn eval(&self, si: &SurfaceInteraction, wi_world: Vector3, wo_world: Vector3) -> Spectrum {
        let frame = make_frame(si);

        let wi = frame.to_local(wi_world).normalize();
        let wo = frame.to_local(wo_world).normalize();
        self.bsdf.eval(si, wi, wo)
    }

    fn bsdf_flags(&self) -> BsdfFlags {
        self.bsdf.flags()
    }
}
