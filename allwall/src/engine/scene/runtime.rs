use std::collections::HashMap;
use std::time::Duration;

use smithay_client_toolkit::{
    compositor::CompositorState,
    output::OutputInfo,
    reexports::client::{Connection, QueueHandle, protocol::wl_output::WlOutput},
    shell::WaylandSurface,
    shell::wlr_layer::{Anchor, Layer, LayerShell, LayerSurface},
};

use crate::{
    config::MergedSceneConfig,
    engine::{
        error::EngineError,
        graphics::{Context, GpuContext, RenderSurface},
        scene::{Fit, Layout, Monitor, MonitorHandle},
    },
    prelude::*,
    sources::{InteractionState, SourceKind, SourceType, grass::GrassSource, media::MediaSource, smoke::SmokeSource},
    transitions::TransitionType,
};

pub struct Scene {
    config: MergedSceneConfig,
    outputs: HashMap<WlOutput, SceneOutput>,
    sources: Vec<SourceType>,
    rotation_interval: Duration,
    transition_duration: Duration,
    transition_type: TransitionType,
    source_kind: SourceKind,
    smoke_config: crate::config::SmokeConfig,
    sources_initialized: bool,
}

struct SceneOutput {
    monitor: Monitor,
    context: Context,
    configured: bool,
}

impl Scene {
    pub fn new(config: MergedSceneConfig, source_kind: SourceKind, smoke_config: crate::config::SmokeConfig) -> Self {
        let transition_duration = config.transition.duration();
        let rotation_interval = config.transition.interval();
        let transition_type = config.transition.r#type;

        Self {
            config,
            outputs: HashMap::new(),
            sources: Vec::new(),
            rotation_interval,
            transition_duration,
            transition_type,
            source_kind,
            smoke_config,
            sources_initialized: false,
        }
    }

    pub fn should_handle_output(&self, output_name: &str) -> bool {
        self.config.monitors.matches(output_name)
    }

    pub fn on_output_added(
        &mut self,
        output: WlOutput,
        info: &OutputInfo,
        gpu: std::sync::Arc<GpuContext>,
        conn: &Connection,
        compositor: &CompositorState,
        layer_shell: &LayerShell,
        qh: &QueueHandle<crate::engine::Engine>,
    ) -> Result<()> {
        let output_name = info.name.as_deref().unwrap_or("unknown");
        if !self.should_handle_output(output_name) {
            return Ok(());
        }

        if self.outputs.contains_key(&output) {
            info!("Output {} already managed by scene", output_name);
            return Ok(());
        }

        info!("Adding output '{}' to scene", output_name);

        let handle = MonitorHandle::new(output_name.to_string());
        let size = info.logical_size.map(|(w, h)| (w as u32, h as u32)).unwrap_or((1920, 1080));

        let surface = compositor.create_surface(qh);
        let layer = layer_shell.create_layer_surface(qh, surface, Layer::Background, Some("wallpaper"), Some(&output));
        layer.set_anchor(Anchor::all());
        layer.set_size(0, 0);
        layer.set_exclusive_zone(-1);

        if let Ok(region) = smithay_client_toolkit::compositor::Region::new(compositor) {
            layer.set_input_region(Some(region.wl_region()));
            region.wl_region().destroy();
        }

        layer.commit();

        let render_surface =
            RenderSurface::new(&gpu, conn, &layer, size).map_err(|e| EngineError::Render(e.to_string()))?;
        let context = Context::from_parts(gpu, render_surface);
        let monitor = Monitor::new(handle, layer, output.clone(), info.clone());

        self.outputs.insert(
            output,
            SceneOutput {
                monitor,
                context,
                configured: false,
            },
        );

        if self.outputs.len() == 1 && !self.sources_initialized {
            self.initialize_sources()?;
        }

        Ok(())
    }

    pub fn on_output_removed(&mut self, output: &WlOutput) {
        if let Some(scene_output) = self.outputs.remove(output) {
            let name = scene_output.monitor.handle().name();
            info!("Removed output '{}' from scene", name);
        }
    }

    pub fn on_output_updated(&mut self, output: &WlOutput, info: &OutputInfo) {
        if let Some(scene_output) = self.outputs.get_mut(output) {
            if let Some(size) = info.logical_size {
                let new_size = (size.0 as u32, size.1 as u32);
                scene_output.context.resize(new_size);
                info!(
                    "Resized output '{}' to {:?}",
                    scene_output.monitor.handle().name(),
                    new_size
                );
            }
        }
    }

    pub fn on_layer_configure(&mut self, layer: &LayerSurface, width: u32, height: u32) {
        for scene_output in self.outputs.values_mut() {
            if std::ptr::eq(scene_output.monitor.layer().wl_surface(), layer.wl_surface()) {
                if width > 0 && height > 0 {
                    scene_output.context.resize((width, height));
                }
                if !scene_output.configured {
                    scene_output.configured = true;
                    info!(
                        "Output '{}' configured with size {}x{}",
                        scene_output.monitor.handle().name(),
                        width,
                        height
                    );
                }
                break;
            }
        }
    }

    fn initialize_sources(&mut self) -> Result<()> {
        if self.outputs.is_empty() {
            return Ok(());
        }

        let num_sources = match self.config.layout {
            Layout::Clone | Layout::Span => 1,
            Layout::Independent => self.outputs.len(),
        };

        self.sources.clear();
        let contexts: Vec<_> = self.outputs.values().collect();

        for i in 0..num_sources {
            let ctx = &contexts[i % contexts.len()].context;
            let mut source = self.create_source(ctx)?;
            source.load(ctx)?;
            source.start_transition(None, self.transition_duration, ctx, self.transition_type);
            self.sources.push(source);
        }

        self.sources_initialized = true;
        info!(
            "Scene initialized with {} sources for {} outputs",
            self.sources.len(),
            self.outputs.len()
        );

        Ok(())
    }

    fn create_source(&self, ctx: &Context) -> Result<SourceType> {
        match self.source_kind {
            SourceKind::Media => {
                let path = self
                    .config
                    .path
                    .as_ref()
                    .ok_or_else(|| Error::Generic("Media source requires path".to_string()))?;
                let source = MediaSource::from_directory(path, ctx)?;
                Ok(SourceType::Media(Box::new(source)))
            },
            SourceKind::Smoke => {
                let source = SmokeSource::new(ctx, self.smoke_config.clone());
                Ok(SourceType::Smoke(Box::new(source)))
            },
            SourceKind::Grass => {
                let source = GrassSource::new(ctx);
                Ok(SourceType::Grass(Box::new(source)))
            },
        }
    }

    pub fn update(&mut self, dt: Duration) {
        for source in &mut self.sources {
            source.update(dt);
        }
    }

    pub fn render(&mut self, state: &InteractionState) {
        if self.sources.is_empty() || self.outputs.is_empty() {
            return;
        }

        match self.config.layout {
            Layout::Clone => self.render_clone(state),
            Layout::Span => self.render_span(state),
            Layout::Independent => self.render_independent(state),
        }
    }

    fn render_clone(&mut self, state: &InteractionState) {
        if self.sources.is_empty() {
            return;
        }
        let source = &mut self.sources[0];
        for scene_output in self.outputs.values_mut() {
            if scene_output.configured {
                source.render(&scene_output.context, state);
            }
        }
    }

    fn render_span(&mut self, state: &InteractionState) {
        if self.sources.is_empty() {
            return;
        }
        let source = &mut self.sources[0];
        if let Some(first) = self.outputs.values().find(|o| o.configured) {
            source.render(&first.context, state);
        }
    }

    fn render_independent(&mut self, state: &InteractionState) {
        for (i, scene_output) in self.outputs.values_mut().enumerate() {
            if i < self.sources.len() && scene_output.configured {
                self.sources[i].render(&scene_output.context, state);
            }
        }
    }

    pub fn advance_source(&mut self) -> Result<()> {
        if self.outputs.is_empty() || self.sources.is_empty() {
            return Ok(());
        }

        let configured_contexts: Vec<_> = self.outputs.values().filter(|o| o.configured).collect();

        if configured_contexts.is_empty() {
            return Ok(());
        }

        match self.config.layout {
            Layout::Clone | Layout::Span => {
                let ctx = &configured_contexts[0].context;
                let new_source = self.sources[0].next(ctx)?;
                let old_source = std::mem::replace(&mut self.sources[0], new_source);
                self.sources[0].start_transition(Some(old_source), self.transition_duration, ctx, self.transition_type);
            },
            Layout::Independent => {
                for (i, scene_output) in configured_contexts.iter().enumerate() {
                    if i < self.sources.len() {
                        let ctx = &scene_output.context;
                        let new_source = self.sources[i].next(ctx)?;
                        let old_source = std::mem::replace(&mut self.sources[i], new_source);
                        self.sources[i].start_transition(
                            Some(old_source),
                            self.transition_duration,
                            ctx,
                            self.transition_type,
                        );
                    }
                }
            },
        }
        Ok(())
    }

    pub fn rotation_interval(&self) -> Duration {
        self.rotation_interval
    }

    pub fn is_media(&self) -> bool {
        matches!(self.source_kind, SourceKind::Media)
    }

    pub fn config(&self) -> &MergedSceneConfig {
        &self.config
    }

    pub fn layout(&self) -> Layout {
        self.config.layout
    }

    pub fn fit(&self) -> Fit {
        self.config.fit
    }

    pub fn monitor_count(&self) -> usize {
        self.outputs.len()
    }

    pub fn has_outputs(&self) -> bool {
        !self.outputs.is_empty()
    }
}
