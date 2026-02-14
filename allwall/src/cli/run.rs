use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use derive_more::Display;
use serde::Deserialize;

use super::AllwallCommand;
use crate::{
	config::{load_config, AppConfig},
	prelude::*,
};

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Display, Deserialize, Default)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum SourceType {
	#[default]
	Media,
	Smoke,
}

#[derive(Parser, Debug)]
#[command(name = "Allwall")]
pub struct Run {
	/// Path to image/video file or directory of images/videos to use as wallpaper
	#[arg(long)]
	pub path: Option<PathBuf>,

	/// Source type: media (images/videos) or smoke
	#[arg(short, long, default_value = "media")]
	pub source: SourceType,

	/// Duration of transitions between images (in seconds)
	#[arg(short = 'd', long)]
	pub transition_duration: Option<u64>,

	/// Interval between image rotations (in seconds)
	#[arg(short, long)]
	pub transition_interval: Option<u64>,

	/// Target framerate
	#[arg(long)]
	pub fps: Option<u32>,
}

impl AllwallCommand for Run {
	async fn execute(&self) -> Result<()> {
		let config = load_config()?.unwrap_or_default();

		if matches!(self.source, SourceType::Media)
			&& self.path.is_none()
			&& config.scenes.is_empty()
		{
			return Err(Error::Generic(
				"Path is required for media source. Use --path <PATH>".to_string(),
			));
		}

		let app_config = AppConfig::from_config(config)?.merge_cli(
			self.path.clone(),
			self.transition_duration,
			self.transition_interval,
			self.fps,
		);

		crate::engine::Engine::run(app_config, self.source)
	}
}
