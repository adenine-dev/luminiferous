use bitflags::bitflags;
use enum_dispatch::enum_dispatch;

use crate::prelude::*;
use crate::{primitive::SurfaceInteraction, spectra::Spectrum};

mod lambertian;
pub use lambertian::*;

mod mirror;
pub use mirror::*;

mod dielectric;
pub use dielectric::*;

mod conductor;
pub use conductor::*;

mod plastic;
pub use plastic::*;

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
        const GlossyReflection = 1 << 3;

        const DeltaReflection = 1 << 4;
        const DeltaTransmission = 1 << 5;

        // Compound
        const Diffuse = Self::DiffuseReflection.bits();
        const Glossy = Self::GlossyReflection.bits();
        const Smooth = Self::Diffuse.bits() | Self::Glossy.bits();
        const Delta = Self::DeltaReflection.bits() | Self::DeltaTransmission.bits();
    }
}

#[enum_dispatch]
pub trait BsdfT {
    fn sample(&self, wi: Vector3, si: &SurfaceInteraction, u1: f32, u2: Point2) -> BsdfSample;

    fn eval(&self, si: &SurfaceInteraction, wi: Vector3, wo: Vector3) -> Spectrum;

    fn flags(&self) -> BsdfFlags;
}

#[enum_dispatch(BsdfT)]
#[derive(Debug, Clone)]
pub enum Bsdf {
    Lambertian(Lambertian),
    Mirror(MirrorBsdf),
    Dielectric(Dielectric),
    Measured(MeasuredBsdf),
    Null(NullBsdf),
    Plastic(PlasticBsdf),
    Conductor(ConductorBsdf),
}
