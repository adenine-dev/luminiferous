use enum_dispatch::enum_dispatch;

mod direct;
pub use direct::*;

use crate::prelude::*;
use crate::{
    bsdfs::{BsdfFlags, BsdfSample},
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
#[derive(Debug, Clone)]
pub enum Material {
    Direct(DirectMaterial),
}
