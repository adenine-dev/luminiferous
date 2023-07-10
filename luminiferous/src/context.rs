use std::{path::Path, time::Instant};

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
    bsdfs::{ior, Conductor, Dielectric},
    maths::{Normal3, Point2},
    scene::SceneBuilder,
    shapes::Triangle,
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
        // let (width, height) = (3840, 2160);
        let (width, height) = (1600, 900);
        // let (width, height) = (800, 450);
        // let (width, height) = (800, 800);
        // let (width, height) = (1200, 1200);
        // let (width, height) = (512, 384);
        // let (width, height) = (320, 180);
        // let (width, height) = (100, 62);

        // let (width, height) = (69, 420);

        //TODO: replace with actual scene loading lol.. this is extremely quick and bad
        let load_obj = |sb: &mut SceneBuilder, path, material, world_to_object| {
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
            // assumes we have normals...
            let normals = mesh
                .normals
                .chunks(3)
                .map(|p| Normal3::new(p[0], p[1], p[2]))
                .collect::<Vec<_>>();
            // assume...
            let uvs = mesh
                .texcoords
                .chunks(2)
                .map(|p| Point2::new(p[0], p[1]))
                .collect::<Vec<_>>();

            let mut tris = vec![Shape::Triangle(Triangle::default()); mesh.indices.len() / 3];
            for i in 0..mesh.indices.len() / 3 {
                tris[i] = Shape::Triangle(Triangle::new(
                    [
                        vertices[mesh.indices[i * 3] as usize],
                        vertices[mesh.indices[i * 3 + 1] as usize],
                        vertices[mesh.indices[i * 3 + 2] as usize],
                    ],
                    [
                        normals[mesh.indices[i * 3] as usize],
                        normals[mesh.indices[i * 3 + 1] as usize],
                        normals[mesh.indices[i * 3 + 2] as usize],
                    ],
                    [
                        *(uvs.get(mesh.indices[i * 3] as usize)).unwrap_or(&Point2::new(1.0, 0.0)),
                        *(uvs.get(mesh.indices[i * 3 + 1] as usize))
                            .unwrap_or(&Point2::new(0.0, 1.0)),
                        *(uvs.get(mesh.indices[i * 3 + 2] as usize))
                            .unwrap_or(&Point2::new(0.0, 1.0)),
                    ],
                ));
            }

            sb.primitives(tris, material, world_to_object);
        };

        let scene = match 1 {
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
                            Point3::new(-10.0, 5.0, -7.0),
                            Point3::new(0.0, 1.0, 0.0),
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
                                Shape::Sphere(Sphere::new(r)),
                                Material::Direct(DirectMaterial::new(Bsdf::Lambertian(
                                    Lambertian::new(Texture::Constant(ConstantTexture::new(
                                        Spectrum::from_rgb(
                                            (z as f32 / N as f32 * 0.7) + 0.2,
                                            (x as f32 / N as f32 * 0.7) + 0.2,
                                            (y as f32 / N as f32 * 0.7) + 0.2,
                                        ),
                                    ))),
                                ))),
                                Some(Transform3::translate(p)),
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
                    Shape::Sphere(Sphere::new(50000.0)),
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.5, 0.5, 0.5))),
                    )))),
                    Some(Transform3::translate(Vector3::new(0.0, -50000.5, 0.0))),
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
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    core::f32::consts::FRAC_PI_2,
                    0.0,
                    0.0,
                )));

                load_obj(
                    &mut sb,
                    "assets/Flamehorn Wyrmling/BabyDragon_C_v01_reduced.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Conductor(Conductor::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.8, 0.8, 0.8))),
                    )))),
                    None,
                );
                load_obj(
                    &mut sb,
                    "assets/Flamehorn Wyrmling/BabyDragon_C_Base_v01_reduced.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.2, 0.2, 0.2))),
                    )))),
                    None,
                );

                sb.primitive(
                    Shape::Sphere(Sphere::new(50000.0)),
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.5, 0.5, 0.5))),
                    )))),
                    Some(Transform3::translate(Vector3::new(0.0, -50000.5, 0.0))),
                );

                sb.light(Light::Environment(Environment::new(
                    // Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.8, 0.8, 0.8))),
                    Texture::Image(ImageTexture::from_path(Path::new(
                        "assets/kloppenheim_07_puresky/kloppenheim_07_puresky_4k.exr",
                    ))),
                )));
                // sb.light(Light::Point(PointLight::new(
                //     Point3::new(100.0, 100.0, -20.0),
                //     Spectrum::from_rgb(1.0, 0.8, 0.6),
                // )));

                sb.build()
            }
            // material test
            2 => {
                let mut sb = SceneBuilder::new();
                sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                    Film::new(
                        UVector2::new(width, height),
                        TentFilter::new(Vector2::splat(1.0)),
                    ),
                    Transform3::new(
                        Matrix4::look_at_rh(
                            // Point3::new(0.0, 2.0, 8.0),
                            Point3::new(0.0, 2.0, 8.0),
                            // Point3::new(0.0, 0.0, 8.0),
                            Point3::new(0.0, 0.0, 0.0),
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    0.4,
                    0.0,
                    0.0,
                )));

                let r = 1.0;

                load_obj(
                    &mut sb,
                    "assets/material_test/backdrop.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Checkerboard(CheckerboardTexture::new(
                            Spectrum::from_rgb(0.3, 0.3, 0.3),
                            Spectrum::from_rgb(0.8, 0.8, 0.8),
                            TextureMapping::new(Matrix3::from_scale(Vector2::splat(11.8))),
                        )),
                    )))),
                    Some(Transform3::translate(Vector3::new(0.0, -r, 0.0))),
                );

                sb.primitive(
                    Shape::Sphere(Sphere::new(r)),
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(1.0, 0.8, 1.0))),
                    )))),
                    None,
                );

                sb.light(Light::Environment(Environment::new(Texture::Image(
                    ImageTexture::from_path(Path::new(
                        "assets/material_test/christmas_photo_studio_07.exr",
                    )),
                ))));

                sb.build()
            }
            _ => {
                panic!("bad scene mode")
            }
        }
        .unwrap();

        let sampler = Sampler::Stratified(StratifiedSampler::new(params.spp, params.seed, true));
        // let sampler = Sampler::Random(RandomSampler::new(params.spp, params.seed));

        const MAX_BOUNCES: u32 = 100;
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
