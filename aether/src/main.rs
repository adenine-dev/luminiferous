use std::path::Path;

use luminiferous::{
    aggregates::{Aggregate, AggregateT, Vector},
    bsdfs::{Bsdf, Lambertian},
    emitters::{Emitter, EmitterT, Environment, PointLight},
    film::Film,
    materials::{DirectMaterial, Material, MaterialT},
    maths::{Extent2, Point3, Ray, UVector2, Vector2, Vector3},
    primitive::Primitive,
    rfilters::*,
    sensors::{ProjectiveCamera, Sensor, SensorT},
    shapes::{Shape, Sphere},
    spectra::{Spectrum, SpectrumT},
};

const SPP: u32 = 30;
const MAX_BOUNCES: u32 = 20;

fn main() {
    println!("initializing");
    // let width = 3840;
    // let height = 2160;
    let width = 512;
    let height = 384;
    // let width = 400;
    // let height = 225;
    // let width = 100;
    // let height = 62;

    let mut film = Film::new(
        UVector2::new(width, height),
        TentFilter::new(Vector2::splat(1.0)),
    );

    let camera = Sensor::ProjectiveCamera(ProjectiveCamera::new_perspective(
        Extent2::new(width as f32, height as f32),
        core::f32::consts::FRAC_PI_2,
    ));

    let world = Aggregate::Vector(Vector::new(vec![
        Primitive::new(
            Shape::Sphere(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)),
            Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                Spectrum::new(0.5, 0.5, 0.5),
            )))),
        ),
        Primitive::new(
            Shape::Sphere(Sphere::new(Vector3::new(0.0, -100.5, -1.0), 100.0)),
            Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                Spectrum::new(0.5, 0.5, 0.5),
            )))),
        ),
    ]));

    let lights = vec![
        // Emitter::Environment(Environment::new(Spectrum::zero())),
        Emitter::Point(PointLight::new(
            Point3::new(1.0, 0.0, 1.0),
            Spectrum::new(1.0, 1.0, 10.0),
        )),
        Emitter::Point(PointLight::new(
            Point3::new(-1.0, 0.0, 1.0),
            Spectrum::new(10.0, 1.0, 1.0),
        )),
        // Emitter::Environment(Environment::new(Spectrum::new(0.5, 0.7, 1.0))),
    ];

    println!("rendering...");

    let test_visibility = |visibility: luminiferous::emitters::Visibility| {
        if let Some(intersection) = world.intersect(visibility.ray) {
            if intersection.t < visibility.end.distance(visibility.ray.o) {
                return false;
            }
        }
        true
    };

    fn li(ray: Ray, depth: u32, world: &Aggregate, lights: &Vec<Emitter>) -> Spectrum {
        fn test_visibility(
            visibility: luminiferous::emitters::Visibility,
            world: &Aggregate,
        ) -> bool {
            if let Some(intersection) = world.intersect(visibility.ray) {
                if intersection.t < visibility.end.distance(visibility.ray.o) {
                    return false;
                }
            }
            true
        }

        let mut radiance = Spectrum::zero();
        if let Some(interaction) = world.intersect(ray) {
            let sample = interaction.primitive.material.sample(ray.d, &interaction);
            let L = sample.spectrum;
            for light in lights.iter() {
                let emitter_sample = light.sample_li(&interaction);
                if test_visibility(emitter_sample.visibility, world) {
                    let f = interaction
                        .primitive
                        .material
                        .eval(emitter_sample.wi, ray.d);
                    radiance += f * emitter_sample.li * emitter_sample.wi.dot(interaction.n).abs();
                }
            }
            if depth + 1 < MAX_BOUNCES {
                radiance += L * li(Ray::new(interaction.p, sample.wo), depth + 1, world, lights);
            }
        } else {
            for light in lights.iter() {
                if light.is_environment() {
                    radiance += light.l_e(ray.d);
                }
            }
            return radiance;
        }

        radiance
    }

    for y in 0..height {
        for x in 0..width {
            for _ in 0..SPP {
                let x = x as f32 + (rand::random::<f32>() - 0.5);
                let y = y as f32 + (rand::random::<f32>() - 0.5);
                let p = Vector2::new(x, y);
                let mut ray = camera.sample_ray(p);

                let mut surface_reflectance = Spectrum::new(1.0, 1.0, 1.0);
                let mut contributed = Spectrum::zero();

                for _ in 0..MAX_BOUNCES {
                    if let Some(interaction) = world.intersect(ray) {
                        let sample = interaction.primitive.material.sample(ray.d, &interaction);
                        let L = sample.spectrum;

                        for light in lights.iter() {
                            let emitted = light.sample_li(&interaction);
                            if test_visibility(emitted.visibility) {
                                let f = interaction.primitive.material.eval(emitted.wi, ray.d);
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
                        for light in lights.iter() {
                            if light.is_environment() {
                                contributed += surface_reflectance * light.l_e(ray.d);
                            }
                        }

                        break;
                    }
                }

                film.apply_sample(p, contributed);
            }
        }
        println!("finished scanline {y}");
    }

    println!("writing output...");
    film.develop(Path::new("output"));

    println!("successfully wrote output :>");
}
