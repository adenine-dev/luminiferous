use bitflags::bitflags;
use enum_dispatch::enum_dispatch;

use crate::prelude::*;
use crate::{primitive::SurfaceInteraction, spectra::Spectrum};

mod lambertian;
pub use lambertian::*;

mod conductor;
pub use conductor::*;

mod dielectric;
pub use dielectric::*;

mod measured;
pub use measured::*;

mod null;
pub use null::*;

mod util;
pub use util::*;

pub struct BsdfSample {
    pub wo: Vector3,
    pub sampled: BsdfFlags,
    pub spectrum: Spectrum,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct BsdfFlags: u32 {
        // Empty flags
        const None = 1 << 0;

        // special case for the null bsdf
        const Null = 1 << 1;

        // Lobes
        const DiffuseReflection = 1 << 2;

        const DeltaReflection = 1 << 3;
        const DeltaTransmission = 1 << 4;

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
#[derive(Debug, Clone)]
pub enum Bsdf {
    Lambertian(Lambertian),
    Conductor(Conductor),
    Dielectric(Dielectric),
    Measured(MeasuredBsdf),
    Null(NullBsdf),
}
