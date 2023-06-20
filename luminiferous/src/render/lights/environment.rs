use crate::{
    maths::{warp, Frame3, Point2, Ray, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
    stats::STATS,
};

use super::{LightSample, LightT, Visibility};

pub struct Environment {
    radiance: Spectrum,
}

impl Environment {
    pub fn new(radiance: Spectrum) -> Self {
        STATS.lights_created.inc();

        Self { radiance }
    }
}

impl LightT for Environment {
    fn is_environment(&self) -> bool {
        true
    }

    fn l_e(&self, _wi: Vector3) -> Spectrum {
        //NOTE: currently wrong but gives fine results for now.
        // let t = 0.5 * (wi.y + 1.0);
        // t * self.radiance + (1.0 - t) * Spectrum::new(1.0, 1.0, 1.0)
        self.radiance
    }

    fn sample_li(&self, interaction: &SurfaceInteraction, u: Point2) -> LightSample {
        let frame = Frame3::new(interaction.n);
        let wi = -frame.to_local(warp::square_to_cosine_hemisphere(u).normalize());

        LightSample {
            wi,
            li: self.l_e(wi),
            visibility: Visibility {
                ray: Ray::new(interaction.p + interaction.n * 1.0e-6, wi),
                end: interaction.p + wi * 1.0e7,
            },
        }
    }
}
