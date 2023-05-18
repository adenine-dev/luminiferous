use std::path::Path;

use crate::{containers::Array2d, Point2, UExtent2};

#[derive(Debug, Clone, Copy, Default)]
struct Pixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

pub struct Film {
    pixels: Array2d<Pixel>,
}

impl Film {
    pub fn new(extent: UExtent2) -> Self {
        Self {
            pixels: Array2d::with_default(extent, Pixel::default()),
        }
    }

    pub fn apply_sample(&mut self, p: Point2, sample: (f32, f32, f32)) {
        let p = p.floor(); // bad filter lol
        self.pixels[p.y as usize][p.x as usize] = Pixel {
            r: sample.0,
            g: sample.1,
            b: sample.2,
        };
    }

    /// Writes the render artifacts to the filesystem into the specified directory.
    pub fn develop(&self, directory: &Path) {
        use exr::prelude::*;

        let path = directory.join("output.exr");
        write_rgba_file(
            path,
            self.pixels.get_extent().x as usize,
            self.pixels.get_extent().y as usize,
            |x, y| {
                let p = self.pixels[y][x];
                (p.r, p.g, p.b, 1.0)
            },
        )
        .unwrap();
    }
}
