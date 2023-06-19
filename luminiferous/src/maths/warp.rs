use super::{Point2, Vector3};

pub fn square_to_uniform_disk_concentric(u: Point2) -> Point2 {
    let u = 2.0 * u - Point2::splat(1.0);
    if u.x == 0.0 && u.y == 0.0 {
        return Point2::splat(0.0);
    }

    let (r, theta) = if u.x.abs() > u.y.abs() {
        (u.x, core::f32::consts::FRAC_PI_4 * (u.y / u.x))
    } else {
        (
            u.y,
            core::f32::consts::FRAC_PI_2 - core::f32::consts::FRAC_PI_4 * (u.x / u.y),
        )
    };

    r * Point2::new(theta.cos(), theta.sin())
}

pub fn square_to_uniform_hemisphere(u: Point2) -> Vector3 {
    let z = u[0];
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = 2.0 * core::f32::consts::PI * u[1];
    Vector3::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn square_to_cosine_hemisphere(u: Point2) -> Vector3 {
    let d = square_to_uniform_disk_concentric(u);
    let z = (1.0 - d.x * d.x - d.y * d.y).max(0.0).sqrt();
    Vector3::new(d.x, d.y, z)
}

//TODO: maybe cosine hemisphere?