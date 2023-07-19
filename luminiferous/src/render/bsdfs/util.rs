use crate::prelude::*;

pub(crate) fn reflect(v: Vector3) -> Vector3 {
    v * Vector3::new(-1.0, -1.0, 1.0)
}

pub(crate) fn reflect_across(v: Vector3, n: Normal3) -> Vector3 {
    -v + 2.0 * v.dot(n) * n
}

pub(crate) fn refract(wi: Vector3, cos_theta_t: f32, eta_ti: f32) -> Vector3 {
    Vector3::new(-eta_ti * wi.x, -eta_ti * wi.y, cos_theta_t)
}

pub(crate) fn spherical_theta(d: Vector3) -> f32 {
    // d.z.acos() ==
    2.0 * (0.5 * ((d.x * d.x) + (d.y * d.y) + ((d.z - 1.0) * (d.z - 1.0))).sqrt()).asin()
}

// https://en.wikipedia.org/wiki/List_of_refractive_indices
pub mod ior {
    pub const VACUUM: f32 = 1.0;
    pub const AIR: f32 = 1.000273;
    pub const WATER_ICE: f32 = 1.31;
    pub const WATER: f32 = 1.333;
    pub const PYREX: f32 = 1.470;
    pub const POLYCARBONATE: f32 = 1.60;
    pub const DIAMOND: f32 = 2.417;
}
