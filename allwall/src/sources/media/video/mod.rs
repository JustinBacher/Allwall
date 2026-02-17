pub mod error;

use std::{iter::once, path::PathBuf, time::Duration};

use gstreamer::{Caps, Pipeline, State, prelude::*};
use gstreamer_app::AppSink;
use gstreamer_video::VideoFrame;

use crate::{
    engine::{Context, Texture},
    prelude::*,
    sources::{
        BasicSource, INDICES, RenderState, Source, SourceType, create_index_buffer, create_pipeline,
        create_texture_binds, create_uniform_binds, create_vertex_buffer,
    },
    transitions::{CircleOrigin, CircleRevealTransition, FadeTransition, Transition, TransitionType},
};

use self::error::VideoError;

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
    _video_path: PathBuf,
    video_dir: PathBuf,
    pipeline: Option<Pipeline>,
    appsink: Option<AppSink>,
    frame_aspect_ratio: f32,
}

impl Video {
    pub fn new(video_path: PathBuf, video_dir: PathBuf, ctx: &Context) -> Result<Self> {
        debug!("Creating Video source from {:?}", video_path);

        if !video_path.exists() {
            return Err(VideoError::FileNotFound(video_path).into());
        }

        let texture = Self::create_placeholder_texture(ctx);

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

        let (pipeline, appsink) = Self::create_pipeline_and_sink(&video_path)?;

        Ok(Self {
            texture,
            texture_bind_group,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            render_pipeline,
            state: RenderState::default(),
            _video_path: video_path,
            video_dir,
            pipeline: Some(pipeline),
            appsink: Some(appsink),
            frame_aspect_ratio: 16.0 / 9.0,
        })
    }

    fn create_placeholder_texture(ctx: &Context) -> Texture {
        let device = ctx.device();
        let size = wgpu::Extent3d {
            width: 1920,
            height: 1080,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("video_placeholder_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Texture::from_existing(texture, view, sampler)
    }

    fn create_pipeline_and_sink(video_path: &PathBuf) -> Result<(Pipeline, AppSink)> {
        let path_str = video_path
            .to_str()
            .ok_or_else(|| VideoError::Generic("Invalid path encoding".to_string()))?;
        let pipeline_str = format!(
            "filesrc location='{}' ! decodebin ! videoconvert ! videoscale ! video/x-raw,format=RGBA,width=1920,height=1080 ! appsink name=sink caps=video/x-raw,format=RGBA",
            path_str
        );

        let pipeline = gstreamer::parse::launch(&pipeline_str)
            .map_err(|e| VideoError::PipelineParse(e.to_string()))?
            .downcast::<Pipeline>()
            .map_err(|_| VideoError::PipelineDowncast)?;

        let appsink = pipeline
            .by_name("sink")
            .ok_or(VideoError::SinkNotFound("sink"))?
            .downcast::<AppSink>()
            .map_err(|_| VideoError::SinkCast)?;

        appsink.set_caps(Some(&Caps::builder("video/x-raw").field("format", "RGBA").build()));

        pipeline
            .set_state(State::Playing)
            .map_err(|e| VideoError::PipelineStart(e.to_string()))?;

        Ok((pipeline, appsink))
    }

    fn pull_frame(&mut self, ctx: &Context) -> Result<()> {
        let appsink = self.appsink.as_ref().ok_or(VideoError::NoFrames)?;

        let sample = appsink
            .try_pull_sample(gstreamer::ClockTime::from_mseconds(10))
            .ok_or_else(|| VideoError::NoFrames)?;

        let buffer = sample.buffer().ok_or(VideoError::BufferNotFound)?;
        let caps = sample
            .caps()
            .ok_or_else(|| VideoError::Generic("No caps in sample".to_string()))?;
        let structure = caps
            .structure(0)
            .ok_or_else(|| VideoError::Generic("No structure in caps".to_string()))?;

        let width = structure.get::<i32>("width").map_err(|e| VideoError::Generic(e.to_string()))? as u32;
        let height = structure.get::<i32>("height").map_err(|e| VideoError::Generic(e.to_string()))? as u32;

        self.frame_aspect_ratio = width as f32 / height as f32;

        let video_info =
            gstreamer_video::VideoInfo::from_caps(&caps).map_err(|e| VideoError::BufferMap(format!("{:?}", e)))?;

        let frame = VideoFrame::from_buffer_readable(buffer.to_owned(), &video_info)
            .map_err(|e| VideoError::BufferMap(format!("{:?}", e)))?;

        let data = frame.plane_data(0).map_err(|e| VideoError::BufferMap(format!("{:?}", e)))?;

        let device = ctx.device();
        let queue = ctx.queue();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("video_frame_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.texture = Texture::from_existing(texture, view, sampler);

        let (texture_bind_group_layout, texture_bind_group) = create_texture_binds(&[&self.texture], ctx);

        let (_, _, _uniform_bind_group) = create_uniform_binds(32, ctx);

        self.texture_bind_group = texture_bind_group;

        let device = ctx.device();
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        self.render_pipeline = create_pipeline(
            ctx,
            &[&texture_bind_group_layout, &uniform_bind_group_layout],
            &device.create_shader_module(wgpu::include_wgsl!("./shaders/video.wgsl")),
            ctx.config(),
        );

        Ok(())
    }

    pub fn directory(&self) -> &PathBuf {
        &self.video_dir
    }

    fn render_normal(&mut self, ctx: &Context) {
        if let Err(e) = self.pull_frame(ctx) {
            debug!("Failed to pull frame: {}", e);
        }

        let queue = ctx.queue();
        let device = ctx.device();
        let surface = ctx.surface();

        debug!(
            "Video rendering, surface aspect: {:.2}, frame aspect: {:.2}",
            ctx.surface_aspect_ratio(),
            self.frame_aspect_ratio
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
            bytemuck::cast_slice(&[ctx.surface_aspect_ratio() / self.frame_aspect_ratio]),
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

impl Drop for Video {
    fn drop(&mut self) {
        if let Some(pipeline) = self.pipeline.take() {
            let _ = pipeline.set_state(State::Null);
        }
    }
}
