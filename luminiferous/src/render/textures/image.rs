use std::{mem::size_of, path::Path};

use crate::prelude::*;
use crate::{
    core::Array2d,
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
};

use super::{SpectralTexture, TextureT};

#[derive(Debug, Clone)]
pub struct ImageTexture<T: Copy> {
    pixels: Array2d<T>,
}

impl<T: Copy> ImageTexture<T> {
    pub fn new(pixels: Array2d<T>) -> Self {
        Self { pixels }
    }
}

impl ImageTexture<Spectrum> {
    pub fn from_path(path: &Path) -> Self {
        let image = image::open(path).unwrap();

        let extent = UExtent2::new(image.width(), image.height());

        let inverse_gamma = |x: f32| -> f32 {
            if x <= 0.04045 {
                x * 1.0 / 12.92
            } else {
                ((x + 0.0550) / 1.055).powf(2.4)
            }
        };

        // mild hack
        // FIXME: have this be a param
        let undo_gamma_correct = !matches!(
            image,
            image::DynamicImage::ImageRgb32F(_) | image::DynamicImage::ImageRgba32F(_)
        );

        let pixels = image
            .to_rgb32f()
            .chunks(3)
            .map(|p| {
                Spectrum::from_rgb(
                    if undo_gamma_correct {
                        inverse_gamma(p[0])
                    } else {
                        p[0]
                    },
                    if undo_gamma_correct {
                        inverse_gamma(p[1])
                    } else {
                        p[1]
                    },
                    if undo_gamma_correct {
                        inverse_gamma(p[2])
                    } else {
                        p[2]
                    },
                )
            })
            .collect::<Vec<_>>();

        STATS.textures_created.inc();
        STATS
            .texture_memory
            .add(size_of::<Spectrum>() as u64 * pixels.len() as u64);
        STATS
            .texture_memory
            .add(size_of::<SpectralTexture>() as u64);

        Self {
            pixels: Array2d::from_1d(extent, pixels),
        }
    }
}

impl<T: Copy> TextureT<T> for ImageTexture<T> {
    fn eval(&self, si: &SurfaceInteraction) -> T {
        self.eval_uv(si.uv)
    }

    fn eval_uv(&self, uv: Point2) -> T {
        let uv = (uv % Point2::ONE + Point2::ONE) % Point2::ONE;

        //TODO: better filtering
        let x = (uv) * (self.pixels.get_extent() - UVector2::splat(1)).as_vec2();

        self.pixels[x.y as usize][x.x as usize]
    }

    fn extent(&self) -> UExtent2 {
        self.pixels.get_extent()
    }
}
