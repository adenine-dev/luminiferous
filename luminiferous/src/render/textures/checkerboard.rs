use std::mem::size_of;

use crate::{
    maths::{Point2, Vector2},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    stats::STATS,
};

use super::{Texture, TextureMapping, TextureT};

#[derive(Clone)]
pub struct CheckerboardTexture {
    a: Spectrum,
    b: Spectrum,
    to_uv: TextureMapping,
}

impl CheckerboardTexture {
    pub fn new(a: Spectrum, b: Spectrum, to_uv: TextureMapping) -> Self {
        STATS.textures_created.inc();
        STATS.texture_memory.add(size_of::<Texture>() as u64);

        Self { a, b, to_uv }
    }
}

impl TextureT for CheckerboardTexture {
    fn eval(&self, si: &SurfaceInteraction) -> Spectrum {
        self.eval_uv(si.uv)
    }

    fn eval_uv(&self, uv: Point2) -> Spectrum {
        let st = self.to_uv.map(uv);
        let mask = st - st.floor();
        if (mask.x > 0.5) == (mask.y > 0.5) {
            self.a
        } else {
            self.b
        }
    }
}
