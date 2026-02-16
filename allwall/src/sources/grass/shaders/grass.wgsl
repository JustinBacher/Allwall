struct GrassUniforms {
	resolution: vec2f,
	time: f32,
	wind_strength: f32,
	blade_height: f32,
	blade_spacing: f32,
	grid_size: vec2f,
	_padding: vec2f,
}

@group(0) @binding(0) var<uniform> uniforms: GrassUniforms;
@group(1) @binding(0) var wind_texture: texture_2d<f32>;
@group(1) @binding(1) var wind_sampler: sampler;

struct BladeVertex {
	@location(0) position: vec3f,
	@location(1) tex_coords: vec2f,
	@location(2) height_factor: f32,
}

struct BladeInstance {
	@location(3) grid_x: f32,
	@location(4) grid_y: f32,
	@location(5) random_seed: f32,
	@location(6) rotation: f32,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4f,
	@location(0) height_factor: f32,
	@location(1) random_seed: f32,
	@location(2) tex_coords: vec2f,
}

fn rotate2d(angle: f32) -> mat2x2f {
	let c = cos(angle);
	let s = sin(angle);
	return mat2x2f(c, -s, s, c);
}

@vertex
fn vs_main(vertex: BladeVertex, instance: BladeInstance) -> VertexOutput {
	var output: VertexOutput;
	
	let grid_pos = vec2f(instance.grid_x, instance.grid_y);
	let normalized_grid = grid_pos / uniforms.grid_size;
	
	let world_x = (normalized_grid.x - 0.5) * 2.0 * (uniforms.resolution.x / uniforms.resolution.y);
	let world_y = (normalized_grid.y - 0.5) * 2.0;
	
	let wind_uv = normalized_grid + vec2f(uniforms.time * 0.05, uniforms.time * 0.03);
	let tex_size = vec2f(textureDimensions(wind_texture, 0));
	let tex_coords = vec2i(wind_uv * tex_size) % vec2i(tex_size);
	let wind = textureLoad(wind_texture, tex_coords, 0).xy;
	
	let wind_offset = wind * uniforms.wind_strength * vertex.height_factor * vertex.height_factor;
	
	let rotated_pos = rotate2d(instance.rotation) * vertex.position.xy;
	
	let base_width = 0.008;
	let width_taper = 1.0 - vertex.height_factor * 0.7;
	let final_x = rotated_pos.x * base_width * width_taper;
	
	let height_scale = uniforms.blade_height / uniforms.resolution.y;
	
	var final_pos = vec3f(
		world_x + final_x + wind_offset.x,
		world_y + rotated_pos.y * height_scale + wind_offset.y * 0.3,
		0.0
	);
	
	output.clip_position = vec4f(final_pos.xy, 0.0, 1.0);
	output.height_factor = vertex.height_factor;
	output.random_seed = instance.random_seed;
	output.tex_coords = vertex.tex_coords;
	
	return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
	let base_green = vec3f(0.2, 0.5, 0.15);
	let tip_green = vec3f(0.4, 0.7, 0.2);
	
	let variation = (input.random_seed - 0.5) * 0.15;
	let color = mix(base_green, tip_green, input.height_factor);
	let varied_color = color + variation;
	
	let final_color = clamp(varied_color, vec3f(0.0), vec3f(1.0));
	
	let tip_fade = 1.0 - smoothstep(0.85, 1.0, input.height_factor);
	
	return vec4f(final_color, tip_fade);
}
