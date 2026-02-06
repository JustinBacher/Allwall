use super::AllwallCommand;
use crate::decode::Decoder;
use crate::prelude::*;
use crate::renderer::State;

#[derive(clap::Parser, Debug)]
#[command()]
pub struct Run {
	#[arg(short, long)]
	video: String,
}

impl AllwallCommand for Run {
	async fn execute(&self) -> Result<()> {
		log::info!("Initializing GStreamer...");
		gstreamer::init().map_err(|e| Error::Generic(f!("Failed to init GStreamer: {:?}", e)))?;

		log::info!("Creating decoder for video: {}", self.video);
		let decoder = Decoder::new(&self.video)?;

		log::info!("Creating window and renderer...");
		let event_loop = winit::event_loop::EventLoop::new()
			.map_err(|e| Error::Generic(f!("Failed to create event loop: {:?}", e)))?;

		let window = std::sync::Arc::new(
			winit::window::WindowBuilder::new()
				.with_title("Video Wallpaper MVP")
				.with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
				.build(&event_loop)
				.map_err(|e| Error::Generic(f!("Failed to create window: {:?}", e)))?,
		);

		let mut state = State::new(&window).await?;
		let decoder = std::sync::Arc::new(decoder);

		log::info!("Starting event loop...");

		event_loop
			.run(move |event, window_target| {
				window_target.set_control_flow(winit::event_loop::ControlFlow::Poll);

				match event {
					winit::event::Event::WindowEvent {
						event: winit::event::WindowEvent::CloseRequested,
						..
					} => {
						log::info!("Close requested. Exiting.");
						window_target.exit();
					}

					winit::event::Event::WindowEvent {
						event: winit::event::WindowEvent::Resized(new_size),
						..
					} => {
						state.resize(new_size);
					}

					winit::event::Event::AboutToWait => {
						if let Err(e) = state.render(&decoder) {
							log::error!("Render error: {:?}", e);
						}
					}

					_ => {}
				}
			})
			.map_err(|e| Error::Generic(f!("Event loop error: {:?}", e)))?;

		Ok(())
	}
}
