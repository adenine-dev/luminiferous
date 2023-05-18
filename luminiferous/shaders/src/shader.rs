#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::spirv;

use shared::{glam::*, integrators::*, ShaderConstants};

enum E {
    X(i32),
    #[allow(dead_code)]
    Y(u32),
}

#[spirv(fragment)]
pub fn fs_main(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] integrator: &Integrator,
    output: &mut Vec4,
) {
    // let e = E::X(1);
    // *output = match e {
    //     E::X(i) => vec4(i as f32, 1.0, 0.0, 1.0),
    //     E::Y(u) => vec4(u as f32, 0.0, 1.0, 1.0),
    // }

    *output = integrator.render_fragment(in_frag_coord.xy().as_ivec2());
}

#[spirv(vertex)]
pub fn vs_main(#[spirv(vertex_index)] id: i32, #[spirv(position, invariant)] out_pos: &mut Vec4) {
    let uv = vec2(((id << 1) & 2) as f32, (id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}
