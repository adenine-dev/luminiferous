use glam::*;

pub trait Integrator {
    fn render_fragment(&self, fragment: IVec2) -> Vec4;
}

mod simple;
pub use simple::*;
