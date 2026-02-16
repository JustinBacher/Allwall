use std::{ffi::c_void, ptr::NonNull};

use client::{Connection, Proxy};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
use smithay_client_toolkit::{
    reexports::client,
    shell::{WaylandSurface, wlr_layer::LayerSurface},
};
use wgpu::{CompositeAlphaMode, Device, PresentMode, Queue, Surface, SurfaceConfiguration, TextureUsages};

use crate::{engine::error::ContextError, prelude::*};

pub struct GpuContext {
    device: Device,
    queue: Queue,
    adapter: wgpu::Adapter,
    instance: wgpu::Instance,
    surface_format: wgpu::TextureFormat,
}

impl GpuContext {
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: None,
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

        let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        Ok(Self {
            device,
            queue,
            adapter,
            instance,
            surface_format,
        })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }
}

pub struct RenderSurface {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
}

impl RenderSurface {
    pub fn new(gpu: &GpuContext, conn: &Connection, layer: &LayerSurface, size: (u32, u32)) -> Result<Self> {
        let raw_layer_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(layer.wl_surface().id().as_ptr() as *mut c_void).ok_or(ContextError::InvalidSurfacePointer)?,
        ));
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(conn.backend().display_ptr() as *mut c_void).ok_or(ContextError::InvalidDisplayPointer)?,
        ));

        let surface = unsafe {
            gpu.instance()
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_window_handle: raw_layer_handle,
                    raw_display_handle,
                })
                .map_err(|e| ContextError::SurfaceCreate(e.to_string()))?
        };

        let surface_caps = surface.get_capabilities(gpu.adapter());
        debug!("Supported present modes: {:?}", surface_caps.present_modes);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(gpu.surface_format());

        debug!("Surface format: {:?}", surface_format);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0.max(1),
            height: size.1.max(1),
            present_mode: PresentMode::AutoVsync,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(gpu.device(), &config);

        Ok(Self { surface, config })
    }

    pub fn resize(&mut self, device: &Device, dimensions: (u32, u32)) {
        let (width, height) = dimensions;
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(device, &self.config);
    }

    pub fn surface(&self) -> &Surface<'static> {
        &self.surface
    }

    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.config.height == 0 {
            1.0
        } else {
            self.config.width as f32 / self.config.height as f32
        }
    }
}

pub struct Context {
    gpu: std::sync::Arc<GpuContext>,
    surface: RenderSurface,
}

impl RenderSurface {
    pub fn config_ref(&self) -> &SurfaceConfiguration {
        &self.config
    }

    pub fn surface_ref(&self) -> &Surface<'static> {
        &self.surface
    }
}

pub struct ContextRef<'a> {
    gpu: &'a GpuContext,
    surface: &'a RenderSurface,
}

impl<'a> ContextRef<'a> {
    pub fn new(gpu: &'a GpuContext, surface: &'a RenderSurface) -> Self {
        Self { gpu, surface }
    }

    pub fn device(&self) -> &'a Device {
        &self.gpu.device
    }

    pub fn queue(&self) -> &'a Queue {
        &self.gpu.queue
    }

    pub fn surface(&self) -> &'a Surface<'static> {
        self.surface.surface()
    }

    pub fn config(&self) -> &'a SurfaceConfiguration {
        self.surface.config()
    }

    pub fn gpu(&self) -> &'a GpuContext {
        self.gpu
    }

    pub fn render_surface(&self) -> &'a RenderSurface {
        self.surface
    }
}

impl Context {
    pub async fn new(conn: &Connection, layer: &LayerSurface, size: (u32, u32)) -> Result<Self> {
        let gpu = GpuContext::new().await?;
        let surface = RenderSurface::new(&gpu, conn, layer, size)?;
        Ok(Self {
            gpu: std::sync::Arc::new(gpu),
            surface,
        })
    }

    pub fn from_parts(gpu: std::sync::Arc<GpuContext>, surface: RenderSurface) -> Self {
        Self { gpu, surface }
    }

    pub fn gpu_arc(&self) -> std::sync::Arc<GpuContext> {
        self.gpu.clone()
    }

    pub fn as_ref(&self) -> ContextRef<'_> {
        ContextRef::new(&self.gpu, &self.surface)
    }

    pub fn resize(&mut self, dimensions: (u32, u32)) {
        self.surface.resize(&self.gpu.device, dimensions);
    }

    pub fn surface_aspect_ratio(&self) -> f32 {
        self.surface.aspect_ratio()
    }

    pub fn device(&self) -> &Device {
        &self.gpu.device
    }

    pub fn queue(&self) -> &Queue {
        &self.gpu.queue
    }

    pub fn surface(&self) -> &Surface<'static> {
        self.surface.surface()
    }

    pub fn config(&self) -> &SurfaceConfiguration {
        self.surface.config()
    }

    pub fn gpu(&self) -> &GpuContext {
        &self.gpu
    }

    pub fn render_surface(&self) -> &RenderSurface {
        &self.surface
    }

    pub fn render_surface_mut(&mut self) -> &mut RenderSurface {
        &mut self.surface
    }
}
