mod context;
mod texture;

use image::DynamicImage;
use rand::seq::SliceRandom;
use std::{
	path::{Path, PathBuf},
	time::{Duration, Instant},
};

use crate::prelude::{Error, Result};
use log::*;

use calloop::EventLoop;
use client::{
	globals::registry_queue_init,
	protocol::{
		wl_output::{self},
		wl_surface,
	},
	Connection, QueueHandle,
};
use smithay_client_toolkit::{
	compositor::{CompositorHandler, CompositorState, Region},
	delegate_compositor, delegate_layer, delegate_output, delegate_registry,
	output::{OutputHandler, OutputState},
	reexports::{calloop_wayland_source::WaylandSource, client},
	registry::{ProvidesRegistryState, RegistryState},
	shell::{
		wlr_layer::{Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface},
		WaylandSurface,
	},
};

use calloop::timer::{TimeoutAction, Timer};

use crate::sources::Source;

const FPS: f32 = 60.0;
const MIN_FPS: f32 = 5.0;

pub use context::Context;
pub use texture::Texture;

pub struct Engine {
	registry_state: RegistryState,
	output_state: OutputState,
	compositor_state: CompositorState,
	layer_shell: LayerShell,

	current_source: Box<dyn Source>,
	img_dir: PathBuf,
	rotation_interval: Duration,
	configured: bool,
	frame_timer: FrameTimer,

	ctx: Context,
	conn: Connection,
	layer: LayerSurface,
}

impl Engine {
	pub fn run(
		img_dir: PathBuf,
		transition_duration: Duration,
		rotation_interval: Duration,
	) -> Result<()> {
		let total_start = Instant::now();
		info!("Starting Allwall...");

		let start = Instant::now();
		info!("Connecting to Wayland...");
		let conn = Connection::connect_to_env().unwrap();
		let (globals, queue) = registry_queue_init::<Engine>(&conn).unwrap();
		let qh = queue.handle();
		info!("Wayland connected in {:?}", start.elapsed());

		let start = Instant::now();
		let registry_state = RegistryState::new(&globals);
		let compositor_state =
			CompositorState::bind(&globals, &qh).expect("Compositor not available");
		let output_state = OutputState::new(&globals, &qh);
		let layer_shell = LayerShell::bind(&globals, &qh).expect("Layer shell not available");
		let surface = compositor_state.create_surface(&qh);
		info!("Wayland protocols bound in {:?}", start.elapsed());

		let start = Instant::now();
		let layer = layer_shell.create_layer_surface(
			&qh,
			surface,
			Layer::Background,
			Some("wallpaper"),
			None,
		);
		layer.set_anchor(Anchor::all());
		layer.set_size(0, 0);
		layer.set_exclusive_zone(-1);

		match Region::new(&compositor_state) {
			Ok(region) => {
				layer.set_input_region(Some(region.wl_region()));
				region.wl_region().destroy();
			}
			Err(e) => {
				warn!("Failed to set input region, background may not have cursor: {e}");
			}
		}

		layer.commit();
		info!("Layer surface created in {:?}", start.elapsed());

		let start = Instant::now();
		let ctx = pollster::block_on(Context::new(&conn, &layer, (256, 256)));
		info!("WGPU context created in {:?}", start.elapsed());

		let start = Instant::now();
		let initial_img = Self::load_random_img(&img_dir)
			.ok_or_else(|| Error::NoImages(format!("No images found in {}", img_dir.display())))?;
		info!("Initial image loaded in {:?}", start.elapsed());

		let start = Instant::now();
		use crate::sources::still::Still;
		let current_source = Box::new(Still::new(&initial_img, &ctx));
		info!("Still source initialized in {:?}", start.elapsed());

		let engine_init_start = Instant::now();
		let mut event_loop: EventLoop<Engine> = EventLoop::try_new().unwrap();
		let event_loop_handler = event_loop.handle();
		let mut engine = Engine {
			conn,
			registry_state,
			output_state,
			compositor_state,
			layer_shell,
			current_source,
			ctx,
			layer,

			img_dir,
			rotation_interval,
			configured: false,
			frame_timer: FrameTimer::new(FPS),
		};

		info!("Engine initialized in {:?}", engine_init_start.elapsed());
		info!("Total startup time: {:?}", total_start.elapsed());

		let _ = event_loop_handler.insert_source(
			Timer::from_deadline(engine.frame_timer.next_frame()),
			|_, _, engine| {
				engine.current_source.render(&engine.ctx);
				TimeoutAction::ToInstant(engine.frame_timer.next_frame())
			},
		);

		let _ = event_loop_handler.insert_source(
			Timer::from_duration(engine.rotation_interval),
			|_, _, engine| {
				match engine.load_img_result() {
					Ok(img) => {
						info!("Loading new image");
						engine.current_source.update_texture(&img, &engine.ctx);
					}
					Err(e) => {
						error!("Could not load new img: {e}");
					}
				}
				TimeoutAction::ToDuration(engine.rotation_interval)
			},
		);

		ctrlc::set_handler({
			let loop_signal = event_loop.get_signal();
			move || {
				info!("SIGTERM/SIGINT/SIGHUP received, exiting");
				loop_signal.stop();
				loop_signal.wakeup();
			}
		})
		.unwrap();

		WaylandSource::new(engine.conn.clone(), queue)
			.insert(event_loop.handle())
			.map_err(|e| Error::Generic(format!("Failed to insert wayland source: {}", e)))?;

		event_loop.run(None, &mut engine, |_| ())?;

		Ok(())
	}

	fn load_random_img(dir: &Path) -> Option<DynamicImage> {
		let mut rng = rand::rng();
		let mut files: Vec<_> = dir
			.read_dir()
			.unwrap()
			.filter_map(std::result::Result::ok)
			.map(|d| d.path())
			.filter(|p| p.is_file())
			.collect();

		if files.is_empty() {
			return None;
		}

		files.shuffle(&mut rng);
		files
			.into_iter()
			.filter_map(|p| {
				info!("Attempting to load image: {}", p.display());
				image::open(&p).ok()
			})
			.next()
	}

	fn load_img_result(&self) -> Result<DynamicImage> {
		Self::load_random_img(&self.img_dir).ok_or_else(|| {
			Error::NoImages(format!(
				"Unable to load any image from {}",
				self.img_dir.display()
			))
		})
	}
}

impl CompositorHandler for Engine {
	fn scale_factor_changed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &wl_surface::WlSurface,
		_new_factor: i32,
	) {
	}

	fn transform_changed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &wl_surface::WlSurface,
		_new_transform: wl_output::Transform,
	) {
	}

	fn frame(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &wl_surface::WlSurface,
		_time: u32,
	) {
	}

	fn surface_enter(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &wl_surface::WlSurface,
		_output: &wl_output::WlOutput,
	) {
	}

	fn surface_leave(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &wl_surface::WlSurface,
		_output: &wl_output::WlOutput,
	) {
	}
}

impl OutputHandler for Engine {
	fn output_state(&mut self) -> &mut OutputState {
		&mut self.output_state
	}

	fn new_output(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: wl_output::WlOutput,
	) {
	}

	fn update_output(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: wl_output::WlOutput,
	) {
	}

	fn output_destroyed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: wl_output::WlOutput,
	) {
	}
}

impl LayerShellHandler for Engine {
	fn configure(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
		_configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
		_serial: u32,
	) {
		info!("Layer configure event: new_size={:?}", _configure.new_size);
		self.ctx.resize(_configure.new_size);
		self.configured = true;
		self.current_source.render(&self.ctx);
	}

	fn closed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
	) {
		warn!("Surface closed");
	}
}

impl ProvidesRegistryState for Engine {
	fn registry(&mut self) -> &mut RegistryState {
		&mut self.registry_state
	}

	fn runtime_add_global(
		&mut self,
		_conn: &client::Connection,
		_qh: &QueueHandle<Self>,
		_name: u32,
		_interface: &str,
		_version: u32,
	) {
	}

	fn runtime_remove_global(
		&mut self,
		_conn: &client::Connection,
		_qh: &QueueHandle<Self>,
		_name: u32,
		_interface: &str,
	) {
	}
}

delegate_compositor!(Engine);
delegate_layer!(Engine);
delegate_output!(Engine);
delegate_registry!(Engine);

struct FrameTimer {
	fps: f32,
	start: Instant,
}

impl FrameTimer {
	fn new(fps: f32) -> Self {
		Self {
			fps,
			start: Instant::now(),
		}
	}

	fn frametime(&self) -> Duration {
		Duration::from_secs_f32(1.0 / self.fps)
	}

	fn start(&mut self) -> bool {
		if (self.start + self.frametime()) > Instant::now() {
			return false;
		}
		self.start = Instant::now();
		true
	}

	fn next_frame(&self) -> Instant {
		self.start + self.frametime()
	}

	fn set_fps(&mut self, fps: f32) {
		self.fps = fps;
	}
}
