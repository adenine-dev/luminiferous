use crate::{
    maths::{max_dimension_v3, permute_v3, Bounds3, Point2, Point3, Ray},
    stats::STATS,
};

use super::{ShapeInteraction, ShapeIntersection, ShapeT};

#[derive(Clone, Default)]
pub struct Triangle {
    a: Point3,
    b: Point3,
    c: Point3,
}

impl Triangle {
    pub fn new(a: Point3, b: Point3, c: Point3) -> Self {
        STATS.shapes_created.inc();
        Self { a, b, c }
    }
}

impl ShapeT for Triangle {
    fn intersect(&self, ray: Ray) -> ShapeIntersection {
        let a_t = self.a - ray.o;
        let b_t = self.b - ray.o;
        let c_t = self.c - ray.o;

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
        let uvs = [
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
        ];

        let a_t = self.a - ray.o;
        let b_t = self.b - ray.o;
        let c_t = self.c - ray.o;

        let kz = max_dimension_v3(ray.d.abs());
        let kx = (kz + 1) % 3;
        let ky = (kx + 1) % 3;
        let d = permute_v3(ray.d, kx, ky, kz);
        let mut p0t = permute_v3(a_t, kx, ky, kz);
        let mut p1t = permute_v3(b_t, kx, ky, kz);
        let mut p2t = permute_v3(c_t, kx, ky, kz);
        let s_x = -d.x / d.z;
        let s_y = -d.y / d.z;

        p0t.x += s_x * p0t.z;
        p0t.y += s_y * p0t.z;
        p1t.x += s_x * p1t.z;
        p1t.y += s_y * p1t.z;
        p2t.x += s_x * p2t.z;
        p2t.y += s_y * p2t.z;

        let e0 = p1t.x * p2t.y - p1t.y * p2t.x;
        let e1 = p2t.x * p0t.y - p2t.y * p0t.x;
        let e2 = p0t.x * p1t.y - p0t.y * p1t.x;

        let det = e0 + e1 + e2;
        let inv_det = det.recip();
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;

        ShapeInteraction {
            intersection,
            p: ray.at(intersection.t),
            uv: b0 * uvs[0] + b1 * uvs[1] + b2 * uvs[2],
            n: (self.a - self.c).cross(self.b - self.c).normalize(),
        }
    }

    fn make_bounds(&self) -> Bounds3 {
        Bounds3::new(self.a, self.b).expand(self.c)
    }
}
