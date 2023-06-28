#![feature(let_chains)]
#![feature(iter_partition_in_place)]

pub mod core;
pub mod maths;
pub mod stats;

mod render;
pub use render::*;

mod context;
pub use context::*;
