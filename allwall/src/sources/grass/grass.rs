use std::{iter::once, time::Instant};

use bytemuck::cast_slice;
use rand::{Rng, SeedableRng, rngs::StdRng};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, Color,
    ColorTargetState, ColorWrites, Extent3d, FilterMode, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout,
    IndexFormat, LoadOp, MultisampleState, Operations, Origin3d, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderModule, ShaderStages, StoreOp, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
    VertexState, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

use super::{
    perlin::generate_wind_texture,
    types::{BladeInstance, BladeVertex, DirtUniforms, DirtVertex, GrassUniforms},
};
use crate::{
    engine::{Context, Texture},
    prelude::*,
    sources::{
        RenderState, Source,
        types::{Vec2f, Vec2u, Vec3f},
    },
};

const WIND_TEXTURE_SIZE: u32 = 256;
const WIND_STRENGTH: f32 = 0.03;
const BLADE_HEIGHT_PERCENT: f32 = 0.085;
const BLADE_SPACING: f32 = 3.5;

#[derive(Debug)]
pub struct GrassSource {
    texture: Texture,
    state: RenderState,
    start_time: Instant,

    dirt_vertex_buffer: Buffer,
    dirt_index_buffer: Buffer,
    dirt_uniform_buffer: Buffer,
    dirt_bind_group: BindGroup,
    dirt_bind_group_layout: BindGroupLayout,
    dirt_pipeline: RenderPipeline,

    blade_vertex_buffer: Buffer,
    blade_index_buffer: Buffer,
    instance_buffer: Buffer,
    instance_count: u32,

    grass_uniform_buffer: Buffer,
    grass_bind_group: BindGroup,
    grass_bind_group_layout: BindGroupLayout,
    grass_pipeline: RenderPipeline,

    wind_texture: Texture,
    wind_bind_group: BindGroup,
    wind_bind_group_layout: BindGroupLayout,

    grid_size: Vec2f,
    current_resolution: Vec2u,
}

impl GrassSource {
    pub fn new(ctx: &Context) -> Self {
        debug!("Creating GrassSource");

        let config = ctx.config();
        let device = ctx.device();

        let texture = Texture::empty(ctx, config.width, config.height);

        let grid_width = (config.width as f32 / BLADE_SPACING).ceil() as u32;
        let grid_height = (config.height as f32 / BLADE_SPACING).ceil() as u32;
        let instance_count = grid_width * grid_height;
        let grid_size = Vec2f::from_u32(grid_width, grid_height);

        let dirt_vertex_buffer = create_dirt_vertex_buffer(ctx);
        let dirt_index_buffer = create_dirt_index_buffer(ctx);

        let dirt_uniforms = DirtUniforms {
            color: Vec3f::new(0.35, 0.25, 0.15),
            padding: 0.0,
        };

        let dirt_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("dirt_uniform"),
            contents: cast_slice(&[dirt_uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let dirt_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("dirt_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let dirt_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &dirt_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: dirt_uniform_buffer.as_entire_binding(),
            }],
            label: Some("dirt_bind_group"),
        });

        let dirt_shader = device.create_shader_module(include_wgsl!("./shaders/dirt.wgsl"));
        let dirt_pipeline = create_dirt_pipeline(ctx, &[&dirt_bind_group_layout], &dirt_shader);

        let blade_vertex_buffer = create_blade_vertex_buffer(ctx);
        let blade_index_buffer = create_blade_index_buffer(ctx);
        let instance_buffer = create_instance_buffer(ctx, grid_width, grid_height);

        let blade_height_pixels = config.height as f32 * BLADE_HEIGHT_PERCENT;
        let grass_uniforms = GrassUniforms {
            resolution: Vec2f::from_u32(config.width, config.height),
            time: 0.0,
            wind_strength: WIND_STRENGTH,
            blade_height: blade_height_pixels,
            blade_spacing: BLADE_SPACING,
            grid_size,
            padding: Vec2f::new(0.0, 0.0),
        };
        let grass_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("grass_uniform"),
            contents: cast_slice(&[grass_uniforms]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let grass_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("grass_bind_group_layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let grass_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &grass_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: grass_uniform_buffer.as_entire_binding(),
            }],
            label: Some("grass_bind_group"),
        });

        let wind_texture = create_wind_texture(ctx);
        let wind_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("wind_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let wind_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &wind_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(wind_texture.view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(wind_texture.sampler()),
                },
            ],
            label: Some("wind_bind_group"),
        });

        let grass_shader = device.create_shader_module(include_wgsl!("./shaders/grass.wgsl"));
        let grass_pipeline =
            create_grass_pipeline(ctx, &[&grass_bind_group_layout, &wind_bind_group_layout], &grass_shader);

        let state = RenderState::default();

        Self {
            texture,
            state,
            start_time: Instant::now(),
            dirt_vertex_buffer,
            dirt_index_buffer,
            dirt_uniform_buffer,
            dirt_bind_group,
            dirt_bind_group_layout,
            dirt_pipeline,
            blade_vertex_buffer,
            blade_index_buffer,
            instance_buffer,
            instance_count,
            grass_uniform_buffer,
            grass_bind_group,
            grass_bind_group_layout,
            grass_pipeline,
            wind_texture,
            wind_bind_group,
            wind_bind_group_layout,
            grid_size,
            current_resolution: Vec2u::new(config.width, config.height),
        }
    }

    fn resize_if_needed(&mut self, ctx: &Context) {
        let config = ctx.config();
        if config.width == self.current_resolution.u && config.height == self.current_resolution.v {
            return;
        }

        debug!(
            "Resizing grass from {:?} to {:?}",
            self.current_resolution,
            [config.width, config.height]
        );

        let grid_width = (config.width as f32 / BLADE_SPACING).ceil() as u32;
        let grid_height = (config.height as f32 / BLADE_SPACING).ceil() as u32;

        self.instance_count = grid_width * grid_height;
        self.grid_size = Vec2f::from_u32(grid_width, grid_height);
        self.current_resolution = Vec2u::new(config.width, config.height);
        self.instance_buffer = create_instance_buffer(ctx, grid_width, grid_height);
    }

    fn update_uniforms(&self, ctx: &Context) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let config = ctx.config();

        let blade_height_pixels = config.height as f32 * BLADE_HEIGHT_PERCENT;
        let uniforms = GrassUniforms {
            resolution: Vec2f::from_u32(config.width, config.height),
            time: elapsed,
            wind_strength: WIND_STRENGTH,
            blade_height: blade_height_pixels,
            blade_spacing: BLADE_SPACING,
            grid_size: self.grid_size,
            padding: Vec2f::new(0.0, 0.0),
        };
        ctx.queue().write_buffer(&self.grass_uniform_buffer, 0, cast_slice(&[uniforms]));
    }

    fn render_normal(&mut self, ctx: &Context) {
        self.resize_if_needed(ctx);
        self.update_uniforms(ctx);

        let surface = ctx.surface();
        let output = match surface.get_current_texture() {
            Ok(o) => o,
            Err(e) => {
                error!("Could not get texture from surface: {e}");
                return;
            },
        };
        let view = output.texture.create_view(&Default::default());

        let device = ctx.device();
        let mut encoder = device.create_command_encoder(&Default::default());

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("grass_render"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.dirt_pipeline);
            render_pass.set_bind_group(0, &self.dirt_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.dirt_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.dirt_index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..1);

            render_pass.set_pipeline(&self.grass_pipeline);
            render_pass.set_bind_group(0, &self.grass_bind_group, &[]);
            render_pass.set_bind_group(1, &self.wind_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.blade_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.blade_index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..18, 0, 0..self.instance_count);
        }

        ctx.queue().submit(once(encoder.finish()));
        output.present();
    }
}

impl Source for GrassSource {
    fn texture(&self) -> &Texture {
        &self.texture
    }

    fn state(&self) -> &RenderState {
        &self.state
    }

    fn load(&mut self, _ctx: &Context) -> Result<()> {
        debug!("Loading GrassSource");
        self.state = RenderState::Displaying;
        Ok(())
    }

    fn start_transition(
        &mut self,
        _previous: Option<crate::sources::SourceType>,
        _duration: std::time::Duration,
        _ctx: &Context,
        _transition_type: crate::transitions::TransitionType,
    ) {
        debug!("Grass source does not support transitions");
        self.state = RenderState::Displaying;
    }

    fn update(&mut self, _dt: std::time::Duration) {}
}

impl crate::sources::ContextualSource for GrassSource {
    const NEEDS: crate::sources::ContextNeeds = crate::sources::ContextNeeds::MOUSE;

    fn render(&mut self, ctx: &Context, state: &crate::sources::InteractionState) {
        match &self.state {
            RenderState::Transitioning(_) => {},
            _ => {
                // Update mouse position if available
                if let Some((x, y)) = state.mouse {
                    // Grass doesn't currently use mouse, but could in the future
                    let _ = (x, y);
                }
                self.render_normal(ctx);
            },
        }
    }
}

fn create_dirt_vertex_buffer(ctx: &Context) -> Buffer {
    const VERTICES: &[DirtVertex] = &[
        DirtVertex {
            position: [-1.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
        },
        DirtVertex {
            position: [-1.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
        },
        DirtVertex {
            position: [1.0, -1.0, 0.0],
            tex_coords: [1.0, 1.0],
        },
        DirtVertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
        },
    ];

    ctx.device().create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: cast_slice(VERTICES),
        usage: BufferUsages::VERTEX,
    })
}

fn create_dirt_index_buffer(ctx: &Context) -> Buffer {
    const INDICES: &[u16] = &[0, 1, 3, 2, 3, 1];
    ctx.device().create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: cast_slice(INDICES),
        usage: BufferUsages::INDEX,
    })
}

fn create_blade_vertex_buffer(ctx: &Context) -> Buffer {
    let vertices: Vec<BladeVertex> = vec![
        BladeVertex {
            position: Vec3f::new(-0.5, 0.0, 0.0),
            tex_coords: Vec2f::new(0.0, 0.0),
            height_factor: 0.0,
            _padding: 0.0,
        },
        BladeVertex {
            position: Vec3f::new(0.5, 0.0, 0.0),
            tex_coords: Vec2f::new(1.0, 0.0),
            height_factor: 0.0,
            _padding: 0.0,
        },
        BladeVertex {
            position: Vec3f::new(-0.4, 0.33, 0.0),
            tex_coords: Vec2f::new(0.0, 0.33),
            height_factor: 0.33,
            _padding: 0.0,
        },
        BladeVertex {
            position: Vec3f::new(0.4, 0.33, 0.0),
            tex_coords: Vec2f::new(1.0, 0.33),
            height_factor: 0.33,
            _padding: 0.0,
        },
        BladeVertex {
            position: Vec3f::new(-0.25, 0.66, 0.0),
            tex_coords: Vec2f::new(0.0, 0.66),
            height_factor: 0.66,
            _padding: 0.0,
        },
        BladeVertex {
            position: Vec3f::new(0.25, 0.66, 0.0),
            tex_coords: Vec2f::new(1.0, 0.66),
            height_factor: 0.66,
            _padding: 0.0,
        },
        BladeVertex {
            position: Vec3f::new(0.0, 1.0, 0.0),
            tex_coords: Vec2f::new(0.5, 1.0),
            height_factor: 1.0,
            _padding: 0.0,
        },
        BladeVertex {
            position: Vec3f::new(0.0, 1.0, 0.0),
            tex_coords: Vec2f::new(0.5, 1.0),
            height_factor: 1.0,
            _padding: 0.0,
        },
    ];

    ctx.device().create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    })
}

fn create_blade_index_buffer(ctx: &Context) -> Buffer {
    const INDICES: &[u16] = &[0, 1, 2, 2, 1, 3, 2, 3, 4, 4, 3, 5, 4, 5, 6, 4, 6, 7];
    ctx.device().create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: cast_slice(INDICES),
        usage: BufferUsages::INDEX,
    })
}

fn create_instance_buffer(ctx: &Context, grid_width: u32, grid_height: u32) -> Buffer {
    let mut rng = StdRng::seed_from_u64(42);
    let mut instances = Vec::with_capacity((grid_width * grid_height) as usize);

    for y in 0..grid_height {
        for x in 0..grid_width {
            let jitter_x = (rng.random::<f32>() - 0.5) * 1.8;
            let jitter_y = (rng.random::<f32>() - 0.5) * 1.8;

            instances.push(BladeInstance {
                grid_x: x as f32 + jitter_x,
                grid_y: y as f32 + jitter_y,
                random_seed: rng.random(),
                rotation: rng.random::<f32>() * std::f32::consts::TAU,
            });
        }
    }

    ctx.device().create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: cast_slice(&instances),
        usage: BufferUsages::VERTEX,
    })
}

fn f32_to_f16(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = (bits >> 31) as u16 & 0x8000;
    let mut exponent = ((bits >> 23) & 0xFF) as i32;
    let mantissa = (bits & 0x7FFFFF) as i32;

    if exponent == 255 {
        if mantissa != 0 {
            return sign | 0x7FFF;
        }
        return sign | 0x7C00;
    }

    exponent = exponent - 127 + 15;

    if exponent <= 0 {
        return sign;
    }

    if exponent >= 31 {
        return sign | 0x7C00;
    }

    let f16_mantissa = (mantissa >> 13) as u16;
    sign | ((exponent as u16) << 10) | f16_mantissa
}

fn create_wind_texture(ctx: &Context) -> Texture {
    let data = generate_wind_texture(WIND_TEXTURE_SIZE, WIND_TEXTURE_SIZE, 42);
    let mut data_bytes = Vec::with_capacity((WIND_TEXTURE_SIZE * WIND_TEXTURE_SIZE * 8) as usize);

    for pixel in data {
        for val in pixel {
            let f16_val = f32_to_f16(val);
            data_bytes.extend_from_slice(&f16_val.to_le_bytes());
        }
    }

    let size = Extent3d {
        width: WIND_TEXTURE_SIZE,
        height: WIND_TEXTURE_SIZE,
        depth_or_array_layers: 1,
    };

    let texture = ctx.device().create_texture(&TextureDescriptor {
        label: Some("wind_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    ctx.queue().write_texture(
        ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &data_bytes,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(WIND_TEXTURE_SIZE * 8),
            rows_per_image: Some(WIND_TEXTURE_SIZE),
        },
        size,
    );

    let view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = ctx.device().create_sampler(&SamplerDescriptor {
        label: Some("wind_sampler"),
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    Texture::from_existing(texture, view, sampler)
}

fn create_dirt_pipeline(
    ctx: &Context,
    bind_group_layouts: &[&BindGroupLayout],
    shader: &ShaderModule,
) -> RenderPipeline {
    let layout = ctx.device().create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    ctx.device().create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[DirtVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: ctx.config().format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

fn create_grass_pipeline(
    ctx: &Context,
    bind_group_layouts: &[&BindGroupLayout],
    shader: &ShaderModule,
) -> RenderPipeline {
    let layout = ctx.device().create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    ctx.device().create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&layout),
        vertex: VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[BladeVertex::desc(), BladeInstance::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: ctx.config().format,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
