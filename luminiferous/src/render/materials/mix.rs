use crate::prelude::*;
use crate::textures::{Texture, TextureT};
use crate::{
    bsdfs::{Bsdf, BsdfFlags, BsdfSample, BsdfT},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

use super::{make_frame, MaterialT};

#[derive(Debug, Clone)]
pub struct MixMaterial {
    a: Bsdf,
    b: Bsdf,
    mask: Texture<f32>,
}

impl MixMaterial {
    pub fn new(a: Bsdf, b: Bsdf, mask: Texture<f32>) -> Self {
        STATS.materials_created.inc();

        Self { a, b, mask }
    }
}

impl MaterialT for MixMaterial {
    fn sample(&self, wi_world: Vector3, si: &SurfaceInteraction, u: Point2) -> BsdfSample {
        let frame = make_frame(si);

        let wi = frame.to_local(wi_world);

        let t = self.mask.eval(si).clamp(0.0, 1.0);

        let mut sample = if t == 0.0 {
            self.a.sample(wi, si, u)
        } else if t == 1.0 {
            self.b.sample(wi, si, u)
        } else {
            let a = self.a.sample(wi, si, u);
            let b = self.b.sample(wi, si, u);
            BsdfSample {
                wo: (a.wo * (1.0 - t) + b.wo * t).normalize(),
                sampled: a.sampled | b.sampled,
                spectrum: a.spectrum * (1.0 - t) + b.spectrum * t,
            }
        };

        sample.wo = frame.to_world(sample.wo);
        sample
    }

    fn eval(&self, si: &SurfaceInteraction, wi_world: Vector3, wo_world: Vector3) -> Spectrum {
        let frame = make_frame(si);
        let wi = frame.to_local(wi_world);
        let wo = frame.to_local(wo_world);

        let t = self.mask.eval(si).clamp(0.0, 1.0);
        if t == 0.0 {
            self.a.eval(si, wi, wo)
        } else if t == 1.0 {
            self.b.eval(si, wi, wo)
        } else {
            let a = self.a.eval(si, wi, wo);
            let b = self.b.eval(si, wi, wo);
            a * (1.0 - t) + b * t
        }
    }

    fn bsdf_flags(&self) -> BsdfFlags {
        self.a.flags() & self.b.flags()
    }
}
