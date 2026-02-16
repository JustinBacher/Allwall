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
}
