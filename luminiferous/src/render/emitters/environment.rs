use crate::{
    maths::{random_in_hemisphere, Ray, Vector3},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

use super::{EmitterSample, EmitterT, Visibility};

pub struct Environment {
    radiance: Spectrum,
}

impl Environment {
    pub fn new(radiance: Spectrum) -> Self {
        Self { radiance }
    }
}

impl EmitterT for Environment {
    fn is_environment(&self) -> bool {
        true
    }

    fn l_e(&self, wi: Vector3) -> Spectrum {
        //NOTE: currently wrong but gives fine results for now.
        // let t = 0.5 * (wi.y + 1.0);
        // t * self.radiance + (1.0 - t) * Spectrum::new(1.0, 1.0, 1.0)
        self.radiance
    }

    fn sample_li(&self, interaction: &SurfaceInteraction) -> EmitterSample {
        let wi = random_in_hemisphere(interaction.n).normalize();

        EmitterSample {
            wi,
            li: self.l_e(wi),
            visibility: Visibility {
                ray: Ray::new(interaction.p, wi),
                end: interaction.p + wi * 10000.0,
            },
        }
    }
}
