mod sphere;
use enum_dispatch::enum_dispatch;
pub use sphere::*;

use crate::maths::{Normal3, Point3, Ray, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct ShapeIntersection {
    pub t: f32,
}

pub struct ShapeInteraction {
    pub intersection: ShapeIntersection,
    pub p: Point3,
    pub n: Normal3,
}

#[enum_dispatch]
pub trait ShapeT {
    fn intersect(&self, ray: Ray) -> ShapeIntersection;

    fn get_surface_interaction(
        &self,
        ray: Ray,
        intersection: ShapeIntersection,
    ) -> ShapeInteraction;
}

#[enum_dispatch(ShapeT)]
pub enum Shape {
    Sphere(Sphere),
}
