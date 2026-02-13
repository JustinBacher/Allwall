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

struct Uniforms {
	progress: f32,
	aspect_ratio: f32,
	center: vec2<f32>,
	surface_to_from_arr: f32,
	surface_to_to_arr: f32,
}

@group(1) @binding(0)
var<uniform> uniforms: Uniforms;

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
	// Scale X coordinate by aspect ratio to make circle circular
	// rather than oval-shaped on non-square displays
	let adjusted_coords = vec2<f32>(in.tex_coords.x * uniforms.aspect_ratio, in.tex_coords.y);
	let adjusted_center = vec2<f32>(uniforms.center.x * uniforms.aspect_ratio, uniforms.center.y);
	let dist = length(adjusted_coords - adjusted_center);
	
	// Max distance is diagonal of normalized space, adjusted for aspect ratio
	let max_dist = sqrt(uniforms.aspect_ratio * uniforms.aspect_ratio + 1.0);
	let radius = uniforms.progress * max_dist;
	let mask = step(radius, dist);

	let from_color = sample_texture(t_from, s_from, in.tex_coords, uniforms.surface_to_from_arr);
	let to_color = sample_texture(t_to, s_to, in.tex_coords, uniforms.surface_to_to_arr);

	return mix(to_color, from_color, mask);
}
