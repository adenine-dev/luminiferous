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
// https://scipoly.com/technical-library/refractive-index-of-polymers-by-index
pub mod ior {
    pub const VACUUM: f32 = 1.0;
    pub const AIR: f32 = 1.000273;
    pub const WATER_ICE: f32 = 1.31;
    pub const WATER: f32 = 1.333;
    pub const PYREX: f32 = 1.470;
    pub const ACRYLIC: f32 = 1.4893;
    pub const POLYPROPYLENE: f32 = 1.49;
    pub const POLYETHYLENE: f32 = 1.51;
    pub const POLYCARBONATE: f32 = 1.60;
    pub const DIAMOND: f32 = 2.417;
}

// returns (fresnel reflection coefficient, cos_theta of the transmitted ray, eta_i, eta_t)
pub(crate) fn fresnel(cos_theta_i: f32, eta: f32) -> (f32, f32, f32, f32) {
    let entering = cos_theta_i >= 0.0;

    let (eta_i, eta_t, cos_theta_i) = if entering {
        (eta, eta.recip(), cos_theta_i)
    } else {
        (eta.recip(), eta, -cos_theta_i)
    };

    let cos_theta_t = (-(-cos_theta_i).mul_add(cos_theta_i, 1.0))
        .mul_add(eta_t * eta_t, 1.0)
        .max(0.0)
        .sqrt();

    let a_parallel =
        (-eta_i).mul_add(cos_theta_t, cos_theta_i) / eta_i.mul_add(cos_theta_t, cos_theta_i);

    let a_perpendicular =
        (-eta_i).mul_add(cos_theta_i, cos_theta_t) / eta_i.mul_add(cos_theta_i, cos_theta_t);

    let r_i = (a_perpendicular * a_perpendicular + a_parallel * a_parallel) / 2.0;

    (r_i, cos_theta_t, eta_i, eta_t)
}

#[inline]
fn fdr_d_eon_irving(inv_eta: f32) -> f32 {
    let inv_eta_2 = inv_eta * inv_eta;
    let inv_eta_3 = inv_eta_2 * inv_eta;
    let inv_eta_4 = inv_eta_3 * inv_eta;
    let inv_eta_5 = inv_eta_4 * inv_eta;

    0.919317 - 3.4793 * inv_eta + 6.75335 * inv_eta_2 - 7.80989 * inv_eta_3 + 4.98554 * inv_eta_4
        - 1.36881 * inv_eta_5
}

#[inline]
fn fdr_egan_hilgeman(eta: f32, inv_eta: f32) -> f32 {
    -1.4399 * (eta * eta) + 0.7099 * eta + 0.6681 + 0.0636 * inv_eta
}

pub(crate) fn fresnel_diffuse_reflectance(eta: f32) -> (f32, f32) {
    let inv_eta = eta.recip();
    if eta < 1.0 {
        // Egan Hilgeman fit
        return (
            fdr_egan_hilgeman(eta, inv_eta),
            fdr_egan_hilgeman(inv_eta, eta),
        );
    }

    // d'Eon Irving fit, better but slower ;-;
    (fdr_d_eon_irving(inv_eta), fdr_d_eon_irving(eta))
}
