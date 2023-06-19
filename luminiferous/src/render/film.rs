use std::{
    path::Path,
    sync::atomic::{AtomicU32, Ordering},
};

use atomic_float::AtomicF32;

use crate::{
    core::Array2d,
    maths::{Extent2, Point2, UBounds2, UExtent2, UVector2, Vector2},
    rfilters::{RFilter, RFilterT},
    spectra::{Spectrum, SpectrumT},
};

#[derive(Debug, Default)]
struct Pixel {
    pub filter_weight_sum: AtomicF32,
    pub contribution_sum_xyz: [AtomicF32; 3],
}

impl Clone for Pixel {
    fn clone(&self) -> Self {
        Pixel {
            filter_weight_sum: AtomicF32::new(self.filter_weight_sum.load(Ordering::SeqCst)),
            contribution_sum_xyz: [
                AtomicF32::new(self.contribution_sum_xyz[0].load(Ordering::SeqCst)),
                AtomicF32::new(self.contribution_sum_xyz[1].load(Ordering::SeqCst)),
                AtomicF32::new(self.contribution_sum_xyz[2].load(Ordering::SeqCst)),
            ],
        }
    }
}

#[derive(Debug)]
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

    pub fn get_extent(&self) -> UExtent2 {
        self.pixels.get_extent()
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

    pub fn apply_sample(&self, p: Point2, sample: Spectrum) {
        let bounds = self.get_sample_bounds(p);

        for i in bounds.min.y..bounds.max.y {
            for j in bounds.min.x..bounds.max.x {
                let p1 = Vector2::new(j as f32, i as f32) + Vector2::splat(0.5) - p;
                let weight = self.filter.eval(p1);

                if weight >= 0.0 {
                    let [x, y, z] = sample.to_rgb();

                    // All of these are relaxed because they aren't read until the end.
                    self.pixels[i as usize][j as usize]
                        .filter_weight_sum
                        .fetch_add(weight, Ordering::Relaxed);
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[0]
                        .fetch_add(x * weight, Ordering::Relaxed);
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[1]
                        .fetch_add(y * weight, Ordering::Relaxed);
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[2]
                        .fetch_add(z * weight, Ordering::Relaxed);
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
                let p = &self.pixels[y][x];
                let filter_weight_sum = p.filter_weight_sum.load(Ordering::Acquire);
                (
                    p.contribution_sum_xyz[0].load(Ordering::Acquire) * (1.0 / filter_weight_sum),
                    p.contribution_sum_xyz[1].load(Ordering::Acquire) * (1.0 / filter_weight_sum),
                    p.contribution_sum_xyz[2].load(Ordering::Acquire) * (1.0 / filter_weight_sum),
                )
            },
        )
        .unwrap();
    }
}
