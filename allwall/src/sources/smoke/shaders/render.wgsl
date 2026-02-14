struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Uniforms {
    resolution: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(in.position, 1.0),
        in.tex_coords,
    );
}

@group(1) @binding(0)
var smoke_texture: texture_2d<f32>;

@group(1) @binding(1)
var smoke_sampler: sampler;

fn vignette(color: vec3<f32>, q: vec2<f32>, v: f32) -> vec3<f32> {
    let vignette_factor = pow(16.0 * q.x * q.y * (1.0 - q.x) * (1.0 - q.y), v);
    return color * (0.3 + 0.8 * vignette_factor);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let density = textureSample(smoke_texture, smoke_sampler, in.tex_coords).b;
	
	let base_color = vec3<f32>(0.1, 0.15, 0.2);
	let fog_color = vec3<f32>(0.8, 0.85, 0.9);
	let smoke_color = vec3<f32>(0.6, 0.7, 0.8);
	
	let color = mix(base_color, fog_color, density * 0.5);
	let final_color = mix(color, smoke_color, density * 0.8);
	
	let vignette_q = in.tex_coords;
	let final_color_vignetted = vignette(final_color, vignette_q, 0.6);
	
	return vec4<f32>(sqrt(final_color_vignetted), 1.0);
}
