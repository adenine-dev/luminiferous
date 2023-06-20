use std::path::Path;

use rayon::prelude::*;

use crate::{
    aggregates::AggregateT,
    cameras::{CameraSample, CameraT},
    core::ProgressBar,
    lights::LightT,
    materials::MaterialT,
    maths::*,
    samplers::{Sampler, SamplerT},
    scene::Scene,
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
};

use super::IntegratorT;

pub struct WhittedIntegrator {
    sampler: Sampler,
    max_depth: u32,
}

impl WhittedIntegrator {
    pub fn new(sampler: Sampler, depth: u32) -> Self {
        Self {
            sampler,
            max_depth: depth,
        }
    }
}

impl IntegratorT for WhittedIntegrator {
    fn render(&self, scene: Scene) {
        let film = scene.camera.get_film();
        let extent = film.get_extent();

        let progress = ProgressBar::new(extent.x as u64 * extent.y as u64, "Rendering");

        for y in 0..extent.y {
            (0..extent.x).into_par_iter().for_each(|x| {
                let mut pixel_sampler = self.sampler.fork((y * extent.x + x) as u64);
                pixel_sampler.begin_pixel(UPoint2::new(x, y));

                while pixel_sampler.advance() {
                    let p = Vector2::new(x as f32, y as f32)
                        + (pixel_sampler.next_2d() - Vector2::splat(0.5));

                    let mut ray = scene.camera.sample_ray(CameraSample {
                        p_film: p,
                        p_lens: pixel_sampler.next_2d(),
                    });
                    STATS.camera_rays_traced.inc();

                    let mut surface_reflectance = Spectrum::from_rgb(1.0, 1.0, 1.0);
                    let mut contributed = Spectrum::zero();

                    let mut i = 1;
                    while i < self.max_depth {
                        if let Some(interaction) = scene.intersect(ray) {
                            let sample = interaction.primitive.material.sample(
                                ray.d,
                                &interaction,
                                pixel_sampler.next_2d(),
                            );
                            let L = sample.spectrum;

                            for light in scene.lights.iter() {
                                let emitted =
                                    light.sample_li(&interaction, pixel_sampler.next_2d());

                                if scene.test_visibility(emitted.visibility) {
                                    let f = interaction.primitive.material.eval(
                                        &interaction,
                                        emitted.wi,
                                        ray.d,
                                    );
                                    contributed += surface_reflectance
                                        * f
                                        * emitted.li
                                        * emitted.wi.dot(interaction.n).abs();
                                }
                            }
                            surface_reflectance *= L;

                            if surface_reflectance.is_black() {
                                break;
                            }

                            ray.o = interaction.p;
                            ray.d = sample.wo;
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
                    film.apply_sample(p, contributed);

                    STATS.path_length.add(i as i64);
                }

                progress.advance(1);
            });
        }

        let path = Path::new("output");
        film.develop(path);
        println!("Successfully wrote output to {path:?}");
    }
}
