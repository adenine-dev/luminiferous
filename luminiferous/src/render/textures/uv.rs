use crate::{
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
};

use super::{TextureMapping, TextureT};

pub struct UvTexture {}

impl UvTexture {
    pub fn new() -> Self {
        Self {}
    }
}

impl TextureT for UvTexture {
    fn eval(&self, si: &SurfaceInteraction) -> Spectrum {
        Spectrum::from_rgb(si.uv[0], si.uv[1], 0.0)
    }
}
