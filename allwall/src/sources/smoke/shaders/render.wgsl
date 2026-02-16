// Final render shader - displays smoke simulation

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
    background_color: vec3<f32>,
    smoke_color: vec3<f32>,
    smoke_intensity: f32,
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let density = textureSample(smoke_texture, smoke_sampler, in.tex_coords).b;
    
    // High contrast visibility
    // Use threshold to make even low density visible
    let visibility = smoothstep(0.0, 0.1, density * uniforms.smoke_intensity);
    
    // Mix with high contrast
    var final_color = mix(uniforms.background_color, uniforms.smoke_color, visibility);
    
    // Add brightness for dense areas
    final_color = final_color + vec3<f32>(density * 0.3);
    
    return vec4<f32>(final_color, 1.0);
}
