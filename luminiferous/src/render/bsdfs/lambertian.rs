use crate::{
    maths::{warp, Point2, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    textures::{Texture, TextureT},
};

use super::{BsdfSample, BsdfT};

pub struct Lambertian {
    reflectance: Texture,
}

impl Lambertian {
    pub fn new(reflectance: Texture) -> Self {
        Self { reflectance }
    }
}

impl BsdfT for Lambertian {
    fn sample(&self, _wi: Vector3, interaction: &SurfaceInteraction, u: Point2) -> BsdfSample {
        let wo = warp::square_to_cosine_hemisphere(u);

        BsdfSample {
            wo,
            spectrum: self.reflectance.eval(interaction),
        }
    }

    fn eval(&self, si: &SurfaceInteraction, _wi: Vector3, _wo: Vector3) -> Spectrum {
        self.reflectance.eval(si) * core::f32::consts::FRAC_1_PI
    }
}
