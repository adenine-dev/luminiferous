use crate::prelude::*;

use super::{ShapeInteraction, ShapeIntersection, ShapeSample, ShapeT};

#[derive(Debug, Clone)]
pub struct Sphere {
    radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        STATS.shapes_created.inc();

        Self { radius }
    }
}

impl ShapeT for Sphere {
    fn intersect(&self, ray: Ray) -> ShapeIntersection {
        let oc = ray.o;
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
        let n = p.normalize();

        let u = n.x.atan2(n.z) / (core::f32::consts::PI * 2.0) + 0.5;
        let v = n.y * 0.5 + 0.5;

        let theta = ((p.z / self.radius).clamp(-1.0, 1.0)).acos();
        let z_r = (p.x * p.x + p.y * p.y).sqrt();
        let cos_phi = p.x / z_r;
        let sin_phi = p.y / z_r;
        let dp_du = Vector3::new(
            -core::f32::consts::TAU * p.y,
            core::f32::consts::TAU * p.x,
            0.0,
        );
        let dp_dv = (core::f32::consts::PI)
            * Vector3::new(p.z * cos_phi, p.z * sin_phi, -self.radius * theta.sin());

        ShapeInteraction {
            p,
            intersection,
            n,
            uv: Point2::new(u, v),
            dp_du,
            dp_dv,
            // shading_frame: Frame3::new(n),
        }
    }

    fn make_bounds(&self) -> Bounds3 {
        Bounds3::new(Vector3::splat(-self.radius), Vector3::splat(self.radius))
    }

    fn area(&self) -> f32 {
        4.0 * core::f32::consts::PI * self.radius * self.radius
    }

    fn sample(&self, u: Point2) -> ShapeSample {
        let s = warp::square_to_uniform_sphere(u);
        let p = self.radius * s;
        let n = s;

        ShapeSample { p, n }
    }
}
