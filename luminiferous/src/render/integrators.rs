mod whitted;
pub use whitted::*;

use enum_dispatch::enum_dispatch;

use crate::scene::Scene;

#[enum_dispatch]
pub trait IntegratorT {
    fn render(&self, scene: Scene);
}

#[enum_dispatch(IntegratorT)]
pub enum Integrator {
    Whitted(WhittedIntegrator),
}
