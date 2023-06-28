use crate::maths::Vector2;

use enum_dispatch::enum_dispatch;

mod box_filter;
pub use box_filter::*;

mod tent_filter;
pub use tent_filter::*;

#[enum_dispatch]
pub trait RFilterT {
    /// Evaluates the filter at the given point and return its weight.
    ///
    /// Note that p must be within the filter's radius.
    fn eval(&self, p: Vector2) -> f32;

    /// Return the radius of the reconstruction filter.
    fn get_radius(&self) -> Vector2;
}

#[enum_dispatch(RFilterT)]
#[derive(Debug, Clone)]
pub enum RFilter {
    Box(BoxFilter),
    Tent(TentFilter),
}
