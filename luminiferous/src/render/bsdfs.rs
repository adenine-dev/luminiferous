use crate::{maths::Vector3, primitive::SurfaceInteraction, spectra::Spectrum};

mod lambertian;
use enum_dispatch::enum_dispatch;
pub use lambertian::*;

pub struct BsdfSample {
    pub wo: Vector3,

    pub spectrum: Spectrum,
}

#[enum_dispatch]
pub trait BsdfT {
    fn sample(&self, wi: Vector3, interaction: &SurfaceInteraction) -> BsdfSample;

    fn eval(&self, wi: Vector3, wo: Vector3) -> Spectrum;
}

#[enum_dispatch(BsdfT)]
pub enum Bsdf {
    Lambertian(Lambertian),
}
