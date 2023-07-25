use crate::prelude::*;
use crate::{
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
    stats::STATS,
    textures::{SpectralTexture, TextureT},
};

use super::{util::reflect, BsdfFlags, BsdfSample, BsdfT};

/// A not strictly physically accurate mirror that perfectly reflects incoming rays.
#[derive(Debug, Clone)]
pub struct ConductorBsdf {
    k: SpectralTexture,
    eta: SpectralTexture,
}

impl ConductorBsdf {
    pub fn new(k: SpectralTexture, eta: SpectralTexture) -> Self {
        STATS.bsdfs_created.inc();

        Self { k, eta }
    }
}

impl BsdfT for ConductorBsdf {
    fn eval(&self, _si: &SurfaceInteraction, _wi: Vector3, _wo: Vector3) -> Spectrum {
        Spectrum::zero()
    }

    fn sample(&self, wi: Vector3, si: &SurfaceInteraction, _u1: f32, _u2: Point2) -> BsdfSample {
        let wo = reflect(wi);

        let k = self.k.eval(si);
        let eta = self.eta.eval(si);

        let cos_theta_o = Frame3::cos_theta(wo);
        let eta_k = k; // NOTE: could be divided by an eta_i value but we don't store that, and it will usually be negligible.

        let cos_theta_o2 = cos_theta_o.powi(2);
        let sin_theta_o2 = 1.0 - cos_theta_o2;
        let eta2 = eta * eta;
        let eta_k2 = eta_k * eta_k;

        let t0 = eta2 - eta_k2 - Spectrum::splat(sin_theta_o2);
        let a2plusb2 = (t0 * t0 + 4.0 * eta2 * eta_k2).sqrt();
        let t1 = a2plusb2 + Spectrum::splat(cos_theta_o2);
        let a = (0.5 * (a2plusb2 + t0)).sqrt();
        let t2 = 2.0 * cos_theta_o * a;
        let r_s = (t1 - t2) / (t1 + t2);

        let t3 = cos_theta_o2 * a2plusb2 + Spectrum::splat(sin_theta_o2 * sin_theta_o2);
        let t4 = t2 * sin_theta_o2;
        let r_p = r_s * (t3 - t4) / (t3 + t4);

        let reflectance = 0.5 * (r_p + r_s);

        BsdfSample {
            wo,
            sampled: self.flags(),
            spectrum: reflectance,
        }
    }

    fn flags(&self) -> super::BsdfFlags {
        BsdfFlags::DeltaReflection
    }
}
