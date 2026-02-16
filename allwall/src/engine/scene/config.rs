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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[cfg_attr(feature = "generate", derive(JsonSchema, serde_nixos::NixosType))]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    /// Mirror content to all monitors (most efficient)
    ///
    /// Renders a single texture and displays it on all monitors.
    /// Best for power efficiency and when you want identical content everywhere.
    #[serde(alias = "clone")]
    #[cfg_attr(feature = "generate", nixos(default = "true"))]
    #[default]
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

/// How wallpaper content fits within monitor bounds
///
/// Controls the scaling and positioning behavior when the wallpaper
/// aspect ratio doesn't match the monitor's aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[cfg_attr(feature = "generate", derive(JsonSchema, serde_nixos::NixosType))]
#[serde(rename_all = "kebab-case")]
pub enum Fit {
    /// Stretch to fill entire area
    ///
    /// Ignores aspect ratio, may cause distortion.
    #[cfg_attr(feature = "generate", nixos(default = "true"))]
    #[default]
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct LayoutConfig {
        layout: Layout,
    }

    #[test]
    fn test_layout_deserialize_clone() {
        let config: LayoutConfig = toml::from_str(r#"layout = "clone""#).unwrap();
        assert_eq!(config.layout, Layout::Clone);
    }

    #[test]
    fn test_layout_deserialize_independent() {
        let config: LayoutConfig = toml::from_str(r#"layout = "independent""#).unwrap();
        assert_eq!(config.layout, Layout::Independent);
    }

    #[test]
    fn test_layout_deserialize_span() {
        let config: LayoutConfig = toml::from_str(r#"layout = "span""#).unwrap();
        assert_eq!(config.layout, Layout::Span);
    }

    #[test]
    fn test_layout_default() {
        assert_eq!(Layout::default(), Layout::Clone);
    }

    #[derive(serde::Deserialize)]
    struct FitConfig {
        fit: Fit,
    }

    #[test]
    fn test_fit_deserialize_stretch() {
        let config: FitConfig = toml::from_str(r#"fit = "stretch""#).unwrap();
        assert_eq!(config.fit, Fit::Stretch);
    }

    #[test]
    fn test_fit_deserialize_center() {
        let config: FitConfig = toml::from_str(r#"fit = "center""#).unwrap();
        assert_eq!(config.fit, Fit::Center);
    }

    #[test]
    fn test_fit_deserialize_cover() {
        let config: FitConfig = toml::from_str(r#"fit = "cover""#).unwrap();
        assert_eq!(config.fit, Fit::Cover);
    }

    #[test]
    fn test_fit_alias_zoom() {
        let config: FitConfig = toml::from_str(r#"fit = "zoom""#).unwrap();
        assert_eq!(config.fit, Fit::Cover);
    }

    #[test]
    fn test_fit_deserialize_contain() {
        let config: FitConfig = toml::from_str(r#"fit = "contain""#).unwrap();
        assert_eq!(config.fit, Fit::Contain);
    }

    #[test]
    fn test_fit_deserialize_tile() {
        let config: FitConfig = toml::from_str(r#"fit = "tile""#).unwrap();
        assert_eq!(config.fit, Fit::Tile);
    }

    #[test]
    fn test_fit_default() {
        assert_eq!(Fit::default(), Fit::Stretch);
    }

    #[test]
    fn test_scene_config_defaults() {
        let config = SceneConfig::default();
        assert!(config.path.is_none());
        assert_eq!(config.layout, Layout::Clone);
        assert_eq!(config.fit, Fit::Cover);
        assert_eq!(config.monitors, MonitorsSpec::Any);
        assert!(config.transition.is_none());
    }

    #[derive(serde::Deserialize)]
    struct SceneConfigWrapper {
        scene: SceneConfig,
    }

    #[test]
    fn test_scene_config_deserialize_full() {
        let config: SceneConfigWrapper = toml::from_str(
            r#"
            [scene]
            path = "/test/path"
            layout = "independent"
            fit = "cover"
            monitors = "DP-1"
            "#,
        )
        .unwrap();

        assert_eq!(config.scene.path, Some(PathBuf::from("/test/path")));
        assert_eq!(config.scene.layout, Layout::Independent);
        assert_eq!(config.scene.fit, Fit::Cover);
    }
}
