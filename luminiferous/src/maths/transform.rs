use std::ops::Mul;

use crate::prelude::*;
use crate::primitive::SurfaceInteraction;

#[derive(Debug, Clone, Copy)]
pub struct Transform3 {
    matrix: Matrix4,
    inverse: Matrix4,
}

impl Transform3 {
    pub fn new(matrix: Matrix4) -> Self {
        Self {
            matrix,
            inverse: matrix.inverse(),
        }
    }

    pub fn identity() -> Self {
        Self::new(Matrix4::IDENTITY)
    }

    pub fn translate(v: Vector3) -> Self {
        let m = Matrix4::from_translation(v);
        Self::new(m)
    }

    pub fn rotate(euler_angles: Point3) -> Self {
        let m = Matrix4::from_euler(
            glam::EulerRot::XYZ,
            euler_angles.x,
            euler_angles.y,
            euler_angles.z,
        );
        Self::new(m)
    }

    pub fn scale(scale: Vector3) -> Self {
        let m = Matrix4::from_scale(scale);
        Self::new(m)
    }

    pub fn inverse(&self) -> Self {
        Self {
            matrix: self.inverse,
            inverse: self.matrix,
        }
    }

    pub fn transform_point(&self, p: Point3) -> Point3 {
        let transformed = self.matrix * p.extend(1.0);
        if transformed.w == 1.0 {
            transformed.truncate()
        } else {
            transformed.truncate() / transformed.w
        }
    }

    pub fn transform_vector(&self, v: Vector3) -> Vector3 {
        (self.matrix * v.extend(0.0)).truncate()
    }

    pub fn transform_normal(&self, n: Normal3) -> Normal3 {
        (self.inverse.transpose() * n.extend(0.0))
            .truncate()
            .normalize()
    }

    pub fn transform_ray(&self, r: Ray) -> Ray {
        let o = self.transform_point(r.o);
        let d = self.transform_vector(r.d);

        Ray::new(o, d)
    }

    pub fn transform_point_inv(&self, p: Point3) -> Point3 {
        let transformed = self.inverse * p.extend(1.0);
        if transformed.w == 1.0 {
            transformed.truncate()
        } else {
            transformed.truncate() / transformed.w
        }
    }

    pub fn transform_vector_inv(&self, v: Vector3) -> Vector3 {
        (self.inverse * v.extend(0.0)).truncate()
    }

    pub fn transform_normal_inv(&self, n: Normal3) -> Normal3 {
        (self.matrix.transpose() * n.extend(0.0))
            .truncate()
            .normalize()
    }

    pub fn transform_ray_inv(&self, r: Ray) -> Ray {
        let o = self.transform_point_inv(r.o);
        let d = self.transform_vector_inv(r.d);

        Ray::new(o, d)
    }

    pub fn transform_bounds(&self, bounds: Bounds3) -> Bounds3 {
        // Adapted from James Arvo's "Transforming Axis-aligned Bounding Boxes" in Graphics Gems.

        let mut min = self.matrix.col(3).truncate();
        let mut max = self.matrix.col(3).truncate();

        for i in 0..3 {
            for j in 0..3 {
                let a = self.matrix.col(i)[j] * bounds.min[j];
                let b = self.matrix.col(i)[j] * bounds.max[j];
                if a < b {
                    min[i] += a;
                    max[i] += b;
                } else {
                    min[i] += b;
                    max[i] += a;
                }
            }
        }

        Bounds3::new(min, max)
    }

    /// transforms the surface interaction by `self`. The `ray` argument is the non-transformed ray.
    pub fn transform_surface_interaction<'a>(
        &self,
        ray: Ray,
        si: SurfaceInteraction<'a>,
    ) -> SurfaceInteraction<'a> {
        SurfaceInteraction {
            primitive: si.primitive,
            t: si.t,
            p: ray.at(si.t),
            n: self.transform_normal(si.n),
            uv: si.uv,
            dp_du: self.transform_normal(si.dp_du),
            dp_dv: self.transform_normal(si.dp_dv),
        }
    }
}

impl Mul for Transform3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            matrix: self.matrix * rhs.matrix,
            inverse: rhs.inverse * self.inverse,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform2 {
    matrix: Matrix3,
}

impl Transform2 {
    pub fn new(matrix: Matrix3) -> Self {
        Self { matrix }
    }

    pub fn transform_point(&self, p: Point2) -> Point2 {
        let transformed = self.matrix * p.extend(1.0);

        transformed.truncate()
    }

    pub fn transform_vector(&self, v: Vector2) -> Vector2 {
        (self.matrix * v.extend(0.0)).truncate()
    }
}
