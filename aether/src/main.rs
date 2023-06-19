use std::path::Path;

#[allow(unused_imports)]
use luminiferous::{
    aggregates::{Aggregate, AggregateT, Vector},
    bsdfs::{Bsdf, Lambertian},
    cameras::{Camera, CameraSample, CameraT, ProjectiveCamera},
    film::Film,
    lights::{Environment, Light, LightT, PointLight},
    materials::{DirectMaterial, Material, MaterialT},
    maths::{Extent2, Point3, Ray, UPoint2, UVector2, Vector2, Vector3},
    primitive::Primitive,
    rfilters::*,
    samplers::{RandomSampler, Sampler, SamplerT, StratifiedSampler},
    shapes::{Shape, Sphere},
    spectra::{Spectrum, SpectrumT},
    textures::{CheckerboardTexture, ConstantTexture, Texture, UvTexture},
};
use luminiferous::{
    integrators::{Integrator, IntegratorT, WhittedIntegrator},
    maths::Matrix3,
    scene::Scene,
    textures::TextureMapping,
};
use rayon::prelude::*;

const SPP: u32 = 16;
const MAX_BOUNCES: u32 = 10;

fn main() {
    println!("initializing");
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
                        ) * Matrix3::from_scale(Vector2::new(2.0f32.sqrt() / 2.0, 1.0 / 2.0) * 8.0),
                    ),
                )),
            )))),
        ),
        Primitive::new(
            Shape::Sphere(Sphere::new(Vector3::new(0.0, -50000.5, -1.0), 50000.0)),
            Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.5, 0.5, 0.5))),
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

    let sampler = Sampler::Stratified(StratifiedSampler::new(SPP, 0, true));
    // let sampler = Sampler::Random(RandomSampler::new(SPP, 0));

    let integrator = Integrator::Whitted(WhittedIntegrator::new(sampler));

    println!("rendering...");

    let scene = Scene::new(lights, world, camera);
    integrator.render(scene);

    // for y in 0..height {
    //     (0..width).into_par_iter().for_each(|x| {
    //         let mut pixel_sampler = sampler.fork((y * width + x) as u64);
    //         pixel_sampler.begin_pixel(UPoint2::new(x, y));
    //         while pixel_sampler.advance() {
    //             let p = Vector2::new(x as f32, y as f32)
    //                 + (pixel_sampler.next_2d() - Vector2::splat(0.5));
    //             // let l = pixel_sampler.next_1d();
    //             // film.apply_sample(p, Spectrum::from_rgb(l, l, l));

    //             // let l = pixel_sampler.next_2d();
    //             // film.apply_sample(p, Spectrum::from_rgb(l.x, l.y, 1.0));
    //             // continue;

    //             // let x = x as f32 + (rand::random::<f32>() - 0.5);
    //             // let y = y as f32 + (rand::random::<f32>() - 0.5);
    //             let mut ray = camera.sample_ray(CameraSample {
    //                 p_film: p,
    //                 p_lens: pixel_sampler.next_2d(),
    //             });

    //             let mut surface_reflectance = Spectrum::from_rgb(1.0, 1.0, 1.0);
    //             let mut contributed = Spectrum::zero();

    //             for _ in 0..MAX_BOUNCES {
    //                 if let Some(interaction) = world.intersect(ray) {
    //                     // let n = (interaction.n + 1.0) / 2.0;
    //                     // contributed = Spectrum::from_rgb(n.x, n.y, n.z);
    //                     let sample = interaction.primitive.material.sample(
    //                         ray.d,
    //                         &interaction,
    //                         pixel_sampler.next_2d(),
    //                     );
    //                     let L = sample.spectrum;

    //                     for light in lights.iter() {
    //                         let emitted = light.sample_li(&interaction, pixel_sampler.next_2d());

    //                         if test_visibility(emitted.visibility) {
    //                             let f = interaction.primitive.material.eval(
    //                                 &interaction,
    //                                 emitted.wi,
    //                                 ray.d,
    //                             );
    //                             contributed += surface_reflectance
    //                                 * f
    //                                 * emitted.li
    //                                 * emitted.wi.dot(interaction.n).abs();
    //                         }
    //                     }
    //                     surface_reflectance *= L;

    //                     if L.is_black() {
    //                         break;
    //                     }

    //                     ray.o = interaction.p;
    //                     ray.d = sample.wo;
    //                 } else {
    //                     for light in lights.iter() {
    //                         if light.is_environment() {
    //                             contributed += surface_reflectance * light.l_e(ray.d);
    //                         }
    //                     }

    //                     break;
    //                 }
    //             }

    //             film.apply_sample(p, contributed);
    //         }
    //     });
    //     println!("finished scanline {y}");
    // }

    // println!("writing output...");
    // film.develop(Path::new("output"));

    println!("successfully wrote output :>");
}
