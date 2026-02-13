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

struct Uniforms {
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var velocity_texture: texture_2d<f32>;

@group(1) @binding(1)
var velocity_sampler: sampler;

fn texel_fetch(coord: vec2<i32>) -> vec4<f32> {
    return textureLoad(velocity_texture, coord, 0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let icoord = vec2<i32>(in.tex_coords * uniforms.resolution);
	let vel_x_left = texel_fetch(icoord + vec2<i32>(-1, 0)).r;
	let vel_x_right = texel_fetch(icoord + vec2<i32>(1, 0)).r;
	let vel_y_bottom = texel_fetch(icoord + vec2<i32>(0, -1)).g;
	let vel_y_top = texel_fetch(icoord + vec2<i32>(0, 1)).g;
	let divergence = (vel_x_right - vel_x_left + vel_y_top - vel_y_bottom) * 0.5;
	return vec4<f32>(divergence, vec3<f32>(1.0));
}
