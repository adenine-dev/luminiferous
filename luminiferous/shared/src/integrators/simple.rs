use glam::*;

use super::Integrator;

pub struct SimpleIntegrator {}

impl Integrator for SimpleIntegrator {
    fn render_fragment(&self, pixel: IVec2) -> Vec4 {
        if (pixel.x) % 2 == 0 {
            vec4(1.0, 0.0, 1.0, 1.0)
        } else {
            vec4(0.0, 0.0, 0.0, 1.0)
        }
    }
}
