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

    pub fn intersects(&self, ray: Ray, mut t_min: f32, mut t_max: f32) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / ray.d[a];
            let mut t0 = (self.min[a] - ray.o[a]) * inv_d;
            let mut t1 = (self.max[a] - ray.o[a]) * inv_d;

            if t0 > t1 {
                (t0, t1) = (t1, t0);
            }

            t_min = t_min.max(t0);
            t_max = t_max.min(t1);

            if t_min >= t_max {
                return false;
            }
        }

        true
    }
}
