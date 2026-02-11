use std::path::Path;

pub enum MediaType {
	Image,
	Video,
}

pub struct MediaDetector;

impl MediaDetector {
	pub const IMAGE_EXTENSIONS: &[&str] = &[
		"png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "ico", "avif",
	];

	pub const VIDEO_EXTENSIONS: &[&str] =
		&["mp4", "webm", "mkv", "avi", "mov", "flv", "wmv", "m4v"];

	pub fn get_media_type(path: &Path) -> Option<MediaType> {
		let extension = path
			.extension()
			.and_then(|ext| ext.to_str())
			.map(|ext| ext.to_lowercase())?;

		if Self::IMAGE_EXTENSIONS.contains(&extension.as_str()) {
			Some(MediaType::Image)
		} else if Self::VIDEO_EXTENSIONS.contains(&extension.as_str()) {
			Some(MediaType::Video)
		} else {
			None
		}
	}

	pub fn is_image(path: &Path) -> bool {
		matches!(Self::get_media_type(path), Some(MediaType::Image))
	}

	pub fn is_video(path: &Path) -> bool {
		matches!(Self::get_media_type(path), Some(MediaType::Video))
	}
}
