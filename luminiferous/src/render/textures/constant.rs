use crate::{primitive::SurfaceInteraction, spectra::Spectrum};

use super::TextureT;

pub struct ConstantTexture {
    value: Spectrum,
}

impl ConstantTexture {
    pub fn new(value: Spectrum) -> Self {
        Self { value }
    }
}

impl TextureT for ConstantTexture {
    fn eval(&self, _si: &SurfaceInteraction) -> Spectrum {
        self.value
    }
}
