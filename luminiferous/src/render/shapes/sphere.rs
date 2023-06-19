use crate::maths::{Point2, Point3, Ray};

use super::{ShapeInteraction, ShapeIntersection, ShapeT};

pub struct Sphere {
    origin: Point3,
    radius: f32,
}

impl Sphere {
    pub fn new(origin: Point3, radius: f32) -> Self {
        Self { origin, radius }
    }
}

impl ShapeT for Sphere {
    fn intersect(&self, ray: Ray) -> ShapeIntersection {
        let oc = ray.o - self.origin;
        let a = ray.d.dot(ray.d);
        let b = 2.0 * oc.dot(ray.d);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        ShapeIntersection {
            t: if discriminant > 0.0 {
                (-b - discriminant.sqrt()) / (2.0 * a)
            } else {
                -1.0
            },
        }
    }

    fn get_surface_interaction(
        &self,
        ray: Ray,
        intersection: ShapeIntersection,
    ) -> ShapeInteraction {
        let p = ray.at(intersection.t);
        let n = (p - self.origin).normalize();

        let u = n.x.atan2(n.z) / (core::f32::consts::PI * 2.0) + 0.5;
        let v = n.y * 0.5 + 0.5;

        ShapeInteraction {
            p,
            intersection,
            n,
            uv: Point2::new(u, v),
        }
    }
}
