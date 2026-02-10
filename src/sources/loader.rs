use image::DynamicImage;
use log::*;
use rand::seq::SliceRandom;
use std::path::PathBuf;

use crate::prelude::*;

pub struct ImageLoader {
	image_dir: PathBuf,
	current_img: Option<DynamicImage>,
}

impl ImageLoader {
	pub fn new(image_dir: PathBuf) -> Result<Self> {
		if !image_dir.is_dir() {
			return Err(Error::NotADirectory(f!("{}", image_dir.display())));
		}
		Ok(Self {
			image_dir,
			current_img: None,
		})
	}

	fn get_random_img(&self) -> Result<DynamicImage> {
		let mut rng = rand::rng();
		let mut files: Vec<_> = self
			.image_dir
			.read_dir()?
			.filter_map(std::result::Result::ok)
			.map(|d| d.path())
			.filter(|p| p.is_file())
			.collect();

		if files.is_empty() {
			return Err(Error::NoImages(f!("{}", self.image_dir.display())));
		}

		files.shuffle(&mut rng);
		files
			.into_iter()
			.filter_map(|p| {
				info!("Attempting to load {}", p.display());
				image::open(&p).ok()
			})
			.next()
			.ok_or_else(|| {
				Error::NoImages(f!(
					"Unable to load any image from {}",
					self.image_dir.display()
				))
			})
	}

	pub fn load_next(&mut self) -> Result<DynamicImage> {
		let img = self.get_random_img()?;
		info!("Successfully loaded image");
		self.current_img = Some(img.clone());
		Ok(img)
	}

	pub fn get_current(&self) -> Option<&DynamicImage> {
		self.current_img.as_ref()
	}
}
