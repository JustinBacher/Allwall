mod still;

use crate::engine::{Context, Texture};

use image::DynamicImage;

use wgpu::util::DeviceExt;

pub trait Animation {
	fn render(&mut self, ctx: &Context);
	fn update_img(&mut self, img: &DynamicImage, ctx: &Context);
	fn state(&self) -> &RenderState;
}

pub enum RenderState {
	Complete,
	Loading,
	Loaded,
	Playing,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Vec3 {
	pub const fn new(x: f32, y: f32, z: f32) -> Self {
		Self { x, y, z }
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec2 {
	pub u: f32,
	pub v: f32,
}

impl Vec2 {
	pub const fn new(u: f32, v: f32) -> Self {
		Self { u, v }
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	position: Vec3,
	tex_coords: Vec2,
}

impl Vertex {
	fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x3,
				},
				wgpu::VertexAttribute {
					offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x2,
				},
			],
		}
	}
}

pub const TOP_LEFT: Vertex = Vertex {
	position: Vec3::new(0.0, 0.0, 0.0),
	tex_coords: Vec2::new(0.0, 0.0),
};

pub const TOP_RIGHT: Vertex = Vertex {
	position: Vec3::new(-1.0, -1.0, 0.0),
	tex_coords: Vec2::new(0.0, 1.0),
};

pub const BOTTOM_LEFT: Vertex = Vertex {
	position: Vec3::new(1.0, -1.0, 0.0),
	tex_coords: Vec2::new(1.0, 1.0),
};

pub const BOTTOM_RIGHT: Vertex = Vertex {
	position: Vec3::new(1.0, 1.0, 0.0),
	tex_coords: Vec2::new(1.0, 0.0),
};

pub const VERTICES: &[Vertex] = &[TOP_LEFT, TOP_RIGHT, BOTTOM_RIGHT, BOTTOM_LEFT];

pub const INDICES: &[u16] = &[0, 1, 3, 2, 3, 1];

pub fn create_vertex_buffer(ctx: &Context) -> wgpu::Buffer {
	ctx.device()
		.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(VERTICES),
			usage: wgpu::BufferUsages::VERTEX,
		})
}

pub fn create_index_buffer(ctx: &Context) -> wgpu::Buffer {
	ctx.device()
		.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: None,
			contents: bytemuck::cast_slice(INDICES),
			usage: wgpu::BufferUsages::INDEX,
		})
}
pub fn create_texture_binds(
	textures: &[&Texture],
	ctx: &Context,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
	let device = ctx.device();
	let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		label: None,
		entries: (0..textures.len())
			.flat_map(|i| {
				[
					wgpu::BindGroupLayoutEntry {
						binding: (i * 2) as u32,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
							view_dimension: wgpu::TextureViewDimension::D2,
							multisampled: false,
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: (i * 2 + 1) as u32,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				]
			})
			.collect::<Vec<_>>()
			.as_slice(),
	});

	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &layout,
		entries: textures
			.iter()
			.enumerate()
			.flat_map(|(i, t)| {
				[
					wgpu::BindGroupEntry {
						binding: i as u32 * 2,
						resource: wgpu::BindingResource::TextureView(t.view()),
					},
					wgpu::BindGroupEntry {
						binding: i as u32 * 2 + 1,
						resource: wgpu::BindingResource::Sampler(t.sampler()),
					},
				]
			})
			.collect::<Vec<_>>()
			.as_slice(),
		label: None,
	});
	(layout, bind_group)
}

pub fn create_uniform_binds(
	size: u64,
	ctx: &Context,
) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
	let buffer = ctx.device().create_buffer(&wgpu::BufferDescriptor {
		label: None,
		size,
		mapped_at_creation: false,
		usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
	});

	let layout = ctx
		.device()
		.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

	let group = ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &layout,
		entries: &[wgpu::BindGroupEntry {
			binding: 0,
			resource: buffer.as_entire_binding(),
		}],
		label: None,
	});

	(buffer, layout, group)
}

pub fn create_pipeline(
	ctx: &Context,
	bind_group_layouts: &[&wgpu::BindGroupLayout],
	shader: &wgpu::ShaderModule,
	config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
	let layout = ctx
		.device()
		.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts,
			immediate_size: 0,
		});

	ctx.device()
		.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&layout),
			vertex: wgpu::VertexState {
				module: shader,
				entry_point: Some("vs_main"),
				buffers: &[Vertex::desc()],
				compilation_options: Default::default(),
			},
			fragment: Some(wgpu::FragmentState {
				module: shader,
				entry_point: Some("fs_main"),
				targets: &[Some(wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
				compilation_options: Default::default(),
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview_mask: None,
			cache: None,
		})
}
