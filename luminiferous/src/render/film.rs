use std::path::Path;

use crate::{
    core::Array2d,
    maths::{Point2, UBounds2, UExtent2, UVector2, Vector2},
    rfilters::{RFilter, RFilterT},
    spectra::{Spectrum, SpectrumT},
};

#[derive(Debug, Clone, Copy, Default)]
struct Pixel {
    pub filter_weight_sum: f32,
    pub contribution_sum_xyz: [f32; 3],
}

pub struct Film {
    pixels: Array2d<Pixel>,
    filter: RFilter,
}

impl Film {
    pub fn new(extent: UExtent2, filter: impl Into<RFilter>) -> Self {
        Self {
            pixels: Array2d::with_default(extent, Pixel::default()),
            filter: filter.into(),
        }
    }

    fn get_sample_bounds(&self, p: Point2) -> UBounds2 {
        let min = ((p - self.filter.get_radius() + Vector2::splat(0.5))
            .floor()
            .as_uvec2())
        .max(UVector2::ZERO);

        let max = ((p + self.filter.get_radius() - Vector2::splat(0.5))
            .ceil()
            .as_uvec2())
        .min(self.pixels.get_extent());

        UBounds2::new(min, max)
    }

    pub fn apply_sample(&mut self, p: Point2, sample: Spectrum) {
        let bounds = self.get_sample_bounds(p);

        for i in bounds.min.y..bounds.max.y {
            for j in bounds.min.x..bounds.max.x {
                let p1 = Vector2::new(j as f32, i as f32) + Vector2::splat(0.5) - p;
                let weight = self.filter.eval(p1);

                if weight >= 0.0 {
                    let [x, y, z] = sample.to_rgb();

                    self.pixels[i as usize][j as usize].filter_weight_sum += weight;
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[0] += x * weight;
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[1] += y * weight;
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[2] += z * weight;
                }
            }
        }
    }

    /// Writes the render artifacts to the filesystem into the specified directory.
    pub fn develop(&self, directory: &Path) {
        use exr::prelude::*;

        let path = directory.join("output.exr");

        write_rgb_file(
            path,
            self.pixels.get_extent().x as usize,
            self.pixels.get_extent().y as usize,
            |x, y| {
                let p = self.pixels[y][x];
                (
                    p.contribution_sum_xyz[0] * (1.0 / p.filter_weight_sum),
                    p.contribution_sum_xyz[1] * (1.0 / p.filter_weight_sum),
                    p.contribution_sum_xyz[2] * (1.0 / p.filter_weight_sum),
                )
            },
        )
        .unwrap();
    }
}
