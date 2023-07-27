// NOTE: this is a *very* incomplete impl of a pbrt loader to match what has been implemented
// This was also mostly written at 4am on zero sleep and jetlag so it is probably the worst code in this repo

// If you are a potential employer please look elsewhere for good code I have written

// FIXME: all of this

use core::panic;
use std::{collections::HashMap, fs, path::Path};

use crate::lights::AreaLight;
use crate::prelude::*;

use crate::primitive::Primitive;
use crate::shapes::Triangle;
use crate::{
    bsdfs::{ior, Bsdf, ConductorBsdf, Dielectric, Lambertian, NullBsdf, PlasticBsdf},
    cameras::{Camera, PerspectiveCamera},
    film::Film,
    lights::{DistantLight, Environment, Light, PointLight},
    loaders::shapes_from_russimp_mesh,
    materials::{DirectMaterial, Material},
    media::MediumInterface,
    rfilters::{RFilter, TentFilter},
    scene::SceneBuilder,
    shapes::{Shape, Sphere},
    spectra::{Spectrum, SpectrumT},
    textures::{ConstantTexture, ImageTexture, SpectralTexture},
};

use super::{Loader, SceneCreationParams};

pub struct PbrtLoader {}

#[derive(Debug)]
#[allow(dead_code)] //incomplete impl
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
    pub fn unwrap_point2s_or(&self, default: Point2) -> Vec<Point2> {
        if let ParameterValue::Float2(p) = self {
            return p.iter().map(|p| Point2::new(p[0], p[1])).collect();
        }

        vec![default]
    }

    #[inline]
    pub fn unwrap_point3s_or(&self, default: Point3) -> Vec<Point3> {
        if let ParameterValue::Float3(p) = self {
            return p.iter().map(|p| Point3::new(p[0], p[1], p[2])).collect();
        }

        vec![default]
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

    #[inline]
    pub fn unwrap_string_or(&self, default: String) -> String {
        if let ParameterValue::String(s) = self {
            s.clone()
        } else {
            default
        }
    }

    #[inline]
    pub fn unwrap_ints(&self) -> &Vec<i32> {
        if let ParameterValue::Integer(f) = self {
            return f;
        }

        panic!("oof")
    }
}

#[derive(Debug, Clone)]
enum UntypedTexture {
    Spectral(SpectralTexture),
    // Float(Texture<f32>),
}

fn load_ply_to_shape_vec(path: &Path) -> Vec<Shape> {
    let p = ply_rs::parser::Parser::<ply_rs::ply::DefaultElement>::new();
    let mut f = std::fs::File::open(path).unwrap();

    let ply = p.read_ply(&mut f).unwrap();
    let vertices = ply.payload["vertex"]
        .iter()
        .map(|v| {
            use ply_rs::ply::Property;
            let Property::Float(x) = v["x"] else {panic!("no x found")};
            let Property::Float(y) = v["y"] else {panic!("no z found")};
            let Property::Float(z) = v["z"] else {panic!("no y found")};
            let Property::Float(nx) = *v.get("nx").unwrap_or(&Property::Float(0.0)) else {panic!("no nx found")};
            let Property::Float(ny) = *v.get("ny").unwrap_or(&Property::Float(0.0)) else {panic!("no ny found")};
            let Property::Float(nz) = *v.get("nz").unwrap_or(&Property::Float(0.0)) else {panic!("no nz found")};

            let Property::Float(s) = *v.get("s").unwrap_or_else(|| v.get("u").expect("no s found")) else {panic!("no s found")};
            let Property::Float(t) = *v.get("t").unwrap_or_else(|| v.get("v").expect("no t found")) else {panic!("no t found")};

            (
                Point3::new(x, y, z),
                Normal3::new(nx, ny, nz),
                Point2::new(s, t),
            )
        })
        .collect::<Vec<_>>();
    ply.payload["face"]
        .iter()
        .flat_map(|face| {
            let idxs = &face["vertex_indices"];
            match idxs {
                ply_rs::ply::Property::ListUInt(list) => {
                    if list.len() == 3 {
                        return vec![[list[0] as usize, list[1] as usize, list[2] as usize]];
                    }

                    vec![
                        [list[0] as usize, list[1] as usize, list[2] as usize],
                        [list[0] as usize, list[2] as usize, list[3] as usize],
                    ]
                }
                ply_rs::ply::Property::ListInt(list) => {
                    if list.len() == 3 {
                        return vec![[list[0] as usize, list[1] as usize, list[2] as usize]];
                    }

                    vec![
                        [list[0] as usize, list[1] as usize, list[2] as usize],
                        [list[0] as usize, list[2] as usize, list[3] as usize],
                    ]
                }
                _ => panic!("oof"),
            }
        })
        .map(|face| {
            let v1 = vertices[face[0]];
            let v2 = vertices[face[1]];
            let v3 = vertices[face[2]];

            let mut n = [v1.1, v2.1, v3.1];
            if n[0] == Normal3::ZERO && n[1] == Normal3::ZERO && n[2] == Normal3::ZERO {
                n = [(v2.0 - v1.0).cross(v3.0 - v1.0); 3];
            }
            n = n.map(|n| n.normalize());
            Shape::Triangle(Triangle::new([v1.0, v2.0, v3.0], n, [v1.2, v2.2, v3.2]))
        })
        .collect()
}

struct GraphicsState {
    pub transform: Transform3,
    pub material: Material,
    pub named_materials: HashMap<String, Material>,
    pub named_textures: HashMap<String, UntypedTexture>,
    pub area_light: Option<Spectrum>,
}

impl Loader for PbrtLoader {
    fn load_from_file(sb: &mut SceneBuilder, path: &Path, sc_params: SceneCreationParams) {
        let string = fs::read_to_string(path).unwrap();

        let mut state = GraphicsState {
            transform: Transform3::identity(),
            material: Material::Direct(DirectMaterial::new(Bsdf::Lambertian(Lambertian::new(
                SpectralTexture::Constant(ConstantTexture::new(Spectrum::from_rgb(0.8, 0.8, 0.8))),
            )))),
            named_materials: HashMap::new(),
            named_textures: HashMap::from([(
                "__default_spectral_texture".to_owned(),
                UntypedTexture::Spectral(SpectralTexture::Constant(ConstantTexture::new(
                    Spectrum::from_rgb(0.1, 0.5, 0.1),
                ))),
            )]),
            area_light: None,
        };

        let path_prefix = path.parent().unwrap();

        let clean_text = string
            .lines()
            .filter_map(|line| line.split('#').next())
            .fold(String::new(), |a, b| a + b + "\n");

        let tokens = {
            let mut prev_c = '\0';
            let mut in_qoute = false;
            let mut in_bracket = false;
            let mut line = 1;

            clean_text
                .split(|c| {
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
                .filter_map(|s| {
                    if s.is_empty() {
                        return None;
                    }
                    Some((0, s))
                })
                .collect::<Vec<_>>()
        };

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
                                panic!("bad parameter at {l}");
                            }
                        }
                        "point3" | "vector3" | "normal3" | "normal" => {
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
                                panic!("bad parameter at {l}");
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
                                panic!("bad parameter at {l}");
                            }
                        }
                        "bool" => {
                            let x: &[_] = &['[', ']', ' '];
                            ParameterValue::Bool(value.trim_matches(x).parse().unwrap())
                        }
                        "string" | "texture" => {
                            ParameterValue::String(value[1..value.len() - 1].to_owned())
                        }
                        "blackbody" | "spectrum" => {
                            warnln!("unsupported type `{typ}` at line {l}");
                            ParameterValue::None
                        }
                        _ => panic!("invalid type `{typ}` at line {l}"),
                    };
                    hm.insert(key, value);
                }
                hm
            }};
        }

        macro_rules! parse_material {
            ($l:ident, $kind:expr, $params:ident) => {{
                match $kind {
                    "\"diffuse\"" => {
                        let reflectance =
                            $params.get("reflectance").unwrap_or(&ParameterValue::None);

                        let texture = if matches!(
                            reflectance,
                            ParameterValue::Spectrum(_) | ParameterValue::Rgb(_)
                        ) {
                            UntypedTexture::Spectral(SpectralTexture::Constant(
                                ConstantTexture::new(reflectance.unwrap_spectrum()),
                            ))
                        } else {
                            let texture_name = reflectance.unwrap_string();
                            // .unwrap_string_or("__default_spectral_texture".to_owned());
                            let texture_name = texture_name.trim().trim_matches('"');
                            state.named_textures.get(texture_name).unwrap().clone()
                        };
                        match texture {
                            UntypedTexture::Spectral(texture) => {
                                Some(Material::Direct(DirectMaterial::new(Bsdf::Lambertian(
                                    Lambertian::new(texture.clone()),
                                ))))
                            }
                            _ => panic!("expected spectral texture found other at line {}", $l),
                        }
                    }
                    "\"coateddiffuse\"" => {
                        let reflectance =
                            $params.get("reflectance").unwrap_or(&ParameterValue::None);

                        let texture = if matches!(
                            reflectance,
                            ParameterValue::Spectrum(_) | ParameterValue::Rgb(_)
                        ) {
                            UntypedTexture::Spectral(SpectralTexture::Constant(
                                ConstantTexture::new(reflectance.unwrap_spectrum()),
                            ))
                        } else {
                            let texture_name = reflectance
                                .unwrap_string_or("__default_spectral_texture".to_owned());
                            let texture_name = texture_name.trim().trim_matches('"');
                            state.named_textures.get(texture_name).unwrap().clone()
                        };
                        match texture {
                            UntypedTexture::Spectral(texture) => Some(Material::Direct(
                                DirectMaterial::new(Bsdf::Plastic(PlasticBsdf::new(
                                    texture.clone(),
                                    ior::AIR,
                                    ior::WATER_ICE,
                                    $params
                                        .get("vroughness")
                                        .unwrap_or(&ParameterValue::None)
                                        .unwrap_float_or(0.1),
                                ))),
                            )),
                            _ => panic!("expected spectral texture found other at line {}", $l),
                        }
                    }
                    "\"conductor\"" => Some(Material::Direct(DirectMaterial::new(
                        Bsdf::Conductor(ConductorBsdf::new(
                            SpectralTexture::Constant(ConstantTexture::new(
                                $params
                                    .get("k")
                                    .unwrap_or(&ParameterValue::None)
                                    .unwrap_spectrum(),
                            )),
                            SpectralTexture::Constant(ConstantTexture::new(
                                $params
                                    .get("eta")
                                    .unwrap_or(&ParameterValue::None)
                                    .unwrap_spectrum(),
                            )),
                        )),
                    ))),
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
                    "\"null\"" => Some(Material::Direct(DirectMaterial::new(Bsdf::Null(
                        NullBsdf::new(),
                    )))),
                    _ => {
                        warnln!("unsupported material kind {} at line {}", $kind, $l);
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
                        _ => warnln!(" unsupported camera kind {kind} at line {l}"),
                    }
                }
                "LightSource" => {
                    let kind = next!();
                    let params = parse_params!();
                    match kind {
                        "\"infinite\"" => {
                            if params.contains_key("L") {
                                sb.light(Light::Environment(Environment::new(
                                    SpectralTexture::Constant(ConstantTexture::new(
                                        params["L"].unwrap_spectrum(),
                                    )),
                                )));
                            } else if params.contains_key("filename") {
                                let filename = &params["filename"].unwrap_string();
                                let filename = if !Path::new(filename).is_absolute() {
                                    path_prefix.join(filename)
                                } else {
                                    Path::new(filename).to_path_buf()
                                };
                                sb.light(Light::Environment(Environment::new(
                                    SpectralTexture::Image(ImageTexture::from_path(&filename)),
                                )));
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
                        _ => warnln!(" unsupported light source kind {kind} at line {l}"),
                    }
                }
                "AreaLightSource" => {
                    let _type = next!();
                    let params = parse_params!();
                    state.area_light = Some(params.get("L").unwrap().unwrap_spectrum());
                }
                "Shape" => {
                    let kind = next!();
                    let params = parse_params!();
                    let shapes = match kind {
                        "\"sphere\"" => {
                            //
                            vec![Shape::Sphere(Sphere::new(
                                params
                                    .get("radius")
                                    .unwrap_or(&ParameterValue::None)
                                    .unwrap_float_or(1.0),
                            ))]
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

                            load_ply_to_shape_vec(filename.as_path())
                        }
                        "\"trianglemesh\"" => {
                            let uvs = params
                                .get("uv")
                                .unwrap_or(&ParameterValue::None)
                                .unwrap_point2s_or(Point2::ZERO);

                            let ps = params
                                .get("P")
                                .unwrap_or(&ParameterValue::None)
                                .unwrap_point3s_or(Point3::ZERO);

                            let ns = params
                                .get("N")
                                .unwrap_or(&ParameterValue::None)
                                .unwrap_point3s_or(Point3::ZERO);
                            let indices = params
                                .get("indices")
                                .unwrap_or(&ParameterValue::None)
                                .unwrap_ints();

                            indices
                                .chunks(3)
                                .map(|idxs| {
                                    let idxs: [i32; 3] = idxs.try_into().unwrap();
                                    Shape::Triangle(Triangle::new(
                                        idxs.map(|i| ps[i as usize]),
                                        idxs.map(|i| ns[i as usize]),
                                        idxs.map(|i| uvs[i as usize]),
                                    ))
                                })
                                .collect()
                        }
                        _ => {
                            warnln!(" unsupported shape kind {kind} at line {l}");
                            vec![]
                        }
                    };
                    if let Some(radiance) = state.area_light {
                        sb.area_lights(AreaLight::multi_new(
                            shapes
                                .into_iter()
                                .map(|s| {
                                    Primitive::new(
                                        s,
                                        0,
                                        None,
                                        Some(state.transform),
                                        MediumInterface::none(),
                                    )
                                })
                                .collect(),
                            radiance,
                        ));
                    } else {
                        sb.primitives(
                            shapes,
                            state.material.clone(),
                            Some(state.transform),
                            MediumInterface::none(),
                        );
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
                        errorln!("Named material {name} at line {l} is not a valid material.");
                    }
                }
                "NamedMaterial" => {
                    let name = next!();
                    let name = &name[1..name.len() - 1];
                    if let Some(mat) = state.named_materials.get(name) {
                        state.material = mat.clone();
                    } else {
                        errorln!("getting unknown named material '{name}'");
                    }
                }

                "Texture" => {
                    let name = next!().trim_matches('"');
                    let typ = next!().trim_matches('"');
                    let class = next!().trim_matches('"');
                    let params = parse_params!();
                    assert!(
                        class == "imagemap",
                        "imagemap is the only texture class supported currently. line {l}"
                    );
                    let texture = match typ {
                        "spectrum" => {
                            let filename = params.get("filename").unwrap().unwrap_string();
                            let filename = filename.trim().trim_matches('"');
                            let filename = if !Path::new(filename).is_absolute() {
                                path_prefix.join(filename)
                            } else {
                                Path::new(filename).to_path_buf()
                            };
                            UntypedTexture::Spectral(SpectralTexture::Image(
                                ImageTexture::from_path(Path::new(&filename)),
                            ))
                        }
                        // TODO: floats
                        _ => {
                            panic!("unsupported image type {typ} at line {l}");
                        }
                    };

                    state.named_textures.insert(name.to_owned(), texture);
                    // dbg!(name, typ, class, params);
                    // panic!();
                }
                "WorldBegin" => {
                    //TODO: rest of the stuff that should happen here..
                    state.transform = Transform3::identity()
                }
                // "Accelerator" | "CoordinateSystem" | "CoordSysTransform" | "Include" | "Import"
                _ => {
                    warnln!("unsupported directive {t} at line {l}");
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
