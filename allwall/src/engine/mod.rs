pub mod error;
pub mod events;
pub mod graphics;
pub mod scene;
pub mod wayland;

use std::{
    path::Path,
    time::{Duration, Instant},
};

use calloop::{
    Interest, Mode, PostAction,
    generic::Generic,
    timer::{TimeoutAction, Timer},
};
use client::{Connection, globals::registry_queue_init};
pub use graphics::{Context, Texture};
use image::DynamicImage;
use rand::seq::SliceRandom;
pub use scene::{Fit, Layout, MonitorHandle, MonitorsSpec, SceneConfig};
use smithay_client_toolkit::{
    compositor::{CompositorState, Region},
    output::OutputState,
    reexports::client,
    registry::RegistryState,
    shell::{
        WaylandSurface,
        wlr_layer::{Anchor, Layer, LayerShell},
    },
};

use crate::{
    cli::ipc::protocol::socket_path,
    config::AppConfig,
    engine::error::EngineError,
    prelude::{Result, error, info, warn},
    sources::{
        BasicSource, InteractionState, Source, SourceKind, SourceType, grass::GrassSource, media::MediaSource,
        smoke::SmokeSource,
    },
    transitions::TransitionType,
};

pub struct Engine {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,

    pub current_source: SourceType,
    pub rotation_interval: Duration,
    pub transition_duration: Duration,
    pub transition_type: TransitionType,
    pub fps: f32,

    pub ctx: Context,
    pub conn: Connection,
    pub layer: smithay_client_toolkit::shell::wlr_layer::LayerSurface,
    pub source_kind: SourceKind,

    pub interaction_state: InteractionState,
}

impl Engine {
    pub fn run(config: AppConfig, source_kind: SourceKind) -> Result<()> {
        let total_start = Instant::now();
        info!("Starting Allwall...");

        let transition_config = if config.scenes.is_empty() {
            config.transition.clone()
        } else {
            config.scenes[0].transition.clone()
        };

        let img_dir = if !config.scenes.is_empty() {
            config.scenes[0].path.clone()
        } else {
            None
        };

        let fps = config.general.fps;

        let start = Instant::now();
        info!("Connecting to Wayland...");
        let conn = Connection::connect_to_env().map_err(|e| EngineError::WaylandConnect(e.to_string()))?;
        let (globals, queue) =
            registry_queue_init::<Engine>(&conn).map_err(|e| EngineError::WaylandRegistry(e.to_string()))?;
        let qh = queue.handle();
        info!("Wayland connected in {:?}", start.elapsed());

        let start = Instant::now();
        let registry_state = RegistryState::new(&globals);
        let compositor_state = CompositorState::bind(&globals, &qh).map_err(|_| EngineError::NoCompositor)?;
        let output_state = OutputState::new(&globals, &qh);
        let layer_shell = LayerShell::bind(&globals, &qh).map_err(|_| EngineError::NoLayerShell)?;
        let surface = compositor_state.create_surface(&qh);
        info!("Wayland protocols bound in {:?}", start.elapsed());

        let start = Instant::now();
        let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Background, Some("wallpaper"), None);
        layer.set_anchor(Anchor::all());
        layer.set_size(0, 0);
        layer.set_exclusive_zone(-1);

        match Region::new(&compositor_state) {
            Ok(region) => {
                layer.set_input_region(Some(region.wl_region()));
                region.wl_region().destroy();
            },
            Err(e) => {
                warn!("Failed to set input region, background may not have cursor: {e}");
            },
        }

        layer.commit();
        info!("Layer surface created in {:?}", start.elapsed());

        let start = Instant::now();
        let ctx = pollster::block_on(Context::new(&conn, &layer, (256, 256)))?;
        info!("WGPU context created in {:?}", start.elapsed());

        let transition_duration = transition_config.duration();
        let rotation_interval = transition_config.interval();
        let transition_type = transition_config.r#type;
        let smoke_config = config.smoke.clone();

        let start = Instant::now();
        let mut current_source: SourceType = match source_kind {
            SourceKind::Media => {
                let img_dir_ref = img_dir.as_ref().ok_or(EngineError::MediaPathRequired)?;
                let mut source = MediaSource::from_directory(img_dir_ref, &ctx)?;
                source.load(&ctx)?;
                info!("Media source initialized in {:?}", start.elapsed());
                SourceType::Media(Box::new(source))
            },
            SourceKind::Smoke => {
                info!("Creating SmokeSource in {:?}", start.elapsed());
                let mut source = SmokeSource::new(&ctx, smoke_config);
                source.load(&ctx)?;
                SourceType::Smoke(Box::new(source))
            },
            SourceKind::Grass => {
                info!("Creating GrassSource in {:?}", start.elapsed());
                let mut source = GrassSource::new(&ctx);
                source.load(&ctx)?;
                SourceType::Grass(Box::new(source))
            },
        };
        current_source.start_transition(None, transition_duration, &ctx, transition_type);

        let engine_init_start = Instant::now();
        let mut event_loop: calloop::EventLoop<Engine> =
            calloop::EventLoop::try_new().map_err(|e| EngineError::EventLoopCreate(e.to_string()))?;
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

            rotation_interval,
            transition_duration,
            transition_type,
            fps: fps as f32,
            source_kind,
            interaction_state: InteractionState::default(),
        };

        info!("Engine initialized in {:?}", engine_init_start.elapsed());
        info!("Total startup time: {:?}", total_start.elapsed());

        let socket = socket_path();
        if let Some(parent) = socket.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let listener = std::os::unix::net::UnixListener::bind(&socket)?;
        listener.set_nonblocking(true)?;
        info!("IPC socket listening at {}", socket.display());

        let _ = event_loop_handler.insert_source(
            Generic::new(listener, Interest::READ, Mode::Level),
            |_readiness, listener, engine| {
                if let Ok((client, _)) = listener.accept()
                    && let Err(e) = engine.handle_ipc_client(client)
                {
                    error!("IPC client error: {e}");
                }
                Ok(PostAction::Continue)
            },
        );

        let frame_fps = engine.fps;
        let _ = event_loop_handler.insert_source(
            Timer::from_duration(Duration::from_secs_f32(1.0 / frame_fps)),
            move |_, _, engine| {
                let dt = Duration::from_secs_f32(1.0 / engine.fps);
                engine.current_source.update(dt);
                engine.current_source.render(&engine.ctx, &engine.interaction_state);
                TimeoutAction::ToDuration(Duration::from_secs_f32(1.0 / engine.fps))
            },
        );

        if matches!(source_kind, SourceKind::Media) {
            let rotation_interval = engine.rotation_interval;
            let transition_duration = engine.transition_duration;
            let transition_type = engine.transition_type;
            let _ = event_loop_handler.insert_source(Timer::from_duration(rotation_interval), move |_, _, engine| {
                match engine.current_source.next(&engine.ctx) {
                    Ok(new_source) => {
                        info!("Starting transition to new image");
                        let old_source = std::mem::replace(&mut engine.current_source, new_source);
                        engine.current_source.start_transition(
                            Some(old_source),
                            transition_duration,
                            &engine.ctx,
                            transition_type,
                        );
                    },
                    Err(e) => {
                        error!("Could not load new img: {e}");
                    },
                }
                TimeoutAction::ToDuration(rotation_interval)
            });
        }

        ctrlc::set_handler({
            let loop_signal = event_loop.get_signal();
            let socket = socket_path();
            move || {
                info!("SIGTERM/SIGINT/SIGHUP received, exiting");
                let _ = std::fs::remove_file(&socket);
                loop_signal.stop();
                loop_signal.wakeup();
            }
        })
        .map_err(|_| EngineError::SignalHandler)?;

        smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource::new(engine.conn.clone(), queue)
            .insert(event_loop.handle())
            .map_err(|e| EngineError::WaylandSourceInsert(e.to_string()))?;

        event_loop.run(None, &mut engine, |_| ())?;

        Ok(())
    }

    fn load_random_img(dir: &Path) -> Option<DynamicImage> {
        let mut rng = rand::rng();
        let mut files: Vec<_> = dir
            .read_dir()
            .ok()?
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
}
