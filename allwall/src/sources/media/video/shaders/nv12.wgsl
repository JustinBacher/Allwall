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
var t_y: texture_2d<f32>;

@group(0) @binding(1)
var s_y: sampler;

@group(0) @binding(2)
var t_uv: texture_2d<f32>;

@group(0) @binding(3)
var s_uv: sampler;

@group(1) @binding(0)
var<uniform> surface_to_video_arr: f32;

fn nv12_to_rgb(y: f32, u: f32, v: f32) -> vec3<f32> {
    let y_norm = y;
    let u_norm = u - 0.5;
    let v_norm = v - 0.5;

    let r = y_norm + 1.402 * v_norm;
    let g = y_norm - 0.344136 * u_norm - 0.714136 * v_norm;
    let b = y_norm + 1.772 * u_norm;

    return vec3<f32>(clamp(r, 0.0, 1.0), clamp(g, 0.0, 1.0), clamp(b, 0.0, 1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = select(
        vec2<f32>(surface_to_video_arr, 1.0),
        vec2<f32>(1.0, 1.0 / surface_to_video_arr),
        surface_to_video_arr > 1.0,
    );

    let uv = in.tex_coords * scale + 0.5 * (vec2<f32>(1.0) - scale);

    let y = textureSample(t_y, s_y, uv).r;
    let uv_sample = textureSample(t_uv, s_uv, uv);
    let u = uv_sample.r;
    let v = uv_sample.g;

    let rgb = nv12_to_rgb(y, u, v);

    return vec4<f32>(rgb, 1.0);
}
