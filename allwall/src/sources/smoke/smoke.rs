use std::{
	iter::once,
	time::{Duration, Instant},
};

use log::debug;
use wgpu::util::DeviceExt;

use crate::{
	engine::{Context, Texture},
	prelude::*,
	sources::{
		create_index_buffer, create_pipeline, create_vertex_buffer, RenderState, Source, INDICES,
	},
};

const SIMULATION_RESOLUTION: u32 = 512;

#[derive(Debug)]
pub struct SmokeSource {
	texture: Texture,
	texture_bind_group: wgpu::BindGroup,
	texture_bind_group_layout: wgpu::BindGroupLayout,

	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,

	uniform_buffer: wgpu::Buffer,
	uniform_bind_group: wgpu::BindGroup,
	uniform_bind_group_layout: wgpu::BindGroupLayout,

	render_pipeline: wgpu::RenderPipeline,

	sim_texture_a: Texture,
	sim_texture_b: Texture,
	sim_texture_c: Texture,

	advection_pipeline: wgpu::RenderPipeline,
	velocity_bind_group_layout: wgpu::BindGroupLayout,
	advection_bind_groups: Vec<wgpu::BindGroup>,

	divergence_texture: Texture,
	divergence_bind_group_layout: wgpu::BindGroupLayout,
	divergence_bind_groups: Vec<wgpu::BindGroup>,
	divergence_pipeline: wgpu::RenderPipeline,

	pressure_texture_a: Texture,
	pressure_texture_b: Texture,
	pressure_bind_group_layout: wgpu::BindGroupLayout,
	pressure_bind_groups: Vec<wgpu::BindGroup>,
	pressure_pipeline: wgpu::RenderPipeline,

	pressure2_pipeline: wgpu::RenderPipeline,

	render_uniform_buffer: wgpu::Buffer,
	render_uniform_bind_group: wgpu::BindGroup,
	render_uniform_bind_group_layout: wgpu::BindGroupLayout,

	state: RenderState,
	start_time: Instant,
	mouse_position: [f32; 2],
	mouse_prev_position: [f32; 2],

	sim_vertex_buffer: wgpu::Buffer,
	sim_index_buffer: wgpu::Buffer,
}

impl SmokeSource {
	pub fn new(ctx: &Context) -> Self {
		debug!("Creating SmokeSource");

		let sim_size = wgpu::Extent3d {
			width: SIMULATION_RESOLUTION,
			height: SIMULATION_RESOLUTION,
			depth_or_array_layers: 1,
		};

		let sim_texture_a = Self::create_sim_texture(ctx, sim_size);
		let sim_texture_b = Self::create_sim_texture(ctx, sim_size);
		let sim_texture_c = Self::create_sim_texture(ctx, sim_size);

		let divergence_texture = Self::create_sim_texture(ctx, sim_size);
		let pressure_texture_a = Self::create_sim_texture(ctx, sim_size);
		let pressure_texture_b = Self::create_sim_texture(ctx, sim_size);

		let surface_size = ctx.config();

		let texture = Texture::empty(ctx, surface_size.width, surface_size.height);

		let device = ctx.device();

		let texture_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("smoke_texture_layout"),
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
				],
			});

		let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &texture_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(sim_texture_a.view()),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(sim_texture_a.sampler()),
				},
			],
			label: Some("smoke_texture_bind_group"),
		});

		let vertex_buffer = create_vertex_buffer(ctx);
		let index_buffer = create_index_buffer(ctx);
		let sim_vertex_buffer = create_sim_vertex_buffer(ctx);
		let sim_index_buffer = create_sim_index_buffer(ctx);

		let uniform_size = 56u64;
		let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("smoke_uniform"),
			size: uniform_size,
			mapped_at_creation: false,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let uniform_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("smoke_uniform_layout"),
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}],
			});

		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &uniform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer: &uniform_buffer,
					offset: 0,
					size: wgpu::BufferSize::new(56),
				}),
			}],
			label: Some("smoke_uniform_bind_group"),
		});

		let velocity_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("velocity_bind_group_layout"),
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
				],
			});

		let advection_bind_groups = vec![
			Self::create_velocity_bind_group(device, &velocity_bind_group_layout, &sim_texture_a),
			Self::create_velocity_bind_group(device, &velocity_bind_group_layout, &sim_texture_b),
			Self::create_velocity_bind_group(device, &velocity_bind_group_layout, &sim_texture_c),
		];

		let divergence_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("divergence_bind_group_layout"),
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
				],
			});

		let divergence_bind_groups = vec![
			Self::create_velocity_bind_group(device, &divergence_bind_group_layout, &sim_texture_a),
			Self::create_velocity_bind_group(device, &divergence_bind_group_layout, &sim_texture_b),
			Self::create_velocity_bind_group(device, &divergence_bind_group_layout, &sim_texture_c),
		];

		let pressure_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("pressure_bind_group_layout"),
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
				],
			});

		let pressure_bind_groups = vec![
			Self::create_velocity_bind_group(
				device,
				&pressure_bind_group_layout,
				&pressure_texture_a,
			),
			Self::create_velocity_bind_group(
				device,
				&pressure_bind_group_layout,
				&pressure_texture_b,
			),
		];

		let advection_shader = ctx
			.device()
			.create_shader_module(wgpu::include_wgsl!("./shaders/advection.wgsl"));

		let advection_pipeline = create_sim_pipeline(
			ctx,
			&[&uniform_bind_group_layout, &velocity_bind_group_layout],
			&advection_shader,
		);

		let divergence_shader = ctx
			.device()
			.create_shader_module(wgpu::include_wgsl!("./shaders/divergence.wgsl"));

		let divergence_pipeline = create_sim_pipeline(
			ctx,
			&[&uniform_bind_group_layout, &divergence_bind_group_layout],
			&divergence_shader,
		);

		let pressure1_shader = ctx
			.device()
			.create_shader_module(wgpu::include_wgsl!("./shaders/pressure1.wgsl"));

		let pressure_pipeline = create_sim_pipeline(
			ctx,
			&[
				&uniform_bind_group_layout,
				&divergence_bind_group_layout,
				&pressure_bind_group_layout,
			],
			&pressure1_shader,
		);

		let pressure2_shader = ctx
			.device()
			.create_shader_module(wgpu::include_wgsl!("./shaders/pressure2.wgsl"));

		let pressure2_pipeline = create_sim_pipeline(
			ctx,
			&[&uniform_bind_group_layout, &pressure_bind_group_layout],
			&pressure2_shader,
		);

		let render_uniform_size = 8u64;
		let render_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("render_uniform"),
			size: render_uniform_size,
			mapped_at_creation: false,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let render_uniform_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: Some("render_uniform_layout"),
				entries: &[wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}],
			});

		let render_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &render_uniform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer: &render_uniform_buffer,
					offset: 0,
					size: wgpu::BufferSize::new(8),
				}),
			}],
			label: Some("render_uniform_bind_group"),
		});

		let render_shader =
			ctx.device().create_shader_module(wgpu::include_wgsl!("./shaders/render.wgsl"));

		let render_pipeline = create_pipeline(
			ctx,
			&[&render_uniform_bind_group_layout, &texture_bind_group_layout],
			&render_shader,
			ctx.config(),
		);

		let state = RenderState::default();

		Self {
			texture,
			texture_bind_group,
			texture_bind_group_layout,
			vertex_buffer,
			index_buffer,
			uniform_buffer,
			uniform_bind_group,
			uniform_bind_group_layout,
			render_pipeline,
			sim_texture_a,
			sim_texture_b,
			sim_texture_c,
			advection_pipeline,
			velocity_bind_group_layout,
			advection_bind_groups,
			divergence_texture,
			divergence_bind_group_layout,
			divergence_bind_groups,
			divergence_pipeline,
			pressure_texture_a,
			pressure_texture_b,
			pressure_bind_group_layout,
			pressure_bind_groups,
			pressure_pipeline,
			pressure2_pipeline,
			render_uniform_buffer,
			render_uniform_bind_group,
			render_uniform_bind_group_layout,
			state,
			start_time: Instant::now(),
			mouse_position: [
				SIMULATION_RESOLUTION as f32 / 2.0,
				SIMULATION_RESOLUTION as f32 / 2.0,
			],
			mouse_prev_position: [
				SIMULATION_RESOLUTION as f32 / 2.0,
				SIMULATION_RESOLUTION as f32 / 2.0,
			],
			sim_vertex_buffer,
			sim_index_buffer,
		}
	}

	fn create_sim_texture(ctx: &Context, size: wgpu::Extent3d) -> Texture {
		Texture::empty_format(
			ctx,
			size.width,
			size.height,
			wgpu::TextureFormat::Rgba16Float,
		)
	}

	fn create_velocity_bind_group(
		device: &wgpu::Device,
		layout: &wgpu::BindGroupLayout,
		texture: &Texture,
	) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(texture.view()),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(texture.sampler()),
				},
			],
			label: None,
		})
	}

	fn update_uniforms(&self, queue: &wgpu::Queue) {
		let elapsed = self.start_time.elapsed().as_secs_f32();
		let uniforms = SmokeUniforms {
			resolution: [SIMULATION_RESOLUTION as f32, SIMULATION_RESOLUTION as f32],
			_pad1: [0.0; 2],
			time: elapsed,
			_pad2: [0.0; 3],
			mouse: self.mouse_position,
			_pad3: [0.0; 2],
			mouse_prev: self.mouse_prev_position,
		};
		queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
	}

	fn run_simulation_pass(
		&self,
		ctx: &Context,
		pipeline: &wgpu::RenderPipeline,
		output: &Texture,
		bind_group: &wgpu::BindGroup,
		clear_color: wgpu::Color,
	) {
		let device = ctx.device();
		let queue = ctx.queue();

		let mut encoder = device.create_command_encoder(&Default::default());

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("simulation_pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: output.view(),
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(clear_color),
						store: wgpu::StoreOp::Store,
					},
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});

			render_pass.set_pipeline(pipeline);
			render_pass.set_vertex_buffer(0, self.sim_vertex_buffer.slice(..));
			render_pass
				.set_index_buffer(self.sim_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
			render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
			render_pass.set_bind_group(1, bind_group, &[]);
			render_pass.draw_indexed(0..6, 0, 0..1);
		}

		queue.submit(once(encoder.finish()));
	}

	pub fn update_mouse_position(&mut self, x: f32, y: f32) {
		self.mouse_prev_position = self.mouse_position;
		self.mouse_position =
			[x * SIMULATION_RESOLUTION as f32, (1.0 - y) * SIMULATION_RESOLUTION as f32];
	}

	fn render_normal(&mut self, ctx: &Context) {
		self.update_uniforms(ctx.queue());

		self.run_simulation_pass(
			ctx,
			&self.advection_pipeline,
			&self.sim_texture_b,
			&self.advection_bind_groups[0],
			wgpu::Color::TRANSPARENT,
		);

		self.mouse_prev_position = self.mouse_position;

		let queue = ctx.queue();
		let device = ctx.device();
		let surface = ctx.surface();

		let output = match surface.get_current_texture() {
			Ok(output) => output,
			Err(e) => {
				log::error!("Could not get texture from surface: {e}");
				return;
			},
		};
		let view = output.texture.create_view(&Default::default());

		let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &self.texture_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(self.sim_texture_b.view()),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(self.sim_texture_b.sampler()),
				},
			],
			label: Some("smoke_texture_bind_group_render"),
		});

		let mut encoder = device.create_command_encoder(&Default::default());
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("smoke_render"),
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
			render_pass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
			render_pass.set_bind_group(1, &texture_bind_group, &[]);
			render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
		}

		queue.write_buffer(
			&self.render_uniform_buffer,
			0,
			bytemuck::cast_slice(&[SIMULATION_RESOLUTION as f32, SIMULATION_RESOLUTION as f32]),
		);
		queue.submit(once(encoder.finish()));
		output.present();
	}
}

impl Source for SmokeSource {
	fn render(&mut self, ctx: &Context) {
		match &self.state {
			RenderState::Transitioning(transition) => {
				transition.render(ctx, self);
			},
			_ => {
				self.render_normal(ctx);
			},
		}
	}

	fn texture(&self) -> &Texture {
		&self.texture
	}

	fn state(&self) -> &RenderState {
		&self.state
	}

	fn load(&mut self, _ctx: &Context) -> Result<()> {
		debug!("Loading SmokeSource");
		self.state = RenderState::Displaying;
		Ok(())
	}

	fn start_transition(
		&mut self,
		previous: Option<Box<dyn Source>>,
		duration: Duration,
		ctx: &Context,
		transition_type: crate::transitions::TransitionType,
	) {
		use crate::transitions::{FadeTransition, Transition};
		debug!(
			"Starting {:?} transition with duration {:?}",
			transition_type, duration
		);
		let transition: Box<dyn Transition> = match transition_type {
			crate::transitions::TransitionType::Fade =>
				Box::new(FadeTransition::new(previous, duration, ctx)),
			_ => Box::new(FadeTransition::new(previous, duration, ctx)),
		};
		self.state = RenderState::Transitioning(transition);
	}

	fn update(&mut self, _dt: Duration) {
		match &mut self.state {
			RenderState::Transitioning(ref mut transition) => {
				let complete = transition.update(_dt);
				if complete {
					debug!("Transition complete, switching to Displaying");
					self.state = RenderState::Displaying;
				}
			},
			_ => {},
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SmokeUniforms {
	resolution: [f32; 2],
	_pad1: [f32; 2],
	time: f32,
	_pad2: [f32; 3],
	mouse: [f32; 2],
	_pad3: [f32; 2],
	mouse_prev: [f32; 2],
}

fn create_sim_pipeline(
	ctx: &Context,
	bind_group_layouts: &[&wgpu::BindGroupLayout],
	shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
	let layout = ctx.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: None,
		bind_group_layouts,
		push_constant_ranges: &[],
	});

	let vertex_layouts = &[wgpu::VertexBufferLayout {
		array_stride: std::mem::size_of::<SimVertex>() as wgpu::BufferAddress,
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
	}];

	ctx.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: None,
		layout: Some(&layout),
		vertex: wgpu::VertexState {
			module: shader,
			entry_point: "vs_main",
			buffers: vertex_layouts,
			compilation_options: Default::default(),
		},
		fragment: Some(wgpu::FragmentState {
			module: shader,
			entry_point: "fs_main",
			targets: &[Some(wgpu::ColorTargetState {
				format: wgpu::TextureFormat::Rgba16Float,
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
		multiview: None,
	})
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SimVertex {
	position: [f32; 3],
	tex_coords: [f32; 2],
}

fn create_sim_vertex_buffer(ctx: &Context) -> wgpu::Buffer {
	const VERTICES: &[SimVertex] = &[
		SimVertex {
			position: [-1.0, 1.0, 0.0],
			tex_coords: [0.0, 0.0],
		},
		SimVertex {
			position: [-1.0, -1.0, 0.0],
			tex_coords: [0.0, 1.0],
		},
		SimVertex {
			position: [1.0, -1.0, 0.0],
			tex_coords: [1.0, 1.0],
		},
		SimVertex {
			position: [1.0, 1.0, 0.0],
			tex_coords: [1.0, 0.0],
		},
	];

	ctx.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: None,
		contents: bytemuck::cast_slice(VERTICES),
		usage: wgpu::BufferUsages::VERTEX,
	})
}

fn create_sim_index_buffer(ctx: &Context) -> wgpu::Buffer {
	const INDICES: &[u16] = &[0, 1, 3, 2, 3, 1];
	ctx.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: None,
		contents: bytemuck::cast_slice(INDICES),
		usage: wgpu::BufferUsages::INDEX,
	})
}
