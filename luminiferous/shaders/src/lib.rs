#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{
    glam::{vec2, vec4, Vec2, Vec4},
    spirv,
};

use shared::ShaderConstants;

#[spirv(fragment)]
pub fn fs_main(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let frag_coord = vec2(in_frag_coord.x, in_frag_coord.y);
    let mut uv = (frag_coord - 0.5 * vec2(constants.width as f32, constants.height as f32))
        / constants.height as f32;
    uv.y = -uv.y;
    *output = vec4(uv.x, uv.y, 0.0, 1.0);
}

#[spirv(vertex)]
pub fn vs_main(#[spirv(vertex_index)] id: i32, #[spirv(position, invariant)] out_pos: &mut Vec4) {
    let uv = vec2(((id << 1) & 2) as f32, (id & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *out_pos = pos.extend(0.0).extend(1.0);
}
