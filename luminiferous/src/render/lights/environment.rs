use crate::prelude::*;
use crate::{
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    textures::{Texture, TextureT},
};

use super::{LightSample, LightT, Visibility};

pub struct Environment {
    radiance: Texture,
}

impl Environment {
    pub fn new(radiance: Texture) -> Self {
        STATS.lights_created.inc();

        Self { radiance }
    }
}

impl LightT for Environment {
    fn is_environment(&self) -> bool {
        true
    }

    fn l_e(&self, wi: Vector3) -> Spectrum {
        self.radiance.eval_uv(
            Point2::new(
                -wi.z.atan2(-wi.x) / core::f32::consts::TAU,
                wi.y.asin() / core::f32::consts::PI,
            ) + Vector2::splat(0.5),
        )
    }

    fn sample_li(&self, si: &SurfaceInteraction, u: Point2) -> LightSample {
        self.sample(si.p, si.n, u)
    }

    fn sample(&self, p: Point3, n: Normal3, u: Point2) -> LightSample {
        let frame = Frame3::new(n);
        let mut wi = -frame.to_local(warp::square_to_uniform_hemisphere(u).normalize());
        if wi.dot(n) < 0.0 {
            wi = -wi;
        }

        LightSample {
            wo: wi,
            li: self.l_e(wi),
            visibility: Visibility {
                ray: Ray::new(p + face_forward(n, wi) * 1e-6, wi),
                end: p + wi * 1.0e7,
            },
        }
    }
}
