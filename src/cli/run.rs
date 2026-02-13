use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use clap_config::ClapConfig;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use super::AllwallCommand;
use crate::prelude::*;
use crate::transitions::TransitionType;

use crate::engine::SourceType;

#[derive(ValueEnum, ClapConfig, Debug, Clone, Serialize, Deserialize, Default, Display)]
#[clap(rename_all = "kebab-case")]
pub enum Mode {
	#[default]
	Stretch,
	Tile,
	Center,
	Zoom,
	Smoke,
}

#[derive(Parser, Debug)]
#[command(name = "Allwall")]
pub struct Run {
	/// Path to image/video file or directory of images/videos to use as wallpaper
	#[arg(long)]
	pub path: Option<PathBuf>,

	/// How images/videos are displayed
	#[arg(short, long, default_value = "stretch")]
	pub mode: Mode,

	/// Whether to play sound when using a video as wallpaper
	#[arg(short, long, default_value_t = true)]
	pub play_audio: bool,

	/// Framerate. Must be greater than 5
	#[arg(short, long, default_value_t = 60)]
	pub fps: u32,

	/// Duration of transitions between images (in seconds)
	#[arg(short = 'd', long, default_value_t = 1)]
	pub transition_duration: u64,

	/// Interval between image rotations (in seconds)
	#[arg(short, long, default_value_t = 10)]
	pub rotation_interval: u64,

	/// Type of transition to use when switching between images
	#[arg(short = 't', long, default_value = "fade")]
	pub transition_type: TransitionType,
}

impl Run {
	pub fn framerate(self) -> f32 {
		1.0 / self.fps as f32
	}
}

impl Mode {
	pub fn as_kebab_case_str(&self) -> &'static str {
		match self {
			Mode::Stretch => "stretch",
			Mode::Tile => "tile",
			Mode::Center => "center",
			Mode::Zoom => "zoom",
			Mode::Smoke => "smoke",
		}
	}

	pub fn as_source_type(&self) -> SourceType {
		match self {
			Mode::Smoke => SourceType::Smoke,
			_ => SourceType::Image,
		}
	}
}

impl std::str::FromStr for Mode {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		match s {
			"stretch" => Ok(Mode::Stretch),
			"tile" => Ok(Mode::Tile),
			"center" => Ok(Mode::Center),
			"zoom" => Ok(Mode::Zoom),
			"smoke" => Ok(Mode::Smoke),
			_ => Err(Error::Generic(f!("Invalid mode: {}", s))),
		}
	}
}

impl AllwallCommand for Run {
	async fn execute(&self) -> Result<()> {
		let config = crate::config::load_config()?;
		let final_config = crate::config::merge_config(config, self)?;

		let path = match final_config.path {
			Some(p) => p,
			None => {
				return Err(Error::Generic(
					"No path provided in CLI or config file".to_string(),
				));
			}
		};

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

		let transition_duration = Duration::from_secs(final_config.transition_duration);
		let rotation_interval = Duration::from_secs(final_config.rotation_interval);
		let transition_type = final_config.transition_type.as_str().try_into()?;
		let feather = final_config.feather;

		let mode: Mode = final_config.mode.parse().unwrap_or(Mode::Stretch);
		let source_type = mode.as_source_type();

		let img_dir_option = if matches!(source_type, SourceType::Smoke) {
			None
		} else {
			Some(img_dir)
		};

		crate::engine::Engine::run(
			img_dir_option,
			source_type,
			transition_duration,
			rotation_interval,
			transition_type,
			feather,
		)
	}
}
