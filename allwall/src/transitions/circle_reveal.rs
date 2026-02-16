use std::{iter::once, time::Duration};

use rand::Rng;
#[cfg(feature = "generate")]
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    engine::{Context, Texture},
    prelude::*,
    sources::{INDICES, create_index_buffer, create_pipeline, create_vertex_buffer},
    transitions::Transition,
};

fn default_feather() -> f32 {
    0.05
}

/// Origin point for circle reveal transition
///
/// Defines where the circle animation starts when revealing the new wallpaper.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema, serde_nixos::NixosType))]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum CircleOrigin {
    /// Start from top-left corner
    TopLeft,

    /// Start from top-right corner
    TopRight,

    /// Start from bottom-left corner
    BottomLeft,

    /// Start from bottom-right corner
    BottomRight,

    /// Start from center
    #[cfg_attr(feature = "generate", nixos(default = "true"))]
    #[default]
    Center,

    /// Start from a random position each time
    Random,
}

/// Options for circle reveal transition
///
/// Configures the appearance of the circle reveal animation.
#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema, serde_nixos::NixosType))]
pub struct CircleOptions {
    /// Edge softness (feathering) for the circle reveal
    ///
    /// Controls how soft or hard the edge of the circle appears.
    /// - 0.0: Hard edge (no feathering)
    /// - 0.05: Slight softness (default)
    /// - 0.1+: More pronounced soft edge
    #[serde(default = "default_feather")]
    #[cfg_attr(feature = "generate", schemars(default = "default_feather"))]
    #[cfg_attr(feature = "generate", nixos(default = "0.05"))]
    pub feather: f32,

    /// Starting point for the circle reveal animation
    #[serde(default)]
    #[cfg_attr(feature = "generate", nixos(default = "\"center\""))]
    pub origin: CircleOrigin,
}

impl Default for CircleOptions {
    fn default() -> Self {
        Self {
            feather: default_feather(),
            origin: CircleOrigin::Center,
        }
    }
}

impl CircleOptions {
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            feather: if other.feather != default_feather() {
                other.feather
            } else {
                self.feather
            },
            origin: if other.origin != CircleOrigin::default() {
                other.origin
            } else {
                self.origin
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_options_defaults() {
        let options = CircleOptions::default();
        assert!((options.feather - 0.05).abs() < f32::EPSILON);
        assert_eq!(options.origin, CircleOrigin::Center);
    }

    #[test]
    fn test_circle_options_merge_feather() {
        let global = CircleOptions {
            feather: 0.1,
            origin: CircleOrigin::Center,
        };
        let scene = CircleOptions {
            feather: 0.2,
            origin: CircleOrigin::Center,
        };

        let merged = global.merge(&scene);
        assert!((merged.feather - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_circle_options_merge_feather_uses_global_when_default() {
        let global = CircleOptions {
            feather: 0.15,
            origin: CircleOrigin::Center,
        };
        let scene = CircleOptions {
            feather: 0.05,
            origin: CircleOrigin::Center,
        };

        let merged = global.merge(&scene);
        assert!((merged.feather - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn test_circle_options_merge_origin() {
        let global = CircleOptions {
            feather: 0.05,
            origin: CircleOrigin::Center,
        };
        let scene = CircleOptions {
            feather: 0.05,
            origin: CircleOrigin::TopLeft,
        };

        let merged = global.merge(&scene);
        assert_eq!(merged.origin, CircleOrigin::TopLeft);
    }

    #[test]
    fn test_circle_options_merge_origin_uses_global_when_default() {
        let global = CircleOptions {
            feather: 0.05,
            origin: CircleOrigin::TopRight,
        };
        let scene = CircleOptions {
            feather: 0.05,
            origin: CircleOrigin::Center,
        };

        let merged = global.merge(&scene);
        assert_eq!(merged.origin, CircleOrigin::TopRight);
    }

    #[derive(serde::Deserialize)]
    struct CircleOriginConfig {
        origin: CircleOrigin,
    }

    #[test]
    fn test_circle_origin_deserialize_top_left() {
        let config: CircleOriginConfig = toml::from_str(r#"origin = "top-left""#).unwrap();
        assert_eq!(config.origin, CircleOrigin::TopLeft);
    }

    #[test]
    fn test_circle_origin_deserialize_top_right() {
        let config: CircleOriginConfig = toml::from_str(r#"origin = "top-right""#).unwrap();
        assert_eq!(config.origin, CircleOrigin::TopRight);
    }

    #[test]
    fn test_circle_origin_deserialize_bottom_left() {
        let config: CircleOriginConfig = toml::from_str(r#"origin = "bottom-left""#).unwrap();
        assert_eq!(config.origin, CircleOrigin::BottomLeft);
    }

    #[test]
    fn test_circle_origin_deserialize_bottom_right() {
        let config: CircleOriginConfig = toml::from_str(r#"origin = "bottom-right""#).unwrap();
        assert_eq!(config.origin, CircleOrigin::BottomRight);
    }

    #[test]
    fn test_circle_origin_deserialize_center() {
        let config: CircleOriginConfig = toml::from_str(r#"origin = "center""#).unwrap();
        assert_eq!(config.origin, CircleOrigin::Center);
    }

    #[test]
    fn test_circle_origin_deserialize_random() {
        let config: CircleOriginConfig = toml::from_str(r#"origin = "random""#).unwrap();
        assert_eq!(config.origin, CircleOrigin::Random);
    }

    #[test]
    fn test_circle_origin_default() {
        assert_eq!(CircleOrigin::default(), CircleOrigin::Center);
    }

    #[derive(serde::Deserialize)]
    struct CircleConfigTest {
        circle: CircleOptions,
    }

    #[test]
    fn test_circle_options_deserialize() {
        let config: CircleConfigTest = toml::from_str(
            r#"
            [circle]
            feather = 0.1
            origin = "top-left"
            "#,
        )
        .unwrap();

        assert!((config.circle.feather - 0.1).abs() < f32::EPSILON);
        assert_eq!(config.circle.origin, CircleOrigin::TopLeft);
    }
}

#[derive(Debug)]
pub struct CircleRevealTransition {
    previous_texture: Option<Texture>,
    elapsed: Duration,
    duration: Duration,
    origin: CircleOrigin,
    center: Option<[f32; 2]>,

    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl CircleRevealTransition {
    pub fn new(previous_texture: Option<Texture>, duration: Duration, origin: CircleOrigin, ctx: &Context) -> Self {
        debug!(
            "Creating CircleRevealTransition with origin {:?} and duration {:?}",
            origin, duration
        );

        let mut rng = rand::rng();

        let center = match origin {
            CircleOrigin::Random => Some([rng.random::<f32>(), rng.random::<f32>()]),
            CircleOrigin::TopLeft => Some([0.0, 0.0]),
            CircleOrigin::TopRight => Some([1.0, 0.0]),
            CircleOrigin::BottomLeft => Some([0.0, 1.0]),
            CircleOrigin::BottomRight => Some([1.0, 1.0]),
            CircleOrigin::Center => Some([0.5, 0.5]),
        };

        let vertex_buffer = create_vertex_buffer(ctx);
        let index_buffer = create_index_buffer(ctx);
        let device = ctx.device();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("circle_reveal_texture_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // WGSL struct alignment: vec2<f32> requires 8-byte alignment
        // progress (f32) at offset 0, aspect_ratio at offset 4
        // center (vec2<f32>) at offset 8
        // surface_to_from_arr (f32) at offset 16
        // surface_to_to_arr (f32) at offset 20
        // Total: 24 bytes
        const UNIFORM_SIZE: u64 = 24;

        #[repr(C)]
        #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct CircleRevealUniforms {
            progress: f32,
            aspect_ratio: f32,
            center: [f32; 2],
            surface_to_from_arr: f32,
            surface_to_to_arr: f32,
        }

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("circle_reveal_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(UNIFORM_SIZE),
                },
                count: None,
            }],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("circle_reveal_uniform"),
            size: UNIFORM_SIZE,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let _uniforms = CircleRevealUniforms {
            progress: 0.0,
            aspect_ratio: ctx.surface_aspect_ratio(),
            center: [0.5, 0.5],
            surface_to_from_arr: 1.0,
            surface_to_to_arr: 1.0,
        };

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(UNIFORM_SIZE),
                }),
            }],
            label: Some("circle_reveal_uniform_bind_group"),
        });

        let shader = ctx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shaders/circle_reveal.wgsl"));

        let render_pipeline = create_pipeline(
            ctx,
            &[&texture_bind_group_layout, &uniform_bind_group_layout],
            &shader,
            ctx.config(),
        );

        Self {
            previous_texture,
            elapsed: Duration::ZERO,
            duration,
            origin,
            center,
            render_pipeline,
            texture_bind_group_layout,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
        }
    }
}

impl Transition for CircleRevealTransition {
    fn update(&mut self, dt: Duration) -> bool {
        self.elapsed += dt;
        let progress = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
        debug!("CircleRevealTransition progress: {:.2}", progress.min(1.0));
        progress >= 1.0
    }

    fn progress(&self) -> f32 {
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
    }

    fn render(&self, ctx: &Context, current_texture: &Texture) {
        let queue = ctx.queue();
        let device = ctx.device();
        let surface = ctx.surface();

        let output = match surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                error!("Could not get texture from surface: {e}");
                return;
            },
        };
        let view = output.texture.create_view(&Default::default());

        let from_texture: &Texture = self.previous_texture.as_ref().unwrap_or(current_texture);

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(from_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(from_texture.sampler()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(current_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(current_texture.sampler()),
                },
            ],
            label: Some("circle_reveal_texture_bind_group"),
        });

        let surface_aspect = ctx.surface_aspect_ratio();
        let from_aspect = from_texture.aspect_ratio();
        let to_aspect = current_texture.aspect_ratio();
        let progress = self.progress();

        let center = self.center.unwrap_or([0.5, 0.5]);

        #[repr(C)]
        #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct CircleRevealUniforms {
            progress: f32,
            aspect_ratio: f32,
            center: [f32; 2],
            surface_to_from_arr: f32,
            surface_to_to_arr: f32,
        }

        let uniforms = CircleRevealUniforms {
            progress,
            aspect_ratio: surface_aspect,
            center,
            surface_to_from_arr: surface_aspect / from_aspect,
            surface_to_to_arr: surface_aspect / to_aspect,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("circle_reveal_transition"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        queue.submit(once(encoder.finish()));
        output.present();
    }

    fn previous_texture(&self) -> Option<&Texture> {
        self.previous_texture.as_ref()
    }
}
