use std::path::Path;

use rayon::prelude::*;

use crate::lights::Light;
use crate::media::MediumInteraction;
use crate::prelude::*;
use crate::primitive::SurfaceInteraction;
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

pub struct PathIntegrator {
    sampler: Sampler,
    max_depth: u32,
    volumetric: bool,
}

impl PathIntegrator {
    pub fn new(sampler: Sampler, depth: u32, volumetric: bool) -> Self {
        Self {
            sampler,
            max_depth: depth,
            volumetric,
        }
    }

    fn sample_light_from_surface(
        &self,
        scene: &Scene,
        light: &Light,
        ray: Ray,
        si: &SurfaceInteraction,
        sampler: &mut Sampler,
    ) -> Spectrum {
        let emitted = light.sample_li(&si.as_interaction(), sampler.next_2d());

        if !scene.unoccluded(emitted.visibility) {
            return Spectrum::zero();
        }
        let f = scene.materials[si.primitive.material_index].eval(&si, emitted.wo, ray.d);

        f * emitted.li * emitted.wo.dot(si.n).abs()
    }

    fn sample_light_from_medium(
        &self,
        scene: &Scene,
        light: &Light,
        ray: Ray,
        mi: &MediumInteraction,
        sampler: &mut Sampler,
    ) -> Spectrum {
        if let Some(pf) = &mi.phase_function && mi.medium.is_some() {
            let emitted = light.sample_li(&mi.as_interaction(), sampler.next_2d());
            let mut transmittance = {
                let mut medium = mi.medium.unwrap();
                let mut ray = emitted.visibility.ray;
                let mut transmittance = Spectrum::splat(1.0);
                
                while let (Some(intersection), _) = scene.intersect(ray) {
                    if scene.materials[intersection.primitive.material_index].bsdf_flags() != BsdfFlags::Null
                    {
                        return Spectrum::zero();
                    }
        
                    let mi = medium.sample(ray, intersection.t, sampler.next_1d());
                    if let Some((mi, tr)) = mi {
                        if mi.valid() {
                            transmittance *= tr;
                            medium = mi.medium.unwrap();
                        }
                    }

                    ray = intersection.as_interaction().spawn_ray_to(emitted.visibility.end);
                }

                let mi = medium.sample(ray, f32::MAX, sampler.next_1d());
                if let Some((mi, tr)) = mi {
                    if mi.valid() {
                        transmittance *= tr;
                    }
                }
        
                transmittance        
            };
            
            if !transmittance.is_black() {
                let f = pf.eval(mi, emitted.wo, ray.d);
                transmittance *= f * emitted.wo.dot(mi.wi).abs();
            }

            return transmittance * emitted.li;
        }

        Spectrum::zero()
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
                            let mut medium = scene.camera.medium();

                            let mut depth = 1;
                            let mut _num_tests = 0;

                            'outer: while depth < self.max_depth {
                                let (interaction, n) = scene.intersect(ray);
                                _num_tests += n;

                                let mi = if let Some(medium) = &medium && self.volumetric  {
                                    medium.sample(
                                        ray,
                                        interaction.as_ref().map(|i| i.t).unwrap_or(1e7),
                                        pixel_sampler.next_1d(),
                                    )
                                } else {
                                    None
                                };

                                if let Some((mi, l)) = &mi {
                                    if mi.phase_function.is_some() {
                                        surface_reflectance *= *l;
                                        for light in scene.lights.iter() {
                                            contributed += surface_reflectance
                                                * self.sample_light_from_medium(
                                                    &scene,
                                                    light,
                                                    ray,
                                                    mi,
                                                    &mut pixel_sampler,
                                                );
                                        }
                                        let sample = mi
                                            .phase_function
                                            .as_ref()
                                            .unwrap()
                                            .sample(mi, pixel_sampler.next_2d());
                                        ray = mi.as_interaction().spawn_ray(sample.wo);
                                    }
                                } else if let Some(si) = interaction {
                                    // let n = si.n;
                                    // contributed = Spectrum::from_rgb(n.x, n.y, n.z);
                                    // break 'outer;

                                    if let Some(area_light_index) = si.primitive.area_light_index {
                                        let l = scene.lights[area_light_index].l_e(-ray.d);

                                        contributed += surface_reflectance * l;
                                        break;
                                    }

                                    let material = &scene.materials[si.primitive.material_index];
                                    let sample =
                                        material.sample(-ray.d, &si, pixel_sampler.next_1d(), pixel_sampler.next_2d());

                                    let l = sample.spectrum;
                                    if l.has_nan() {
                                        break;
                                    }
                                    if sample.sampled.contains(BsdfFlags::Smooth) {
                                        for light in scene.lights.iter() {
                                            contributed += surface_reflectance
                                                * self.sample_light_from_surface(
                                                    &scene,
                                                    light,
                                                    ray,
                                                    &si,
                                                    &mut pixel_sampler,
                                                );
                                        }
                                    }
                                    surface_reflectance *= l;

                                    if surface_reflectance.is_black() {
                                        break;
                                    }

                                    if sample.sampled != BsdfFlags::Null {
                                        ray = si.as_interaction().spawn_ray(sample.wo);
                                    } else {
                                        depth -= 1;
                                        ray = Ray::new(si.p + ray.d * 1e-5, ray.d);
                                    }
                                    medium = si.target_medium(sample.wo);

                                    // ray = interaction.spawn_ray(sample.wo);
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

                            STATS.path_length.add(depth as i64);

                            if contributed.is_black() {
                                STATS.zero_radiance_paths.inc();
                            }

                            tile.apply_sample(tp, contributed);
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
