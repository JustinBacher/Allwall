use std::os::fd::RawFd;

use crate::prelude::*;

/// Represents a DMA-BUF plane with its file descriptor, offset, and stride
#[derive(Debug, Clone)]
pub struct DmabufPlane {
    pub fd: RawFd,
    pub offset: u32,
    pub stride: u32,
}

/// Represents a video frame in DMA-BUF format
#[derive(Debug, Clone)]
pub struct DmabufFrame {
    pub width: u32,
    pub height: u32,
    pub planes: Vec<DmabufPlane>,
    pub format: DmabufFormat,
}

/// Supported DMA-BUF formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmabufFormat {
    /// NV12: Y plane followed by interleaved UV plane
    Nv12,
    /// NV21: Y plane followed by interleaved VU plane
    Nv21,
}

impl DmabufFrame {
    /// Create a new DMA-BUF frame
    pub fn new(width: u32, height: u32, planes: Vec<DmabufPlane>, format: DmabufFormat) -> Self {
        Self {
            width,
            height,
            planes,
            format,
        }
    }

    /// Get the Y plane (first plane for NV12/NV21)
    pub fn y_plane(&self) -> Option<&DmabufPlane> {
        self.planes.get(0)
    }

    /// Get the UV plane (second plane for NV12/NV21)
    pub fn uv_plane(&self) -> Option<&DmabufPlane> {
        self.planes.get(1)
    }

    /// Check if this frame has valid NV12/NV21 plane configuration
    pub fn is_valid_nv(&self) -> bool {
        self.planes.len() == 2 && matches!(self.format, DmabufFormat::Nv12 | DmabufFormat::Nv21)
    }
}

/// Import a DMA-BUF frame into WGPU textures
///
/// This is a placeholder that creates empty textures. Full implementation requires:
/// 1. Using wgpu-hal to access the raw Vulkan device
/// 2. Creating VkDeviceMemory from the DMA-BUF FDs using Vulkan external memory extensions
/// 3. Creating VkImages bound to that memory
/// 4. Converting back to wgpu Textures via hal device interface
///
/// The vulkan_ext module contains the Vulkan-level code needed for this integration.
///
/// # Errors
///
/// Returns an error if the frame format is invalid.
pub fn import_dmabuf_frame(device: &wgpu::Device, frame: &DmabufFrame) -> Result<(wgpu::Texture, wgpu::Texture)> {
    if !frame.is_valid_nv() {
        return Err(Error::Generic(
            "DMA-BUF frame must have exactly 2 planes (Y and UV) for NV12/NV21 format".to_string(),
        ));
    }

    debug!(
        "DMA-BUF import requested for {}x{} frame (Tier 1 CUDA→DMA-BUF integration in progress)",
        frame.width, frame.height
    );

    // Create temporary textures as placeholders for now
    // TODO: Replace with actual Vulkan external memory import when HAL integration is complete
    let y_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("dmabuf_y_plane_placeholder"),
        size: wgpu::Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let uv_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("dmabuf_uv_plane_placeholder"),
        size: wgpu::Extent3d {
            width: frame.width / 2,
            height: frame.height / 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rg8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    Ok((y_tex, uv_tex))
}
