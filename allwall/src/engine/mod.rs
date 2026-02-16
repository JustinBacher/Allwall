pub mod error;
pub mod events;
pub mod graphics;
pub mod scene;
pub mod wayland;

use std::time::{Duration, Instant};

use calloop::{
    Interest, Mode, PostAction,
    generic::Generic,
    timer::{TimeoutAction, Timer},
};
pub use graphics::{Context, GpuContext, RenderSurface, Texture};
use scene::Scene;
use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputState,
    reexports::client::{self, Connection, globals::registry_queue_init},
    registry::RegistryState,
    shell::wlr_layer::LayerShell,
};

use crate::{
    cli::ipc::protocol::socket_path,
    config::AppConfig,
    engine::error::EngineError,
    prelude::{Result, error, info},
    sources::{InteractionState, SourceKind},
};
pub use scene::{Fit, Layout, MonitorsSpec, SceneConfig};

pub struct Engine {
    pub registry_state: RegistryState,
    pub output_state: OutputState,
    pub compositor_state: CompositorState,
    pub layer_shell: LayerShell,
    pub conn: Connection,
    pub gpu: std::sync::Arc<GpuContext>,
    pub scenes: Vec<Scene>,
    pub fps: f32,
    pub source_kind: SourceKind,
    pub interaction_state: InteractionState,
    pub qh: client::QueueHandle<Engine>,
}

impl Engine {
    pub fn run(config: AppConfig, source_kind: SourceKind) -> Result<()> {
        let total_start = Instant::now();
        info!("Starting Allwall...");

        let fps = config.general.fps;
        let smoke_config = config.smoke.clone();

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
        info!("Wayland protocols bound in {:?}", start.elapsed());

        let start = Instant::now();
        let gpu = pollster::block_on(GpuContext::new())?;
        info!("WGPU context created in {:?}", start.elapsed());

        let start = Instant::now();
        let gpu = std::sync::Arc::new(gpu);
        let scenes = create_scenes(&config, source_kind, smoke_config);
        info!("Scenes created in {:?}", start.elapsed());

        if scenes.is_empty() {
            return Err(EngineError::NoScenes.into());
        }

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
            gpu,
            scenes,
            fps: fps as f32,
            source_kind,
            interaction_state: InteractionState::default(),
            qh,
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
                for scene in &mut engine.scenes {
                    scene.update(dt);
                    scene.render(&engine.interaction_state);
                }
                TimeoutAction::ToDuration(Duration::from_secs_f32(1.0 / engine.fps))
            },
        );

        for scene_idx in 0..engine.scenes.len() {
            if engine.scenes[scene_idx].is_media() {
                let rotation_interval = engine.scenes[scene_idx].rotation_interval();
                let _ =
                    event_loop_handler.insert_source(Timer::from_duration(rotation_interval), move |_, _, engine| {
                        if let Some(scene) = engine.scenes.get_mut(scene_idx) {
                            if let Err(e) = scene.advance_source() {
                                error!("Could not advance source: {e}");
                            }
                        }
                        TimeoutAction::ToDuration(rotation_interval)
                    });
            }
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

        info!("Starting event loop - outputs will be handled via Wayland events");
        event_loop.run(None, &mut engine, |_| ())?;

        Ok(())
    }
}

fn create_scenes(config: &AppConfig, source_kind: SourceKind, smoke_config: crate::config::SmokeConfig) -> Vec<Scene> {
    if config.scenes.is_empty() {
        let scene_config = crate::config::MergedSceneConfig {
            path: None,
            layout: Default::default(),
            fit: Default::default(),
            monitors: Default::default(),
            transition: config.transition.clone(),
        };
        info!("Creating default scene (matches all monitors)");
        return vec![Scene::new(scene_config, source_kind, smoke_config)];
    }

    config
        .scenes
        .iter()
        .enumerate()
        .map(|(i, scene_config)| {
            info!(
                "Creating scene {} (monitors: {:?})",
                i,
                scene_config
                    .monitors
                    .monitors()
                    .map(|m| m.iter().map(|h| h.name()).collect::<Vec<_>>())
                    .unwrap_or_else(|| vec!["*"])
            );
            Scene::new(scene_config.clone(), source_kind, smoke_config.clone())
        })
        .collect()
}
