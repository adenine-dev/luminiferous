use enum_dispatch::enum_dispatch;

mod vector;
pub use vector::*;

mod bvh;
pub use bvh::*;

use crate::{
    maths::{Bounds3, Ray},
    primitive::{Intersection, SurfaceInteraction},
};

#[enum_dispatch]
pub trait AggregateT {
    fn intersect_p(&self, ray: Ray) -> (Option<Intersection>, usize);

    fn intersect(&self, ray: Ray) -> (Option<SurfaceInteraction>, usize) {
        let (i, u) = self.intersect_p(ray);

        (i.map(|i| i.get_surface_interaction(ray)), u)
    }

    fn bounds(&self) -> Bounds3;
}

#[enum_dispatch(AggregateT)]
pub enum Aggregate {
    Vector(Vector),
    Bvh(Bvh),
}
