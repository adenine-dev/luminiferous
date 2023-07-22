use super::{linear_types::*, Ray};

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

    pub fn extent(&self) -> UExtent2 {
        self.max - self.min
    }

    pub fn width(&self) -> u32 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> u32 {
        self.max.y - self.min.y
    }

    pub fn union(&self, other: UBounds2) -> Self {
        Self::new(self.min.min(other.min), self.max.max(other.max))
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Bounds3 {
    pub min: Point3,
    pub max: Point3,
}

impl Bounds3 {
    pub fn new(p1: Point3, p2: Point3) -> Self {
        Self {
            min: p1.min(p2),
            max: p1.max(p2),
        }
    }

    pub fn from_point(p: Point3) -> Self {
        Self { min: p, max: p }
    }

    pub fn union(&self, other: Bounds3) -> Self {
        Self::new(self.min.min(other.min), self.max.max(other.max))
    }

    pub fn expand(&self, p: Point3) -> Self {
        Self::new(self.min.min(p), self.max.max(p))
    }

    pub fn pad(&self, padding: f32) -> Self {
        Self {
            min: self.min - padding,
            max: self.max + padding,
        }
    }

    pub fn centroid(&self) -> Point3 {
        (self.min + self.max) * 0.5
    }

    pub fn diagonal(&self) -> Vector3 {
        self.max - self.min
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.diagonal();
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    pub fn offset(&self, p: Point3) -> Vector3 {
        let mut o = p - self.min;

        for c in 0..3 {
            if self.max[c] > self.min[c] {
                o[c] /= self.max[c] - self.min[c];
            }
        }

        o
    }

    pub fn max_extent_idx(&self) -> usize {
        let d = self.diagonal();
        if d.x > d.y && d.x > d.z {
            0
        } else if d.y > d.z {
            1
        } else {
            2
        }
    }

    pub fn intersects(&self, ray: Ray, t_min: f32, t_max: f32) -> bool {
        // for whatever reason this runs significantly (~15%) faster than the traditional loop method.
        // only tested on arm
        let inv_d = ray.d.recip();
        let t0 = (self.min - ray.o) * inv_d;
        let t1 = (self.max - ray.o) * inv_d;

        t_min.max(t0.min(t1).max_element()) < t_max.min(t0.max(t1).min_element())
    }
}
