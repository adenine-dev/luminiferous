use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{
    aggregates::{Aggregate, Vector},
    bsdfs::{Bsdf, Lambertian},
    cameras::{Camera, ProjectiveCamera},
    film::Film,
    integrators::{Integrator, IntegratorT, WhittedIntegrator},
    lights::{Light, PointLight},
    materials::{DirectMaterial, Material},
    maths::{Matrix3, Point3, UVector2, Vector2, Vector3},
    primitive::Primitive,
    rfilters::TentFilter,
    samplers::{Sampler, StratifiedSampler},
    scene::Scene,
    shapes::{Shape, Sphere},
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
    textures::{CheckerboardTexture, ConstantTexture, Texture, TextureMapping},
};

pub struct Context {
    scene: Scene,
    integrator: Integrator,
}

pub struct ContextParams {
    pub seed: u64,
    pub spp: u32,
}

impl Context {
    //TODO: load scene from file
    pub fn new(params: ContextParams) -> Self {
        let start = Instant::now();

        let ctx = match 0 {
            0 => {
                // let width = 3840;
                // let height = 2160;
                // let width = 512;
                // let height = 384;
                let width = 400;
                let height = 225;
                // let width = 100;
                // let height = 62;

                let film = Film::new(
                    UVector2::new(width, height),
                    TentFilter::new(Vector2::splat(1.0)),
                );

                let camera = Camera::Projective(ProjectiveCamera::new_perspective(
                    film,
                    core::f32::consts::FRAC_PI_2,
                    0.0,
                    0.0,
                ));

                let world = Aggregate::Vector(Vector::new(vec![
                    Primitive::new(
                        Shape::Sphere(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)),
                        Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                            // Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.7, 0.2, 0.8))),
                            // Texture::Uv(UvTexture::new()),
                            Texture::Checkerboard(CheckerboardTexture::new(
                                Spectrum::from_rgb(0.8, 0.8, 0.8),
                                Spectrum::from_rgb(0.1, 0.1, 0.1),
                                TextureMapping::new(
                                    Matrix3::from_axis_angle(
                                        Vector3::new(1.0, 1.0, 1.0),
                                        core::f32::consts::FRAC_PI_4,
                                    ) * Matrix3::from_scale(
                                        Vector2::new(2.0f32.sqrt() / 2.0, 1.0 / 2.0) * 32.0,
                                    ),
                                ),
                            )),
                        )))),
                    ),
                    Primitive::new(
                        Shape::Sphere(Sphere::new(Vector3::new(0.0, -50000.5, -1.0), 50000.0)),
                        Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                            Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                                0.5, 0.5, 0.5,
                            ))),
                            // Texture::Checkerboard(CheckerboardTexture::new(
                            //     Spectrum::from_rgb(0.5, 0.5, 0.5),
                            //     Spectrum::from_rgb(0.1, 0.1, 0.1),
                            // )),
                        )))),
                    ),
                ]));

                let lights = vec![
                    // Emitter::Environment(Environment::new(Spectrum::zero())),
                    Light::Point(PointLight::new(
                        Point3::new(1.0, 0.0, 1.0),
                        Spectrum::from_rgb(1.0, 1.0, 10.0),
                    )),
                    Light::Point(PointLight::new(
                        Point3::new(-1.0, 0.0, 1.0),
                        Spectrum::from_rgb(10.0, 1.0, 1.0),
                    )),
                    // Emitter::Point(PointLight::new(
                    //     Point3::new(0.0, 100.0, -1.0),
                    //     Spectrum::from_rgb(5.0, 5.0, 5.0),
                    // )),
                    // Emitter::Environment(Environment::new(Spectrum::from_rgb(1.0, 1.0, 1.0))),
                ];

                let sampler =
                    Sampler::Stratified(StratifiedSampler::new(params.spp, params.seed, true));

                const MAX_BOUNCES: u32 = 10;
                let integrator = Integrator::Whitted(WhittedIntegrator::new(sampler, MAX_BOUNCES));

                let scene = Scene::new(lights, world, camera);

                Self { scene, integrator }
            }
            _ => {
                panic!("bad scene mode")
            }
        };

        let duration = start.elapsed();
        STATS.init_duration.add(duration.as_millis() as u64);

        ctx
    }

    pub fn run(self) {
        let start = Instant::now();
        self.integrator.render(self.scene);
        let duration = start.elapsed();
        STATS.render_duration.add(duration.as_millis() as u64);

        STATS.print();
    }
}
