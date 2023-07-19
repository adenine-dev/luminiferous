use crate::prelude::*;
use crate::spectra::SpectrumT;
use crate::{
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    textures::{SpectralTexture, TextureT},
};

use super::{LightSample, LightT, Visibility};

pub struct Environment {
    radiance: SpectralTexture,
}

impl Environment {
    pub fn new(radiance: SpectralTexture) -> Self {
        STATS.lights_created.inc();
        let extent = radiance.extent();

        Self { radiance }
    }
}

impl LightT for Environment {
    fn is_environment(&self) -> bool {
        true
    }

    fn l_e(&self, wi: Vector3) -> Spectrum {
        let wi = ((Matrix4::from_axis_angle(Vector3::Y, -core::f32::consts::FRAC_PI_3 - 0.2)
            * wi.extend(0.0))
        .truncate())
        .normalize();
        self.radiance.eval_uv(
            Point2::new(
                -wi.z.atan2(wi.x) / core::f32::consts::TAU,
                -wi.y.asin() / core::f32::consts::PI,
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
                ray: Ray::new(p + n * 1e-6, wi),
                end: p + wi * 1.0e7,
            },
        }
    }
}
