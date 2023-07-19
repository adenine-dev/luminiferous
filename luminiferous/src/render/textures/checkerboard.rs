use std::mem::size_of;

use crate::prelude::*;
use crate::primitive::SurfaceInteraction;

use super::{SpectralTexture, TextureMapping, TextureT};

#[derive(Debug, Clone)]
pub struct CheckerboardTexture<T: Copy> {
    a: T,
    b: T,
    to_uv: TextureMapping,
}

impl<T: Copy> CheckerboardTexture<T> {
    pub fn new(a: T, b: T, to_uv: TextureMapping) -> Self {
        STATS.textures_created.inc();
        STATS
            .texture_memory
            .add(size_of::<SpectralTexture>() as u64);

        Self { a, b, to_uv }
    }
}

impl<T: Copy> TextureT<T> for CheckerboardTexture<T> {
    fn eval(&self, si: &SurfaceInteraction) -> T {
        self.eval_uv(si.uv)
    }

    fn eval_uv(&self, uv: Point2) -> T {
        let st = self.to_uv.map(uv);
        let mask = st - st.floor();
        if (mask.x > 0.5) == (mask.y > 0.5) {
            self.a
        } else {
            self.b
        }
    }

    fn extent(&self) -> UExtent2 {
        UExtent2::splat(1)
    }
}
