use std::path::PathBuf;

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
	/// Path to image/video file or directry of images/videos to use as wallpaper
	#[arg(short, long)]
	path: PathBuf,

	/// How the images/videos are displayed
	#[arg(short, long, default_value_t = Mode::default())]
	mode: Mode,

	/// Whether to play sound when using a video as wallpaper
	#[arg(short, long, default_value_t = true)]
	play_audio: bool,

	/// Framerate. Must be greater than 5
	#[arg(value_parser = value_parser!(u32).range(5..), alias = "framerate", default_value_t = 60)]
	fps: u32,
}

impl Run {
	pub fn framerate(self) -> f32 {
		1.0 / self.fps as f32
	}
}

impl AllwallCommand for Run {
	async fn execute(&self) -> Result<()> {
		Ok(())
	}
}
