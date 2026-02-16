use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::sources::types::{Vec2f, Vec3f};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GrassUniforms {
    pub resolution: Vec2f,
    pub time: f32,
    pub wind_strength: f32,
    pub blade_height: f32,
    pub blade_spacing: f32,
    pub grid_size: Vec2f,
    pub padding: Vec2f,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DirtUniforms {
    pub color: Vec3f,
    pub padding: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct BladeVertex {
    pub position: Vec3f,
    pub tex_coords: Vec2f,
    pub height_factor: f32,
    pub _padding: f32,
}

impl BladeVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<BladeVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<Vec3f>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct BladeInstance {
    pub grid_x: f32,
    pub grid_y: f32,
    pub random_seed: f32,
    pub rotation: f32,
}

impl BladeInstance {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<BladeInstance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: VertexFormat::Float32,
                },
                VertexAttribute {
                    offset: size_of::<f32>() as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 5,
                    format: VertexFormat::Float32,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 6,
                    format: VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DirtVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl DirtVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<DirtVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<Vec3f>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_grass_uniforms_size() {
        assert_eq!(size_of::<GrassUniforms>(), size_of::<[f32; 10]>());
    }

    #[test]
    fn test_grass_uniforms_zeroable() {
        let zero = GrassUniforms::zeroed();
        assert!((zero.resolution.u).abs() < f32::EPSILON);
        assert!((zero.resolution.v).abs() < f32::EPSILON);
        assert!((zero.time).abs() < f32::EPSILON);
        assert!((zero.wind_strength).abs() < f32::EPSILON);
        assert!((zero.blade_height).abs() < f32::EPSILON);
        assert!((zero.blade_spacing).abs() < f32::EPSILON);
        assert!((zero.grid_size.u).abs() < f32::EPSILON);
        assert!((zero.grid_size.v).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dirt_uniforms_size() {
        assert_eq!(size_of::<DirtUniforms>(), size_of::<[f32; 4]>());
    }

    #[test]
    fn test_dirt_uniforms_zeroable() {
        let zero = DirtUniforms::zeroed();
        assert!((zero.color.x).abs() < f32::EPSILON);
        assert!((zero.color.y).abs() < f32::EPSILON);
        assert!((zero.color.z).abs() < f32::EPSILON);
        assert!((zero.padding).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blade_vertex_size() {
        assert_eq!(size_of::<BladeVertex>(), size_of::<[f32; 7]>());
    }

    #[test]
    fn test_blade_vertex_zeroable() {
        let zero = BladeVertex::zeroed();
        assert!((zero.position.x).abs() < f32::EPSILON);
        assert!((zero.position.y).abs() < f32::EPSILON);
        assert!((zero.position.z).abs() < f32::EPSILON);
        assert!((zero.tex_coords.u).abs() < f32::EPSILON);
        assert!((zero.tex_coords.v).abs() < f32::EPSILON);
        assert!((zero.height_factor).abs() < f32::EPSILON);
    }

    #[test]
    fn test_blade_instance_size() {
        assert_eq!(size_of::<BladeInstance>(), size_of::<[f32; 4]>());
    }

    #[test]
    fn test_blade_instance_zeroable() {
        let zero = BladeInstance::zeroed();
        assert!((zero.grid_x).abs() < f32::EPSILON);
        assert!((zero.grid_y).abs() < f32::EPSILON);
        assert!((zero.random_seed).abs() < f32::EPSILON);
        assert!((zero.rotation).abs() < f32::EPSILON);
    }

    #[test]
    fn test_dirt_vertex_size() {
        assert_eq!(size_of::<DirtVertex>(), size_of::<[f32; 5]>());
    }

    #[test]
    fn test_dirt_vertex_zeroable() {
        let zero = DirtVertex::zeroed();
        assert!((zero.position[0]).abs() < f32::EPSILON);
        assert!((zero.position[1]).abs() < f32::EPSILON);
        assert!((zero.position[2]).abs() < f32::EPSILON);
        assert!((zero.tex_coords[0]).abs() < f32::EPSILON);
        assert!((zero.tex_coords[1]).abs() < f32::EPSILON);
    }
}
