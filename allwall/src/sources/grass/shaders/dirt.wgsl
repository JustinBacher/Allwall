struct DirtUniforms {
	color: vec3f,
	_padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: DirtUniforms;

struct VertexInput {
	@location(0) position: vec3f,
	@location(1) tex_coords: vec2f,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4f,
	@location(0) tex_coords: vec2f,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
	var output: VertexOutput;
	output.clip_position = vec4f(input.position.xy, 0.0, 1.0);
	output.tex_coords = input.tex_coords;
	return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
	return vec4f(uniforms.color, 1.0);
}
