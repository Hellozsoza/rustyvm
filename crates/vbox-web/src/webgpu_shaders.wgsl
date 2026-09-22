[[stage(vertex)]]
fn vs_main([[location(0)]] position: vec2<f32>, [[location(1)]] tex_coord: vec2<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main([[location(0)]] tex_coord: vec2<f32>) -> [[location(0)]] vec4<f32> {
    return textureSample(frame_texture, frame_sampler, tex_coord);
}

[[stage(vertex)]]
fn vs_main_flat([[location(0)]] position: vec2<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main_flat() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
