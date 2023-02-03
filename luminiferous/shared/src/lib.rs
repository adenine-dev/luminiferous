#![no_std]

use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
}

pub use glam;
pub use libm;

mod math;
pub use math::*;

pub mod integrators;
pub mod shapes;
