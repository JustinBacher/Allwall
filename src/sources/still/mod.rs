use std::iter::once;

use image::DynamicImage;
use log::{debug, error};

use crate::sources::{
	create_index_buffer, create_pipeline, create_texture_binds, create_uniform_binds,
	create_vertex_buffer, RendererState, Source, INDICES,
};

use crate::engine::{Context, Texture};

pub struct Still {
	texture: Texture,
	texture_bind_group: wgpu::BindGroup,

	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,

	uniform_buffer: wgpu::Buffer,
	uniform_bind_group: wgpu::BindGroup,

	render_pipeline: wgpu::RenderPipeline,

	state: RendererState,
}

impl Still {
	pub fn new(img: &DynamicImage, ctx: &Context) -> Self {
		debug!("Creating Still source from image");
		let texture = Texture::from_image(img, ctx);

		let (texture_bind_group_layout, texture_bind_group) =
			create_texture_binds(&[&texture], ctx);

		let vertex_buffer = create_vertex_buffer(ctx);
		let index_buffer = create_index_buffer(ctx);

		let (uniform_buffer, uniform_bind_group_layout, uniform_bind_group) =
			create_uniform_binds(32, ctx);

		let render_pipeline = create_pipeline(
			ctx,
			&[&texture_bind_group_layout, &uniform_bind_group_layout],
			&ctx.device()
				.create_shader_module(wgpu::include_wgsl!("./shaders/static.wgsl")),
			ctx.config(),
		);

		let state = RendererState::default();

		Self {
			texture,
			texture_bind_group,
			vertex_buffer,
			index_buffer,
			uniform_buffer,
			uniform_bind_group,
			render_pipeline,
			state,
		}
	}

	pub fn update_texture(&mut self, img: &DynamicImage, ctx: &Context) {
		debug!("Updating Still texture");
		let texture = Texture::from_image(img, ctx);
		self.texture = texture;
		let (_, bindgroup) = create_texture_binds(&[&self.texture], ctx);
		self.texture_bind_group = bindgroup;
	}
}

impl Source for Still {
	fn render(&self, ctx: &Context) {
		let queue = ctx.queue();
		let device = ctx.device();
		let surface = ctx.surface();

		debug!(
			"Still rendering, surface aspect: {:.2}, texture aspect: {:.2}",
			ctx.surface_aspect_ratio(),
			self.texture.aspect_ratio()
		);

		let output = surface.get_current_texture();
		if let Err(e) = output {
			error!("Could not get texture from surface: {e}");
			return;
		}
		let output = output.unwrap();
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

		debug!("Still render complete");
	}

	fn texture(&self) -> &Texture {
		&self.texture
	}

	fn update_texture(&mut self, img: &DynamicImage, ctx: &Context) {
		debug!("Updating Still texture via Source trait");
		let texture = Texture::from_image(img, ctx);
		self.texture = texture;
		let (_, bindgroup) = create_texture_binds(&[&self.texture], ctx);
		self.texture_bind_group = bindgroup;
	}

	fn state(&self) -> &RendererState {
		&self.state
	}
}
