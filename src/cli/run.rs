use std::path::PathBuf;
use std::time::Duration;

use clap::{value_parser, Parser, ValueEnum};
use clap_config::ClapConfig;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use super::AllwallCommand;
use crate::prelude::*;

#[derive(ValueEnum, Debug, Clone, Serialize, Deserialize, Default, Display)]
#[clap(rename_all = "kebab-case")]
enum Mode {
	#[default]
	Stretch,
	Tile,
	Center,
	Zoom,
}

#[derive(ClapConfig, Parser, Debug)]
#[command(name = "Allwall")]
pub struct Run {
	/// Path to image/video file or directory of images/videos to use as wallpaper
	#[arg(long)]
	path: PathBuf,

	/// How the images/videos are displayed
	#[arg(short, long, default_value = "stretch")]
	mode: Mode,

	/// Whether to play sound when using a video as wallpaper
	#[arg(short, long, default_value_t = true)]
	play_audio: bool,

	/// Framerate. Must be greater than 5
	#[arg(short, long, default_value_t = 60)]
	fps: u32,

	/// Duration of transitions between images (in seconds)
	#[arg(short, long, default_value_t = 1)]
	transition_duration: u64,

	/// Interval between image rotations (in seconds)
	#[arg(short, long, default_value_t = 10)]
	rotation_interval: u64,
}

impl Run {
	pub fn framerate(self) -> f32 {
		1.0 / self.fps as f32
	}
}

impl AllwallCommand for Run {
	async fn execute(&self) -> Result<()> {
		let path = &self.path;

		if !path.exists() {
			return Err(Error::Generic(f!(
				"Path does not exist: {}",
				path.display()
			)));
		}

		let img_dir = if path.is_file() {
			path.parent().unwrap().to_path_buf()
		} else {
			path.clone()
		};

		let transition_duration = Duration::from_secs(self.transition_duration);
		let rotation_interval = Duration::from_secs(self.rotation_interval);

		crate::engine::Engine::run(img_dir, transition_duration, rotation_interval)
	}
}
