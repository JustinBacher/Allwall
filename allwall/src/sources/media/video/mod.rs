pub mod decoder;
pub mod error;

use std::{iter::once, path::PathBuf, time::Duration};

use decoder::Decoder;

use crate::{
    engine::{Context, Texture},
    prelude::*,
    sources::{
        BasicSource, INDICES, RenderState, Source, SourceType, create_index_buffer, create_pipeline,
        create_texture_binds, create_uniform_binds, create_vertex_buffer,
    },
    transitions::{CircleOrigin, CircleRevealTransition, FadeTransition, Transition, TransitionType},
};

#[derive(Debug)]
pub struct Video {
    texture: Texture,
    texture_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    state: RenderState,
    decoder: Decoder,
    video_path: PathBuf,
    video_dir: PathBuf,
}

impl Video {
    pub fn new(video_path: PathBuf, video_dir: PathBuf, ctx: &Context) -> Result<Self> {
        debug!("Creating Video source from: {:?}", video_path);

        let decoder = Decoder::new(&video_path)?;
        let (width, height) = Self::get_video_dimensions(&decoder);

        let texture = Texture::empty_writable(ctx, width, height);
        let (texture_bind_group_layout, texture_bind_group) = create_texture_binds(&[&texture], ctx);

        let vertex_buffer = create_vertex_buffer(ctx);
        let index_buffer = create_index_buffer(ctx);
        let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) = create_uniform_binds(32, ctx);

        let render_pipeline = create_pipeline(
            ctx,
            &[&texture_bind_group_layout, &uniform_bind_group_layout],
            &ctx.device().create_shader_module(wgpu::include_wgsl!("./shaders/video.wgsl")),
            ctx.config(),
        );

        let state = RenderState::default();

        Ok(Self {
            texture,
            texture_bind_group,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
            state,
            decoder,
            video_path,
            video_dir,
        })
    }

    pub fn video_path(&self) -> &PathBuf {
        &self.video_path
    }

    pub fn directory(&self) -> &PathBuf {
        &self.video_dir
    }

    fn get_video_dimensions(decoder: &Decoder) -> (u32, u32) {
        if let Some(caps) = decoder.appsink().caps() {
            if let Some(s) = caps.structure(0) {
                let width = s.get::<i32>("width").unwrap_or(1920) as u32;
                let height = s.get::<i32>("height").unwrap_or(1080) as u32;
                return (width, height);
            }
        }
        (1920, 1080)
    }

    fn update_texture_from_frame(&mut self, ctx: &Context) {
        if let Some(sample) = self.decoder.pull_sample() {
            if let Some(buffer) = sample.buffer() {
                if let Ok(map) = buffer.map_readable() {
                    let data = map.as_slice();
                    let size = self.texture.size();

                    ctx.queue().write_texture(
                        self.texture.texture().as_image_copy(),
                        data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * size.width),
                            rows_per_image: Some(size.height),
                        },
                        size,
                    );
                }
            }
        }
    }

    fn render_normal(&self, ctx: &Context) {
        let queue = ctx.queue();
        let device = ctx.device();
        let surface = ctx.surface();

        debug!(
            "Video rendering, surface aspect: {:.2}, texture aspect: {:.2}",
            ctx.surface_aspect_ratio(),
            self.texture.aspect_ratio()
        );

        let output = match surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                error!("Could not get texture from surface: {e}");
                return;
            },
        };
        let view = output.texture.create_view(&Default::default());

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[ctx.surface_aspect_ratio() / self.texture.aspect_ratio()]),
        );

        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        queue.submit(once(encoder.finish()));
        output.present();

        debug!("Video render complete");
    }
}

impl Source for Video {
    fn texture(&self) -> &Texture {
        &self.texture
    }

    fn state(&self) -> &RenderState {
        &self.state
    }

    fn load(&mut self, _ctx: &Context) -> Result<()> {
        debug!("Loading Video source");
        self.state = RenderState::Displaying;
        Ok(())
    }

    fn start_transition(
        &mut self,
        previous: Option<SourceType>,
        duration: Duration,
        ctx: &Context,
        transition_type: TransitionType,
    ) {
        debug!("Starting {:?} transition with duration {:?}", transition_type, duration);
        let previous_texture = previous.map(|s| s.texture().clone());
        let transition: Box<dyn Transition> = match transition_type {
            TransitionType::Fade => Box::new(FadeTransition::new(previous_texture, duration, ctx)),
            TransitionType::CircleTopLeft => Box::new(CircleRevealTransition::new(
                previous_texture,
                duration,
                CircleOrigin::TopLeft,
                ctx,
            )),
            TransitionType::CircleTopRight => Box::new(CircleRevealTransition::new(
                previous_texture,
                duration,
                CircleOrigin::TopRight,
                ctx,
            )),
            TransitionType::CircleBottomLeft => Box::new(CircleRevealTransition::new(
                previous_texture,
                duration,
                CircleOrigin::BottomLeft,
                ctx,
            )),
            TransitionType::CircleBottomRight => Box::new(CircleRevealTransition::new(
                previous_texture,
                duration,
                CircleOrigin::BottomRight,
                ctx,
            )),
            TransitionType::CircleCenter => Box::new(CircleRevealTransition::new(
                previous_texture,
                duration,
                CircleOrigin::Center,
                ctx,
            )),
            TransitionType::CircleRandom => Box::new(CircleRevealTransition::new(
                previous_texture,
                duration,
                CircleOrigin::Random,
                ctx,
            )),
        };
        self.state = RenderState::Transitioning(transition);
    }

    fn update(&mut self, dt: Duration) {
        if let RenderState::Transitioning(transition) = &mut self.state {
            let complete = transition.update(dt);
            if complete {
                debug!("Transition complete, switching to Displaying");
                self.state = RenderState::Displaying;
            }
        }
    }
}

impl BasicSource for Video {
    fn render(&mut self, ctx: &Context) {
        self.update_texture_from_frame(ctx);
        match &self.state {
            RenderState::Transitioning(transition) => {
                transition.render(ctx, &self.texture);
            },
            _ => {
                self.render_normal(ctx);
            },
        }
    }
}
