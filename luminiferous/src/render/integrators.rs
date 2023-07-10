mod path;
pub use path::*;

mod vol_path;
pub use vol_path::*;

use enum_dispatch::enum_dispatch;

use crate::scene::Scene;

#[enum_dispatch]
pub trait IntegratorT {
    fn render(&self, scene: Scene);
}

#[enum_dispatch(IntegratorT)]
pub enum Integrator {
    Path(PathIntegrator),
    VolPath(VolPathIntegrator),
}
