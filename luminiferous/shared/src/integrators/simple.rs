use glam::*;

use crate::{shapes::Sphere, Ray};

use super::Integrate;

#[derive(Clone, Copy)]
pub struct SimpleIntegrator {
    size: Vec2,
}

impl SimpleIntegrator {
    pub fn new(size: Vec2) -> Self {
        SimpleIntegrator { size }
    }
}

impl Integrate for SimpleIntegrator {
    fn render_fragment(&self, pixel: IVec2) -> Vec4 {
        let uv = (pixel.as_vec2() / (self.size - Vec2::splat(1.0))).extend(0.0);
        let aspect_ratio = self.size.x / self.size.y;
        let size = vec3(aspect_ratio * 2.0, 2.0, 0.0);
        let origin = Vec3::splat(0.0);
        let focal_length = 1.0;
        let llc = origin - size / 2.0 - vec3(0.0, 0.0, focal_length);
        let pixel_ray = Ray::new(origin, (llc + uv * size - origin).normalize());

        let sphere = Sphere::new(vec3(0.0, 0.0, -2.0), 0.5);
        if sphere.intersect(pixel_ray) > 0.0 {
            vec4(1.0, 0.0, 1.0, 1.0)
        } else {
            uv.extend(1.0)
        }
    }
}
