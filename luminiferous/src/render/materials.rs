use enum_dispatch::enum_dispatch;

mod direct;
pub use direct::*;

use crate::{bsdfs::BsdfSample, maths::Vector3, primitive::SurfaceInteraction, spectra::Spectrum};

#[enum_dispatch]
pub trait MaterialT {
    fn sample(&self, wi: Vector3, interaction: &SurfaceInteraction) -> BsdfSample;

    fn eval(&self, wi: Vector3, wo: Vector3) -> Spectrum;
}

#[enum_dispatch(MaterialT)]
pub enum Material {
    Direct(DirectMaterial),
}
