use crate::{
    maths::{warp, Frame3, Point2, Ray, Vector2, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    stats::STATS,
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
        //NOTE: currently wrong but gives fine results for now.
        // let t = 0.5 * (wi.y + 1.0);
        // t * self.radiance + (1.0 - t) * Spectrum::new(1.0, 1.0, 1.0)

        self.radiance.eval_uv(
            Point2::new(
                -wi.z.atan2(wi.x) / core::f32::consts::TAU,
                wi.y.asin() / core::f32::consts::PI,
            ) + Vector2::splat(0.5),
        )
    }

    fn sample_li(&self, si: &SurfaceInteraction, u: Point2) -> LightSample {
        let frame = Frame3::new(si.n);
        let mut wi = -frame.to_local(warp::square_to_uniform_hemisphere(u).normalize());
        if wi.dot(si.n) < 0.0 {
            wi = -wi;
        }

        LightSample {
            wi,
            li: self.l_e(wi),
            visibility: Visibility {
                ray: Ray::new(si.p + (si.n * 1.0e-6), wi),
                end: si.p + wi * 1.0e7,
            },
        }
    }
}
