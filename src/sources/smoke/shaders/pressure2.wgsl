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
var pressure_texture: texture_2d<f32>;

@group(1) @binding(1)
var pressure_sampler: sampler;

fn texel_fetch_pre(coord: vec2<i32>) -> vec4<f32> {
    return textureLoad(pressure_texture, coord, 0);
}

fn get_pre(icoord: vec2<i32>, p: vec2<i32>) -> f32 {
    return texel_fetch_pre(icoord + p).r;
}

fn get_pre_weighted(icoord: vec2<i32>) -> f32 {
	var p: f32 = 0.0;
	p = p + 1.0 * get_pre(icoord, vec2<i32>(-10, 0));
	p = p + 10.0 * get_pre(icoord, vec2<i32>(-9, -1));
	p = p + 10.0 * get_pre(icoord, vec2<i32>(-9, 1));
	p = p + 45.0 * get_pre(icoord, vec2<i32>(-8, -2));
	p = p + 100.0 * get_pre(icoord, vec2<i32>(-8, 0));
	p = p + 45.0 * get_pre(icoord, vec2<i32>(-8, 2));
	p = p + 120.0 * get_pre(icoord, vec2<i32>(-7, -3));
	p = p + 450.0 * get_pre(icoord, vec2<i32>(-7, -1));
	p = p + 450.0 * get_pre(icoord, vec2<i32>(-7, 1));
	p = p + 120.0 * get_pre(icoord, vec2<i32>(-7, 3));
	p = p + 210.0 * get_pre(icoord, vec2<i32>(-6, -4));
	p = p + 1200.0 * get_pre(icoord, vec2<i32>(-6, -2));
	p = p + 2025.0 * get_pre(icoord, vec2<i32>(-6, 0));
	p = p + 1200.0 * get_pre(icoord, vec2<i32>(-6, 2));
	p = p + 210.0 * get_pre(icoord, vec2<i32>(-6, 4));
	p = p + 252.0 * get_pre(icoord, vec2<i32>(-5, -5));
	p = p + 2100.0 * get_pre(icoord, vec2<i32>(-5, -3));
	p = p + 5400.0 * get_pre(icoord, vec2<i32>(-5, -1));
	p = p + 5400.0 * get_pre(icoord, vec2<i32>(-5, 1));
	p = p + 2100.0 * get_pre(icoord, vec2<i32>(-5, 3));
	p = p + 252.0 * get_pre(icoord, vec2<i32>(-5, 5));
	p = p + 210.0 * get_pre(icoord, vec2<i32>(-4, -6));
	p = p + 2520.0 * get_pre(icoord, vec2<i32>(-4, -4));
	p = p + 9450.0 * get_pre(icoord, vec2<i32>(-4, -2));
	p = p + 14400.0 * get_pre(icoord, vec2<i32>(-4, 0));
	p = p + 9450.0 * get_pre(icoord, vec2<i32>(-4, 2));
	p = p + 2520.0 * get_pre(icoord, vec2<i32>(-4, 4));
	p = p + 210.0 * get_pre(icoord, vec2<i32>(-4, 6));
	p = p + 120.0 * get_pre(icoord, vec2<i32>(-3, -7));
	p = p + 2100.0 * get_pre(icoord, vec2<i32>(-3, -5));
	p = p + 11340.0 * get_pre(icoord, vec2<i32>(-3, -3));
	p = p + 25200.0 * get_pre(icoord, vec2<i32>(-3, -1));
	p = p + 25200.0 * get_pre(icoord, vec2<i32>(-3, 1));
	p = p + 11340.0 * get_pre(icoord, vec2<i32>(-3, 3));
	p = p + 2100.0 * get_pre(icoord, vec2<i32>(-3, 5));
	p = p + 120.0 * get_pre(icoord, vec2<i32>(-3, 7));
	p = p + 45.0 * get_pre(icoord, vec2<i32>(-2, -8));
	p = p + 1200.0 * get_pre(icoord, vec2<i32>(-2, -6));
	p = p + 9450.0 * get_pre(icoord, vec2<i32>(-2, -4));
	p = p + 30240.0 * get_pre(icoord, vec2<i32>(-2, -2));
	p = p + 44100.0 * get_pre(icoord, vec2<i32>(-2, 0));
	p = p + 30240.0 * get_pre(icoord, vec2<i32>(-2, 2));
	p = p + 9450.0 * get_pre(icoord, vec2<i32>(-2, 4));
	p = p + 1200.0 * get_pre(icoord, vec2<i32>(-2, 6));
	p = p + 45.0 * get_pre(icoord, vec2<i32>(-2, 8));
	p = p + 10.0 * get_pre(icoord, vec2<i32>(-1, -9));
	p = p + 450.0 * get_pre(icoord, vec2<i32>(-1, -7));
	p = p + 5400.0 * get_pre(icoord, vec2<i32>(-1, -5));
	p = p + 25200.0 * get_pre(icoord, vec2<i32>(-1, -3));
	p = p + 52920.0 * get_pre(icoord, vec2<i32>(-1, -1));
	p = p + 52920.0 * get_pre(icoord, vec2<i32>(-1, 1));
	p = p + 25200.0 * get_pre(icoord, vec2<i32>(-1, 3));
	p = p + 5400.0 * get_pre(icoord, vec2<i32>(-1, 5));
	p = p + 450.0 * get_pre(icoord, vec2<i32>(-1, 7));
	p = p + 10.0 * get_pre(icoord, vec2<i32>(-1, 9));
	p = p + 1.0 * get_pre(icoord, vec2<i32>(0, -10));
	p = p + 100.0 * get_pre(icoord, vec2<i32>(0, -8));
	p = p + 2025.0 * get_pre(icoord, vec2<i32>(0, -6));
	p = p + 14400.0 * get_pre(icoord, vec2<i32>(0, -4));
	p = p + 44100.0 * get_pre(icoord, vec2<i32>(0, -2));
	p = p + 63504.0 * get_pre(icoord, vec2<i32>(0, 0));
	p = p + 44100.0 * get_pre(icoord, vec2<i32>(0, 2));
	p = p + 14400.0 * get_pre(icoord, vec2<i32>(0, 4));
	p = p + 2025.0 * get_pre(icoord, vec2<i32>(0, 6));
	p = p + 100.0 * get_pre(icoord, vec2<i32>(0, 8));
	p = p + 1.0 * get_pre(icoord, vec2<i32>(0, 10));
	p = p + 10.0 * get_pre(icoord, vec2<i32>(1, -9));
	p = p + 450.0 * get_pre(icoord, vec2<i32>(1, -7));
	p = p + 5400.0 * get_pre(icoord, vec2<i32>(1, -5));
	p = p + 25200.0 * get_pre(icoord, vec2<i32>(1, -3));
	p = p + 52920.0 * get_pre(icoord, vec2<i32>(1, -1));
	p = p + 52920.0 * get_pre(icoord, vec2<i32>(1, 1));
	p = p + 25200.0 * get_pre(icoord, vec2<i32>(1, 3));
	p = p + 5400.0 * get_pre(icoord, vec2<i32>(1, 5));
	p = p + 450.0 * get_pre(icoord, vec2<i32>(1, 7));
	p = p + 10.0 * get_pre(icoord, vec2<i32>(1, 9));
	p = p + 45.0 * get_pre(icoord, vec2<i32>(2, -8));
	p = p + 1200.0 * get_pre(icoord, vec2<i32>(2, -6));
	p = p + 9450.0 * get_pre(icoord, vec2<i32>(2, -4));
	p = p + 30240.0 * get_pre(icoord, vec2<i32>(2, -2));
	p = p + 44100.0 * get_pre(icoord, vec2<i32>(2, 0));
	p = p + 30240.0 * get_pre(icoord, vec2<i32>(2, 2));
	p = p + 9450.0 * get_pre(icoord, vec2<i32>(2, 4));
	p = p + 1200.0 * get_pre(icoord, vec2<i32>(2, 6));
	p = p + 45.0 * get_pre(icoord, vec2<i32>(2, 8));
	p = p + 120.0 * get_pre(icoord, vec2<i32>(3, -7));
	p = p + 2100.0 * get_pre(icoord, vec2<i32>(3, -5));
	p = p + 11340.0 * get_pre(icoord, vec2<i32>(3, -3));
	p = p + 25200.0 * get_pre(icoord, vec2<i32>(3, -1));
	p = p + 25200.0 * get_pre(icoord, vec2<i32>(3, 1));
	p = p + 11340.0 * get_pre(icoord, vec2<i32>(3, 3));
	p = p + 2100.0 * get_pre(icoord, vec2<i32>(3, 5));
	p = p + 120.0 * get_pre(icoord, vec2<i32>(3, 7));
	p = p + 210.0 * get_pre(icoord, vec2<i32>(4, -6));
	p = p + 2520.0 * get_pre(icoord, vec2<i32>(4, -4));
	p = p + 9450.0 * get_pre(icoord, vec2<i32>(4, -2));
	p = p + 14400.0 * get_pre(icoord, vec2<i32>(4, 0));
	p = p + 9450.0 * get_pre(icoord, vec2<i32>(4, 2));
	p = p + 2520.0 * get_pre(icoord, vec2<i32>(4, 4));
	p = p + 210.0 * get_pre(icoord, vec2<i32>(4, 6));
	p = p + 252.0 * get_pre(icoord, vec2<i32>(5, -5));
	p = p + 2100.0 * get_pre(icoord, vec2<i32>(5, -3));
	p = p + 5400.0 * get_pre(icoord, vec2<i32>(5, -1));
	p = p + 5400.0 * get_pre(icoord, vec2<i32>(5, 1));
	p = p + 2100.0 * get_pre(icoord, vec2<i32>(5, 3));
	p = p + 252.0 * get_pre(icoord, vec2<i32>(5, 5));
	p = p + 210.0 * get_pre(icoord, vec2<i32>(6, -4));
	p = p + 1200.0 * get_pre(icoord, vec2<i32>(6, -2));
	p = p + 2025.0 * get_pre(icoord, vec2<i32>(6, 0));
	p = p + 1200.0 * get_pre(icoord, vec2<i32>(6, 2));
	p = p + 210.0 * get_pre(icoord, vec2<i32>(6, 4));
	p = p + 120.0 * get_pre(icoord, vec2<i32>(7, -3));
	p = p + 450.0 * get_pre(icoord, vec2<i32>(7, -1));
	p = p + 450.0 * get_pre(icoord, vec2<i32>(7, 1));
	p = p + 120.0 * get_pre(icoord, vec2<i32>(7, 3));
	p = p + 45.0 * get_pre(icoord, vec2<i32>(8, -2));
	p = p + 100.0 * get_pre(icoord, vec2<i32>(8, 0));
	p = p + 45.0 * get_pre(icoord, vec2<i32>(8, 2));
	p = p + 10.0 * get_pre(icoord, vec2<i32>(9, -1));
	p = p + 10.0 * get_pre(icoord, vec2<i32>(9, 1));
	p = p + 1.0 * get_pre(icoord, vec2<i32>(10, 0));
	return p / 1048576.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let icoord = vec2<i32>(in.tex_coords * uniforms.resolution);
	let div = texel_fetch_pre(icoord).g;
	let p = get_pre_weighted(icoord) - div;
	return vec4<f32>(p, 1.0, 1.0, 1.0);
}
