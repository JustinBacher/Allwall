use std::iter::once;
use std::time::Duration;

use image::DynamicImage;
use log::{debug, error};

use crate::prelude::Result;
use crate::sources::{
	create_index_buffer, create_pipeline, create_texture_binds, create_uniform_binds,
	create_vertex_buffer, RenderState, Source, INDICES,
};
use crate::transitions::{
	CircleOrigin, CircleRevealTransition, FadeTransition, Transition, TransitionType,
};

use crate::engine::{Context, Texture};

#[derive(Debug)]
pub struct Still {
	texture: Texture,
	texture_bind_group: wgpu::BindGroup,

	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,

	uniform_buffer: wgpu::Buffer,
	uniform_bind_group: wgpu::BindGroup,

	render_pipeline: wgpu::RenderPipeline,

	state: RenderState,
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

		let state = RenderState::default();

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

	fn render_normal(&self, ctx: &Context) {
		let queue = ctx.queue();
		let device = ctx.device();
		let surface = ctx.surface();

		debug!(
			"Still rendering, surface aspect: {:.2}, texture aspect: {:.2}",
			ctx.surface_aspect_ratio(),
			self.texture.aspect_ratio()
		);

		let output = match surface.get_current_texture() {
			Ok(output) => output,
			Err(e) => {
				error!("Could not get texture from surface: {e}");
				return;
			}
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

		debug!("Still render complete");
	}
}

impl Source for Still {
	fn render(&mut self, ctx: &Context) {
		match &self.state {
			RenderState::Transitioning(transition) => {
				transition.render(ctx, self);
			}
			_ => {
				self.render_normal(ctx);
			}
		}
	}

	fn texture(&self) -> &Texture {
		&self.texture
	}

	fn state(&self) -> &RenderState {
		&self.state
	}

	fn load(&mut self, _ctx: &Context) -> Result<()> {
		debug!("Loading Still source");
		self.state = RenderState::Displaying;
		Ok(())
	}

	fn start_transition(
		&mut self,
		previous: Option<Box<dyn Source>>,
		duration: Duration,
		ctx: &Context,
		transition_type: TransitionType,
	) {
		debug!(
			"Starting {:?} transition with duration {:?}",
			transition_type, duration
		);
		let transition: Box<dyn Transition> = match transition_type {
			TransitionType::Fade => Box::new(FadeTransition::new(previous, duration, ctx)),
			TransitionType::CircleTopLeft => Box::new(CircleRevealTransition::new(
				previous,
				duration,
				CircleOrigin::TopLeft,
				ctx,
			)),
			TransitionType::CircleTopRight => Box::new(CircleRevealTransition::new(
				previous,
				duration,
				CircleOrigin::TopRight,
				ctx,
			)),
			TransitionType::CircleBottomLeft => Box::new(CircleRevealTransition::new(
				previous,
				duration,
				CircleOrigin::BottomLeft,
				ctx,
			)),
			TransitionType::CircleBottomRight => Box::new(CircleRevealTransition::new(
				previous,
				duration,
				CircleOrigin::BottomRight,
				ctx,
			)),
			TransitionType::CircleCenter => Box::new(CircleRevealTransition::new(
				previous,
				duration,
				CircleOrigin::Center,
				ctx,
			)),
			TransitionType::CircleRandom => Box::new(CircleRevealTransition::new(
				previous,
				duration,
				CircleOrigin::Random,
				ctx,
			)),
		};
		self.state = RenderState::Transitioning(transition);
	}

	fn update(&mut self, dt: Duration) {
		match &mut self.state {
			RenderState::Transitioning(ref mut transition) => {
				let complete = transition.update(dt);
				if complete {
					debug!("Transition complete, switching to Displaying");
					self.state = RenderState::Displaying;
				}
			}
			_ => {}
		}
	}
}
