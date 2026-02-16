// Advection shader - moves velocity and density fields

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

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn noise(p: vec3<f32>) -> f32 {
    let ip = floor(p);
    var f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    let h = hash(ip.xy + vec2<f32>(0.0, ip.z));
    return mix(h, hash(ip.xy + vec2<f32>(1.0, ip.z)), f.x);
}

fn fbm(p: vec3<f32>, octaves: i32) -> vec2<f32> {
    var v = vec2<f32>(0.0);
    var a = 0.5;
    var pp = p;
    for (var i: i32 = 0; i < octaves; i++) {
        v = v + vec2<f32>(noise(pp), noise(pp + vec3<f32>(100.0))) * a;
        pp = pp * 2.0;
        a = a * 0.5;
    }
    return v;
}

@group(1) @binding(0)
var velocity_texture: texture_2d<f32>;

@group(1) @binding(1)
var velocity_sampler: sampler;

fn sample_velocity(coord: vec2<f32>) -> vec4<f32> {
    let uv = coord / uniforms.resolution;
    return textureSample(velocity_texture, velocity_sampler, uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_coord = in.tex_coords * uniforms.resolution;
    let uv = frag_coord / uniforms.resolution;
    
    // Resolution scaling factor for consistent appearance
    let scale = min(uniforms.resolution.x, uniforms.resolution.y) / 512.0;

    // Read current state
    var veld = sample_velocity(frag_coord);
    var velocity = veld.xy;
    var density = veld.z;

    // CRITICAL FIX: Advection with proper velocity scaling
    // Scale velocity to pixel space for correct displacement
    let upstream = frag_coord - velocity * uniforms.resolution * 0.02;
    let upstream_veld = sample_velocity(upstream);
    velocity = upstream_veld.xy;
    density = upstream_veld.z;

    // Subtle turbulence
    let center = uniforms.resolution * 0.5;
    let world_uv = (frag_coord - center) / uniforms.resolution.y;
    let turb = fbm(vec3<f32>(world_uv * 8.0, uniforms.time * 0.2), 4);
    velocity = velocity + turb * 0.05 * scale;

    // CONTINUOUS EMISSION: Entire top edge
    let emit_height = uniforms.resolution.y * 0.1;
    if (frag_coord.y < emit_height) {
        let emit_strength = 1.0 - (frag_coord.y / emit_height);
        // Gentle downward velocity
        velocity.y = velocity.y + emit_strength * 0.15 * scale;
        // Light smoke emission
        density = density + emit_strength * 0.006 * scale;
    }

    // BURST EMISSION: Random probability-based bursts
    // Use time-based seed for consistent randomness per frame
    let burst_seed = floor(uniforms.time * 10.0);
    
    // Probability check (30% chance per frame)
    if (hash(vec2<f32>(burst_seed, 0.0)) < 0.3) {
        // Number of bursts: 2-8 (random)
        let num_bursts = 2 + i32(hash(vec2<f32>(burst_seed, 1.0)) * 6.0);
        
        for (var i: i32 = 0; i < num_bursts; i++) {
            // Random position for this burst
            let burst_x = hash(vec2<f32>(burst_seed, f32(i) * 2.0)) * uniforms.resolution.x;
            let burst_y = hash(vec2<f32>(burst_seed, f32(i) * 2.0 + 1.0)) * uniforms.resolution.y;
            let burst_center = vec2<f32>(burst_x, burst_y);
            
            // Distance from burst center
            let dist = length(frag_coord - burst_center);
            
            // Burst radius: 20-60 pixels (scaled by resolution)
            let burst_radius = (20.0 + hash(vec2<f32>(burst_seed, f32(i) * 3.0)) * 40.0) * scale;
            
            if (dist < burst_radius) {
                let burst_strength = 1.0 - (dist / burst_radius);
                // Higher density injection for bursts
                density = density + burst_strength * 0.1 * scale;
                // Random velocity direction
                let angle = hash(vec2<f32>(burst_seed, f32(i) * 4.0)) * 6.283;
                let burst_vel = vec2<f32>(cos(angle), sin(angle)) * 0.3;
                velocity = velocity + burst_vel * burst_strength * scale;
            }
        }
    }

    // MOUSE INTERACTION (if engine supports it)
    let mouse_pos = uniforms.mouse;
    let mouse_delta = mouse_pos - uniforms.mouse_prev;
    let to_mouse = frag_coord - mouse_pos;
    let dist = length(to_mouse);
    let influence_radius = 60.0 * scale;

    if (dist < influence_radius) {
        let influence = 1.0 - (dist / influence_radius);
        if (length(mouse_delta) > 0.01) {
            // Push smoke
            velocity = velocity + normalize(mouse_delta) * influence * 3.0 * scale;
        }
        // Emit at mouse
        density = density + influence * 0.05 * scale;
    }

    // Gentle damping
    velocity = velocity * 0.998;
    density = density * 0.995;
    
    density = clamp(density, 0.0, 2.0);

    return vec4<f32>(velocity, density, 1.0);
}
