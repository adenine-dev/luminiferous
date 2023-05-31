use enum_dispatch::enum_dispatch;

mod vector;
pub use vector::*;

use crate::{maths::Ray, primitive::SurfaceInteraction};

#[enum_dispatch]
pub trait AggregateT {
    fn intersect(&self, ray: Ray) -> Option<SurfaceInteraction>;
}

#[enum_dispatch(AggregateT)]
pub enum Aggregate {
    Vector(Vector),
}
