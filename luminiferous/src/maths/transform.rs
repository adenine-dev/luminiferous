use crate::maths::*;

#[derive(Debug)]
pub struct Transform3 {
    matrix: Matrix4,
    inverse_t: Matrix4,
}

impl Transform3 {
    pub fn new(matrix: Matrix4) -> Self {
        Self {
            matrix,
            inverse_t: matrix.inverse().transpose(),
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

    pub fn transform_normal(&self, n: Vector3) -> Vector3 {
        (self.inverse_t * n.extend(0.0)).truncate()
    }

    pub fn transform_ray(&self, r: Ray) -> Ray {
        let o = self.transform_point(r.o);
        let d = self.transform_vector(r.d);

        Ray::new(o, d)
    }
}

pub struct Transform2 {
    matrix: Matrix3,
}

impl Transform2 {
    pub fn new(matrix: Matrix3) -> Self {
        Self { matrix }
    }

    pub fn transform_point(&self, p: Point2) -> Point2 {
        let transformed = self.matrix * p.extend(1.0);
        // if transformed.z == 1.0 {
        transformed.truncate()
        // } else {
        //     transformed.truncate() / transformed.z
        // }
    }

    pub fn transform_vector(&self, v: Vector2) -> Vector2 {
        (self.matrix * v.extend(0.0)).truncate()
    }
}
