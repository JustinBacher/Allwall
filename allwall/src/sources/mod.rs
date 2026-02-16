pub mod error;
pub mod grass;
pub mod media;
pub mod smoke;
pub mod types;

use std::{fmt, time::Duration};

use bitflags::bitflags;
use bytemuck::cast_slice;
use clap::ValueEnum;
use derive_more::Display;
use serde::Deserialize;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, BlendState, Buffer, BufferBindingType, BufferDescriptor, BufferUsages,
    ColorTargetState, ColorWrites, Face, FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    ShaderModule, ShaderStages, SurfaceConfiguration, TextureSampleType, TextureViewDimension, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    engine::{Context, Texture},
    prelude::Result,
    sources::types::{FVertex, Vec2f, Vec3f},
    transitions::TransitionType,
};

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct ContextNeeds: u8 {
        const NONE = 0;
        const MOUSE = 0b01;
        const WINDOWS = 0b10;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InteractionState {
    pub mouse: Option<(f32, f32)>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Display, Deserialize, Default)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    #[default]
    Media,
    Smoke,
    Grass,
}

pub enum SourceType {
    Media(Box<media::MediaSource>),
    Smoke(Box<smoke::SmokeSource>),
    Grass(Box<grass::GrassSource>),
}

impl SourceType {
    pub fn needs(&self) -> ContextNeeds {
        match self {
            SourceType::Grass(g) => g.needs(),
            _ => ContextNeeds::NONE,
        }
    }

    pub fn render(&mut self, ctx: &Context, state: &InteractionState) {
        match self {
            SourceType::Media(m) => m.render(ctx),
            SourceType::Smoke(s) => s.render(ctx),
            SourceType::Grass(g) => g.render(ctx, state),
        }
    }

    pub fn texture(&self) -> &Texture {
        match self {
            SourceType::Media(m) => m.texture(),
            SourceType::Smoke(s) => s.texture(),
            SourceType::Grass(g) => g.texture(),
        }
    }

    pub fn state(&self) -> &RenderState {
        match self {
            SourceType::Media(m) => m.state(),
            SourceType::Smoke(s) => s.state(),
            SourceType::Grass(g) => g.state(),
        }
    }

    pub fn load(&mut self, ctx: &Context) -> Result<()> {
        match self {
            SourceType::Media(m) => m.load(ctx),
            SourceType::Smoke(s) => s.load(ctx),
            SourceType::Grass(g) => g.load(ctx),
        }
    }

    pub fn start_transition(
        &mut self,
        previous: Option<SourceType>,
        duration: Duration,
        ctx: &Context,
        transition_type: TransitionType,
    ) {
        match self {
            SourceType::Media(m) => m.start_transition(previous, duration, ctx, transition_type),
            SourceType::Smoke(s) => s.start_transition(previous, duration, ctx, transition_type),
            SourceType::Grass(g) => g.start_transition(previous, duration, ctx, transition_type),
        }
    }

    pub fn update(&mut self, dt: Duration) {
        match self {
            SourceType::Media(m) => m.update(dt),
            SourceType::Smoke(s) => s.update(dt),
            SourceType::Grass(g) => g.update(dt),
        }
    }

    pub fn next(&self, ctx: &Context) -> Result<SourceType> {
        match self {
            SourceType::Media(m) => m.next(ctx).map(|s| SourceType::Media(Box::new(s))),
            SourceType::Smoke(_) | SourceType::Grass(_) => {
                Err(error::SourceError::UnsupportedOperation("next".to_string()).into())
            },
        }
    }

    pub fn prev(&self, ctx: &Context) -> Result<SourceType> {
        match self {
            SourceType::Media(m) => m.prev(ctx).map(|s| SourceType::Media(Box::new(s))),
            SourceType::Smoke(_) | SourceType::Grass(_) => {
                Err(error::SourceError::UnsupportedOperation("prev".to_string()).into())
            },
        }
    }
}

impl fmt::Debug for SourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceType::Media(m) => m.fmt(f),
            SourceType::Smoke(s) => s.fmt(f),
            SourceType::Grass(g) => g.fmt(f),
        }
    }
}

pub trait Source: fmt::Debug {
    fn texture(&self) -> &Texture;
    fn state(&self) -> &RenderState;
    fn load(&mut self, ctx: &Context) -> Result<()>;
    fn start_transition(
        &mut self,
        previous: Option<SourceType>,
        duration: Duration,
        ctx: &Context,
        transition_type: TransitionType,
    );
    fn update(&mut self, dt: Duration);
    fn next(&self, ctx: &Context) -> Result<Self>
    where
        Self: Sized,
    {
        let _ = ctx;
        Err(error::SourceError::UnsupportedOperation("next".to_string()).into())
    }
    fn prev(&self, ctx: &Context) -> Result<Self>
    where
        Self: Sized,
    {
        let _ = ctx;
        Err(error::SourceError::UnsupportedOperation("prev".to_string()).into())
    }
}

pub trait BasicSource: Source {
    fn render(&mut self, ctx: &Context);
}

pub trait ContextualSource: Source {
    const NEEDS: ContextNeeds;

    fn needs(&self) -> ContextNeeds {
        Self::NEEDS
    }

    fn render(&mut self, ctx: &Context, state: &InteractionState);
}

#[derive(Debug, Default)]
pub enum RenderState {
    #[default]
    Loading,
    Displaying,
    Transitioning(Box<dyn crate::transitions::Transition>),
}

pub const VERTICES: &[FVertex] = &[
    FVertex {
        position: Vec3f::new(-1.0, 1.0, 0.0),
        tex_coords: Vec2f::new(0.0, 0.0),
    },
    FVertex {
        position: Vec3f::new(-1.0, -1.0, 0.0),
        tex_coords: Vec2f::new(0.0, 1.0),
    },
    FVertex {
        position: Vec3f::new(1.0, -1.0, 0.0),
        tex_coords: Vec2f::new(1.0, 1.0),
    },
    FVertex {
        position: Vec3f::new(1.0, 1.0, 0.0),
        tex_coords: Vec2f::new(1.0, 0.0),
    },
];

pub const INDICES: &[u16] = &[0, 1, 3, 2, 3, 1];

pub fn create_vertex_buffer(ctx: &Context) -> Buffer {
    ctx.device().create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: cast_slice(VERTICES),
        usage: BufferUsages::VERTEX,
    })
}

pub fn create_index_buffer(ctx: &Context) -> Buffer {
    ctx.device().create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: cast_slice(INDICES),
        usage: BufferUsages::INDEX,
    })
}
pub fn create_texture_binds(textures: &[&Texture], ctx: &Context) -> (BindGroupLayout, BindGroup) {
    let device = ctx.device();
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: (0..textures.len())
            .flat_map(|i| {
                [
                    BindGroupLayoutEntry {
                        binding: (i * 2) as u32,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: (i * 2 + 1) as u32,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ]
            })
            .collect::<Vec<_>>()
            .as_slice(),
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &layout,
        entries: textures
            .iter()
            .enumerate()
            .flat_map(|(i, t)| {
                [
                    BindGroupEntry {
                        binding: i as u32 * 2,
                        resource: BindingResource::TextureView(t.view()),
                    },
                    BindGroupEntry {
                        binding: i as u32 * 2 + 1,
                        resource: BindingResource::Sampler(t.sampler()),
                    },
                ]
            })
            .collect::<Vec<_>>()
            .as_slice(),
        label: None,
    });
    (layout, bind_group)
}

pub fn create_uniform_binds(size: u64, ctx: &Context) -> (Buffer, BindGroupLayout, BindGroup) {
    let buffer = ctx.device().create_buffer(&BufferDescriptor {
        label: None,
        size,
        mapped_at_creation: false,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let layout = ctx.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::all(),
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let group = ctx.device().create_bind_group(&BindGroupDescriptor {
        layout: &layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: None,
    });

    (buffer, layout, group)
}

pub fn create_pipeline(
    ctx: &Context,
    bind_group_layouts: &[&BindGroupLayout],
    shader: &ShaderModule,
    config: &SurfaceConfiguration,
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
            buffers: &[FVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
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
