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
var t_from: texture_2d<f32>;

@group(0) @binding(1)
var s_from: sampler;

@group(0) @binding(2)
var t_to: texture_2d<f32>;

@group(0) @binding(3)
var s_to: sampler;

@group(1) @binding(0)
var<uniform> progress: f32;

@group(1) @binding(1)
var<uniform> surface_to_from_arr: f32;

@group(1) @binding(2)
var<uniform> surface_to_to_arr: f32;

fn sample_texture(tex: texture_2d<f32>, samp: sampler, coords: vec2<f32>, aspect_ratio: f32) -> vec4<f32> {
	let scale = select(
		vec2<f32>(aspect_ratio, 1.0),
		vec2<f32>(1.0, 1.0 / aspect_ratio),
		aspect_ratio > 1.0,
	);
	return textureSample(tex, samp, coords * scale + 0.5 * (vec2<f32>(1.0) - scale));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let from_color = sample_texture(t_from, s_from, in.tex_coords, surface_to_from_arr);
	let to_color = sample_texture(t_to, s_to, in.tex_coords, surface_to_to_arr);
	return mix(from_color, to_color, progress);
}
