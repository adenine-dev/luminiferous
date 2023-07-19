use std::path::Path;

use rayon::prelude::*;

use crate::prelude::*;
use crate::{
    bsdfs::BsdfFlags,
    cameras::{CameraSample, CameraT},
    core::ProgressBar,
    film::TileProvider,
    lights::LightT,
    materials::MaterialT,
    media::MediumT,
    phase_functions::PhaseFunctionT,
    samplers::{Sampler, SamplerT},
    scene::Scene,
    spectra::{Spectrum, SpectrumT},
};

use super::IntegratorT;

pub struct VolPathIntegrator {
    sampler: Sampler,
    max_depth: u32,
}

impl VolPathIntegrator {
    pub fn new(sampler: Sampler, depth: u32) -> Self {
        Self {
            sampler,
            max_depth: depth,
        }
    }
}

impl IntegratorT for VolPathIntegrator {
    fn render(&self, scene: Scene) {
        let film = scene.camera.get_film();
        let extent = film.get_extent();

        let progress = ProgressBar::new(extent.x as u64 * extent.y as u64, "Rendering");

        let tile_size = 16;
        TileProvider::new(extent, tile_size)
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

                            let mut medium = scene.camera.medium();

                            let mut depth = 1;
                            let mut _num_tests = 0;
                            while depth < self.max_depth {
                                let (interaction, _n) = scene.intersect(ray);
                                _num_tests += _n;

                                let mi = if let Some(medium) = &medium {
                                    medium.sample(
                                        ray,
                                        interaction.as_ref().map(|i| i.t).unwrap_or(1e7),
                                        pixel_sampler.next_1d(),
                                    )
                                } else {
                                    None
                                };

                                if let Some((mi, l)) = mi {
                                    if let Some(pf) = &mi.phase_function {
                                        for light in scene.lights.iter() {
                                            let emitted =
                                                light.sample(mi.p, mi.wi, pixel_sampler.next_2d());

                                            if scene.test_visibility(emitted.visibility) {
                                                let f = pf.eval(&mi, emitted.wo, ray.d);
                                                contributed += surface_reflectance
                                                    * f
                                                    * emitted.li
                                                    * emitted.wo.dot(mi.wi).abs();
                                            }
                                        }
                                        surface_reflectance *= l;
                                        let sample = pf.sample(pixel_sampler.next_2d());
                                        ray = Ray::new(mi.p, sample.wo);
                                    }
                                } else if let Some(interaction) = interaction {
                                    let material =
                                        &scene.materials[interaction.primitive.material_index];
                                    let sample = material.sample(
                                        -ray.d,
                                        &interaction,
                                        pixel_sampler.next_2d(),
                                    );
                                    let l = sample.spectrum;
                                    if (sample.sampled & BsdfFlags::Smooth) == BsdfFlags::Smooth {
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

                                    if sample.sampled != BsdfFlags::Null {
                                        ray = interaction.spawn_ray(sample.wo);
                                    } else {
                                        depth -= 1;
                                        ray = Ray::new(interaction.p + ray.d * 1e-5, ray.d);
                                    }
                                    medium = interaction.target_medium(sample.wo);

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
                                depth += 1;
                            }

                            if contributed.is_black() {
                                STATS.zero_radiance_paths.inc();
                            }

                            tile.apply_sample(tp, contributed);

                            STATS.path_length.add(depth as i64);
                        }
                        progress.advance(1);
                    }
                }

                film.apply_tile(tile);
            });

        let path = Path::new("output");
        film.develop(path);
        infoln!("Successfully wrote output to {path:?}");
    }
}
