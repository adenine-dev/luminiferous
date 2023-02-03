use glam::Vec3;
use libm::sqrtf;

use crate::Ray;

pub struct Sphere {
    pub origin: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(origin: Vec3, radius: f32) -> Self {
        Self { origin, radius }
    }

    pub fn intersect(&self, ray: Ray) -> f32 {
        let oc = ray.origin - self.origin;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let d = b * b - 4.0 * a * c;
        if d < 0.0 {
            -1.0
        } else {
            (-b - sqrtf(d)) / (2.0 * a)
        }
    }
}
