use std::{
    mem::size_of,
    ops::Range,
    path::Path,
    sync::{atomic::Ordering, Mutex},
};

use atomic_float::AtomicF32;
use rayon::{iter::plumbing::*, prelude::*};

use crate::{
    core::{Array2d, TevReporter},
    maths::{Point2, UBounds2, UExtent2, UPoint2, UVector2, Vector2},
    rfilters::{RFilter, RFilterT},
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
};

#[derive(Debug, Default)]
struct Pixel {
    pub filter_weight_sum: AtomicF32,
    pub contribution_sum_xyz: [AtomicF32; 3],
}

impl Pixel {
    fn xyz(&self) -> (f32, f32, f32) {
        let filter_weight_sum = self.filter_weight_sum.load(Ordering::Acquire);
        (
            self.contribution_sum_xyz[0].load(Ordering::Acquire) * (1.0 / filter_weight_sum),
            self.contribution_sum_xyz[1].load(Ordering::Acquire) * (1.0 / filter_weight_sum),
            self.contribution_sum_xyz[2].load(Ordering::Acquire) * (1.0 / filter_weight_sum),
        )
    }
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

        let tev_reporter = TevReporter::new(false, extent);

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

        self.report_tile(tile.bounds);
    }

    pub fn report_tile(&self, bounds: UBounds2) {
        if self.tev_reporter.lock().is_ok_and(|r| r.is_connected()) {
            let mut pixels = Vec::with_capacity(
                ((bounds.max.y - bounds.min.y) * (bounds.max.x - bounds.min.x)) as usize * 3,
            );

            for y in bounds.min.y..bounds.max.y {
                for x in bounds.min.x..bounds.max.x {
                    let (x, y, z) = self.pixels[y as usize][x as usize].xyz();
                    pixels.push(x);
                    pixels.push(y);
                    pixels.push(z);
                }
            }

            if let Ok(mut r) = self.tev_reporter.lock() {
                r.update_pixels(bounds, pixels, false);
            }
        }
    }

    /// Writes the render artifacts to the filesystem into the specified directory.
    pub fn develop(&self, directory: &Path) {
        use exr::prelude::*;

        let path = directory.join("output.exr");

        if self.tev_reporter.lock().is_ok_and(|r| r.is_connected()) {
            if let Ok(mut r) = self.tev_reporter.lock() {
                r.update_pixels(
                    UBounds2::new(UPoint2::splat(0), UPoint2::splat(0)),
                    vec![],
                    true,
                );
            }
        }

        write_rgb_file(
            path,
            self.pixels.get_extent().x as usize,
            self.pixels.get_extent().y as usize,
            |x, y| {
                let p = &self.pixels[y][x];
                p.xyz()
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

pub struct TileProvider {
    range: Range<u32>,
    tile_size: UExtent2,
    extent: UExtent2,
}

impl TileProvider {
    pub fn new(extent: UExtent2, tile_size: u32) -> Self {
        let tile_size = UExtent2::splat(tile_size);

        let num_tiles = (extent.as_vec2() / tile_size.as_vec2()).ceil().as_uvec2();
        Self {
            range: 0..((num_tiles.x.max(num_tiles.y) + 1).pow(2)),

            tile_size,
            extent,
        }
    }

    #[inline]
    fn map_n(n: u32, tile_size: UExtent2, extent: UExtent2) -> Option<UBounds2> {
        // adapted from https://oeis.org/A174344
        let k = |n: f32| core::f32::consts::FRAC_PI_2 * ((4.0 * n - 3.0).sqrt().floor());

        let p = |x| {
            (
                (1..=x).map(|n: u32| k(n as f32).cos()).sum::<f32>(),
                (1..=x).map(|n| k(n as f32).sin()).sum::<f32>() - 0.5,
            )
        };
        let (x, y) = p(n);

        let min = (Point2::new(x, y) * tile_size.as_vec2() + extent.as_vec2() / 2.0)
            .round()
            .clamp(Point2::ZERO, extent.as_vec2())
            .as_uvec2();

        // for typical scanlines
        // let x = (n * tile_size.x) % extent.x;
        // let y = (n * tile_size.x) / extent.x * tile_size.y;

        // let min = UPoint2::new(x, y);
        // let max = UPoint2::new(x + tile_size.x, y + tile_size.y).min(extent);
        let max = (min + tile_size).min(extent);

        let ret = UBounds2::new(min, max);

        if ret.area() == 0 {
            None
        } else {
            Some(ret)
        }
    }
}

impl IntoIterator for TileProvider {
    type IntoIter = TileProviderIntoIterator;
    type Item = UBounds2;

    fn into_iter(self) -> Self::IntoIter {
        TileProviderIntoIterator { provider: self }
    }
}

pub struct TileProviderIntoIterator {
    provider: TileProvider,
}

impl Iterator for TileProviderIntoIterator {
    type Item = UBounds2;

    fn next(&mut self) -> Option<Self::Item> {
        self.provider
            .range
            .find_map(|n| TileProvider::map_n(n, self.provider.tile_size, self.provider.extent))
    }
}

impl ParallelIterator for TileProvider {
    type Item = UBounds2;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.range
            // .par_bridge()
            .into_par_iter()
            .filter_map(|n| Self::map_n(n, self.tile_size, self.extent))
            .drive_unindexed(consumer)
    }
}
