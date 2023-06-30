use enum_dispatch::enum_dispatch;

mod sphere;
pub use sphere::*;

mod triangle;
pub use triangle::*;

use crate::maths::{Bounds3, Normal3, Point2, Point3, Ray};

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
}

#[enum_dispatch(ShapeT)]
#[derive(Clone)]
pub enum Shape {
    Sphere(Sphere),
    Triangle(Triangle),
}
