use crate::maths::{Ray, Vector2};

mod projective;
pub use projective::*;

use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait SensorT {
    fn sample_ray(&self, p: Vector2) -> Ray;
}

#[enum_dispatch(SensorT)]
#[derive(Debug)]
pub enum Sensor {
    ProjectiveCamera(ProjectiveCamera),
}
