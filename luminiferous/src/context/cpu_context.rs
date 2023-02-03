use shared::{
    glam::*,
    integrators::{Integrate, SimpleIntegrator},
};

use super::{Context, RenderResult};
use crate::Config;

pub struct CpuContext {
    config: Config,
}

impl CpuContext {
    pub fn new(config: Config) -> Self {
        CpuContext { config }
    }
}

impl Context for CpuContext {
    fn render(&self) -> RenderResult {
        let mut buffer =
            Vec::with_capacity(self.config.width as usize * self.config.height as usize);

        let integrator =
            SimpleIntegrator::new(vec2(self.config.width as f32, self.config.height as f32));

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                buffer.push(integrator.render_fragment(ivec2(x as i32, y as i32)));
            }
        }

        Ok(super::RenderOutput {
            image_data: buffer
                .iter()
                .flat_map(|c| c.to_array().map(|x| x.to_le_bytes()))
                .flatten()
                .collect(),
        })
    }
}
