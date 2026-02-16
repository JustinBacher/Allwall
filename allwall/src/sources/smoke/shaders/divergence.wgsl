// Divergence calculation shader
// Computes divergence of velocity field for pressure projection

// TODO for MVP improvements:
// - Add support for variable grid spacing
// - Consider higher-order divergence schemes for better accuracy
// - Add buoyancy terms for temperature/density effects
// - Implement boundary condition flags (wall vs open)

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

// Sample with boundary clamping - returns zero velocity at boundaries
fn sample_velocity_clamped(icoord: vec2<i32>) -> vec2<f32> {
    let res = vec2<i32>(uniforms.resolution);
    
    // Check bounds
    if (icoord.x < 0 || icoord.x >= res.x || icoord.y < 0 || icoord.y >= res.y) {
        return vec2<f32>(0.0, 0.0); // Zero velocity outside bounds (wall boundary)
    }
    
    return texel_fetch(icoord).xy;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let icoord = vec2<i32>(in.tex_coords * uniforms.resolution);
    
    // Use clamped sampling for boundary conditions
    let vel_x_left = sample_velocity_clamped(icoord + vec2<i32>(-1, 0)).x;
    let vel_x_right = sample_velocity_clamped(icoord + vec2<i32>(1, 0)).x;
    let vel_y_bottom = sample_velocity_clamped(icoord + vec2<i32>(0, -1)).y;
    let vel_y_top = sample_velocity_clamped(icoord + vec2<i32>(0, 1)).y;
    
    let divergence = (vel_x_right - vel_x_left + vel_y_top - vel_y_bottom) * 0.5;
    return vec4<f32>(divergence, vec3<f32>(1.0));
}
