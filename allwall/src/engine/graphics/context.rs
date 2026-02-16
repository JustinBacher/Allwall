use std::{ffi::c_void, ptr::NonNull};

use client::{Connection, Proxy};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
use smithay_client_toolkit::{
    reexports::client,
    shell::{WaylandSurface, wlr_layer::LayerSurface},
};
use wgpu::{CompositeAlphaMode, Device, PresentMode, Queue, Surface, SurfaceConfiguration, TextureUsages};

use crate::{engine::error::ContextError, prelude::*};

pub struct Context {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
}

impl Context {
    pub async fn new(conn: &Connection, layer: &LayerSurface, size: (u32, u32)) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let raw_layer_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(layer.wl_surface().id().as_ptr() as *mut c_void).ok_or(ContextError::InvalidSurfacePointer)?,
        ));
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(conn.backend().display_ptr() as *mut c_void).ok_or(ContextError::InvalidDisplayPointer)?,
        ));

        // SAFETY: The raw display handle comes from a valid Wayland Connection that remains
        // alive for the duration of this call. The raw window handle comes from a valid
        // LayerSurface that is managed by smithay-client-toolkit. Both pointers are valid
        // and non-null, having been checked by NonNull::new with proper error handling.
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_window_handle: raw_layer_handle,
                    raw_display_handle,
                })
                .map_err(|e| ContextError::SurfaceCreate(e.to_string()))?
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .ok_or(ContextError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits {
                        max_texture_dimension_2d: 16384,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .map_err(|e| ContextError::DeviceCreate(e.to_string()))?;

        let surface_caps = surface.get_capabilities(&adapter);
        debug!("Supported present modes: {:?}", surface_caps.present_modes);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .ok_or(ContextError::NoSrgbFormat)?;

        debug!("Surface format: {:?}", surface_format);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self {
            device,
            queue,
            surface,
            config,
        })
    }

    pub fn resize(&mut self, dimensions: (u32, u32)) {
        let (width, height) = dimensions;
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn surface_aspect_ratio(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }
}
