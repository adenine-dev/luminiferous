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
