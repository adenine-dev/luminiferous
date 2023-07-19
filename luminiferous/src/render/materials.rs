use enum_dispatch::enum_dispatch;

mod direct;
pub use direct::*;

mod mix;
pub use mix::*;

use crate::prelude::*;
use crate::{
    bsdfs::{BsdfFlags, BsdfSample},
    primitive::SurfaceInteraction,
    spectra::Spectrum,
};

pub(crate) fn make_frame(interaction: &SurfaceInteraction) -> Frame3 {
    // currently using Gram-Schmidt orthogonalization but maybe deal with mesh tangents?

    if interaction.dp_du.x == 0.0 && interaction.dp_du.y == 0.0 && interaction.dp_du.z == 0.0 {
        // handle singularity
        return Frame3::new(interaction.n);
    }

    let s =
        -(interaction.n * -interaction.n.dot(interaction.dp_du) + interaction.dp_du).normalize();
    Frame3 {
        n: interaction.n,
        s,
        t: interaction.n.cross(s),
    }
}

#[enum_dispatch]
pub trait MaterialT {
    fn sample(&self, wo_world: Vector3, interaction: &SurfaceInteraction, u: Point2) -> BsdfSample;

    fn eval(&self, si: &SurfaceInteraction, wi_world: Vector3, wo_world: Vector3) -> Spectrum;

    fn bsdf_flags(&self) -> BsdfFlags;
}

#[enum_dispatch(MaterialT)]
#[derive(Debug, Clone)]
pub enum Material {
    Direct(DirectMaterial),
    Mix(MixMaterial),
}
