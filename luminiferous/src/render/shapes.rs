use enum_dispatch::enum_dispatch;

mod sphere;
pub use sphere::*;

mod triangle;
pub use triangle::*;

use crate::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct ShapeIntersection {
    pub t: f32,
}

pub struct ShapeInteraction {
    pub intersection: ShapeIntersection,
    pub p: Point3,
    pub n: Normal3,
    pub uv: Point2,

    pub dp_du: Vector3,
    pub dp_dv: Vector3,
    // pub shading_frame: Frame3,
}

pub struct ShapeSample {
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

    fn make_bounds(&self) -> Bounds3;

    /// Returns the surface area of the shape.
    fn area(&self) -> f32;

    fn sample(&self, u: Point2) -> ShapeSample;

    /// If possible transforms the shape by the specified transform and returns true.
    /// Otherwise does not transform the shape and returns false.
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
