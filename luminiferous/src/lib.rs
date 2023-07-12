#![feature(let_chains)]
#![feature(iter_partition_in_place)]
#![feature(iter_array_chunks)]

pub mod core;
pub mod loaders;
pub mod maths;
pub mod stats;

mod render;
pub use render::*;

mod context;
pub use context::*;
