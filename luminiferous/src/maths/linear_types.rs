// This file exists to deal with point/vector/extent/etc differences
// TODO: use nti to make these type differences concrete

use glam::*;

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

pub type Matrix3 = Mat3;

// TODO: rewrite all this to be custom and just put these on the impl
pub fn permute_v3(v: Vector3, x: usize, y: usize, z: usize) -> Vector3 {
    Vector3::new(v[x], v[y], v[z])
}

pub fn max_dimension_v3(v: Vector3) -> usize {
    if v.x > v.y && v.x > v.z {
        0
    } else if v.y > v.z {
        1
    } else {
        2
    }
}

pub fn face_forward(n: Normal3, v: Vector3) -> Normal3 {
    if 0.0 > n.dot(v) {
        -n
    } else {
        n
    }
}

pub fn spherical_direction(sin_theta: f32, cos_theta: f32, phi: f32) -> Vector3 {
    Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta)
}

pub fn spherical_direction_in(
    sin_theta: f32,
    cos_theta: f32,
    phi: f32,
    x: Vector3,
    y: Vector3,
    z: Vector3,
) -> Vector3 {
    sin_theta * phi.cos() * x + sin_theta * phi.sin() * y + cos_theta * z
}
