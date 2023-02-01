#![feature(async_closure)]

mod context;
use std::{error::Error, path::PathBuf};

use context::*;

#[derive(Clone)]
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub gpu: bool,
    pub output_path: PathBuf,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let context: Box<dyn Context> = if config.gpu {
        Box::new(GpuContext::new(config.clone()))
    } else {
        Box::new(CpuContext::new(config.clone()))
    };

    let result = context.render()?;

    image::save_buffer(
        config.output_path,
        &result.image_data,
        config.width,
        config.height,
        image::ColorType::Rgba32F,
    )?;

    Ok(())
}
