use enum_dispatch::enum_dispatch;

mod sphere;
pub use sphere::*;

mod triangle;
pub use triangle::*;

use crate::maths::{Bounds3, Normal3, Point2, Point3, Ray, Transform3};

#[derive(Debug, Copy, Clone)]
pub struct ShapeIntersection {
    pub t: f32,
}

pub struct ShapeInteraction {
    pub intersection: ShapeIntersection,
    pub p: Point3,
    pub n: Normal3,
    pub uv: Point2,
}

#[enum_dispatch]
pub trait ShapeT {
    fn intersect(&self, ray: Ray) -> ShapeIntersection;

    fn get_surface_interaction(
        &self,
        ray: Ray,
        intersection: ShapeIntersection,
    ) -> ShapeInteraction;

    fn make_bounds(&self) -> Bounds3;

    /// If possible transforms the shape by the specified transform and returns true. Otherwise does not transform the shape and returns false.
    fn transform(&mut self, _transform: &Transform3) -> bool {
        false
    }
}

#[enum_dispatch(ShapeT)]
#[derive(Debug, Clone)]
pub enum Shape {
    Sphere(Sphere),
    Triangle(Triangle),
}
