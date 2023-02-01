pub struct RenderOutput {
    pub image_data: Vec<u8>,
}

pub type RenderResult = Result<RenderOutput, Box<dyn Error>>;

pub trait Context {
    fn render(&self) -> RenderResult;
}

mod cpu_context;
use std::error::Error;

pub use cpu_context::*;

mod gpu_context;
pub use gpu_context::*;
