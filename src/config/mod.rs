use std::path::PathBuf;

use serde::Deserialize;

use crate::cli::Run;
use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub struct Config {
	pub path: Option<PathBuf>,

	#[serde(default)]
	pub general: General,

	#[serde(default)]
	pub audio: Audio,

	#[serde(default)]
	pub rotation: Rotation,

	#[serde(default)]
	pub transition: Transition,
}

#[derive(Debug, Deserialize, Default)]
pub struct General {
	#[serde(default = "default_mode")]
	pub mode: String,
	#[serde(default = "default_fps")]
	pub fps: u32,
}

#[derive(Debug, Deserialize, Default)]
pub struct Audio {
	#[serde(default = "default_play_audio")]
	pub play_audio: bool,
	#[serde(default = "default_volume")]
	pub volume: u8,
}

#[derive(Debug, Deserialize, Default)]
pub struct Rotation {
	#[serde(default = "default_rotation_interval")]
	pub interval: u64,
}

#[derive(Debug, Deserialize, Default)]
pub struct Transition {
	#[serde(default = "default_transition_type")]
	pub r#type: String,
	#[serde(default = "default_transition_duration")]
	pub duration: u64,

	#[serde(default)]
	pub circle: CircleConfig,

	#[serde(default)]
	pub fade: FadeConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct CircleConfig {
	#[serde(default)]
	pub feather: Option<f32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct FadeConfig;

#[derive(Debug)]
pub struct FinalConfig {
	pub path: Option<PathBuf>,
	pub mode: String,
	pub play_audio: bool,
	pub volume: u8,
	pub fps: u32,
	pub transition_duration: u64,
	pub rotation_interval: u64,
	pub transition_type: String,
	pub feather: Option<f32>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			path: None,
			general: General::default(),
			audio: Audio::default(),
			rotation: Rotation::default(),
			transition: Transition::default(),
		}
	}
}

fn default_mode() -> String {
	"stretch".to_string()
}

fn default_fps() -> u32 {
	60
}

fn default_play_audio() -> bool {
	true
}

fn default_volume() -> u8 {
	100
}

fn default_rotation_interval() -> u64 {
	10
}

fn default_transition_type() -> String {
	"fade".to_string()
}

fn default_transition_duration() -> u64 {
	1
}

pub fn load_config() -> Result<Option<Config>> {
	let xdg_dirs = xdg::BaseDirectories::new();
	let config_path = xdg_dirs.place_config_file("allwall/config.toml");

	let config_path = match config_path {
		Ok(path) => path,
		Err(e) => {
			log::warn!("Failed to determine config path: {}", e);
			return Ok(None);
		}
	};

	if !config_path.exists() {
		return Ok(None);
	}

	let config_content = std::fs::read_to_string(&config_path).map_err(|e| {
		Error::Generic(f!(
			"Failed to read config file at {}: {}",
			config_path.display(),
			e
		))
	})?;

	let config: Config = toml::from_str(&config_content).map_err(|e| {
		Error::Generic(f!(
			"Failed to parse config file at {}: {}",
			config_path.display(),
			e
		))
	})?;

	Ok(Some(config))
}

pub fn merge_config(config: Option<Config>, cli: &Run) -> Result<FinalConfig> {
	let config = config.unwrap_or_default();

	let path = cli.path.clone().or(config.path);

	let mode = cli.mode.as_kebab_case_str().to_string();

	let play_audio = cli.play_audio;
	let volume = config.audio.volume;

	let fps = cli.fps;

	let transition_duration = cli.transition_duration;
	let rotation_interval = cli.rotation_interval;

	let transition_type = cli.transition_type.as_kebab_case_str().to_string();

	let feather = config.transition.circle.feather;

	Ok(FinalConfig {
		path,
		mode,
		play_audio,
		volume,
		fps,
		transition_duration,
		rotation_interval,
		transition_type,
		feather,
	})
}
