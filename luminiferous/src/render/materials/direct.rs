use crate::{
    bsdfs::{Bsdf, BsdfSample, BsdfT},
    maths::{Frame3, Point2, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    stats::STATS,
};

use super::MaterialT;

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
    fn sample(&self, wi_world: Vector3, interaction: &SurfaceInteraction, u: Point2) -> BsdfSample {
        let frame = Frame3::new(interaction.n);
        let wi = frame.to_local(wi_world);
        let mut sample = self.bsdf.sample(wi, interaction, u);
        sample.wo = frame.to_world(sample.wo);
        sample
    }

    fn eval(&self, si: &SurfaceInteraction, wi_world: Vector3, wo_world: Vector3) -> Spectrum {
        let frame = Frame3::new(si.n);
        let wi = frame.to_local(wi_world);
        let wo = frame.to_local(wo_world);
        self.bsdf.eval(si, wi, wo)
    }

    // fn sample(&self, wi: Vector3, interaction: &SurfaceInteraction, u: Point2) -> BsdfSample {
    //     self.bsdf.sample(wi, interaction, u)
    // }

    // fn eval(&self, wi: Vector3, wo: Vector3) -> Spectrum {
    //     self.bsdf.eval(wi, wo)
    // }
}
