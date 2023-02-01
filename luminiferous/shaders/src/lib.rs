#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::spirv;

use shared::{
    glam::*,
    integrators::{Integrator, SimpleIntegrator},
    ShaderConstants,
};

#[spirv(fragment)]
pub fn fs_main(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let integrator = SimpleIntegrator {};
    *output = integrator.render_fragment(in_frag_coord.xy().as_ivec2());
}

#[spirv(vertex)]
pub fn vs_main(#[spirv(vertex_index)] id: i32, #[spirv(position, invariant)] out_pos: &mut Vec4) {
    let uv = vec2(((id << 1) & 2) as f32, (id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}
