use std::path::Path;

use crate::{
    core::Array2d,
    maths::{Color, Point2, UBounds2, UExtent2, UVector2, Vector2},
    rfilters::{RFilter, RFilterT},
};

#[derive(Debug, Clone, Copy, Default)]
struct Pixel {
    pub filter_weight_sum: f32,
    pub contribution_sum: Color,
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

    pub fn apply_sample(&mut self, p: Point2, sample: Color) {
        let bounds = self.get_sample_bounds(p);

        for y in bounds.min.y..bounds.max.y {
            for x in bounds.min.x..bounds.max.x {
                let p1 = Vector2::new(x as f32, y as f32) + Vector2::splat(0.5) - p;
                let weight = self.filter.eval(p1);

                if weight >= 0.0 {
                    self.pixels[y as usize][x as usize].filter_weight_sum += weight;
                    self.pixels[y as usize][x as usize].contribution_sum.r += sample.r * weight;
                    self.pixels[y as usize][x as usize].contribution_sum.g += sample.g * weight;
                    self.pixels[y as usize][x as usize].contribution_sum.b += sample.b * weight;
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
                    p.contribution_sum.r * (1.0 / p.filter_weight_sum),
                    p.contribution_sum.g * (1.0 / p.filter_weight_sum),
                    p.contribution_sum.b * (1.0 / p.filter_weight_sum),
                )
            },
        )
        .unwrap();
    }
}
