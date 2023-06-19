use crate::{
    film::Film,
    maths::{Point2, Ray},
};

mod projective;
pub use projective::*;

use enum_dispatch::enum_dispatch;

pub struct CameraSample {
    // in the range of the film extent
    pub p_film: Point2,
    // in the range of [0, 1)^2
    pub p_lens: Point2,
}

#[enum_dispatch]
pub trait CameraT {
    fn sample_ray(&self, sample: CameraSample) -> Ray;

    fn get_film(&self) -> &Film;
}

#[enum_dispatch(CameraT)]
#[derive(Debug)]
pub enum Camera {
    Projective(ProjectiveCamera),
}