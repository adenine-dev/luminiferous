use std::path::Path;

use rayon::prelude::*;

use crate::{
    bsdfs::BsdfFlags,
    cameras::{CameraSample, CameraT},
    core::ProgressBar,
    film::TileProvider,
    lights::LightT,
    materials::MaterialT,
    maths::*,
    samplers::{Sampler, SamplerT},
    scene::Scene,
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
};

use super::IntegratorT;

pub struct PathIntegrator {
    sampler: Sampler,
    max_depth: u32,
}

impl PathIntegrator {
    pub fn new(sampler: Sampler, depth: u32) -> Self {
        Self {
            sampler,
            max_depth: depth,
        }
    }
}

impl IntegratorT for PathIntegrator {
    fn render(&self, scene: Scene) {
        let film = scene.camera.get_film();
        let extent = film.get_extent();

        let progress = ProgressBar::new(extent.x as u64 * extent.y as u64, "Rendering");

        let tile_size = 16;
        TileProvider::new(extent, tile_size)
            // .into_iter()
            .into_par_iter()
            .for_each(|bounds| {
                let tile = film.create_tile(bounds);

                let tile_extent = tile.get_extent();
                for ty in 0..tile_extent.y {
                    for tx in 0..tile_extent.x {
                        let x = tile.bounds.min.x + tx;
                        let y = tile.bounds.min.y + ty;

                        let mut pixel_sampler = self.sampler.fork((y * extent.x + x) as u64);
                        pixel_sampler.begin_pixel(UPoint2::new(x, y));
                        while pixel_sampler.advance() {
                            let offset = pixel_sampler.next_2d() - Vector2::splat(0.5);
                            let p = Point2::new(x as f32, y as f32) + offset;
                            let tp = Point2::new(tx as f32, ty as f32) + offset;

                            let mut ray = scene.camera.sample_ray(CameraSample {
                                p_film: p,
                                p_lens: pixel_sampler.next_2d(),
                            });
                            STATS.camera_rays_traced.inc();

                            let mut surface_reflectance = Spectrum::from_rgb(1.0, 1.0, 1.0);
                            let mut contributed = Spectrum::zero();

                            let mut i = 1;
                            let mut _num_tests = 0;
                            while i < self.max_depth {
                                let (interaction, _n) = scene.intersect(ray);
                                // _num_tests += _n;
                                // tile.apply_sample(
                                //     tp,
                                //     Spectrum::from_rgb(
                                //         _num_tests as f32,
                                //         _num_tests as f32,
                                //         _num_tests as f32,
                                //     ),
                                // );
                                // break;
                                if let Some(interaction) = interaction {
                                    // let n = 0.5 * (interaction.n + 1.0);
                                    // tile.apply_sample(tp, Spectrum::from_rgb(n.x, n.y, n.z));
                                    // break;

                                    let material =
                                        &scene.materials[interaction.primitive.material_index];
                                    let sample = material.sample(
                                        -ray.d,
                                        &interaction,
                                        pixel_sampler.next_2d(),
                                    );
                                    let l = sample.spectrum;
                                    if (material.bsdf_flags() & BsdfFlags::Smooth)
                                        == BsdfFlags::Smooth
                                    {
                                        for light in scene.lights.iter() {
                                            let emitted = light
                                                .sample_li(&interaction, pixel_sampler.next_2d());

                                            if scene.test_visibility(emitted.visibility) {
                                                let f =
                                                    material.eval(&interaction, emitted.wo, ray.d);
                                                contributed += surface_reflectance
                                                    * f
                                                    * emitted.li
                                                    * emitted.wo.dot(interaction.n).abs();
                                            }
                                        }
                                    }

                                    surface_reflectance *= l;

                                    if surface_reflectance.is_black() {
                                        break;
                                    }

                                    ray = interaction.spawn_ray(sample.wo);

                                    // contributed = Spectrum::from_rgb(sample.wo.x, sample.wo.y, sample.wo.z);
                                    // break;
                                } else {
                                    for light in scene.lights.iter() {
                                        if light.is_environment() {
                                            contributed += surface_reflectance * light.l_e(ray.d);
                                        }
                                    }

                                    break;
                                }
                                i += 1;
                            }

                            if contributed.is_black() {
                                STATS.zero_radiance_paths.inc();
                            }

                            tile.apply_sample(tp, contributed);

                            STATS.path_length.add(i as i64);
                        }

                        progress.advance(1);
                    }
                }

                film.apply_tile(tile);
            });

        let path = Path::new("output");
        film.develop(path);
        println!("Successfully wrote output to {path:?}");
    }
}
