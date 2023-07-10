use bitflags::bitflags;
use enum_dispatch::enum_dispatch;

use crate::{
    maths::{Point2, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

mod lambertian;
pub use lambertian::*;

mod conductor;
pub use conductor::*;

mod dielectric;
pub use dielectric::*;

mod util;
pub use util::*;

pub struct BsdfSample {
    pub wo: Vector3,

    pub spectrum: Spectrum,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BsdfFlags: u32 {
        /// Empty flags
        const None = 1 << 0;

        // Lobes
        const DiffuseReflection = 1 << 1;

        const DeltaReflection = 1 << 2;
        const DeltaTransmission = 1 << 3;

        // Compound
        const Diffuse = Self::DiffuseReflection.bits();
        const Smooth = Self::Diffuse.bits();
        const Delta = Self::DeltaReflection.bits() | Self::DeltaTransmission.bits();
    }
}

#[enum_dispatch]
pub trait BsdfT {
    fn sample(&self, wi: Vector3, si: &SurfaceInteraction, u: Point2) -> BsdfSample;

    fn eval(&self, si: &SurfaceInteraction, wi: Vector3, wo: Vector3) -> Spectrum;

    fn flags(&self) -> BsdfFlags;
}

#[enum_dispatch(BsdfT)]
#[derive(Clone)]
pub enum Bsdf {
    Lambertian(Lambertian),
    Conductor(Conductor),
    Dielectric(Dielectric),
}
