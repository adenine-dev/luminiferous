use std::path::Path;

use russimp::{
    mesh::{Mesh, PrimitiveType},
    scene::PostProcess,
    Vector3D,
};

use crate::{
    bsdfs::{Bsdf, Lambertian},
    cameras::{Camera, PerspectiveCamera},
    film::Film,
    lights::{Environment, Light, PointLight},
    materials::{DirectMaterial, Material},
    maths::{Matrix4, Normal3, Point2, Point3, Transform3, Vector2, Vector3},
    media::MediumInterface,
    rfilters::{RFilter, TentFilter},
    scene::SceneBuilder,
    shapes::{Shape, Triangle},
    spectra::{Spectrum, SpectrumT},
    textures::{ConstantTexture, Texture},
};

use super::{Loader, SceneCreationParams, SceneResult};

// loads a file using assimp
pub struct AssimpLoader {}

impl AssimpLoader {}

//TODO: the unhappy path is awful here.. need proper error handling

pub(crate) fn shapes_from_russimp_mesh(mesh: &Mesh) -> Vec<Shape> {
    if (mesh.primitive_types & PrimitiveType::Triangle) != PrimitiveType::Triangle as u32 {
        println!(
            "[WARN]: mesh '{}' is non triangular and won't be loaded, it is {}",
            mesh.name, mesh.primitive_types
        );
        return vec![];
    }

    // assimp doesn't like blender normals, so we have this hack
    // https://github.com/assimp/assimp/issues/816
    let bad_normals = |n: &Vector3D| !(n.x.is_normal() && n.x.is_normal() && n.x.is_normal());

    if mesh.normals.iter().any(bad_normals) {
        println!(
            "[WARN]: mesh '{}' has invalid normals, shading flat.",
            mesh.name
        );
    }

    // let material = &imp_scene.materials[mesh.material_index as usize];
    let default_uvs = [
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(1.0, 1.0),
    ];

    mesh.faces
        .iter()
        .map(|face| {
            let indices = &face.0;
            assert!(face.0.len() == 3);

            let vertices = [
                mesh.vertices[indices[0] as usize],
                mesh.vertices[indices[1] as usize],
                mesh.vertices[indices[2] as usize],
            ]
            .map(|v| Vector3::new(v.x, v.y, v.z));

            let mut normals = if !mesh.normals.is_empty() {
                [
                    mesh.normals[indices[0] as usize],
                    mesh.normals[indices[1] as usize],
                    mesh.normals[indices[2] as usize],
                ]
            } else {
                [Vector3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                }; 3]
            };

            if normals.iter().any(bad_normals) {
                normals = [(vertices[1] - vertices[0])
                    .cross(vertices[2] - vertices[0])
                    .normalize(); 3]
                    .map(|n| Vector3D {
                        x: n.x,
                        y: n.y,
                        z: n.z,
                    })
            }

            let normals = normals.map(|n| Normal3::new(n.x, n.y, n.z).normalize());

            Shape::Triangle(Triangle::new(
                vertices,
                normals,
                if mesh.texture_coords.is_empty() {
                    default_uvs
                } else {
                    [
                        (mesh.texture_coords[0].as_ref(), 0),
                        (mesh.texture_coords[0].as_ref(), 1),
                        (mesh.texture_coords[0].as_ref(), 2),
                    ]
                    .map(|(uv, i)| {
                        uv.map(|uv| {
                            Point2::new(uv[indices[i] as usize].x, uv[indices[i] as usize].y)
                        })
                        .unwrap_or(default_uvs[i])
                    })
                },
            ))
        })
        .collect()
}

impl Loader for AssimpLoader {
    fn load_from_file(sb: &mut SceneBuilder, path: &Path, params: SceneCreationParams) {
        let imp_scene = russimp::scene::Scene::from_file(
            path.to_str().ok_or("bad path :<").unwrap(),
            vec![
                PostProcess::Triangulate,
                PostProcess::JoinIdenticalVertices,
                PostProcess::SortByPrimitiveType,
                PostProcess::PreTransformVertices,
            ],
        )
        .unwrap();

        // dbg!(&imp_scene);

        if !imp_scene.cameras.is_empty() {
            let cam = &imp_scene.cameras[0];
            sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                Film::new(
                    params.extent,
                    RFilter::Tent(TentFilter::new(Vector2::splat(1.0))),
                ),
                Transform3::new(
                    Matrix4::look_at_rh(
                        Point3::new(cam.position.x, cam.position.y, cam.position.z),
                        Vector3::new(cam.look_at.x, cam.look_at.y, cam.look_at.z),
                        Vector3::new(cam.up.x, cam.up.y, cam.up.z),
                    )
                    .inverse(),
                ),
                // convert to vfov
                2.0 * ((cam.horizontal_fov / 2.0).tan() * (params.extent.y as f32)
                    / (params.extent.x as f32))
                    .atan(),
                0.0,
                0.0,
                None,
            )));

            if cam.aspect != 0.0 && cam.aspect != params.extent.x as f32 / params.extent.y as f32 {
                println!("[WARN]: camera has different aspect ratio than expected extent (expected: {}, actual: {})", cam.aspect, params.extent.x as f32 / params.extent.y as f32);
            }

            if imp_scene.cameras.len() > 1 {
                println!("[WARN]: multiple cameras defined in scene using first one.");
            }
        }

        for light in imp_scene.lights {
            match light.light_source_type {
                russimp::light::LightSourceType::Point => {
                    sb.light(Light::Point(PointLight::new(
                        Point3::new(light.pos.x, light.pos.y, light.pos.z),
                        // NOTE: there are other colors associated with the light, tho we don't support the way this is separated so shrug
                        Spectrum::from_rgb(
                            light.color_diffuse.r,
                            light.color_diffuse.g,
                            light.color_diffuse.b,
                        ),
                    )));
                }
                _ => println!(
                    "[WARN]: light '{}' is not a supported type of light.",
                    light.name
                ),
            }
        }

        for mesh in imp_scene.meshes {
            if (mesh.primitive_types & PrimitiveType::Triangle) != PrimitiveType::Triangle as u32 {
                println!(
                    "[WARN]: mesh '{}' is non triangular and won't be loaded, it is {}",
                    mesh.name, mesh.primitive_types
                );
                continue;
            }

            // assimp doesn't like blender normals, so we have this hack
            // https://github.com/assimp/assimp/issues/816
            let bad_normals = mesh
                .normals
                .iter()
                .any(|n| !(n.x.is_normal() && n.x.is_normal() && n.x.is_normal()));

            if bad_normals {
                println!(
                    "[WARN]: mesh '{}' has invalid normals, shading flat.",
                    mesh.name
                );
            }

            // let material = &imp_scene.materials[mesh.material_index as usize];
            let default_uvs = [
                Point2::new(0.0, 0.0),
                Point2::new(1.0, 0.0),
                Point2::new(1.0, 1.0),
            ];

            let triangles = mesh
                .faces
                .iter()
                .map(|face| {
                    let indices = &face.0;
                    assert!(face.0.len() == 3);

                    let vertices = [
                        mesh.vertices[indices[0] as usize],
                        mesh.vertices[indices[1] as usize],
                        mesh.vertices[indices[2] as usize],
                    ]
                    .map(|v| Vector3::new(v.x, v.y, v.z));

                    Shape::Triangle(Triangle::new(
                        vertices,
                        if mesh.normals.is_empty() || bad_normals {
                            [(vertices[1] - vertices[0])
                                .cross(vertices[2] - vertices[0])
                                .normalize(); 3]
                        } else {
                            [
                                mesh.normals[indices[0] as usize],
                                mesh.normals[indices[1] as usize],
                                mesh.normals[indices[2] as usize],
                            ]
                            .map(|n| Normal3::new(n.x, n.y, n.z).normalize())
                        },
                        if mesh.texture_coords.is_empty() {
                            default_uvs
                        } else {
                            [
                                (mesh.texture_coords[0].as_ref(), 0),
                                (mesh.texture_coords[0].as_ref(), 1),
                                (mesh.texture_coords[0].as_ref(), 2),
                            ]
                            .map(|(uv, i)| {
                                uv.map(|uv| {
                                    Point2::new(
                                        uv[indices[i] as usize].x,
                                        uv[indices[i] as usize].y,
                                    )
                                })
                                .unwrap_or(default_uvs[i])
                            })
                        },
                    ))
                })
                .collect();

            // NOTE: materials aren't exported from blender yet shrug
            let material =
                Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                    Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.6, 0.3, 0.6))),
                ))));
            sb.primitives(triangles, material, None, MediumInterface::none());
        }
    }
}
