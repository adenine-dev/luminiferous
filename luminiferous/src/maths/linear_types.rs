// This file exists to deal with point/vector/extent/etc differences
// TODO: use nti to make these type differences concrete

use glam::*;
use rand::prelude::*;

pub type Point2 = Vec2;
pub type Point3 = Vec3;

pub type Vector2 = Vec2;
pub type Vector3 = Vec3;

pub type Normal2 = Vec2;
pub type Normal3 = Vec3;

pub type Extent2 = Vec2;
pub type Extent3 = Vec3;

pub type IPoint2 = IVec2;
pub type IPoint3 = IVec3;

pub type IVector2 = IVec2;
pub type IVector3 = IVec3;

pub type IExtent2 = IVec2;
pub type IExtent3 = IVec3;

pub type UPoint2 = UVec2;
pub type UPoint3 = UVec3;

pub type UVector2 = UVec2;
pub type UVector3 = UVec3;

pub type UExtent2 = UVec2;
pub type UExtent3 = UVec3;

pub type Matrix4 = Mat4;

pub fn random_in_unit_sphere() -> Vector3 {
    let mut rng = rand::thread_rng();
    let side = rand::distributions::Uniform::new(-1.0, 1.0);

    loop {
        let ret = Vector3::new(
            side.sample(&mut rng),
            side.sample(&mut rng),
            side.sample(&mut rng),
        );
        if ret.length_squared() <= 1.0 {
            return ret;
        }
    }
}

pub fn random_in_hemisphere(n: Vector3) -> Vector3 {
    let in_unit_sphere = random_in_unit_sphere();
    if in_unit_sphere.dot(n) > 0.0 {
        in_unit_sphere
    } else {
        -in_unit_sphere
    }
}
