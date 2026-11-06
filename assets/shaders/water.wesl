import bevy_pbr::render::forward_io::VertexOutput;
@group(2) @binding(0) var<uniform> settings: vec4<f32>;
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.world_position.xz;
    let wave = sin(p.x * 4.5 + p.y * 2.3 + settings.w) * sin(p.y * 6.0 - settings.w * 1.3);
    let glint = pow(max(0.0, wave), 12.0);
    return vec4<f32>(settings.xyz + wave * 0.018 + glint * vec3<f32>(0.16, 0.24, 0.20), 1.0);
}
