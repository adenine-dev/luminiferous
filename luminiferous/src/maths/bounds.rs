use super::linear_types::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct UBounds2 {
    pub min: UPoint2,
    pub max: UPoint2,
}

impl UBounds2 {
    pub const fn new(min: UPoint2, max: UPoint2) -> Self {
        Self { min, max }
    }

    pub const fn area(&self) -> u32 {
        self.min.x.abs_diff(self.max.x) * self.min.y.abs_diff(self.max.y)
    }
}
