mod constant;
pub use constant::*;

mod checkerboard;
pub use checkerboard::*;

mod uv;
pub use uv::*;

use enum_dispatch::enum_dispatch;

use crate::{
    maths::{Matrix3, Point2, Transform2},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

#[enum_dispatch]
pub trait TextureT {
    fn eval(&self, si: &SurfaceInteraction) -> Spectrum;
}

#[enum_dispatch(TextureT)]
pub enum Texture {
    Constant(ConstantTexture),
    Checkerboard(CheckerboardTexture),
    Uv(UvTexture),
}

pub struct TextureMapping {
    pub transform: Transform2,
}

impl TextureMapping {
    pub fn new(transform: Matrix3) -> Self {
        Self {
            transform: Transform2::new(transform),
        }
    }

    pub fn map(&self, uv: Point2) -> Point2 {
        self.transform.transform_point(uv)
    }
}
