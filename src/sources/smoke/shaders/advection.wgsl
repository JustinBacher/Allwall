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
    time: f32,
    mouse: vec2<f32>,
    mouse_prev: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(in.position, 1.0),
        in.tex_coords,
    );
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

fn noise(p: vec3<f32>) -> f32 {
	let ip = floor(p);
    let f = p - ip;
    let s = vec3<f32>(7.0, 157.0, 113.0);
    let h = vec4<f32>(0.0, s.yz, s.y + s.z) + dot(ip, s);
    let f2 = f * f * (3.0 - 2.0 * f);
    let h2 = mix(fract(sin(h) * 43758.5), fract(sin(h + s.xxxx) * 43758.5), f2.x);
    let h3 = mix(h2.xz, h2.yw, f2.y);
    return mix(h3.x, h3.y, f2.z);
}

fn fbm(p: vec3<f32>, octave_num: i32) -> vec2<f32> {
	var acc: vec2<f32> = vec2<f32>(0.0);
	var freq: f32 = 1.0;
	var amp: f32 = 0.5;
    let shift = vec3<f32>(100.0);
	var fp = p;
	for (var i: i32 = 0; i < octave_num; i++) {
		acc = acc + vec2<f32>(noise(fp), noise(fp + vec3<f32>(0.0, 0.0, 10.0))) * amp;
        fp = fp * 2.0 + shift;
        amp = amp * 0.5;
	}
	return acc;
}

@group(1) @binding(0)
var velocity_texture: texture_2d<f32>;

@group(1) @binding(1)
var velocity_sampler: sampler;

fn sample_velocity(coord: vec2<f32>) -> vec3<f32> {
	let veld = textureSample(velocity_texture, velocity_sampler, coord / uniforms.resolution);
	let left = textureSample(velocity_texture, velocity_sampler, (coord + vec2<f32>(-1.0, 0.0)) / uniforms.resolution).r;
	let right = textureSample(velocity_texture, velocity_sampler, (coord + vec2<f32>(1.0, 0.0)) / uniforms.resolution).r;
	let bottom = textureSample(velocity_texture, velocity_sampler, (coord + vec2<f32>(0.0, -1.0)) / uniforms.resolution).r;
	let top = textureSample(velocity_texture, velocity_sampler, (coord + vec2<f32>(0.0, 1.0)) / uniforms.resolution).r;
	let grad = vec2<f32>(right - left, top - bottom) * 0.5;
    return vec3<f32>(veld.xy - grad, veld.z);
}

const DISSIPATION: f32 = 0.95;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let frag_coord = in.tex_coords * uniforms.resolution;

	var velocity = sample_velocity(frag_coord).xy;
	let veld = sample_velocity(frag_coord - DISSIPATION * velocity);
	var density = veld.z;
	velocity = veld.xy;

	let uv = (2.0 * frag_coord - uniforms.resolution) / uniforms.resolution.y;
	let detail_noise = fbm(vec3<f32>(uv * 40.0, uniforms.time * 0.5 + 30.0), 7) - vec2<f32>(0.5);
	velocity = velocity + detail_noise * 0.2;
	density = density + length(detail_noise) * 0.01;

	let injection_noise = fbm(vec3<f32>(uv * 1.5, uniforms.time * 0.1 + 30.0), 7) - vec2<f32>(0.5);
	velocity = velocity + injection_noise * 0.1;
	density = density + max((length(injection_noise) * 0.04), 0.0);

	let mouse_uv = uniforms.mouse / uniforms.resolution;
	let mouse_prev_uv = uniforms.mouse_prev / uniforms.resolution;
	let mouse_delta = mouse_uv - mouse_prev_uv;

	let mouse_influence_radius = 0.1;
	let dist = distance(uv, mouse_uv);
	if (dist < mouse_influence_radius && length(mouse_delta) > 0.0001) {
		let influence = (mouse_influence_radius - dist) / mouse_influence_radius;
		velocity = velocity - mouse_delta * influence * 15.0;
		density = max(0.0, density - influence * 0.1);
	}

	density = min(1.0, density);
	density = density * 0.99;

    let vignette_q = frag_coord / uniforms.resolution;
	let vignette = mix(1.0, pow(16.0 * vignette_q.x * vignette_q.y * (1.0 - vignette_q.x) * (1.0 - vignette_q.y), 1.0), 0.02);
	let veld_final = vec3<f32>(velocity, density) * vignette;

	return vec4<f32>(veld_final, 1.0);
}
