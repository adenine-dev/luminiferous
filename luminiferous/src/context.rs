use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};

#[allow(unused_imports)] // make prototyping easier FIXME: remove
use crate::{
    aggregates::{Aggregate, Bvh, Vector},
    bsdfs::{Bsdf, Lambertian},
    cameras::{Camera, PerspectiveCamera},
    film::Film,
    integrators::{Integrator, IntegratorT, PathIntegrator},
    lights::{Environment, Light, PointLight},
    materials::{DirectMaterial, Material},
    maths::{Matrix3, Matrix4, Point3, Transform3, UVector2, Vector2, Vector3},
    primitive::Primitive,
    rfilters::TentFilter,
    samplers::{RandomSampler, Sampler, StratifiedSampler},
    scene::Scene,
    shapes::{Shape, Sphere},
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
    textures::{CheckerboardTexture, ConstantTexture, Texture, TextureMapping, UvTexture},
};
use crate::{
    bsdfs::Conductor, maths::Transform2, scene::SceneBuilder, shapes::Triangle,
    textures::ImageTexture,
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
        println!("initializing...");

        let start = Instant::now();

        // let width = 3840;
        // let height = 2160;
        // let width = 1600;
        // let height = 900;
        // let width = 800;
        // let height = 450;
        // let width = 512;
        // let height = 384;
        // let width = 640;
        // let height = 360;
        let width = 320;
        let height = 180;
        // let width = 100;
        // let height = 62;

        let scene = match 0 {
            // sphere pyramid
            0 => {
                let mut scene_builder = SceneBuilder::new();
                scene_builder.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                    Film::new(
                        UVector2::new(width, height),
                        TentFilter::new(Vector2::splat(1.0)),
                    ),
                    Transform3::new(
                        Matrix4::look_at_rh(
                            // Point3::new(-6.0, 6.0, 6.0),
                            Point3::new(10.0, 3.0, 10.0),
                            Point3::new(0.0, 0.0, 0.0),
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    core::f32::consts::FRAC_PI_2,
                    0.0,
                    0.0,
                )));

                const N: u32 = 20;

                for n in 0..N {
                    let w = N - n;
                    for x in 0..w {
                        for z in 0..w {
                            let r = 0.5;
                            let y = n;
                            let p = Point3::new(
                                x as f32 - (N as f32 / 2.0) + r + (n as f32 / 2.0),
                                y as f32 - (n as f32 / 3.0f32.sqrt() * r),
                                z as f32 - (N as f32 / 2.0) + r + (n as f32 / 2.0),
                            );

                            scene_builder.primitive(
                                Shape::Sphere(Sphere::new(p, r)),
                                Material::Direct(DirectMaterial::new(Bsdf::Lambertian(
                                    Lambertian::new(Texture::Constant(ConstantTexture::new(
                                        Spectrum::from_rgb(
                                            (z as f32 / N as f32 * 0.7) + 0.2,
                                            (x as f32 / N as f32 * 0.7) + 0.2,
                                            (y as f32 / N as f32 * 0.7) + 0.2,
                                        ),
                                    ))),
                                ))),
                            );
                        }
                    }
                }

                // for x in 0..N {
                //     for y in 0..N {
                //         for z in 0..N {
                //             let R = 0.5;
                //             let p = Point3::new(
                //                 x as f32 - (N as f32 / 2.0) + R,
                //                 y as f32,
                //                 z as f32 - (N as f32 / 2.0) + R,
                //             );

                //             world.push(Primitive::new(
                //                 Shape::Sphere(Sphere::new(p, R)),
                //                 Material::Direct(DirectMaterial::new(Bsdf::Lambertian(
                //                     Lambertian::new(Texture::Constant(ConstantTexture::new(
                //                         Spectrum::from_rgb(
                //                             (x as f32 / N as f32 * 0.7) + 0.2,
                //                             (y as f32 / N as f32 * 0.7) + 0.2,
                //                             (z as f32 / N as f32 * 0.7) + 0.2,
                //                         ),
                //                     ))),
                //                 ))),
                //             ));
                //         }
                //     }
                // }

                scene_builder.primitive(
                    Shape::Sphere(Sphere::new(Vector3::new(0.0, -50000.5, 0.0), 50000.0)),
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.5, 0.5, 0.5))),
                    )))),
                );

                scene_builder.light(Light::Environment(Environment::new(Texture::Constant(
                    ConstantTexture::new(Spectrum::from_rgb(0.8, 0.8, 0.8)),
                ))));

                scene_builder.build()
            }
            // dragon
            1 => {
                let mut sb = SceneBuilder::new();
                sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                    Film::new(
                        UVector2::new(width, height),
                        TentFilter::new(Vector2::splat(1.0)),
                    ),
                    Transform3::new(
                        Matrix4::look_at_rh(
                            Point3::new(12.0, 20.0, 18.0),
                            Point3::new(0.0, 13.0, 0.0),
                            // Point3::new(1.0, 1.0, 1.0),
                            // Point3::splat(0.0),
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    core::f32::consts::FRAC_PI_2,
                    0.0,
                    0.0,
                )));

                let mut load = |path, material| {
                    let (models, _) = tobj::load_obj(
                        path,
                        // "assets/bunny/bunny.obj",
                        &tobj::GPU_LOAD_OPTIONS,
                    )
                    .expect("oof");

                    let mesh = &models[0].mesh;
                    let vertices = mesh
                        .positions
                        .chunks(3)
                        .map(|p| Point3::new(p[0], p[1], p[2]))
                        .collect::<Vec<_>>();
                    let mut tris =
                        vec![Shape::Triangle(Triangle::default()); mesh.indices.len() / 3];
                    for i in 0..mesh.indices.len() / 3 {
                        tris[i] = Shape::Triangle(Triangle::new(
                            vertices[mesh.indices[i * 3] as usize],
                            vertices[mesh.indices[i * 3 + 1] as usize],
                            vertices[mesh.indices[i * 3 + 2] as usize],
                        ));
                    }

                    sb.primitives(tris, material);
                };

                load(
                    "assets/Flamehorn Wyrmling/BabyDragon_C_v01_reduced.obj",
                    Material::Direct(DirectMaterial::new(
                        Bsdf::Conductor(Conductor::new(Texture::Constant(ConstantTexture::new(
                            Spectrum::from_rgb(0.8, 0.8, 0.8),
                        )))), //     Bsdf::Lambertian(Lambertian::new(
                              //     Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.8, 0.5, 0.8))),
                              // ))
                    )),
                );
                load(
                    "assets/Flamehorn Wyrmling/BabyDragon_C_Base_v01_reduced.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.2, 0.2, 0.2))),
                    )))),
                );

                sb.primitive(
                    Shape::Sphere(Sphere::new(Vector3::new(0.0, -50000.5, 0.0), 50000.0)),
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.5, 0.5, 0.5))),
                    )))),
                );

                sb.light(Light::Environment(Environment::new(
                    Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.8, 0.8, 0.8))), // Texture::Image(ImageTexture::from_path(Path::new(
                                                                                                //     "assets/kloppenheim_07_puresky/kloppenheim_07_puresky_4k.exr",
                                                                                                // ))),
                )));
                // sb.light(Light::Point(PointLight::new(
                //     Point3::new(100.0, 100.0, -20.0),
                //     Spectrum::from_rgb(1.0, 0.8, 0.6),
                // )));

                sb.build()
            }
            2 => {
                let mut sb = SceneBuilder::new();
                sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                    Film::new(
                        UVector2::new(width, height),
                        TentFilter::new(Vector2::splat(1.0)),
                    ),
                    Transform3::new(
                        Matrix4::look_at_rh(
                            Point3::new(1.0, 0.0, 0.0),
                            Point3::new(0.0, 0.0, 0.0),
                            // Point3::new(1.0, 1.0, 1.0),
                            // Point3::splat(0.0),
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    core::f32::consts::FRAC_PI_2,
                    0.0,
                    0.0,
                )));

                sb.primitive(
                    Shape::Sphere(Sphere::new(Vector3::new(0.0, -50000.5, 0.0), 50000.0)),
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.5, 0.5, 0.5))),
                    )))),
                );

                sb.primitive(
                    Shape::Sphere(Sphere::new(Vector3::new(0.0, 0.0, 0.0), 0.5)),
                    // Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                    //     Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.8, 0.8, 0.8))),
                    // )))),
                    Material::Direct(DirectMaterial::new(Bsdf::Conductor(Conductor::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(1.0, 1.0, 1.0))),
                    )))),
                );
                sb.primitive(
                    Shape::Sphere(Sphere::new(Vector3::new(0.0, 0.0, -1.0), 0.5)),
                    Material::Direct(DirectMaterial::new(Bsdf::Conductor(Conductor::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(1.0, 1.0, 1.0))),
                    )))),
                );
                sb.primitive(
                    Shape::Sphere(Sphere::new(Vector3::new(0.0, 0.0, 1.0), 0.5)),
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.8, 0.5, 0.8))),
                    )))),
                );

                sb.light(Light::Environment(Environment::new(
                    // Texture::Constant(
                    //     ConstantTexture::new(Spectrum::from_rgb(0.8, 0.8, 0.8)),
                    // )
                    Texture::Image(ImageTexture::from_path(Path::new(
                        "assets/kloppenheim_07_puresky/kloppenheim_07_puresky_4k.exr",
                    ))),
                )));

                sb.build()
            }
            _ => {
                panic!("bad scene mode")
            }
        }
        .unwrap();

        let sampler = Sampler::Stratified(StratifiedSampler::new(params.spp, params.seed, true));

        const MAX_BOUNCES: u32 = 10;
        let integrator = Integrator::Path(PathIntegrator::new(sampler, MAX_BOUNCES));

        let duration = start.elapsed();
        let ctx = Self { scene, integrator };
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
