use core::panic;
use std::{
    mem::size_of,
    path::Path,
    sync::{atomic::Ordering, Mutex},
};

use atomic_float::AtomicF32;

use crate::{
    core::{Array2d, TevReporter},
    maths::{Bounds3, Extent2, Point2, UBounds2, UExtent2, UPoint2, UVector2, Vector2},
    rfilters::{RFilter, RFilterT},
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
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

    tev_reporter: Mutex<TevReporter>,
}

impl Film {
    pub fn new(extent: UExtent2, filter: impl Into<RFilter>) -> Self {
        STATS
            .film_memory
            .add(extent.x as u64 * extent.y as u64 * size_of::<Pixel>() as u64);

        let mut tev_reporter = TevReporter::new(false);

        tev_reporter.create_image(extent);

        Self {
            pixels: Array2d::with_default(extent, Pixel::default()),
            filter: filter.into(),
            tev_reporter: Mutex::new(tev_reporter),
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

    pub fn create_tile(&self, bounds: UBounds2) -> FilmTile {
        FilmTile::new(bounds, self.filter.clone())
    }

    pub fn apply_tile(&self, tile: FilmTile) {
        let bounds = tile.border_bounds;
        // bounds.min += tile.border_size;
        // bounds.max -= tile.border_size;

        for i in bounds.min.y..bounds.max.y {
            for j in bounds.min.x..bounds.max.x {
                let tx = j - bounds.min.x;
                let ty = i - bounds.min.y;

                let p = &tile.pixels[ty as usize][tx as usize];
                let weight = p.filter_weight_sum.load(Ordering::Acquire);
                let x = p.contribution_sum_xyz[0].load(Ordering::Acquire);
                let y = p.contribution_sum_xyz[1].load(Ordering::Acquire);
                let z = p.contribution_sum_xyz[2].load(Ordering::Acquire);

                let i = i
                    .saturating_sub(tile.border_size.y)
                    .min(self.get_extent().y - 1);
                let j = j
                    .saturating_sub(tile.border_size.x)
                    .min(self.get_extent().x - 1);

                self.pixels[i as usize][j as usize]
                    .filter_weight_sum
                    .fetch_add(weight, Ordering::Release);
                self.pixels[i as usize][j as usize].contribution_sum_xyz[0]
                    .fetch_add(x, Ordering::Release);
                self.pixels[i as usize][j as usize].contribution_sum_xyz[1]
                    .fetch_add(y, Ordering::Release);
                self.pixels[i as usize][j as usize].contribution_sum_xyz[2]
                    .fetch_add(z, Ordering::Release);
            }
        }
    }

    pub fn update(&self, p: Point2) {
        // if self.tev_reporter.lock().is_ok_and(|r| r.is_connected()) {
        //     let bounds = self.get_sample_bounds(p);
        //     let mut pixels = Vec::with_capacity(
        //         ((bounds.max.y - bounds.min.y) * (bounds.max.x - bounds.min.x)) as usize * 3,
        //     );

        //     for y in bounds.min.y..bounds.max.y {
        //         for x in bounds.min.x..bounds.max.x {
        //             let y = y as usize;
        //             let x = x as usize;
        //             let p = &self.pixels[y][x];
        //             let filter_weight_sum = p.filter_weight_sum.load(Ordering::Acquire);
        //             pixels.push(
        //                 p.contribution_sum_xyz[0].load(Ordering::Acquire)
        //                     * (1.0 / filter_weight_sum),
        //             );
        //             pixels.push(
        //                 p.contribution_sum_xyz[1].load(Ordering::Acquire)
        //                     * (1.0 / filter_weight_sum),
        //             );
        //             pixels.push(
        //                 p.contribution_sum_xyz[2].load(Ordering::Acquire)
        //                     * (1.0 / filter_weight_sum),
        //             );
        //         }
        //     }

        //     if let Ok(mut r) = self.tev_reporter.lock() {
        //         r.update_pixels(bounds, &pixels);
        //     }
        // }
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

pub struct FilmTile {
    pixels: Array2d<Pixel>,
    pub bounds: UBounds2,
    pub border_bounds: UBounds2,
    pub border_size: UVector2,
    pub filter: RFilter,
}

impl FilmTile {
    pub fn new(bounds: UBounds2, filter: RFilter) -> Self {
        let border_size = (filter.get_radius() - 0.5).ceil().as_uvec2();
        let border_bounds = UBounds2::new(bounds.min, bounds.max + border_size * 2);
        FilmTile {
            pixels: Array2d::with_default(border_bounds.extent(), Pixel::default()),
            bounds,
            border_bounds,
            border_size,
            filter,
        }
    }

    pub fn get_extent(&self) -> UExtent2 {
        self.bounds.extent()
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
        // panic!("{p:?}");
        let p = p + self.border_size.as_vec2();
        let bounds = self.get_sample_bounds(p);

        for i in bounds.min.y..bounds.max.y {
            for j in bounds.min.x..bounds.max.x {
                let i = (i).min(self.pixels.get_extent().y - 1);
                let j = (j).min(self.pixels.get_extent().x - 1);
                let p1 = Vector2::new(j as f32, i as f32) + Vector2::splat(0.5) - p;
                let weight = self.filter.eval(p1);

                if weight >= 0.0 {
                    let [x, y, z] = sample.to_rgb();

                    // All of these are relaxed because they aren't read until the end.
                    self.pixels[i as usize][j as usize]
                        .filter_weight_sum
                        .fetch_add(weight, Ordering::Release);
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[0]
                        .fetch_add(x * weight, Ordering::Release);
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[1]
                        .fetch_add(y * weight, Ordering::Release);
                    self.pixels[i as usize][j as usize].contribution_sum_xyz[2]
                        .fetch_add(z * weight, Ordering::Release);
                }
            }
        }
    }
}
