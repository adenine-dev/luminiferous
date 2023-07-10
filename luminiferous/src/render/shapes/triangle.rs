use crate::{
    maths::{max_dimension_v3, permute_v3, Bounds3, Normal3, Point2, Point3, Ray, Transform3},
    stats::STATS,
};

use super::{ShapeInteraction, ShapeIntersection, ShapeT};

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
        let a_t = self.v[0] - ray.o;
        let b_t = self.v[1] - ray.o;
        let c_t = self.v[2] - ray.o;

        let kz = max_dimension_v3(ray.d.abs());
        let kx = (kz + 1) % 3;
        let ky = (kx + 1) % 3;
        let d = permute_v3(ray.d, kx, ky, kz);
        let mut a_t = permute_v3(a_t, kx, ky, kz);
        let mut b_t = permute_v3(b_t, kx, ky, kz);
        let mut c_t = permute_v3(c_t, kx, ky, kz);

        let s_x = -d.x / d.z;
        let s_y = -d.y / d.z;
        let s_z = d.z.recip();
        a_t.x += s_x * a_t.z;
        a_t.y += s_y * a_t.z;
        b_t.x += s_x * b_t.z;
        b_t.y += s_y * b_t.z;
        c_t.x += s_x * c_t.z;
        c_t.y += s_y * c_t.z;

        let e0 = b_t.x * c_t.y - b_t.y * c_t.x;
        let e1 = c_t.x * a_t.y - c_t.y * a_t.x;
        let e2 = a_t.x * b_t.y - a_t.y * b_t.x;

        let det = e0 + e1 + e2;

        if ((e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0)) || det == 0.0
        {
            return ShapeIntersection { t: -1.0 };
        }

        a_t.z *= s_z;
        b_t.z *= s_z;
        c_t.z *= s_z;
        let t_scaled = e0 * a_t.z + e1 * b_t.z + e2 * c_t.z;

        if (det < 0.0 && (t_scaled >= 0.0/*|| t_scaled < t_max * det*/))
            || (det > 0.0 && (t_scaled <= 0.0/*|| t_scaled > t_max * det*/))
        {
            return ShapeIntersection { t: -1.0 };
        }

        let inv_det = det.recip();
        let t = t_scaled * inv_det;

        ShapeIntersection { t }
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

        ShapeInteraction {
            intersection,
            p: ray.at(intersection.t),
            uv: b0 * self.uv[0] + b1 * self.uv[1] + b2 * self.uv[2],
            n: b0 * self.n[0] + b1 * self.n[1] + b2 * self.n[2],
            // TODO: option for flat shading
            // n: (self.v[1] - self.v[0])
            //     .cross(self.v[2] - self.v[0])
            //     .normalize(),
        }
    }

    fn make_bounds(&self) -> Bounds3 {
        Bounds3::new(self.v[0], self.v[1])
            .expand(self.v[2])
            .pad(1e-6) // deal with the case that the bounds have zero volume
    }

    fn transform(&mut self, transform: &Transform3) -> bool {
        self.v = self.v.map(|v| transform.transform_point(v));
        self.n = self.n.map(|n| transform.transform_normal(n));

        true
    }
}
