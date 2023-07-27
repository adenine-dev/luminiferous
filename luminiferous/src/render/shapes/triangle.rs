use crate::prelude::*;

use super::{ShapeInteraction, ShapeIntersection, ShapeSample, ShapeT};

#[derive(Debug, Clone, Default)]
pub struct Triangle {
    v: [Point3; 3],
    n: [Normal3; 3],
    uv: [Point2; 3],
}

impl Triangle {
    pub fn new(v: [Point3; 3], n: [Normal3; 3], uv: [Point2; 3]) -> Self {
        STATS.shapes_created.inc();
        Self { v, n, uv }
    }
}

impl ShapeT for Triangle {
    fn intersect(&self, ray: Ray) -> ShapeIntersection {
        // uses the Möller Trumbore intersection algorithm

        const EPSILON: f32 = 1e-7;

        let edge1 = self.v[1] - self.v[0];
        let edge2 = self.v[2] - self.v[0];
        let h = ray.d.cross(edge2);
        let a = edge1.dot(h);

        if -EPSILON < a && a < EPSILON {
            return ShapeIntersection { t: -1.0 };
        }

        let f = 1.0 / a;
        let s = ray.o - self.v[0];
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) {
            return ShapeIntersection { t: -1.0 };
        }

        let q = s.cross(edge1);
        let v = f * ray.d.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return ShapeIntersection { t: -1.0 };
        }

        ShapeIntersection {
            t: f * edge2.dot(q),
        }
    }

    fn get_surface_interaction(
        &self,
        ray: Ray,
        intersection: ShapeIntersection,
    ) -> ShapeInteraction {
        let p = ray.at(intersection.t);
        let v1 = self.v[0];
        let v2 = self.v[1];
        let v3 = self.v[2];

        let f1 = v1 - p;
        let f2 = v2 - p;
        let f3 = v3 - p;

        let det = (v1 - v2).cross(v1 - v3).length();
        let b0 = f2.cross(f3).length() / det;
        let b1 = f3.cross(f1).length() / det;
        let b2 = f1.cross(f2).length() / det;

        let duv02 = self.uv[1] - self.uv[0];
        let duv12 = self.uv[2] - self.uv[0];
        let dp02 = v2 - v1;
        let dp12 = v3 - v1;

        let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        let (dp_du, dp_dv) = if determinant == 0.0 {
            let frame = Frame3::new((v3 - v1).cross(v2 - v1).normalize());
            (frame.s, frame.t)
        } else {
            let invdet = determinant.recip();
            (
                (duv12.y * dp02 - (duv02.y * dp12)) * invdet,
                (-duv12.x * dp02 + (duv02.x * dp12)) * invdet,
            )
        };

        ShapeInteraction {
            intersection,
            p: ray.at(intersection.t),
            uv: b0 * self.uv[0] + b1 * self.uv[1] + b2 * self.uv[2],
            n: face_forward(b0 * self.n[0] + b1 * self.n[1] + b2 * self.n[2], -ray.d),

            dp_du,
            dp_dv,
        }
    }

    fn make_bounds(&self) -> Bounds3 {
        Bounds3::new(self.v[0], self.v[1])
            .expand(self.v[2])
            .pad(1e-5) // deal with the case that the bounds have zero volume
    }

    fn transform(&mut self, transform: &Transform3) -> bool {
        self.v = self.v.map(|v| transform.transform_point(v));
        self.n = self.n.map(|n| transform.transform_normal(n));

        true
    }

    fn area(&self) -> f32 {
        0.5 * (self.v[1] - self.v[0])
            .cross(self.v[2] - self.v[0])
            .length()
    }

    fn sample(&self, u: Point2) -> ShapeSample {
        let (b0, b1) = warp::square_to_barycentric(u);
        let b2 = 1.0 - b0 - b1;

        ShapeSample {
            p: b0 * self.v[0] + b1 * self.v[1] + b2 * self.v[2],
            n: b0 * self.n[0] + b1 * self.n[1] + b2 * self.n[2],
        }
    }
}
