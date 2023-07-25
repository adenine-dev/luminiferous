use std::{path::Path, time::Instant};

#[allow(unused_imports)] // make prototyping easier FIXME: remove
use crate::{
    aggregates::{Aggregate, Bvh, Vector},
    bsdfs::PlasticBsdf,
    bsdfs::{ior, Dielectric, MeasuredBsdf},
    bsdfs::{Bsdf, Lambertian},
    bsdfs::{MirrorBsdf, NullBsdf},
    cameras::{Camera, PerspectiveCamera},
    core::Array2d,
    film::Film,
    integrators::{Integrator, IntegratorT, PathIntegrator},
    lights::Spotlight,
    lights::{AreaLight, DistantLight},
    lights::{Environment, Light, PointLight},
    loaders::{AssimpLoader, Loader},
    loaders::{PbrtLoader, SceneCreationParams},
    materials::MixMaterial,
    materials::{DirectMaterial, Material},
    media::{HomogeneousMedium, Medium, MediumInterface},
    phase_functions::HenyeyGreensteinPhaseFunction,
    phase_functions::{IsotropicPhaseFunction, PhaseFunction},
    prelude::*,
    primitive::Primitive,
    rfilters::TentFilter,
    samplers::{RandomSampler, Sampler, StratifiedSampler},
    scene::Scene,
    scene::SceneBuilder,
    shapes::Triangle,
    shapes::{Shape, Sphere},
    spectra::{Spectrum, SpectrumT},
    textures::ImageTexture,
    textures::{CheckerboardTexture, ConstantTexture, SpectralTexture, TextureMapping},
    textures::{Texture, UvTexture},
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
        infoln!("initializing...");

        let start = Instant::now();
        // let (width, height) = (3840, 2160);
        // let (width, height) = (2160, 3840);
        // let (width, height) = (1600, 900);
        // let (width, height) = (1280, 720);
        let (width, height) = (720, 1280);
        // let (width, height) = (800, 450);
        // let (width, height) = (800, 800);
        // let (width, height) = (1200, 1200);
        // let (width, height) = (512, 384);
        // let (width, height) = (320, 180);
        // let (width, height) = (100, 62);

        // let (width, height) = (69, 420);

        //TODO: replace with actual scene loading lol.. this is extremely quick and bad
        let load_obj =
            |sb: &mut SceneBuilder, path, material, world_to_object, medium_interface| {
                let (models, _) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS).expect("oof");

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
                    .map(|p| Normal3::new(p[0], p[1], p[2]).normalize())
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
                            *(uvs.get(mesh.indices[i * 3] as usize))
                                .unwrap_or(&Point2::new(1.0, 0.0)),
                            *(uvs.get(mesh.indices[i * 3 + 1] as usize))
                                .unwrap_or(&Point2::new(0.0, 1.0)),
                            *(uvs.get(mesh.indices[i * 3 + 2] as usize))
                                .unwrap_or(&Point2::new(0.0, 1.0)),
                        ],
                    ));
                }

                sb.primitives(tris, material, world_to_object, medium_interface);
            };

        let scene = match 4 {
            0 => {
                let mut sb = SceneBuilder::new();
                sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                    Film::new(
                        UVector2::new(width, height),
                        TentFilter::new(Vector2::splat(1.0)),
                    ),
                    Transform3::new(
                        Matrix4::look_at_rh(
                            Point3::new(-2.0, 2.0, 2.0),
                            Point3::new(0.0, 0.0, 0.0),
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    core::f32::consts::FRAC_PI_2,
                    0.0,
                    0.0,
                    None,
                )));

                sb.primitive(
                    Shape::Sphere(Sphere::new(50000.0)),
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                            0.5, 0.5, 0.5,
                        ))),
                    )))),
                    Some(Transform3::translate(Vector3::new(0.0, -50000.5, 0.0))),
                    MediumInterface::none(),
                );

                sb.light(Light::Environment(Environment::new(
                    SpectralTexture::Image(ImageTexture::from_path(Path::new(
                        "assets/material_test/christmas_photo_studio_07.exr",
                    ))),
                )));

                sb.build()
            }
            // dragon
            1 => {
                // let outside = Some(Medium::Homogeneous(HomogeneousMedium::new(
                //     PhaseFunction::Isotropic(IsotropicPhaseFunction::new()),
                //     Spectrum::splat(1.0),
                //     Spectrum::splat(0.005),
                //     1.0,
                // )));
                // let outside = None;
                let mut sb = SceneBuilder::new();

                // load_obj(
                //     &mut sb,
                //     "assets/Flamehorn Wyrmling/BabyDragon_C_v01_reduced.obj",
                //     // Material::Direct(DirectMaterial::new(Bsdf::Null(NullBsdf::new()))),
                //     Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                //         SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                //             1.0, 0.6, 0.6,
                //         ))),
                //     )))),
                //     None,
                //     MediumInterface::new(None, outside.clone()),
                //     // MediumInterface::new(
                //     //     Some(Medium::Homogeneous(HomogeneousMedium::new(
                //     //         PhaseFunction::Isotropic(IsotropicPhaseFunction::new()),
                //     //         Spectrum::from_rgb(1.0, 0.8, 0.8),
                //     //         Spectrum::from_rgb(1.0, 1.0, 1.0),
                //     //         1.0,
                //     //     ))),
                //     //     outside.clone(),
                //     // ),
                // );
                // load_obj(
                //     &mut sb,
                //     "assets/Flamehorn Wyrmling/BabyDragon_C_Base_v01_reduced.obj",
                //     Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                //         SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                //             0.2, 0.2, 0.2,
                //         ))),
                //     )))),
                //     None,
                //     MediumInterface::new(None, outside.clone()),
                // );

                // sb.primitive(
                //     Shape::Sphere(Sphere::new(60000.0)),
                //     Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                //         SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                //             0.5, 0.5, 0.5,
                //         ))),
                //     )))),
                //     Some(Transform3::translate(Vector3::new(0.0, -60000.5, 0.0))),
                //     MediumInterface::new(None, outside.clone()),
                // );
                // sb.primitive(
                //     Shape::Sphere(Sphere::new(5.0)),
                //     Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                //         SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                //             0.5, 0.5, 0.5,
                //         ))),
                //     )))),
                //     None,
                //     // Some(Transform3::translate(Vector3::new(0.0, 0.0, 0.0))),
                //     MediumInterface::new(None, outside.clone()),
                // );

                sb.load_with::<PbrtLoader>(
                    Path::new("assets/scenes/dragon/scene-v4.pbrt"),
                    SceneCreationParams {
                        extent: UExtent2::new(width, height),
                    },
                );

                // sb.light(Light::Environment(Environment::new(
                //     SpectralTexture::Constant(ConstantTexture::new(Spectrum::splat(0.01))),
                //     // SpectralTexture::Image(ImageTexture::from_path(Path::new(
                //     //     "assets/kloppenheim_07_puresky/kloppenheim_07_puresky_4k.exr",
                //     // ))),
                // )));

                sb.light(Light::Spot(Spotlight::new(
                    Point3::ZERO,
                    Spectrum::splat(20000.0),
                    10.0,
                    10.0,
                    Some(Transform3::new(
                        Matrix4::look_to_rh(
                            Point3::new(0.0, 45.0, 45.0 * 0.69651 / 0.692312),
                            Vector3::new(0.0, 0.692312, 0.69651),
                            Vector3::Y,
                        )
                        .inverse(),
                    )),
                )));

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
                            // Point3::new(0.0, 0.0, 8.0),
                            Point3::new(0.0, 5.0, 10.0),
                            Point3::new(0.0, 1.0, 0.0),
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    0.3,
                    0.0,
                    0.0,
                    None,
                )));

                load_obj(
                    &mut sb,
                    "assets/material_test/backdrop.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        SpectralTexture::Checkerboard(CheckerboardTexture::new(
                            Spectrum::from_rgb(0.3, 0.3, 0.3),
                            Spectrum::from_rgb(0.8, 0.8, 0.8),
                            TextureMapping::new(Matrix3::from_scale(Vector2::splat(11.8))),
                        )),
                    )))),
                    None,
                    // Some(Transform3::translate(Vector3::new(0.0, 0.0, 0.0))),
                    MediumInterface::none(),
                );

                load_obj(
                    &mut sb,
                    "assets/material_test/mori_knob.obj",
                    // Material::Direct(DirectMaterial::new(Bsdf::Conductor(ConductorBsdf::new(
                    //     SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                    //         // 3.540174, 2.311131, 1.668593,
                    //         // 3.98316, 2.385721, 1.603215,
                    //         9.223869, 6.269523, 4.837001,
                    //     ))),
                    //     SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                    //         // 0.265787, 0.19561, 0.22092,
                    //         // 0.143119, 0.374957, 1.442479,
                    //         1.65746, 0.880369, 0.521229,
                    //     ))),
                    // )))),
                    Material::Direct(DirectMaterial::new(Bsdf::Plastic(PlasticBsdf::new(
                        SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(
                            0.4, 0.4, 0.95,
                        ))),
                        ior::POLYPROPYLENE,
                        ior::AIR,
                        0.5,
                    )))),
                    None,
                    // Some(Transform3::translate(Vector3::new(0.0, -r, 0.0))),
                    MediumInterface::none(),
                );

                sb.light(Light::Environment(Environment::new(
                    SpectralTexture::Image(ImageTexture::from_path(Path::new(
                        "assets/material_test/christmas_photo_studio_07.exr",
                        // "assets/venice_sunset_4k.exr",
                    ))),
                )));

                sb.build()
            }
            // bunny
            3 => {
                let mut sb = SceneBuilder::new();
                sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                    Film::new(
                        UVector2::new(width, height),
                        TentFilter::new(Vector2::splat(1.0)),
                    ),
                    Transform3::new(
                        Matrix4::look_at_rh(
                            Point3::new(0.0, 3.0, 8.0) * 8.0,
                            Point3::new(0.0, 0.51, 0.0) * 11.0,
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    0.3,
                    0.0,
                    0.0,
                    None,
                )));

                load_obj(
                    &mut sb,
                    "assets/material_test/backdrop.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        SpectralTexture::Checkerboard(CheckerboardTexture::new(
                            Spectrum::from_rgb(0.3, 0.3, 0.3),
                            Spectrum::from_rgb(0.8, 0.8, 0.8),
                            TextureMapping::new(Matrix3::from_scale(
                                Vector2::splat(11.8) * Vector2::new(2.0, 1.0),
                            )),
                        )),
                    )))),
                    Some(Transform3::scale(
                        Vector3::splat(2.5) * (Vector3::X + Vector3::splat(1.0)),
                    )),
                    MediumInterface::none(),
                );

                load_obj(
                    &mut sb,
                    "assets/stanford_bunny/stanford_bunny.obj",
                    // Material::Direct(DirectMaterial::new(Bsdf::Dielectric(Dielectric::new(
                    //     ior::POLYCARBONATE,
                    //     ior::AIR,
                    //     Spectrum::from_rgb(1.0, 1.0, 1.0),
                    // )))),
                    Material::Direct(DirectMaterial::new(Bsdf::Null(NullBsdf::new()))),
                    Some(
                        Transform3::translate(Vector3::new(0.1, 0.0, 0.0))
                            * Transform3::scale(Vector3::splat(10.0))
                            * Transform3::rotate(Vector3::new(0.0, 0.7, 0.0)),
                    ),
                    // MediumInterface::none(),
                    MediumInterface::new(
                        Some(Medium::Homogeneous(HomogeneousMedium::new(
                            // PhaseFunction::Isotropic(IsotropicPhaseFunction::new()),
                            PhaseFunction::HenyeyGreenstein(HenyeyGreensteinPhaseFunction::new(
                                0.7,
                            )),
                            Spectrum::from_rgb(0.85, 1.0, 0.85),
                            Spectrum::splat(1.0),
                            1.0,
                        ))),
                        None,
                    ),
                );

                sb.light(Light::Environment(Environment::new(
                    SpectralTexture::Image(ImageTexture::from_path(Path::new(
                        "assets/material_test/christmas_photo_studio_07.exr",
                    ))),
                )));

                sb.build()
            }
            // loader tests
            4 => {
                let mut sb = SceneBuilder::new();
                sb.load_with::<PbrtLoader>(
                    Path::new("assets/scenes/staircase/scene-v4.pbrt"),
                    SceneCreationParams {
                        extent: UExtent2::new(width, height),
                    },
                );

                // sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                //     Film::new(
                //         UVector2::new(width, height),
                //         TentFilter::new(Vector2::splat(1.0)),
                //     ),
                //     Transform3::new(
                //         Matrix4::look_at_rh(
                //             // Point3::new(-6.0, 6.0, 6.0),
                //             Point3::new(0.0, 1.7, 4.8),
                //             Point3::new(0.0, 1.7, -1.0),
                //             Vector3::Y,
                //         )
                //         .inverse(),
                //     ),
                //     (59.0f32).to_radians(),
                //     0.0,
                //     0.0,
                //     None,
                // )));

                sb.area_light(AreaLight::new(
                    Primitive {
                        shape: Shape::Sphere(Sphere::new(0.5)),
                        material_index: 0,
                        area_light_index: None,
                        world_to_object: Some(Transform3::translate(Vector3::new(-0.5, 7.5, -1.0))),
                        medium_interface: MediumInterface::none(),
                    },
                    Spectrum::splat(10.0),
                ));
                // sb.light(Light::Environment(Environment::new(
                //     SpectralTexture::Constant(ConstantTexture::new(Spectrum::splat(1.0))),
                // )));
                sb.build()
            }
            // material scene
            5 => {
                let mut sb = SceneBuilder::new();
                sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                    Film::new(
                        UVector2::new(width, height),
                        TentFilter::new(Vector2::splat(1.0)),
                    ),
                    Transform3::new(
                        Matrix4::look_at_rh(
                            Point3::new(1.0, 0.6, 0.0),
                            Point3::new(0.0, 0.4, 0.0),
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    core::f32::consts::FRAC_PI_4 + 0.05,
                    0.0,
                    0.0,
                    None,
                )));

                load_obj(
                    &mut sb,
                    "assets/scenes/vintage_oil_lamps/floor.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Measured(
                        MeasuredBsdf::load_from_file(Path::new(
                            "assets/brdfs/paper_white_rgb.bsdf",
                        ))
                        .unwrap(),
                    ))),
                    None,
                    MediumInterface::none(),
                );

                let mut oil_lamp = |font_bsdf, bowl_bsdf, transform| {
                    load_obj(
                        &mut sb,
                        "assets/scenes/vintage_oil_lamps/font.obj",
                        Material::Direct(DirectMaterial::new(Bsdf::Measured(
                            MeasuredBsdf::load_from_file(Path::new(font_bsdf)).unwrap(),
                        ))),
                        Some(transform),
                        MediumInterface::none(),
                    );

                    load_obj(
                        &mut sb,
                        "assets/scenes/vintage_oil_lamps/chimney.obj",
                        Material::Direct(DirectMaterial::new(Bsdf::Dielectric(Dielectric::new(
                            ior::AIR,
                            ior::PYREX,
                            Spectrum::from_rgb(1.0, 1.0, 1.0),
                        )))),
                        Some(transform),
                        MediumInterface::none(),
                    );

                    sb.primitive(
                        Shape::Sphere(Sphere::new(0.1)),
                        Material::Direct(DirectMaterial::new(Bsdf::Measured(
                            MeasuredBsdf::load_from_file(Path::new(bowl_bsdf)).unwrap(),
                        ))),
                        Some(transform * Transform3::translate(Vector3::new(0.0, 0.265, 0.0))),
                        MediumInterface::none(),
                    );
                };

                // front row
                let size = 0.4;
                oil_lamp(
                    "assets/brdfs/vch_golden_yellow_rgb.bsdf",
                    "assets/brdfs/cc_iris_purple_gem_rgb.bsdf",
                    Transform3::translate(Vector3::new(0.0, 0.0, -size)),
                );
                oil_lamp(
                    "assets/brdfs/cc_ibiza_sunset_rgb.bsdf",
                    "assets/brdfs/cg_sunflower_rgb.bsdf",
                    Transform3::identity(),
                );
                oil_lamp(
                    "assets/brdfs/aniso_brushed_aluminium_1_rgb.bsdf",
                    "assets/brdfs/colodur_napoli_4f_rgb.bsdf",
                    Transform3::translate(Vector3::new(0.0, 0.0, size)),
                );

                // back row
                oil_lamp(
                    "assets/brdfs/weta_brushed_steel_satin_pink_rgb.bsdf",
                    "assets/brdfs/aniso_morpho_melenaus_rgb.bsdf",
                    Transform3::translate(Vector3::new(-size, 0.0, -size * 0.5)),
                );
                oil_lamp(
                    "assets/brdfs/aniso_metallic_paper_copper_rgb.bsdf",
                    "assets/brdfs/satin_rosaline_rgb.bsdf",
                    Transform3::translate(Vector3::new(-size, 0.0, size * 0.5)),
                );

                sb.light(Light::Environment(Environment::new(
                    SpectralTexture::Image(ImageTexture::from_path(Path::new(
                        "assets/material_test/christmas_photo_studio_07.exr",
                    ))),
                )));

                sb.build()
            }
            // hg
            6 => {
                let mut sb = SceneBuilder::new();
                sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                    Film::new(
                        UVector2::new(width, height),
                        TentFilter::new(Vector2::splat(1.0)),
                    ),
                    Transform3::new(
                        Matrix4::look_at_rh(
                            Point3::new(0.0, 3.0, 8.0) * 8.0,
                            Point3::new(0.0, 0.51, 0.0) * 11.0,
                            Vector3::Y,
                        )
                        .inverse(),
                    ),
                    0.3,
                    0.0,
                    0.0,
                    None,
                )));

                load_obj(
                    &mut sb,
                    "assets/material_test/backdrop.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                        SpectralTexture::Checkerboard(CheckerboardTexture::new(
                            Spectrum::from_rgb(0.3, 0.3, 0.3),
                            Spectrum::from_rgb(0.8, 0.8, 0.8),
                            TextureMapping::new(Matrix3::from_scale(
                                Vector2::splat(11.8) * Vector2::new(2.0, 1.0),
                            )),
                        )),
                    )))),
                    Some(Transform3::scale(
                        Vector3::splat(2.5) * (Vector3::X + Vector3::splat(1.0)),
                    )),
                    MediumInterface::none(),
                );

                load_obj(
                    &mut sb,
                    "assets/stanford_bunny/stanford_bunny.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Dielectric(Dielectric::new(
                        ior::POLYCARBONATE,
                        ior::AIR,
                        Spectrum::from_rgb(1.0, 1.0, 1.0),
                    )))),
                    // Material::Direct(DirectMaterial::new(Bsdf::Null(NullBsdf::new()))),
                    Some(
                        Transform3::translate(Vector3::new(-6.0 + 0.1, 0.0, 0.0))
                            * Transform3::scale(Vector3::splat(8.0))
                            * Transform3::rotate(Vector3::new(0.0, 0.7, 0.0)),
                    ),
                    // MediumInterface::none(),
                    MediumInterface::new(
                        Some(Medium::Homogeneous(HomogeneousMedium::new(
                            // PhaseFunction::Isotropic(IsotropicPhaseFunction::new()),
                            PhaseFunction::HenyeyGreenstein(HenyeyGreensteinPhaseFunction::new(
                                0.7,
                            )),
                            Spectrum::from_rgb(0.85, 1.0, 0.85),
                            Spectrum::splat(1.0),
                            1.0,
                        ))),
                        None,
                    ),
                );

                load_obj(
                    &mut sb,
                    "assets/stanford_bunny/stanford_bunny.obj",
                    Material::Direct(DirectMaterial::new(Bsdf::Dielectric(Dielectric::new(
                        ior::POLYCARBONATE,
                        ior::AIR,
                        Spectrum::from_rgb(1.0, 1.0, 1.0),
                    )))),
                    // Material::Direct(DirectMaterial::new(Bsdf::Null(NullBsdf::new()))),
                    Some(
                        Transform3::translate(Vector3::new(6.0 + 0.1, 0.0, 0.0))
                            * Transform3::scale(Vector3::splat(8.0))
                            * Transform3::rotate(Vector3::new(0.0, -0.7, 0.0)),
                    ),
                    // MediumInterface::none(),
                    MediumInterface::new(
                        Some(Medium::Homogeneous(HomogeneousMedium::new(
                            // PhaseFunction::Isotropic(IsotropicPhaseFunction::new()),
                            PhaseFunction::HenyeyGreenstein(HenyeyGreensteinPhaseFunction::new(
                                -0.7,
                            )),
                            Spectrum::from_rgb(0.85, 1.0, 0.85),
                            Spectrum::splat(1.0),
                            1.0,
                        ))),
                        None,
                    ),
                );

                sb.light(Light::Environment(Environment::new(
                    SpectralTexture::Image(ImageTexture::from_path(Path::new(
                        "assets/material_test/christmas_photo_studio_07.exr",
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
        // let sampler = Sampler::Random(RandomSampler::new(params.spp, params.seed));

        const MAX_BOUNCES: u32 = 24;
        let integrator = Integrator::Path(PathIntegrator::new(sampler, MAX_BOUNCES, false));

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
