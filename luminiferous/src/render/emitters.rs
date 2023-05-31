use enum_dispatch::enum_dispatch;

mod environment;
pub use environment::*;

mod point;
pub use point::*;

use crate::{
    maths::{Point3, Ray, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

pub struct Visibility {
    pub ray: Ray,
    pub end: Point3,
}

pub struct EmitterSample {
    pub li: Spectrum,
    pub wi: Vector3,
    pub visibility: Visibility,
}

#[enum_dispatch]
pub trait EmitterT {
    fn is_environment(&self) -> bool;

    fn l_e(&self, wi: Vector3) -> Spectrum;

    fn sample_li(&self, interaction: &SurfaceInteraction) -> EmitterSample;
}

#[enum_dispatch(EmitterT)]
pub enum Emitter {
    Environment(Environment),
    Point(PointLight),
}
