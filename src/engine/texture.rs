use std::path::Path;

use crate::prelude::Result;
use image::GenericImageView;

use crate::engine::Context;

pub struct Texture {
	texture: wgpu::Texture,
	size: wgpu::Extent3d,
	view: wgpu::TextureView,
	sampler: wgpu::Sampler,
}

impl Texture {
	pub fn open(path: &Path, ctx: &Context) -> Result<Self> {
		let img = image::open(path)?;
		Ok(Self::from_image(&img, ctx))
	}

	pub fn from_image(img: &image::DynamicImage, ctx: &Context) -> Self {
		let device = ctx.device();
		let queue = ctx.queue();
		let rgba = img.to_rgba8();
		let (width, height) = img.dimensions();

		let size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};

		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label: None,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
			view_formats: &[],
		});

		queue.write_texture(
			texture.as_image_copy(),
			&rgba,
			wgpu::TexelCopyBufferLayout {
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
			mipmap_filter: wgpu::MipmapFilterMode::Nearest,
			..Default::default()
		});

		Self {
			texture,
			size,
			view,
			sampler,
		}
	}

	// pub fn texture(&self) -> &wgpu::Texture {
	//     &self.texture
	// }

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
}
