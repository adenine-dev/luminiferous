use enum_dispatch::enum_dispatch;

mod environment;
pub use environment::*;

mod point;
pub use point::*;

mod distant;
pub use distant::*;

use crate::prelude::*;
use crate::{primitive::SurfaceInteraction, spectra::Spectrum};

pub struct Visibility {
    pub ray: Ray,
    pub end: Point3,
}

pub struct LightSample {
    pub li: Spectrum,
    pub wo: Vector3,
    pub visibility: Visibility,
}

#[enum_dispatch]
pub trait LightT {
    fn is_environment(&self) -> bool;

    fn l_e(&self, wi: Vector3) -> Spectrum;

    fn sample_li(&self, interaction: &SurfaceInteraction, u: Point2) -> LightSample;

    fn sample(&self, p: Point3, n: Normal3, u: Point2) -> LightSample;
}

#[enum_dispatch(LightT)]
pub enum Light {
    Environment(Environment),
    Point(PointLight),
    Distant(DistantLight),
}
