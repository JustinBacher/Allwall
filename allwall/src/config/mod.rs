mod general;

use std::{collections::HashSet, path::PathBuf};

pub use general::{GeneralConfig, GpuSelection};
#[cfg(feature = "generate")]
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
	engine::{MonitorsSpec, SceneConfig},
	prelude::*,
	transitions::config::TransitionConfig,
};

/// Root configuration for allwall
///
/// This is the main configuration structure that defines all settings
/// for the wallpaper daemon. Configuration is loaded from
/// `$XDG_CONFIG_HOME/allwall/config.toml`.
///
/// # Example
///
/// ```toml
/// [general]
/// fps = 30
/// gpu = "auto"
///
/// [transition]
/// type = "fade"
/// duration = 1
/// interval = 10
///
/// [[scene]]
/// path = "wallpapers/nature"
/// layout = "clone"
/// fit = "cover"
/// monitors = "*"
/// ```
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema))]
pub struct Config {
	/// General engine settings
	#[serde(default)]
	pub general: GeneralConfig,

	/// Global transition configuration
	#[serde(default)]
	pub transition: TransitionConfig,

	/// Scene configurations for monitor assignments
	///
	/// Each scene defines a wallpaper configuration for one or more monitors.
	/// If no scenes are defined, a default scene using all monitors is created.
	#[serde(default, rename = "scene")]
	pub scenes: Vec<SceneConfig>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			general: GeneralConfig::default(),
			transition: TransitionConfig::default(),
			scenes: Vec::new(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct MergedSceneConfig {
	pub path: Option<PathBuf>,
	pub layout: crate::engine::Layout,
	pub fit: crate::engine::Fit,
	pub monitors: MonitorsSpec,
	pub transition: TransitionConfig,
}

impl MergedSceneConfig {
	pub fn from_scene(scene: &SceneConfig, global_transition: &TransitionConfig) -> Self {
		Self {
			path: scene.path.clone(),
			layout: scene.layout,
			fit: scene.fit,
			monitors: scene.monitors.clone(),
			transition: global_transition.merge(scene.transition.as_ref()),
		}
	}
}

#[derive(Debug)]
pub struct AppConfig {
	pub general: GeneralConfig,
	pub transition: TransitionConfig,
	pub scenes: Vec<MergedSceneConfig>,
}

impl AppConfig {
	pub fn from_config(config: Config) -> Result<Self> {
		validate_monitor_overlaps(&config.scenes)?;

		let scenes: Vec<MergedSceneConfig> = config
			.scenes
			.iter()
			.map(|s| MergedSceneConfig::from_scene(s, &config.transition))
			.collect();

		Ok(Self {
			general: config.general,
			transition: config.transition,
			scenes,
		})
	}

	pub fn merge_cli(
		mut self,
		path: Option<PathBuf>,
		transition_duration: Option<u64>,
		transition_interval: Option<u64>,
		fps: Option<u32>,
	) -> Self {
		if let Some(p) = path {
			if !self.scenes.is_empty() {
				self.scenes[0].path = Some(p);
			} else {
				self.scenes.push(MergedSceneConfig {
					path: Some(p),
					layout: Default::default(),
					fit: Default::default(),
					monitors: MonitorsSpec::Any,
					transition: self.transition.clone(),
				});
			}
		}

		if let Some(d) = transition_duration {
			self.transition.duration = d;
			for scene in &mut self.scenes {
				scene.transition.duration = d;
			}
		}

		if let Some(i) = transition_interval {
			self.transition.interval = i;
			for scene in &mut self.scenes {
				scene.transition.interval = i;
			}
		}

		if let Some(f) = fps {
			self.general.fps = f;
		}

		self
	}
}

fn validate_monitor_overlaps(scenes: &[SceneConfig]) -> Result<()> {
	let mut claimed: HashSet<String> = HashSet::new();

	for scene in scenes {
		match &scene.monitors {
			MonitorsSpec::Any =>
				if !claimed.is_empty() {
					let monitors: Vec<&String> = claimed.iter().collect();
					return Err(Error::Generic(format!(
						"Monitors {:?} are claimed by other scenes, but this scene uses 'any' which conflicts",
						monitors
					)));
				},
			MonitorsSpec::Specific(handles) =>
				for handle in handles {
					let name = handle.name();
					if claimed.contains(name) {
						return Err(Error::Generic(format!(
							"Monitor '{}' is claimed by multiple scenes",
							name
						)));
					}
					claimed.insert(name.to_string());
				},
		}
	}

	Ok(())
}

pub fn load_config() -> Result<Option<Config>> {
	let xdg_dirs = xdg::BaseDirectories::new();
	let config_path = xdg_dirs.place_config_file("allwall/config.toml");

	let config_path = match config_path {
		Ok(path) => path,
		Err(e) => {
			log::warn!("Failed to determine config path: {}", e);
			return Ok(None);
		},
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
