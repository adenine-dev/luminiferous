use crate::{
    bsdfs::{Bsdf, BsdfSample, BsdfT},
    maths::Vector3,
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

use super::MaterialT;

pub struct DirectMaterial {
    bsdf: Bsdf,
}

impl DirectMaterial {
    pub fn new(bsdf: Bsdf) -> Self {
        Self { bsdf }
    }
}

impl MaterialT for DirectMaterial {
    fn sample(&self, wi: Vector3, interaction: &SurfaceInteraction) -> BsdfSample {
        self.bsdf.sample(wi, interaction)
    }

    fn eval(&self, wi: Vector3, wo: Vector3) -> Spectrum {
        self.bsdf.eval(wi, wo)
    }
}
