use std::{iter::once, time::Duration};

use crate::{
    engine::{Context, Texture},
    prelude::*,
    sources::{INDICES, create_index_buffer, create_pipeline, create_vertex_buffer},
    transitions::Transition,
};

#[derive(Debug)]
pub struct FadeTransition {
    previous_texture: Option<Texture>,
    elapsed: Duration,
    duration: Duration,
    progress: f32,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    progress_buffer: wgpu::Buffer,
    from_aspect_buffer: wgpu::Buffer,
    to_aspect_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl FadeTransition {
    pub fn new(previous_texture: Option<Texture>, duration: Duration, ctx: &Context) -> Self {
        debug!("Creating FadeTransition with duration {:?}", duration);

        let vertex_buffer = create_vertex_buffer(ctx);
        let index_buffer = create_index_buffer(ctx);

        let device = ctx.device();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fade_texture_layout"),
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

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fade_uniform_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let progress_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("progress_uniform"),
            size: 4,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let from_aspect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("from_aspect_uniform"),
            size: 4,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let to_aspect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("to_aspect_uniform"),
            size: 4,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: progress_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: from_aspect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: to_aspect_buffer.as_entire_binding(),
                },
            ],
            label: Some("fade_uniform_bind_group"),
        });

        let shader = ctx.device().create_shader_module(wgpu::include_wgsl!("./shaders/fade.wgsl"));

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
            progress: 0.0,
            render_pipeline,
            texture_bind_group_layout,
            vertex_buffer,
            index_buffer,
            progress_buffer,
            from_aspect_buffer,
            to_aspect_buffer,
            uniform_bind_group,
        }
    }
}

impl Transition for FadeTransition {
    fn update(&mut self, dt: Duration) -> bool {
        self.elapsed += dt;
        self.progress = (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
        debug!("FadeTransition progress: {:.2}", self.progress);
        self.progress >= 1.0
    }

    fn progress(&self) -> f32 {
        self.progress
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
            label: Some("fade_texture_bind_group"),
        });

        let surface_aspect = ctx.surface_aspect_ratio();
        let from_aspect = from_texture.aspect_ratio();
        let to_aspect = current_texture.aspect_ratio();

        queue.write_buffer(&self.progress_buffer, 0, bytemuck::cast_slice(&[self.progress]));
        queue.write_buffer(
            &self.from_aspect_buffer,
            0,
            bytemuck::cast_slice(&[surface_aspect / from_aspect]),
        );
        queue.write_buffer(
            &self.to_aspect_buffer,
            0,
            bytemuck::cast_slice(&[surface_aspect / to_aspect]),
        );

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fade_transition"),
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
