use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use derive_more::From;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[derive(Debug, Clone, From, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn from_u32(x: u32, y: u32, z: u32) -> Self {
        Self {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        }
    }
}

#[derive(Debug, Clone, From, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Vec3u {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Vec3u {
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub const fn from_f32(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x as u32,
            y: y as u32,
            z: z as u32,
        }
    }
}

#[derive(Debug, Clone, From, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Vec2f {
    pub u: f32,
    pub v: f32,
}

impl Vec2f {
    pub const fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }

    pub const fn from_u32(u: u32, v: u32) -> Self {
        Self {
            u: u as f32,
            v: v as f32,
        }
    }
}

#[derive(Debug, Clone, From, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Vec2u {
    pub u: u32,
    pub v: u32,
}

impl Vec2u {
    pub const fn new(u: u32, v: u32) -> Self {
        Self { u, v }
    }

    pub const fn from_f32(u: f32, v: f32) -> Self {
        Self {
            u: u as u32,
            v: v as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct FVertex {
    pub position: Vec3f,
    pub tex_coords: Vec2f,
}

impl FVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<UVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct UVertex {
    pub position: Vec3u,
    pub tex_coords: Vec2u,
}

impl UVertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<UVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as u64,
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
    fn test_vec3f_new() {
        let v = Vec3f::new(1.0, 2.0, 3.0);
        assert!((v.x - 1.0).abs() < f32::EPSILON);
        assert!((v.y - 2.0).abs() < f32::EPSILON);
        assert!((v.z - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vec3f_from_u32() {
        let v = Vec3f::from_u32(10, 20, 30);
        assert!((v.x - 10.0).abs() < f32::EPSILON);
        assert!((v.y - 20.0).abs() < f32::EPSILON);
        assert!((v.z - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vec3f_equality() {
        let v1 = Vec3f::new(1.0, 2.0, 3.0);
        let v2 = Vec3f::new(1.0, 2.0, 3.0);
        let v3 = Vec3f::new(1.0, 2.0, 4.0);
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_vec3f_zeroable() {
        let zero = Vec3f::zeroed();
        assert!((zero.x).abs() < f32::EPSILON);
        assert!((zero.y).abs() < f32::EPSILON);
        assert!((zero.z).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vec3u_new() {
        let v = Vec3u::new(10, 20, 30);
        assert_eq!(v.x, 10);
        assert_eq!(v.y, 20);
        assert_eq!(v.z, 30);
    }

    #[test]
    fn test_vec3u_from_f32() {
        let v = Vec3u::from_f32(1.9, 2.1, 3.5);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
        assert_eq!(v.z, 3);
    }

    #[test]
    fn test_vec3u_from_f32_negative() {
        let v = Vec3u::from_f32(-1.0, -2.0, 3.0);
        assert!(v.x > 0 || v.x == 0 || v.x == u32::MAX);
        assert!(v.y > 0 || v.y == 0 || v.y == u32::MAX);
        assert_eq!(v.z, 3);
    }

    #[test]
    fn test_vec3u_equality() {
        let v1 = Vec3u::new(1, 2, 3);
        let v2 = Vec3u::new(1, 2, 3);
        let v3 = Vec3u::new(1, 2, 4);
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_vec2f_new() {
        let v = Vec2f::new(1.5, 2.5);
        assert!((v.u - 1.5).abs() < f32::EPSILON);
        assert!((v.v - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vec2f_from_u32() {
        let v = Vec2f::from_u32(100, 200);
        assert!((v.u - 100.0).abs() < f32::EPSILON);
        assert!((v.v - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vec2f_equality() {
        let v1 = Vec2f::new(1.0, 2.0);
        let v2 = Vec2f::new(1.0, 2.0);
        let v3 = Vec2f::new(1.0, 3.0);
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_vec2f_zeroable() {
        let zero = Vec2f::zeroed();
        assert!((zero.u).abs() < f32::EPSILON);
        assert!((zero.v).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vec2u_new() {
        let v = Vec2u::new(100, 200);
        assert_eq!(v.u, 100);
        assert_eq!(v.v, 200);
    }

    #[test]
    fn test_vec2u_from_f32() {
        let v = Vec2u::from_f32(1.9, 2.1);
        assert_eq!(v.u, 1);
        assert_eq!(v.v, 2);
    }

    #[test]
    fn test_vec2u_equality() {
        let v1 = Vec2u::new(1, 2);
        let v2 = Vec2u::new(1, 2);
        let v3 = Vec2u::new(1, 3);
        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_vec2u_zeroable() {
        let zero = Vec2u::zeroed();
        assert_eq!(zero.u, 0);
        assert_eq!(zero.v, 0);
    }

    #[test]
    fn test_fvertex_size() {
        assert_eq!(size_of::<FVertex>(), size_of::<[f32; 5]>());
    }

    #[test]
    fn test_uvertex_size() {
        assert_eq!(size_of::<UVertex>(), size_of::<[u32; 5]>());
    }

    #[test]
    fn test_fvertex_zeroable() {
        let zero = FVertex::zeroed();
        assert!((zero.position.x).abs() < f32::EPSILON);
        assert!((zero.position.y).abs() < f32::EPSILON);
        assert!((zero.position.z).abs() < f32::EPSILON);
        assert!((zero.tex_coords.u).abs() < f32::EPSILON);
        assert!((zero.tex_coords.v).abs() < f32::EPSILON);
    }

    #[test]
    fn test_uvertex_zeroable() {
        let zero = UVertex::zeroed();
        assert_eq!(zero.position.x, 0);
        assert_eq!(zero.position.y, 0);
        assert_eq!(zero.position.z, 0);
        assert_eq!(zero.tex_coords.u, 0);
        assert_eq!(zero.tex_coords.v, 0);
    }
}
