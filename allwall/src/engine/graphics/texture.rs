use std::{path::Path, sync::Arc};

use image::GenericImageView;

use super::Context;
use crate::prelude::{Result, info};

#[derive(Clone)]
pub struct Texture {
    texture: Arc<wgpu::Texture>,
    size: wgpu::Extent3d,
    view: Arc<wgpu::TextureView>,
    sampler: Arc<wgpu::Sampler>,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("size", &self.size)
            .field("aspect_ratio", &self.aspect_ratio())
            .finish_non_exhaustive()
    }
}

impl Texture {
    pub fn open(path: &Path, ctx: &Context) -> Result<Self> {
        let img = image::open(path)?;
        Ok(Self::from_image(&img, ctx))
    }

    pub fn from_image(img: &image::DynamicImage, ctx: &Context) -> Self {
        info!("Texture::from_image called");
        let device = ctx.device();
        let queue = ctx.queue();
        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        info!("Creating texture: {}x{}, RGBA format", width, height);

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("wallpaper_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        info!("Texture created successfully");

        queue.write_texture(
            texture.as_image_copy(),
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture: Arc::new(texture),
            size,
            view: Arc::new(view),
            sampler: Arc::new(sampler),
        }
    }

    pub fn empty(ctx: &Context, width: u32, height: u32) -> Self {
        Self::empty_format(ctx, width, height, wgpu::TextureFormat::Rgba8UnormSrgb)
    }

    pub fn empty_writable(ctx: &Context, width: u32, height: u32) -> Self {
        Self::empty_writable_format(ctx, width, height, wgpu::TextureFormat::Rgba8UnormSrgb)
    }

    pub fn empty_format(ctx: &Context, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        let device = ctx.device();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("empty_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture: Arc::new(texture),
            size,
            view: Arc::new(view),
            sampler: Arc::new(sampler),
        }
    }

    pub fn empty_writable_format(ctx: &Context, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        let device = ctx.device();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("writable_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture: Arc::new(texture),
            size,
            view: Arc::new(view),
            sampler: Arc::new(sampler),
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn aspect_ratio(&self) -> f32 {
        let width = self.size.width as f32;
        let height = self.size.height as f32;
        width / height
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.size
    }

    pub fn from_existing(texture: wgpu::Texture, view: wgpu::TextureView, sampler: wgpu::Sampler) -> Self {
        let size = texture.size();
        Self {
            texture: Arc::new(texture),
            size,
            view: Arc::new(view),
            sampler: Arc::new(sampler),
        }
    }

    /// Update texture with raw NV12 pixel data
    ///
    /// NV12 format: Y plane at full resolution, UV plane at half resolution (4:2:0 chroma subsampling)
    /// This method should be called on textures created with R8Unorm (Y) or Rg8Unorm (UV) formats.
    ///
    /// # Arguments
    /// - `queue`: WGPU queue for submitting writes
    /// - `data`: Raw pixel data for this plane
    /// - `width`: Plane width in pixels
    /// - `height`: Plane height in pixels
    /// - `stride`: Row stride in bytes (may be larger than width * bytes_per_pixel for alignment)
    pub fn update_from_raw_pixels(
        &self,
        queue: &wgpu::Queue,
        data: &[u8],
        width: u32,
        height: u32,
        stride: u32,
    ) -> Result<()> {
        if width == 0 || height == 0 {
            return Err(crate::prelude::Error::Generic("Invalid texture dimensions".to_string()));
        }

        // Determine bytes per pixel based on texture format
        let _bytes_per_pixel = match self.texture.format() {
            wgpu::TextureFormat::R8Unorm => 1,
            wgpu::TextureFormat::Rg8Unorm => 2,
            fmt => {
                return Err(crate::prelude::Error::Generic(format!(
                    "Unsupported texture format for NV12 update: {:?}",
                    fmt
                )));
            },
        };

        // Validate that we have enough data
        let expected_size = (stride as usize) * (height as usize);
        if data.len() < expected_size {
            return Err(crate::prelude::Error::Generic(format!(
                "Insufficient data: expected {} bytes, got {}",
                expected_size,
                data.len()
            )));
        }

        // Write texture data with proper stride handling
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Ok(())
    }
}
