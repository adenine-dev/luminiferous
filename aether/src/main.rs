#![feature(core_intrinsics)]

use luminiferous as lum;

fn main() {
    lum::run(lum::Config {
        width: 1280,
        height: 960,
        gpu: true,
        output_path: "output/image.exr".into(),
    })
    .unwrap();
}
