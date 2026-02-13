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
var divergence_texture: texture_2d<f32>;

@group(1) @binding(1)
var divergence_sampler: sampler;

@group(2) @binding(0)
var pressure_texture: texture_2d<f32>;

@group(2) @binding(1)
var pressure_sampler: sampler;

fn texel_fetch_div(coord: vec2<i32>) -> vec4<f32> {
    return textureLoad(divergence_texture, coord, 0);
}

fn texel_fetch_pre(coord: vec2<i32>) -> vec4<f32> {
    return textureLoad(pressure_texture, coord, 0);
}

fn get_div(icoord: vec2<i32>, p: vec2<i32>) -> f32 {
    return texel_fetch_div(icoord + p).r;
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

fn get_div_weighted(icoord: vec2<i32>) -> f32 {
	var p: f32 = 0.0;
	p = p + 1.0 * get_div(icoord, vec2<i32>(-9, 0));
	p = p + 9.0 * get_div(icoord, vec2<i32>(-8, -1));
	p = p + 4.0 * get_div(icoord, vec2<i32>(-8, 0));
	p = p + 9.0 * get_div(icoord, vec2<i32>(-8, 1));
	p = p + 36.0 * get_div(icoord, vec2<i32>(-7, -2));
	p = p + 32.0 * get_div(icoord, vec2<i32>(-7, -1));
	p = p + 97.0 * get_div(icoord, vec2<i32>(-7, 0));
	p = p + 32.0 * get_div(icoord, vec2<i32>(-7, 1));
	p = p + 36.0 * get_div(icoord, vec2<i32>(-7, 2));
	p = p + 84.0 * get_div(icoord, vec2<i32>(-6, -3));
	p = p + 112.0 * get_div(icoord, vec2<i32>(-6, -2));
	p = p + 436.0 * get_div(icoord, vec2<i32>(-6, -1));
	p = p + 320.0 * get_div(icoord, vec2<i32>(-6, 0));
	p = p + 436.0 * get_div(icoord, vec2<i32>(-6, 1));
	p = p + 112.0 * get_div(icoord, vec2<i32>(-6, 2));
	p = p + 84.0 * get_div(icoord, vec2<i32>(-6, 3));
	p = p + 126.0 * get_div(icoord, vec2<i32>(-5, -4));
	p = p + 224.0 * get_div(icoord, vec2<i32>(-5, -3));
	p = p + 1092.0 * get_div(icoord, vec2<i32>(-5, -2));
	p = p + 1280.0 * get_div(icoord, vec2<i32>(-5, -1));
	p = p + 2336.0 * get_div(icoord, vec2<i32>(-5, 0));
	p = p + 1280.0 * get_div(icoord, vec2<i32>(-5, 1));
	p = p + 1092.0 * get_div(icoord, vec2<i32>(-5, 2));
	p = p + 224.0 * get_div(icoord, vec2<i32>(-5, 3));
	p = p + 126.0 * get_div(icoord, vec2<i32>(-5, 4));
	p = p + 126.0 * get_div(icoord, vec2<i32>(-4, -5));
	p = p + 280.0 * get_div(icoord, vec2<i32>(-4, -4));
	p = p + 1694.0 * get_div(icoord, vec2<i32>(-4, -3));
	p = p + 2752.0 * get_div(icoord, vec2<i32>(-4, -2));
	p = p + 6656.0 * get_div(icoord, vec2<i32>(-4, -1));
	p = p + 6464.0 * get_div(icoord, vec2<i32>(-4, 0));
	p = p + 6656.0 * get_div(icoord, vec2<i32>(-4, 1));
	p = p + 2752.0 * get_div(icoord, vec2<i32>(-4, 2));
	p = p + 1694.0 * get_div(icoord, vec2<i32>(-4, 3));
	p = p + 280.0 * get_div(icoord, vec2<i32>(-4, 4));
	p = p + 126.0 * get_div(icoord, vec2<i32>(-4, 5));
	p = p + 84.0 * get_div(icoord, vec2<i32>(-3, -6));
	p = p + 224.0 * get_div(icoord, vec2<i32>(-3, -5));
	p = p + 1694.0 * get_div(icoord, vec2<i32>(-3, -4));
	p = p + 3520.0 * get_div(icoord, vec2<i32>(-3, -3));
	p = p + 11016.0 * get_div(icoord, vec2<i32>(-3, -2));
	p = p + 16128.0 * get_div(icoord, vec2<i32>(-3, -1));
	p = p + 24608.0 * get_div(icoord, vec2<i32>(-3, 0));
	p = p + 16128.0 * get_div(icoord, vec2<i32>(-3, 1));
	p = p + 11016.0 * get_div(icoord, vec2<i32>(-3, 2));
	p = p + 3520.0 * get_div(icoord, vec2<i32>(-3, 3));
	p = p + 1694.0 * get_div(icoord, vec2<i32>(-3, 4));
	p = p + 224.0 * get_div(icoord, vec2<i32>(-3, 5));
	p = p + 84.0 * get_div(icoord, vec2<i32>(-3, 6));
	p = p + 36.0 * get_div(icoord, vec2<i32>(-2, -7));
	p = p + 112.0 * get_div(icoord, vec2<i32>(-2, -6));
	p = p + 1092.0 * get_div(icoord, vec2<i32>(-2, -5));
	p = p + 2752.0 * get_div(icoord, vec2<i32>(-2, -4));
	p = p + 11016.0 * get_div(icoord, vec2<i32>(-2, -3));
	p = p + 21664.0 * get_div(icoord, vec2<i32>(-2, -2));
	p = p + 47432.0 * get_div(icoord, vec2<i32>(-2, -1));
	p = p + 59712.0 * get_div(icoord, vec2<i32>(-2, 0));
	p = p + 47432.0 * get_div(icoord, vec2<i32>(-2, 1));
	p = p + 21664.0 * get_div(icoord, vec2<i32>(-2, 2));
	p = p + 11016.0 * get_div(icoord, vec2<i32>(-2, 3));
	p = p + 2752.0 * get_div(icoord, vec2<i32>(-2, 4));
	p = p + 1092.0 * get_div(icoord, vec2<i32>(-2, 5));
	p = p + 112.0 * get_div(icoord, vec2<i32>(-2, 6));
	p = p + 36.0 * get_div(icoord, vec2<i32>(-2, 7));
	p = p + 9.0 * get_div(icoord, vec2<i32>(-1, -8));
	p = p + 32.0 * get_div(icoord, vec2<i32>(-1, -7));
	p = p + 436.0 * get_div(icoord, vec2<i32>(-1, -6));
	p = p + 1280.0 * get_div(icoord, vec2<i32>(-1, -5));
	p = p + 6656.0 * get_div(icoord, vec2<i32>(-1, -4));
	p = p + 16128.0 * get_div(icoord, vec2<i32>(-1, -3));
	p = p + 47432.0 * get_div(icoord, vec2<i32>(-1, -2));
	p = p + 92224.0 * get_div(icoord, vec2<i32>(-1, -1));
	p = p + 163476.0 * get_div(icoord, vec2<i32>(-1, 0));
	p = p + 92224.0 * get_div(icoord, vec2<i32>(-1, 1));
	p = p + 47432.0 * get_div(icoord, vec2<i32>(-1, 2));
	p = p + 16128.0 * get_div(icoord, vec2<i32>(-1, 3));
	p = p + 6656.0 * get_div(icoord, vec2<i32>(-1, 4));
	p = p + 1280.0 * get_div(icoord, vec2<i32>(-1, 5));
	p = p + 436.0 * get_div(icoord, vec2<i32>(-1, 6));
	p = p + 32.0 * get_div(icoord, vec2<i32>(-1, 7));
	p = p + 9.0 * get_div(icoord, vec2<i32>(-1, 8));
	p = p + 1.0 * get_div(icoord, vec2<i32>(0, -9));
	p = p + 4.0 * get_div(icoord, vec2<i32>(0, -8));
	p = p + 97.0 * get_div(icoord, vec2<i32>(0, -7));
	p = p + 320.0 * get_div(icoord, vec2<i32>(0, -6));
	p = p + 2336.0 * get_div(icoord, vec2<i32>(0, -5));
	p = p + 6464.0 * get_div(icoord, vec2<i32>(0, -4));
	p = p + 24608.0 * get_div(icoord, vec2<i32>(0, -3));
	p = p + 59712.0 * get_div(icoord, vec2<i32>(0, -2));
	p = p + 163476.0 * get_div(icoord, vec2<i32>(0, -1));
	p = p + 409744.0 * get_div(icoord, vec2<i32>(0, 0));
	p = p + 163476.0 * get_div(icoord, vec2<i32>(0, 1));
	p = p + 59712.0 * get_div(icoord, vec2<i32>(0, 2));
	p = p + 24608.0 * get_div(icoord, vec2<i32>(0, 3));
	p = p + 6464.0 * get_div(icoord, vec2<i32>(0, 4));
	p = p + 2336.0 * get_div(icoord, vec2<i32>(0, 5));
	p = p + 320.0 * get_div(icoord, vec2<i32>(0, 6));
	p = p + 97.0 * get_div(icoord, vec2<i32>(0, 7));
	p = p + 4.0 * get_div(icoord, vec2<i32>(0, 8));
	p = p + 1.0 * get_div(icoord, vec2<i32>(0, 9));
	p = p + 9.0 * get_div(icoord, vec2<i32>(1, -8));
	p = p + 32.0 * get_div(icoord, vec2<i32>(1, -7));
	p = p + 436.0 * get_div(icoord, vec2<i32>(1, -6));
	p = p + 1280.0 * get_div(icoord, vec2<i32>(1, -5));
	p = p + 6656.0 * get_div(icoord, vec2<i32>(1, -4));
	p = p + 16128.0 * get_div(icoord, vec2<i32>(1, -3));
	p = p + 47432.0 * get_div(icoord, vec2<i32>(1, -2));
	p = p + 92224.0 * get_div(icoord, vec2<i32>(1, -1));
	p = p + 163476.0 * get_div(icoord, vec2<i32>(1, 0));
	p = p + 92224.0 * get_div(icoord, vec2<i32>(1, 1));
	p = p + 47432.0 * get_div(icoord, vec2<i32>(1, 2));
	p = p + 16128.0 * get_div(icoord, vec2<i32>(1, 3));
	p = p + 6656.0 * get_div(icoord, vec2<i32>(1, 4));
	p = p + 1280.0 * get_div(icoord, vec2<i32>(1, 5));
	p = p + 436.0 * get_div(icoord, vec2<i32>(1, 6));
	p = p + 32.0 * get_div(icoord, vec2<i32>(1, 7));
	p = p + 9.0 * get_div(icoord, vec2<i32>(1, 8));
	p = p + 36.0 * get_div(icoord, vec2<i32>(2, -7));
	p = p + 112.0 * get_div(icoord, vec2<i32>(2, -6));
	p = p + 1092.0 * get_div(icoord, vec2<i32>(2, -5));
	p = p + 2752.0 * get_div(icoord, vec2<i32>(2, -4));
	p = p + 11016.0 * get_div(icoord, vec2<i32>(2, -3));
	p = p + 21664.0 * get_div(icoord, vec2<i32>(2, -2));
	p = p + 47432.0 * get_div(icoord, vec2<i32>(2, -1));
	p = p + 59712.0 * get_div(icoord, vec2<i32>(2, 0));
	p = p + 47432.0 * get_div(icoord, vec2<i32>(2, 1));
	p = p + 21664.0 * get_div(icoord, vec2<i32>(2, 2));
	p = p + 11016.0 * get_div(icoord, vec2<i32>(2, 3));
	p = p + 2752.0 * get_div(icoord, vec2<i32>(2, 4));
	p = p + 1092.0 * get_div(icoord, vec2<i32>(2, 5));
	p = p + 112.0 * get_div(icoord, vec2<i32>(2, 6));
	p = p + 36.0 * get_div(icoord, vec2<i32>(2, 7));
	p = p + 84.0 * get_div(icoord, vec2<i32>(3, -6));
	p = p + 224.0 * get_div(icoord, vec2<i32>(3, -5));
	p = p + 1694.0 * get_div(icoord, vec2<i32>(3, -4));
	p = p + 3520.0 * get_div(icoord, vec2<i32>(3, -3));
	p = p + 11016.0 * get_div(icoord, vec2<i32>(3, -2));
	p = p + 16128.0 * get_div(icoord, vec2<i32>(3, -1));
	p = p + 24608.0 * get_div(icoord, vec2<i32>(3, 0));
	p = p + 16128.0 * get_div(icoord, vec2<i32>(3, 1));
	p = p + 11016.0 * get_div(icoord, vec2<i32>(3, 2));
	p = p + 3520.0 * get_div(icoord, vec2<i32>(3, 3));
	p = p + 1694.0 * get_div(icoord, vec2<i32>(3, 4));
	p = p + 224.0 * get_div(icoord, vec2<i32>(3, 5));
	p = p + 84.0 * get_div(icoord, vec2<i32>(3, 6));
	p = p + 126.0 * get_div(icoord, vec2<i32>(4, -5));
	p = p + 280.0 * get_div(icoord, vec2<i32>(4, -4));
	p = p + 1694.0 * get_div(icoord, vec2<i32>(4, -3));
	p = p + 2752.0 * get_div(icoord, vec2<i32>(4, -2));
	p = p + 6656.0 * get_div(icoord, vec2<i32>(4, -1));
	p = p + 6464.0 * get_div(icoord, vec2<i32>(4, 0));
	p = p + 6656.0 * get_div(icoord, vec2<i32>(4, 1));
	p = p + 2752.0 * get_div(icoord, vec2<i32>(4, 2));
	p = p + 1694.0 * get_div(icoord, vec2<i32>(4, 3));
	p = p + 280.0 * get_div(icoord, vec2<i32>(4, 4));
	p = p + 126.0 * get_div(icoord, vec2<i32>(4, 5));
	p = p + 126.0 * get_div(icoord, vec2<i32>(5, -4));
	p = p + 224.0 * get_div(icoord, vec2<i32>(5, -3));
	p = p + 1092.0 * get_div(icoord, vec2<i32>(5, -2));
	p = p + 1280.0 * get_div(icoord, vec2<i32>(5, -1));
	p = p + 2336.0 * get_div(icoord, vec2<i32>(5, 0));
	p = p + 1280.0 * get_div(icoord, vec2<i32>(5, 1));
	p = p + 1092.0 * get_div(icoord, vec2<i32>(5, 2));
	p = p + 224.0 * get_div(icoord, vec2<i32>(5, 3));
	p = p + 126.0 * get_div(icoord, vec2<i32>(5, 4));
	p = p + 84.0 * get_div(icoord, vec2<i32>(6, -3));
	p = p + 112.0 * get_div(icoord, vec2<i32>(6, -2));
	p = p + 436.0 * get_div(icoord, vec2<i32>(6, -1));
	p = p + 320.0 * get_div(icoord, vec2<i32>(6, 0));
	p = p + 436.0 * get_div(icoord, vec2<i32>(6, 1));
	p = p + 112.0 * get_div(icoord, vec2<i32>(6, 2));
	p = p + 84.0 * get_div(icoord, vec2<i32>(6, 3));
	p = p + 36.0 * get_div(icoord, vec2<i32>(7, -2));
	p = p + 32.0 * get_div(icoord, vec2<i32>(7, -1));
	p = p + 97.0 * get_div(icoord, vec2<i32>(7, 0));
	p = p + 32.0 * get_div(icoord, vec2<i32>(7, 1));
	p = p + 36.0 * get_div(icoord, vec2<i32>(7, 2));
	p = p + 9.0 * get_div(icoord, vec2<i32>(8, -1));
	p = p + 4.0 * get_div(icoord, vec2<i32>(8, 0));
	p = p + 9.0 * get_div(icoord, vec2<i32>(8, 1));
	p = p + 1.0 * get_div(icoord, vec2<i32>(9, 0));
	return p / 1048576.0;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let icoord = vec2<i32>(in.tex_coords * uniforms.resolution);
	let div = get_div_weighted(icoord);
	let p = get_pre_weighted(icoord) - div;
	return vec4<f32>(p, div, 1.0, 1.0);
}
