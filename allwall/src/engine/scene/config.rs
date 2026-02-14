use std::path::PathBuf;

#[cfg(feature = "generate")]
use schemars::JsonSchema;
use serde::Deserialize;

use super::monitor::MonitorsSpec;
use crate::transitions::config::TransitionConfig;

/// Monitor layout strategy for multi-monitor setups
///
/// Determines how wallpaper content is distributed across monitors
/// in a scene. This affects rendering efficiency and visual consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema, serde_nixos::NixosType))]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
	/// Mirror content to all monitors (most efficient)
	///
	/// Renders a single texture and displays it on all monitors.
	/// Best for power efficiency and when you want identical content everywhere.
	#[serde(alias = "clone")]
	#[cfg_attr(feature = "generate", nixos(default = "true"))]
	Clone,

	/// Each monitor has its own wallpaper instance
	///
	/// Each monitor gets its own random selection from the source path.
	/// Transitions occur independently per monitor.
	Independent,

	/// Stretch single wallpaper across all monitors
	///
	/// Creates a virtual canvas spanning all monitors.
	/// Best for panoramic wallpapers that span multiple displays.
	Span,
}

impl Default for Layout {
	fn default() -> Self {
		Self::Clone
	}
}

/// How wallpaper content fits within monitor bounds
///
/// Controls the scaling and positioning behavior when the wallpaper
/// aspect ratio doesn't match the monitor's aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema, serde_nixos::NixosType))]
#[serde(rename_all = "kebab-case")]
pub enum Fit {
	/// Stretch to fill entire area
	///
	/// Ignores aspect ratio, may cause distortion.
	Stretch,

	/// Center at original size
	///
	/// No scaling, may not fill the screen.
	Center,

	/// Fill entire area, may crop edges
	///
	/// Maintains aspect ratio while filling the screen.
	/// This is the recommended option for most wallpapers.
	#[serde(alias = "zoom")]
	#[cfg_attr(feature = "generate", nixos(default = "true"))]
	Cover,

	/// Fit entirely within bounds
	///
	/// Maintains aspect ratio, may show letterboxing.
	Contain,

	/// Repeat pattern to fill screen
	///
	/// Best for small repeating patterns or textures.
	Tile,
}

impl Default for Fit {
	fn default() -> Self {
		Self::Cover
	}
}

/// Scene configuration for monitor assignment
///
/// A scene defines a wallpaper configuration that applies to one or more
/// monitors. Multiple scenes can be defined to have different wallpapers
/// on different monitor groups.
///
/// # Example
///
/// ```toml
/// [[scene]]
/// path = "wallpapers/panorama"
/// layout = "span"
/// fit = "cover"
/// monitors = ["DP-1", "DP-2"]
///
/// [[scene]]
/// path = "wallpapers/minimal"
/// layout = "clone"
/// fit = "center"
/// monitors = ["HDMI-A-1"]
/// ```
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema))]
pub struct SceneConfig {
	/// Path to wallpaper directory or file
	///
	/// For media sources, this should point to a directory containing
	/// images or videos. Supports common image formats (PNG, JPG, WebP)
	/// and video formats (MP4, WebM, GIF).
	///
	/// Relative paths are resolved from the config file location.
	pub path: Option<PathBuf>,

	/// Monitor layout strategy
	#[serde(default)]
	pub layout: Layout,

	/// How wallpaper fits within monitor bounds
	#[serde(default)]
	pub fit: Fit,

	/// Which monitors this scene applies to
	///
	/// Accepts:
	/// - `"*"` or `"any"`: All monitors
	/// - `"DP-1"`: Single monitor by name
	/// - `["DP-1", "HDMI-A-1"]`: Multiple specific monitors
	#[serde(default)]
	pub monitors: MonitorsSpec,

	/// Scene-specific transition settings
	///
	/// When specified, these override the global transition configuration
	/// for this scene only.
	#[serde(default)]
	pub transition: Option<TransitionConfig>,
}

impl Default for SceneConfig {
	fn default() -> Self {
		Self {
			path: None,
			layout: Layout::Clone,
			fit: Fit::Cover,
			monitors: MonitorsSpec::Any,
			transition: None,
		}
	}
}
