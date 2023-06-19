use crate::{maths::Vector2, primitive::SurfaceInteraction, spectra::Spectrum};

use super::{TextureMapping, TextureT};

pub struct CheckerboardTexture {
    a: Spectrum,
    b: Spectrum,
    to_uv: TextureMapping,
}

impl CheckerboardTexture {
    pub fn new(a: Spectrum, b: Spectrum, to_uv: TextureMapping) -> Self {
        Self { a, b, to_uv }
    }
}

impl TextureT for CheckerboardTexture {
    fn eval(&self, si: &SurfaceInteraction) -> Spectrum {
        let st = self.to_uv.map(si.uv);
        let mask = st - st.floor();
        if (mask.x > 0.5) == (mask.y > 0.5) {
            self.a
        } else {
            self.b
        }
    }
}
