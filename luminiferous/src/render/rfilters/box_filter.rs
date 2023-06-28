use crate::maths::Vector2;

use super::RFilterT;

#[derive(Debug, Clone)]
pub struct BoxFilter {
    radius: Vector2,
}

impl BoxFilter {
    pub fn new(radius: Vector2) -> Self {
        Self { radius }
    }
}

impl RFilterT for BoxFilter {
    fn eval(&self, _p: Vector2) -> f32 {
        1.0
    }

    fn get_radius(&self) -> Vector2 {
        self.radius
    }
}
