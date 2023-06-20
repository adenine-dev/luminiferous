use std::mem::size_of;

use crate::{
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
};

use super::{Texture, TextureMapping, TextureT};

pub struct UvTexture {}

impl UvTexture {
    pub fn new() -> Self {
        STATS.textures_created.inc();
        STATS.texture_memory.add(size_of::<Texture>() as u64);

        Self {}
    }
}

impl TextureT for UvTexture {
    fn eval(&self, si: &SurfaceInteraction) -> Spectrum {
        Spectrum::from_rgb(si.uv[0], si.uv[1], 0.0)
    }
}
