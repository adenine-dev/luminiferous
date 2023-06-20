use std::mem::size_of;

use crate::{primitive::SurfaceInteraction, spectra::Spectrum, stats::STATS};

use super::{Texture, TextureT};

pub struct ConstantTexture {
    value: Spectrum,
}

impl ConstantTexture {
    pub fn new(value: Spectrum) -> Self {
        STATS.textures_created.inc();
        STATS.texture_memory.add(size_of::<Texture>() as u64);

        Self { value }
    }
}

impl TextureT for ConstantTexture {
    fn eval(&self, _si: &SurfaceInteraction) -> Spectrum {
        self.value
    }
}
