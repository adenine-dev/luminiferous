use crate::prelude::*;
use crate::{
    primitive::Interaction,
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

        Self { radiance }
    }
}

impl LightT for Environment {
    fn is_environment(&self) -> bool {
        true
    }

    fn l_e(&self, wi: Vector3) -> Spectrum {
        // let wi = ((Matrix4::from_axis_angle(Vector3::Y, -core::f32::consts::FRAC_PI_3 - 0.2)
        //     * wi.extend(0.0))
        // .truncate())
        // .normalize();
        self.radiance.eval_uv(
            Point2::new(
                -wi.z.atan2(wi.x) / core::f32::consts::TAU,
                -wi.y.asin() / core::f32::consts::PI,
            ) + Vector2::splat(0.5),
        )
    }

    fn sample_li(&self, interaction: &Interaction, u: Point2) -> LightSample {
        let frame = Frame3::new(interaction.n);
        let mut wo = -frame.to_local(warp::square_to_uniform_hemisphere(u).normalize());
        if wo.dot(interaction.n) < 0.0 {
            wo = -wo;
        }

        let visibility_ray = interaction.spawn_ray(wo);

        LightSample {
            wo,
            li: self.l_e(wo),
            visibility: Visibility {
                ray: visibility_ray,
                end: visibility_ray.at(1.0e7),
            },
        }
    }
}
