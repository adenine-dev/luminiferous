use std::mem::size_of;

use crate::{
    maths::{Point2, Vector3},
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
};

use super::{Texture, TextureMapping, TextureT};
#[derive(Clone)]
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

        //TODO: debug texture that does more?
        // let n = (si.n + Vector3::splat(1.0)) / 2.0;
        // Spectrum::from_rgb(n.x, n.y, n.z)
    }

    fn eval_uv(&self, uv: Point2) -> Spectrum {
        Spectrum::from_rgb(uv[0], uv[1], 0.0)
    }
}
