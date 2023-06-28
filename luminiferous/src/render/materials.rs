use enum_dispatch::enum_dispatch;

mod direct;
pub use direct::*;

use crate::{
    bsdfs::{BsdfFlags, BsdfSample},
    maths::{Point2, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

#[enum_dispatch]
pub trait MaterialT {
    fn sample(&self, wo_world: Vector3, interaction: &SurfaceInteraction, u: Point2) -> BsdfSample;

    fn eval(&self, si: &SurfaceInteraction, wi_world: Vector3, wo_world: Vector3) -> Spectrum;

    fn bsdf_flags(&self) -> BsdfFlags;
}

#[enum_dispatch(MaterialT)]
#[derive(Clone)]
pub enum Material {
    Direct(DirectMaterial),
}
