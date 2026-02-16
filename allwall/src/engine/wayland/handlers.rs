use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_compositor, delegate_layer, delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::client::{
        self, Connection, QueueHandle,
        protocol::{wl_output, wl_surface},
    },
    registry::{ProvidesRegistryState, RegistryState},
    shell::wlr_layer::{self, LayerShellHandler},
};

use crate::{
    engine::Engine,
    prelude::{info, warn},
};

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

    fn frame(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _time: u32) {}

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

impl LayerShellHandler for Engine {
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &wlr_layer::LayerSurface,
        configure: wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (width, height) = configure.new_size;
        info!("Layer configure event: new_size=({}, {})", width, height);

        for scene in &mut self.scenes {
            scene.on_layer_configure(layer, width, height);
        }

        for scene in &mut self.scenes {
            scene.render(&self.interaction_state);
        }
    }

    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &wlr_layer::LayerSurface) {
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

impl OutputHandler for Engine {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, conn: &Connection, qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        let Some(info) = self.output_state.info(&output) else {
            return;
        };
        let output_name = info.name.as_deref().unwrap_or("unknown");
        info!("New output detected: {}", output_name);

        for scene in &mut self.scenes {
            if scene.should_handle_output(output_name) {
                if let Err(e) = scene.on_output_added(
                    output.clone(),
                    &info,
                    self.gpu.clone(),
                    conn,
                    &self.compositor_state,
                    &self.layer_shell,
                    qh,
                ) {
                    warn!("Failed to add output '{}' to scene: {}", output_name, e);
                }
            }
        }
    }

    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        let Some(info) = self.output_state.info(&output) else {
            return;
        };
        let output_name = info.name.as_deref().unwrap_or("unknown");
        info!("Output updated: {}", output_name);

        for scene in &mut self.scenes {
            scene.on_output_updated(&output, &info);
        }
    }

    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        info!("Output destroyed");
        for scene in &mut self.scenes {
            scene.on_output_removed(&output);
        }
    }
}

delegate_compositor!(Engine);
delegate_layer!(Engine);
delegate_output!(Engine);
delegate_registry!(Engine);
