// NOTE: this is a *very* incomplete impl of a pbrt loader to match what has been implemented
// This was also mostly written at 4am on zero sleep and jetlag so it is probably the worst code in this repo

// If you are a potential employer please look elsewhere for good code I have written

// FIXME: all of this

use core::panic;
use std::{collections::HashMap, fs, path::Path};

use crate::{
    bsdfs::{Bsdf, Dielectric, Lambertian},
    cameras::{Camera, PerspectiveCamera},
    film::Film,
    lights::{DistantLight, Environment, Light, PointLight},
    loaders::shapes_from_russimp_mesh,
    materials::{DirectMaterial, Material},
    maths::*,
    media::MediumInterface,
    rfilters::{RFilter, TentFilter},
    scene::SceneBuilder,
    shapes::{Shape, Sphere},
    spectra::{Spectrum, SpectrumT},
    textures::{ConstantTexture, ImageTexture, Texture},
};

use super::{Loader, SceneCreationParams};

pub struct PbrtLoader {}

#[derive(Debug)]
enum ParameterValue {
    Integer(Vec<i32>),
    Float(Vec<f32>),
    Float2(Vec<[f32; 2]>),
    Float3(Vec<[f32; 3]>),
    Spectrum(Spectrum),
    Rgb(Spectrum),
    Blackbody(Spectrum),
    Bool(bool),
    String(String),
    None,
}

impl ParameterValue {
    #[inline]
    pub fn unwrap_float_or(&self, default: f32) -> f32 {
        if let ParameterValue::Float(f) = self {
            if f.len() == 1 {
                return f[0];
            }
        }

        default
    }

    #[inline]
    pub fn unwrap_point3_or(&self, default: Point3) -> Point3 {
        if let ParameterValue::Float3(p) = self {
            if p.len() == 1 {
                return Point3::new(p[0][0], p[0][1], p[0][2]);
            }
        }

        default
    }

    #[inline]
    pub fn unwrap_spectrum(&self) -> Spectrum {
        match self {
            ParameterValue::Rgb(s) => *s,
            ParameterValue::Blackbody(s) => *s,
            ParameterValue::Spectrum(s) => *s,
            _ => panic!("oof"),
        }
    }

    #[inline]
    pub fn unwrap_string(&self) -> String {
        if let ParameterValue::String(s) = self {
            return s.clone();
        }

        panic!("oof");
    }
}

struct GraphicsState {
    pub transform: Transform3,
    pub material: Material,
    pub named_materials: HashMap<String, Material>,
}

impl Loader for PbrtLoader {
    fn load_from_file(sb: &mut SceneBuilder, path: &Path, sc_params: SceneCreationParams) {
        let string = fs::read_to_string(path).unwrap();

        let mut state = GraphicsState {
            transform: Transform3::identity(),
            material: Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                Texture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.8, 0.8, 0.8))),
            )))),
            named_materials: HashMap::new(),
        };

        let path_prefix = path.parent().unwrap();

        let tokens = string
            .lines()
            .filter_map(|line| {
                if line.is_empty() {
                    return None;
                }
                line.split('#').next().map(|s| s.trim())
            })
            .enumerate()
            .flat_map(|(l, line)| {
                let mut prev_c = '\0';
                let mut in_qoute = false;
                let mut in_bracket = false;
                line.split(move |c| {
                    if c == '"' && prev_c != '\\' {
                        in_qoute = !in_qoute
                    }
                    if c == '[' && !in_bracket && !in_qoute {
                        in_bracket = true;
                    }
                    if c == ']' && in_bracket && !in_qoute {
                        in_bracket = false;
                    }
                    prev_c = c;
                    !in_qoute && !in_bracket && c.is_whitespace()
                })
                .map(move |s| (l, s))
            })
            .filter(|s| !s.1.is_empty())
            .collect::<Vec<_>>();

        let mut token_iter = tokens.into_iter().peekable();

        macro_rules! next {
            () => {
                token_iter.next().unwrap().1
            };
        }

        macro_rules! parse_next {
            () => {{
                next!().parse().unwrap()
            }};
            ($T:ty) => {
                next!().parse::<$T>().unwrap()
            };
        }

        macro_rules! parse_params {
            () => {{
                let mut hm = HashMap::new();
                while token_iter
                    .peek()
                    .map(|(_, t)| t.starts_with('"') && t.ends_with('"'))
                    .unwrap_or(false)
                {
                    let (l, key) = token_iter.next().unwrap();
                    let (typ, key) = key
                        .split_once(' ')
                        .expect(&format!("invalid key {key} at line {l}"));
                    let key = &key[..key.len() - 1];
                    let typ = &typ[1..];
                    let value = next!();
                    let value = match typ {
                        "integer" => {
                            if value.starts_with('[') && value.ends_with(']') {
                                ParameterValue::Integer(
                                    value[1..value.len() - 1]
                                        .split_whitespace()
                                        .map(|s| s.parse().unwrap())
                                        .collect(),
                                )
                            } else {
                                ParameterValue::Integer(vec![value.parse().unwrap()])
                            }
                        }

                        "float" => {
                            if value.starts_with('[') && value.ends_with(']') {
                                ParameterValue::Float(
                                    value[1..value.len() - 1]
                                        .split_whitespace()
                                        .map(|s| s.parse().unwrap())
                                        .collect(),
                                )
                            } else {
                                ParameterValue::Float(vec![value.parse().unwrap()])
                            }
                        }
                        "point2" | "vector2" => {
                            if value.starts_with('[') && value.ends_with(']') {
                                ParameterValue::Float2(
                                    value[1..value.len() - 1]
                                        .split_whitespace()
                                        .array_chunks()
                                        .map(|[x, y]| [x.parse().unwrap(), y.parse().unwrap()])
                                        .collect(),
                                )
                            } else {
                                panic!("idk line {l}, idk");
                            }
                        }
                        "point3" | "vector3" | "normal3" => {
                            if value.starts_with('[') && value.ends_with(']') {
                                ParameterValue::Float3(
                                    value[1..value.len() - 1]
                                        .split_whitespace()
                                        .array_chunks()
                                        .map(|[x, y, z]| {
                                            [
                                                x.parse().unwrap(),
                                                y.parse().unwrap(),
                                                z.parse().unwrap(),
                                            ]
                                        })
                                        .collect(),
                                )
                            } else {
                                panic!("idk line {l}, idk");
                            }
                        }
                        "rgb" => {
                            if value.starts_with('[') && value.ends_with(']') {
                                let mut iter = value[1..value.len() - 1].split_whitespace();
                                ParameterValue::Rgb(Spectrum::from_rgb(
                                    iter.next().unwrap().parse().unwrap(),
                                    iter.next().unwrap().parse().unwrap(),
                                    iter.next().unwrap().parse().unwrap(),
                                ))
                            } else {
                                panic!("sadness noises line {l}, idk");
                            }
                        }
                        "bool" => ParameterValue::Bool(value.parse().unwrap()),
                        "string" => ParameterValue::String(value[1..value.len() - 1].to_owned()),
                        "blackbody" | "spectrum" => {
                            println!("[WARN] unsupported type {typ} at line {l}");
                            ParameterValue::None
                        }
                        _ => panic!("invalid type {typ} at line {l}"),
                    };
                    hm.insert(key, value);
                }
                hm
            }};
        }

        macro_rules! parse_material {
            ($l:ident, $kind:expr, $params:ident) => {{
                match $kind {
                    "\"diffuse\"" => Some(Material::Direct(DirectMaterial::new(Bsdf::Lambertian(
                        Lambertian::new(Texture::Constant(ConstantTexture::new(
                            Spectrum::from_rgb(0.5, 0.5, 0.5),
                        ))),
                    )))),
                    "\"dielectric\"" => Some(Material::Direct(DirectMaterial::new(
                        Bsdf::Dielectric(Dielectric::new(
                            1.0,
                            $params
                                .get("eta")
                                .unwrap_or(&ParameterValue::None)
                                .unwrap_float_or(1.5),
                            Spectrum::splat(1.0),
                        )),
                    ))),
                    _ => {
                        println!("[WARN]: unsupported material kind {} at line {}", $kind, $l);
                        None
                    }
                }
            }};
        }

        while let Some((l, t)) = token_iter.next() {
            match t {
                // transforms
                "Identity" => state.transform = Transform3::identity(),
                "Translate" => {
                    state.transform = state.transform
                        * Transform3::translate(Vector3::new(
                            parse_next!(),
                            parse_next!(),
                            parse_next!(),
                        ))
                }
                "Scale" => {
                    state.transform = state.transform
                        * Transform3::scale(Vector3::new(
                            parse_next!(),
                            parse_next!(),
                            parse_next!(),
                        ))
                }
                "Rotate" => {
                    let a = parse_next!(f32).to_radians();
                    state.transform = state.transform
                        * Transform3::new(Matrix4::from_axis_angle(
                            Vector3::new(parse_next!(), parse_next!(), parse_next!()),
                            a,
                        ))
                }
                "LookAt" => {
                    let eye = Point3::new(parse_next!(), parse_next!(), parse_next!());
                    let at = Point3::new(parse_next!(), parse_next!(), parse_next!());
                    let up = Vector3::new(parse_next!(), parse_next!(), parse_next!());
                    state.transform =
                        state.transform * Transform3::new(Matrix4::look_at_rh(eye, at, up))
                }
                "Transform" => {
                    let values = next!();
                    let values = values
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect::<Vec<_>>();
                    state.transform =
                        Transform3::new(Matrix4::from_cols_array(&values.try_into().unwrap()))
                }
                "ConcatTransform" => {
                    state.transform = state.transform
                        * Transform3::new(
                            Matrix4::from_cols_array_2d(&[
                                [parse_next!(), parse_next!(), parse_next!(), parse_next!()],
                                [parse_next!(), parse_next!(), parse_next!(), parse_next!()],
                                [parse_next!(), parse_next!(), parse_next!(), parse_next!()],
                                [parse_next!(), parse_next!(), parse_next!(), parse_next!()],
                            ])
                            .transpose(),
                        )
                }
                // entities
                "Camera" => {
                    let kind = next!();
                    assert!(kind == "\"perspective\"");
                    match kind {
                        "\"perspective\"" => {
                            let params = parse_params!();
                            sb.camera(Camera::Projective(PerspectiveCamera::new_perspective(
                                Film::new(
                                    sc_params.extent,
                                    RFilter::Tent(TentFilter::new(Vector2::splat(1.0))),
                                ),
                                state.transform.inverse(),
                                params
                                    .get("fov")
                                    .unwrap_or(&ParameterValue::None)
                                    .unwrap_float_or(90.0)
                                    .to_radians(),
                                params
                                    .get("lensradius")
                                    .unwrap_or(&ParameterValue::None)
                                    .unwrap_float_or(0.0),
                                params
                                    .get("focaldistance")
                                    .unwrap_or(&ParameterValue::None)
                                    .unwrap_float_or(0.0),
                                None,
                            )));
                        }
                        _ => println!("[WARN]: unsupported camera kind {kind} at line {l}"),
                    }
                }
                "LightSource" => {
                    let kind = next!();
                    let params = parse_params!();
                    match kind {
                        "\"infinite\"" => {
                            if params.contains_key("L") {
                                sb.light(Light::Environment(Environment::new(Texture::Constant(
                                    ConstantTexture::new(params["L"].unwrap_spectrum()),
                                ))));
                            } else if params.contains_key("filename") {
                                let filename = &params["filename"].unwrap_string();
                                let filename = if !Path::new(filename).is_absolute() {
                                    path_prefix.join(filename)
                                } else {
                                    Path::new(filename).to_path_buf()
                                };
                                sb.light(Light::Environment(Environment::new(Texture::Image(
                                    ImageTexture::from_path(&filename),
                                ))));
                            }
                        }
                        "\"distant\"" => {
                            let to = params["to"].unwrap_point3_or(Point3::new(0.0, 0.0, 1.0));
                            let from = params["from"].unwrap_point3_or(Point3::new(0.0, 0.0, 0.0));
                            sb.light(Light::Distant(DistantLight::new(
                                from - to,
                                params["L"].unwrap_spectrum(),
                            )));
                        }
                        "\"point\"" => {
                            sb.light(Light::Point(PointLight::new(
                                params
                                    .get("from")
                                    .unwrap_or(&ParameterValue::None)
                                    .unwrap_point3_or(Point3::ZERO),
                                params
                                    .get("l")
                                    .unwrap_or(&ParameterValue::None)
                                    .unwrap_spectrum(),
                            )));
                        }
                        _ => println!("[WARN]: unsupported light source kind {kind} at line {l}"),
                    }
                }
                "Shape" => {
                    let kind = next!();
                    let params = parse_params!();
                    match kind {
                        "\"sphere\"" => {
                            sb.primitive(
                                Shape::Sphere(Sphere::new(
                                    params
                                        .get("radius")
                                        .unwrap_or(&ParameterValue::None)
                                        .unwrap_float_or(1.0),
                                )),
                                state.material.clone(),
                                Some(state.transform),
                                MediumInterface::none(),
                            );
                        }
                        "\"plymesh\"" => {
                            let filename = params["filename"].unwrap_string();
                            let filename = &filename.trim();
                            let filename = &filename[1..filename.len() - 1];
                            let filename = if !Path::new(filename).is_absolute() {
                                path_prefix.join(filename)
                            } else {
                                Path::new(filename).to_path_buf()
                            };

                            let imp_scene = russimp::scene::Scene::from_file(
                                filename.as_path().to_str().unwrap(),
                                vec![
                                    russimp::scene::PostProcess::Triangulate,
                                    // russimp::scene::PostProcess::JoinIdenticalVertices,
                                    russimp::scene::PostProcess::SortByPrimitiveType,
                                    russimp::scene::PostProcess::PreTransformVertices,
                                ],
                            )
                            .unwrap();
                            let shapes = imp_scene
                                .meshes
                                .iter()
                                .flat_map(shapes_from_russimp_mesh)
                                .collect::<Vec<_>>();

                            sb.primitives(
                                shapes,
                                state.material.clone(),
                                Some(state.transform),
                                MediumInterface::none(),
                            );
                        }
                        _ => println!("[WARN]: unsupported shape kind {kind} at line {l}"),
                    }
                }
                "Material" => {
                    let kind = next!();
                    let params = parse_params!();

                    if let Some(mat) = parse_material!(l, kind, params) {
                        state.material = mat;
                    }
                }
                "MakeNamedMaterial" => {
                    let name = next!();
                    let params = parse_params!();
                    if let Some(mat) = parse_material!(
                        l,
                        params
                            .get("type")
                            .unwrap_or(&ParameterValue::None)
                            .unwrap_string()
                            .as_str()
                            .trim(),
                        params
                    ) {
                        state
                            .named_materials
                            .insert(name[1..name.len() - 1].to_owned(), mat);
                    } else {
                        println!(
                            "[ERROR] Named material {name} at line {l} is not a valid material."
                        );
                    }
                }
                "NamedMaterial" => {
                    let name = next!();
                    let name = &name[1..name.len() - 1];
                    if let Some(mat) = state.named_materials.get(name) {
                        state.material = mat.clone();
                    } else {
                        println!("[ERROR] getting unknown named material '{name}'");
                        dbg!(&state.named_materials);
                    }
                }
                "WorldBegin" => {
                    //TODO: rest of the stuff that should happen here..
                    state.transform = Transform3::identity()
                }
                // "Accelerator" | "CoordinateSystem" | "CoordSysTransform" | "Include" | "Import"
                _ => {
                    println!("[WARN]: unsupported directive {t} at line {l}");
                    loop {
                        let t = token_iter.peek();
                        if t.is_none() {
                            break;
                        }
                        let (_, t) = t.unwrap();
                        if t.starts_with(|p: char| p.is_uppercase()) {
                            break;
                        }
                        token_iter.next();
                    }
                }
            }
        }
    }
}
