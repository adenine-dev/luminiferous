use std::mem::size_of;

use crate::prelude::*;
use crate::primitive::SurfaceInteraction;

use super::{SpectralTexture, TextureT};

#[derive(Debug, Clone)]
pub struct ConstantTexture<T: Copy> {
    value: T,
}

impl<T: Copy> ConstantTexture<T> {
    pub fn new(value: T) -> Self {
        STATS.textures_created.inc();
        STATS
            .texture_memory
            .add(size_of::<SpectralTexture>() as u64);

        Self { value }
    }
}

impl<T: Copy> TextureT<T> for ConstantTexture<T> {
    fn eval(&self, _si: &SurfaceInteraction) -> T {
        self.value
    }

    fn eval_uv(&self, _uv: Point2) -> T {
        self.value
    }

    fn extent(&self) -> UExtent2 {
        UExtent2::splat(1)
    }
}
