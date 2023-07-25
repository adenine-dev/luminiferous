use crate::{
    prelude::*,
    primitive::SurfaceInteraction,
    spectra::{Spectrum, SpectrumT},
    textures::{SpectralTexture, TextureT},
};

use super::{fresnel, fresnel_diffuse_reflectance, reflect, BsdfFlags, BsdfSample, BsdfT};

#[derive(Debug, Clone)]
pub struct PlasticBsdf {
    diffuse_reflectance: SpectralTexture,
    eta: f32,
    fdr_i: f32,
    // fdr_t: f32,
    alpha: f32,
}

impl PlasticBsdf {
    pub fn new(diffuse_reflectance: SpectralTexture, eta_i: f32, eta_t: f32, alpha: f32) -> Self {
        STATS.bsdfs_created.inc();
        let eta = eta_i / eta_t;
        let (fdr_i, _fdr_t) = fresnel_diffuse_reflectance(eta);
        Self {
            diffuse_reflectance,
            eta,
            fdr_i,
            // fdr_t,
            alpha,
        }
    }
}

impl BsdfT for PlasticBsdf {
    fn eval(&self, si: &SurfaceInteraction, wi: Vector3, wo: Vector3) -> Spectrum {
        let cos_theta_i = Frame3::cos_theta(wi);
        let cos_theta_o = Frame3::cos_theta(wo);

        if cos_theta_i <= 0.0 && cos_theta_o <= 0.0 {
            return Spectrum::zero();
        }

        let r_i = fresnel(cos_theta_i, self.eta).0;
        let r_o = fresnel(cos_theta_o, self.eta).0;

        self.diffuse_reflectance.eval(si)
            * core::f32::consts::FRAC_1_PI
            * self.fdr_i
            * self.eta.powi(2).recip()
            * (1.0 - r_i)
            * (1.0 - r_o)
    }

    fn sample(&self, wi: Vector3, si: &SurfaceInteraction, u1: f32, u2: Point2) -> BsdfSample {
        let cos_theta_i = Frame3::cos_theta(wi);

        let (r_i, _, _, _) = fresnel(cos_theta_i, self.eta);

        if u1 < r_i {
            // specular
            BsdfSample {
                //FIXME: this is a mildly hacky way to do roughness, should prob fix it
                wo: (reflect(wi) + warp::square_to_uniform_sphere(u2) * self.alpha).normalize(),
                sampled: self.flags(),
                spectrum: Spectrum::splat(1.0),
            }
        } else {
            // diffuse
            let wo = warp::square_to_cosine_hemisphere(u2);
            let diffuse_reflectance = self.diffuse_reflectance.eval(si);
            let r_o = fresnel(Frame3::cos_theta(wo), self.eta).0;

            let mut spectrum = diffuse_reflectance;
            spectrum /= 1.0 - self.fdr_i;
            spectrum *= 1.0 - r_o;
            // spectrum /= self.eta.recip().powi(2) * (1.0 - self.fdr_i) * (1.0 - r_o);
            // * (1.0 - r_i) / (1.0 - r_i);

            BsdfSample {
                wo,
                sampled: self.flags(),
                spectrum,
            }
        }
    }

    fn flags(&self) -> BsdfFlags {
        BsdfFlags::GlossyReflection | BsdfFlags::DiffuseReflection
    }
}
