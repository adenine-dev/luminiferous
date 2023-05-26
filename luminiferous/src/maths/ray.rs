use super::linear_types::*;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub o: Point3,
    pub d: Vector3,
}

impl Ray {
    pub fn new(o: Point3, d: Vector3) -> Self {
        Self { o, d }
    }
    pub fn at(&self, t: f32) -> Point3 {
        self.o + t * self.d
    }
}
