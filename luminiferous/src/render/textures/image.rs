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
        let image = exr::prelude::read_first_rgba_layer_from_file(
            path,
            |resolution, _| {
                let default_pixel = [0.0, 0.0, 0.0, 0.0];
                let empty_line = vec![default_pixel; resolution.width()];
                let empty_image = vec![empty_line; resolution.height()];
                empty_image
            },
            |pixel_vector, position, (r, g, b, a): (f32, f32, f32, f32)| {
                pixel_vector[position.y()][position.x()] = [r, g, b, a]
            },
        )
        .unwrap();
        let extent = UExtent2::new(
            image.layer_data.size.0 as u32,
            image.layer_data.size.1 as u32,
        );

        let pixels = image
            .layer_data
            .channel_data
            .pixels
            .iter()
            .flatten()
            .map(|p| Spectrum::from_rgb(p[0], p[1], p[2]))
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
        //TODO: better filtering
        let x = (uv) * (self.pixels.get_extent() - UVector2::splat(1)).as_vec2();

        self.pixels[x.y as usize][x.x as usize]
    }

    fn extent(&self) -> UExtent2 {
        self.pixels.get_extent()
    }
}
