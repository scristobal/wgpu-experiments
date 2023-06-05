

@group(0) @binding(0) var t_depth: texture_2d<f32>;
@group(0) @binding(1) var t_sample: sampler_comparison;

struct Camera {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(1) @binding(0) var<uniform> camera: Camera;




@vertex
fn vs_main(@location(0) position: vec3<f32>, @location(1) uv: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}