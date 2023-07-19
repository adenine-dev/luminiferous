mod constant;
pub use constant::*;

mod checkerboard;
pub use checkerboard::*;

mod uv;
pub use uv::*;

mod image;
pub use self::image::*;

use crate::prelude::*;
use crate::primitive::SurfaceInteraction;
use crate::spectra::Spectrum;

pub trait TextureT<T> {
    fn eval(&self, si: &SurfaceInteraction) -> T;

    fn eval_uv(&self, uv: Point2) -> T;

    fn extent(&self) -> UExtent2;
}

#[derive(Debug, Clone)]
pub enum SpectralTexture {
    Constant(ConstantTexture<Spectrum>),
    Checkerboard(CheckerboardTexture<Spectrum>),
    Uv(UvTexture),
    Image(ImageTexture<Spectrum>),
}

// enum_dispatch really doesn't like this so we impl it manually
impl TextureT<Spectrum> for SpectralTexture {
    fn eval(&self, si: &SurfaceInteraction) -> Spectrum {
        match self {
            SpectralTexture::Constant(inner) => inner.eval(si),
            SpectralTexture::Checkerboard(inner) => inner.eval(si),
            SpectralTexture::Uv(inner) => inner.eval(si),
            SpectralTexture::Image(inner) => inner.eval(si),
        }
    }

    fn eval_uv(&self, uv: Point2) -> Spectrum {
        match self {
            SpectralTexture::Constant(inner) => inner.eval_uv(uv),
            SpectralTexture::Checkerboard(inner) => inner.eval_uv(uv),
            SpectralTexture::Uv(inner) => inner.eval_uv(uv),
            SpectralTexture::Image(inner) => inner.eval_uv(uv),
        }
    }

    fn extent(&self) -> UExtent2 {
        match self {
            SpectralTexture::Constant(inner) => inner.extent(),
            SpectralTexture::Checkerboard(inner) => inner.extent(),
            SpectralTexture::Uv(inner) => inner.extent(),
            SpectralTexture::Image(inner) => inner.extent(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Texture<T: Copy> {
    Constant(ConstantTexture<T>),
    Checkerboard(CheckerboardTexture<T>),
    Image(ImageTexture<T>),
}

// enum_dispatch really doesn't like this so we impl it manually
impl<T: Copy> TextureT<T> for Texture<T> {
    fn eval(&self, si: &SurfaceInteraction) -> T {
        match self {
            Texture::Constant(inner) => inner.eval(si),
            Texture::Checkerboard(inner) => inner.eval(si),
            Texture::Image(inner) => inner.eval(si),
        }
    }

    fn eval_uv(&self, uv: Point2) -> T {
        match self {
            Texture::Constant(inner) => inner.eval_uv(uv),
            Texture::Checkerboard(inner) => inner.eval_uv(uv),
            Texture::Image(inner) => inner.eval_uv(uv),
        }
    }

    fn extent(&self) -> UExtent2 {
        match self {
            Texture::Constant(inner) => inner.extent(),
            Texture::Checkerboard(inner) => inner.extent(),
            Texture::Image(inner) => inner.extent(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
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
