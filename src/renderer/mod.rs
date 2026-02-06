use crate::decode::Decoder;
use crate::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 2],
	tex_coord: [f32; 2],
}

const VERTICES: &[Vertex] = &[
	Vertex {
		position: [-1.0, -1.0],
		tex_coord: [0.0, 1.0],
	},
	Vertex {
		position: [1.0, -1.0],
		tex_coord: [1.0, 1.0],
	},
	Vertex {
		position: [-1.0, 1.0],
		tex_coord: [0.0, 0.0],
	},
	Vertex {
		position: [-1.0, 1.0],
		tex_coord: [0.0, 0.0],
	},
	Vertex {
		position: [1.0, -1.0],
		tex_coord: [1.0, 1.0],
	},
	Vertex {
		position: [1.0, 1.0],
		tex_coord: [1.0, 0.0],
	},
];

const SHADER: &str = r#"
struct VertexInput {
	@location(0) position: vec2<f32>,
	@location(1) tex_coord: vec2<f32>,
}

struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	@location(0) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(
	model: VertexInput,
) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
	out.tex_coord = model.tex_coord;
	return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return textureSample(video_texture, video_sampler, in.tex_coord);
}

@group(0) @binding(0)
var video_texture: texture_2d<f32>;

@group(0) @binding(1)
var video_sampler: sampler;
"#;

pub struct State {
	window: std::sync::Arc<winit::window::Window>,
	surface: wgpu::Surface<'static>,
	device: wgpu::Device,
	queue: wgpu::Queue,
	texture: Option<wgpu::Texture>,
	render_pipeline: Option<wgpu::RenderPipeline>,
	vertex_buffer: wgpu::Buffer,
	bind_group_layout: wgpu::BindGroupLayout,
	sampler: wgpu::Sampler,
	config: wgpu::SurfaceConfiguration,
}

impl State {
	pub async fn new(window: &std::sync::Arc<winit::window::Window>) -> Result<Self> {
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			backends: wgpu::Backends::all(),
			..Default::default()
		});

		let surface = instance
			.create_surface(window.clone())
			.map_err(|e| Error::Generic(f!("Failed to create surface: {:?}", e)))?;

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::HighPerformance,
				compatible_surface: Some(&surface),
				..Default::default()
			})
			.await
			.ok_or(Error::Static("No adapter found"))?;

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: Some("Device"),
					required_features: wgpu::Features::empty(),
					required_limits: wgpu::Limits::default(),
				},
				None,
			)
			.await
			.map_err(|e| Error::Generic(f!("Failed to request device: {:?}", e)))?;

		let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Vertex Buffer"),
			size: std::mem::size_of_val(VERTICES) as u64,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(VERTICES));

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Texture Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some("Texture Sampler"),
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Linear,
			..Default::default()
		});

		let size = window.inner_size();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			view_formats: vec![],
			desired_maximum_frame_latency: 2,
		};

		surface.configure(&device, &config);

		log::info!(
			"Renderer created. Window size: {}x{}",
			size.width,
			size.height
		);

		Ok(Self {
			window: window.clone(),
			surface,
			device,
			queue,
			texture: None,
			render_pipeline: None,
			vertex_buffer,
			bind_group_layout,
			sampler,
			config,
		})
	}

	fn update_frame(&mut self, decoder: &std::sync::Arc<Decoder>) -> Result<()> {
		if let Some(sample) = decoder
			.appsink()
			.try_pull_sample(gstreamer::ClockTime::from_mseconds(0))
		{
			let buffer = sample.buffer().ok_or(Error::Static("No buffer"))?;

			let map = buffer
				.map_readable()
				.map_err(|_| Error::Static("Buffer not readable"))?;
			let data = map.as_slice();

			let caps = sample.caps().ok_or(Error::Static("No caps"))?;
			let info = caps.structure(0).ok_or(Error::Static("No structure"))?;

			let width = info
				.get::<i32>("width")
				.map_err(|_| Error::Static("No width"))? as u32;
			let height = info
				.get::<i32>("height")
				.map_err(|_| Error::Static("No height"))? as u32;

			log::debug!("Video frame: {}x{}", width, height);

			let texture_desc = wgpu::TextureDescriptor {
				label: Some("Video Texture"),
				size: wgpu::Extent3d {
					width,
					height,
					depth_or_array_layers: 1,
				},
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Rgba8UnormSrgb,
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				view_formats: &[],
			};

			let texture = self.device.create_texture(&texture_desc);

			self.queue.write_texture(
				wgpu::ImageCopyTexture {
					texture: &texture,
					mip_level: 0,
					origin: wgpu::Origin3d::ZERO,
					aspect: wgpu::TextureAspect::All,
				},
				data,
				wgpu::ImageDataLayout {
					offset: 0,
					bytes_per_row: Some(4 * width),
					rows_per_image: Some(height),
				},
				texture_desc.size,
			);

			self.texture = Some(texture);

			if self.render_pipeline.is_none() {
				self.create_render_pipeline()?;
			}
		}

		Ok(())
	}

	fn create_render_pipeline(&mut self) -> Result<()> {
		let shader = self
			.device
			.create_shader_module(wgpu::ShaderModuleDescriptor {
				label: Some("Shader"),
				source: wgpu::ShaderSource::Wgsl(SHADER.into()),
			});

		let layout = self
			.device
			.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[&self.bind_group_layout],
				push_constant_ranges: &[],
			});

		let render_pipeline = self
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: Some("Render Pipeline"),
				layout: Some(&layout),
				vertex: wgpu::VertexState {
					module: &shader,
					entry_point: "vs_main",
					buffers: &[wgpu::VertexBufferLayout {
						array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
						step_mode: wgpu::VertexStepMode::Vertex,
						attributes: &[
							wgpu::VertexAttribute {
								offset: 0,
								shader_location: 0,
								format: wgpu::VertexFormat::Float32x2,
							},
							wgpu::VertexAttribute {
								offset: 8,
								shader_location: 1,
								format: wgpu::VertexFormat::Float32x2,
							},
						],
					}],
					compilation_options: Default::default(),
				},
				fragment: Some(wgpu::FragmentState {
					module: &shader,
					entry_point: "fs_main",
					targets: &[Some(wgpu::ColorTargetState {
						format: wgpu::TextureFormat::Bgra8UnormSrgb,
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
			});

		self.render_pipeline = Some(render_pipeline);

		Ok(())
	}

	pub fn render(&mut self, decoder: &std::sync::Arc<Decoder>) -> Result<()> {
		self.update_frame(decoder)?;

		let frame = match self.surface.get_current_texture() {
			Ok(frame) => frame,
			Err(e) => {
				log::error!("Failed to get current texture: {}", e);
				return Err(Error::Generic(f!("Surface error: {:?}", e)));
			}
		};

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		if let (Some(texture), Some(pipeline)) = (&self.texture, &self.render_pipeline) {
			let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

			let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("Texture Bind Group"),
				layout: &self.bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&texture_view),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&self.sampler),
					},
				],
			});

			let mut encoder = self
				.device
				.create_command_encoder(&wgpu::CommandEncoderDescriptor {
					label: Some("Render Encoder"),
				});

			{
				let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					label: Some("Render Pass"),
					color_attachments: &[Some(wgpu::RenderPassColorAttachment {
						view: &view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(wgpu::Color {
								r: 0.0,
								g: 0.0,
								b: 0.0,
								a: 1.0,
							}),
							store: wgpu::StoreOp::Store,
						},
					})],
					depth_stencil_attachment: None,
					timestamp_writes: None,
					occlusion_query_set: None,
				});

				render_pass.set_pipeline(pipeline);
				render_pass.set_bind_group(0, &bind_group, &[]);
				render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
				render_pass.draw(0..6, 0..1);
			}

			self.queue.submit(Some(encoder.finish()));
		}

		frame.present();

		Ok(())
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
		}
	}
}
