use crate::{prelude::*, primitive::Interaction, spectra::Spectrum};

use super::{LightSample, LightT, Visibility};

pub struct Spotlight {
    p: Point3,
    radiance: Spectrum,
    cos_width: f32,
    cos_falloff_start: f32,
    world_to_light: Option<Transform3>,
}

impl Spotlight {
    pub fn new(
        p: Point3,
        radiance: Spectrum,
        width: f32,
        falloff_start: f32,
        world_to_light: Option<Transform3>,
    ) -> Self {
        STATS.lights_created.inc();
        Self {
            p: world_to_light.map(|t| t.transform_point(p)).unwrap_or(p),
            radiance,
            cos_width: width.to_radians().cos(),
            cos_falloff_start: falloff_start.to_radians().cos(),
            world_to_light,
        }
    }
}

impl LightT for Spotlight {
    fn is_environment(&self) -> bool {
        false
    }

    fn l_e(&self, wi: Vector3) -> Spectrum {
        let w_l = self
            .world_to_light
            .map(|t| t.transform_vector(wi))
            .unwrap_or(wi);
        let cos_theta = Frame3::cos_theta(w_l);
        let falloff = {
            if cos_theta < self.cos_width {
                0.0
            } else if cos_theta >= self.cos_falloff_start {
                1.0
            } else {
                let d = (cos_theta - self.cos_width) / (self.cos_falloff_start - self.cos_width);
                d.powi(4)
            }
        };

        self.radiance * falloff
    }

    fn sample_li(&self, interaction: &Interaction, _u: Point2) -> LightSample {
        let wo = (self.p - interaction.p).normalize();

        LightSample {
            wo,
            li: self.l_e(-wo) / self.p.distance_squared(interaction.p),
            visibility: Visibility {
                ray: interaction.spawn_ray(wo),
                end: self.p,
            },
        }
    }
}
