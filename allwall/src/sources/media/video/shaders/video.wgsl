struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(in.position, 1.0),
        in.tex_coords,
    );
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> surface_to_video_arr: f32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = select(
        vec2<f32>(surface_to_video_arr, 1.0),
        vec2<f32>(1.0, 1.0 / surface_to_video_arr),
        surface_to_video_arr > 1.0,
    );

    return textureSample(
        t_diffuse,
        s_diffuse,
        in.tex_coords * scale + 0.5 * (vec2<f32>(1.0) - scale),
    );
}
