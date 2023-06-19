use rayon::prelude::*;

use crate::{
    aggregates::AggregateT,
    cameras::{CameraSample, CameraT},
    lights::LightT,
    materials::MaterialT,
    maths::*,
    samplers::{Sampler, SamplerT},
    scene::Scene,
    spectra::{Spectrum, SpectrumT},
};

use super::IntegratorT;

const SPP: u32 = 16;
const MAX_BOUNCES: u32 = 10;

pub struct WhittedIntegrator {
    sampler: Sampler,
}

impl WhittedIntegrator {
    pub fn new(sampler: Sampler) -> Self {
        Self { sampler }
    }
}

impl IntegratorT for WhittedIntegrator {
    fn render(&self, scene: Scene) {
        let film = scene.camera.get_film();
        let extent = film.get_extent();

        for y in 0..extent.y {
            (0..extent.x).into_par_iter().for_each(|x| {
                let mut pixel_sampler = self.sampler.fork((y * extent.x + x) as u64);
                pixel_sampler.begin_pixel(UPoint2::new(x, y));
                while pixel_sampler.advance() {
                    let p = Vector2::new(x as f32, y as f32)
                        + (pixel_sampler.next_2d() - Vector2::splat(0.5));
                    // let l = pixel_sampler.next_1d();
                    // film.apply_sample(p, Spectrum::from_rgb(l, l, l));

                    // let l = pixel_sampler.next_2d();
                    // film.apply_sample(p, Spectrum::from_rgb(l.x, l.y, 1.0));
                    // continue;

                    // let x = x as f32 + (rand::random::<f32>() - 0.5);
                    // let y = y as f32 + (rand::random::<f32>() - 0.5);
                    let mut ray = scene.camera.sample_ray(CameraSample {
                        p_film: p,
                        p_lens: pixel_sampler.next_2d(),
                    });

                    let mut surface_reflectance = Spectrum::from_rgb(1.0, 1.0, 1.0);
                    let mut contributed = Spectrum::zero();

                    for _ in 0..MAX_BOUNCES {
                        if let Some(interaction) = scene.aggregate.intersect(ray) {
                            // let n = (interaction.n + 1.0) / 2.0;
                            // contributed = Spectrum::from_rgb(n.x, n.y, n.z);
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

                            if L.is_black() {
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
                    }

                    film.apply_sample(p, contributed);
                }
            });
            println!("finished scanline {y}");
        }
    }
}
