use super::{Normal3, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Frame3 {
    pub n: Normal3,
    pub s: Vector3,
    pub t: Vector3,
}

impl Frame3 {
    /// Returns (s, t) where n, s, and t form an orthonormal basis.
    #[inline]
    pub fn coordinate_system(n: Normal3) -> (Vector3, Vector3) {
        let sign = n.z.signum();
        let a = -(sign + n.z).recip();
        let b = n.x * n.y * a;

        (
            Vector3::new(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x),
            Vector3::new(b, sign + n.y * n.y * a, -n.y),
        )
    }

    pub fn new(n: Normal3) -> Self {
        let (s, t) = Self::coordinate_system(n);

        Self { n, s, t }
    }

    pub fn to_local(&self, v: Vector3) -> Vector3 {
        Vector3::new(v.dot(self.s), v.dot(self.t), v.dot(self.n))
    }

    pub fn to_world(&self, v: Vector3) -> Vector3 {
        self.n * v.z + self.t * v.y + self.s * v.x
    }

    pub fn cos_theta(v: Vector3) -> f32 {
        v.z
    }
}
