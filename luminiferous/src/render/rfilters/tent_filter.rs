use crate::maths::Vector2;

use super::RFilterT;

#[derive(Debug, Clone)]
pub struct TentFilter {
    radius: Vector2,
}

impl TentFilter {
    pub fn new(radius: Vector2) -> Self {
        Self { radius }
    }
}

impl RFilterT for TentFilter {
    fn eval(&self, p: Vector2) -> f32 {
        (self.radius.x - p.x.abs()).max(0.0) * (self.radius.y - p.y.abs()).max(0.0)
    }

    fn get_radius(&self) -> Vector2 {
        self.radius
    }
}
