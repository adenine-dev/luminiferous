use crate::prelude::*;
use crate::{
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
};

use super::fresnel;
use super::{
    util::{reflect, refract},
    BsdfFlags, BsdfSample, BsdfT,
};

#[derive(Debug, Clone)]
pub struct Dielectric {
    eta: f32,
    t: Spectrum,
}

impl Dielectric {
    pub fn new(eta_i: f32, eta_t: f32, t: Spectrum) -> Self {
        STATS.bsdfs_created.inc();

        Self {
            eta: eta_i / eta_t,
            t,
        }
    }
}

impl BsdfT for Dielectric {
    fn eval(&self, _si: &SurfaceInteraction, _wi: Vector3, _wo: Vector3) -> Spectrum {
        Spectrum::zero()
    }

    fn sample(&self, wi: Vector3, _si: &SurfaceInteraction, u1: f32, u2: Point2) -> BsdfSample {
        let cos_theta_i = Frame3::cos_theta(wi);
        // let entering = cos_theta_i >= 0.0;

        // let (eta_i, eta_t, cos_theta_i) = if entering {
        //     (self.eta, self.eta.recip(), cos_theta_i)
        // } else {
        //     (self.eta.recip(), self.eta, -cos_theta_i)
        // };

        // let cos_theta_t = (-(-cos_theta_i).mul_add(cos_theta_i, 1.0))
        //     .mul_add(eta_t * eta_t, 1.0)
        //     .max(0.0)
        //     .sqrt();

        // let a_parallel =
        //     (-eta_i).mul_add(cos_theta_t, cos_theta_i) / eta_i.mul_add(cos_theta_t, cos_theta_i);

        // let a_perpendicular =
        //     (-eta_i).mul_add(cos_theta_i, cos_theta_t) / eta_i.mul_add(cos_theta_i, cos_theta_t);

        // let r_i = (a_perpendicular * a_perpendicular + a_parallel * a_parallel) / 2.0;

        let (r_i, cos_theta_t, _eta_i, eta_t) = fresnel(cos_theta_i, self.eta);

        let wo = if u1 <= r_i {
            reflect(wi)
        } else {
            refract(
                wi,
                cos_theta_t * if cos_theta_i >= 0.0 { -1.0 } else { 1.0 },
                eta_t,
            )
        };

        BsdfSample {
            wo,
            sampled: self.flags(),
            spectrum: self.t,
        }
    }

    fn flags(&self) -> BsdfFlags {
        BsdfFlags::DeltaTransmission | BsdfFlags::DeltaReflection
    }
}
