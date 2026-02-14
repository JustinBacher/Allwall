//! Main Crate Error

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Generic error: {0}")]
	Generic(String),
	#[error("Static error: {0}")]
	Static(&'static str),

	#[error(transparent)]
	IO(#[from] std::io::Error),

	#[error(transparent)]
	Image(#[from] image::ImageError),

	#[error(transparent)]
	Calloop(#[from] calloop::Error),

	#[error("Wayland error: {0}")]
	Wayland(String),

	#[error(transparent)]
	WGPU(#[from] wgpu::Error),

	#[error("No images found in directory: {0}")]
	NoImages(String),

	#[error("Not a directory: {0}")]
	NotADirectory(String),

	#[error("Surface error: {0}")]
	Surface(String),
}
