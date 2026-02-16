// Subtract pressure gradient shader
// Projects velocity field to be divergence-free
// u_new = u_old - grad(p)

// TODO for MVP improvements:
// - Add vorticity confinement for more turbulent smoke
// - Implement MacCormack advection for less numerical diffusion
// - Consider fractional step methods for better accuracy
// - Add velocity limiters for stability

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

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(in.position, 1.0),
        in.tex_coords,
    );
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var velocity_texture: texture_2d<f32>;

@group(1) @binding(1)
var velocity_sampler: sampler;

@group(2) @binding(0)
var pressure_texture: texture_2d<f32>;

@group(2) @binding(1)
var pressure_sampler: sampler;

// Boundary-aware sampling - clamps to edge
fn texel_fetch_velocity(coord: vec2<i32>) -> vec4<f32> {
    let res = vec2<i32>(uniforms.resolution);
    let clamped = clamp(coord, vec2<i32>(0), res - vec2<i32>(1));
    return textureLoad(velocity_texture, clamped, 0);
}

fn texel_fetch_pressure(coord: vec2<i32>) -> f32 {
    let res = vec2<i32>(uniforms.resolution);
    let clamped = clamp(coord, vec2<i32>(0), res - vec2<i32>(1));
    return textureLoad(pressure_texture, clamped, 0).r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let icoord = vec2<i32>(in.tex_coords * uniforms.resolution);
    
    // Get current velocity
    let vel = texel_fetch_velocity(icoord);
    
    // Calculate pressure gradient with boundary clamping
    let p_left = texel_fetch_pressure(icoord + vec2<i32>(-1, 0));
    let p_right = texel_fetch_pressure(icoord + vec2<i32>(1, 0));
    let p_bottom = texel_fetch_pressure(icoord + vec2<i32>(0, -1));
    let p_top = texel_fetch_pressure(icoord + vec2<i32>(0, 1));
    
    let grad = vec2<f32>(p_right - p_left, p_top - p_bottom) * 0.5;
    
    // Subtract gradient from velocity
    let new_vel = vel.xy - grad;
    
    // Boundary condition: zero velocity at edges (wall)
    let edge_margin = 1;
    var final_vel = new_vel;
    if (icoord.x <= edge_margin || icoord.x >= i32(uniforms.resolution.x) - edge_margin ||
        icoord.y <= edge_margin || icoord.y >= i32(uniforms.resolution.y) - edge_margin) {
        final_vel = vec2<f32>(0.0, 0.0);
    }
    
    return vec4<f32>(final_vel, vel.z, 1.0);
}
