pub mod error;
mod general;
mod source;

use std::{collections::HashSet, fs, path::PathBuf};

pub use general::{GeneralConfig, GpuSelection};
#[cfg(feature = "generate")]
use schemars::JsonSchema;
use serde::Deserialize;
pub use source::{EmissionMode, SmokeConfig};

use crate::{
    engine::{Fit, Layout, MonitorsSpec, SceneConfig},
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
#[derive(Debug, Default, Deserialize)]
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

    /// Smoke source configuration
    #[serde(default)]
    pub smoke: SmokeConfig,
}

#[derive(Debug, Clone)]
pub struct MergedSceneConfig {
    pub path: Option<PathBuf>,
    pub layout: Layout,
    pub fit: Fit,
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
    pub smoke: SmokeConfig,
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
            smoke: config.smoke,
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
            MonitorsSpec::Any => {
                if !claimed.is_empty() {
                    let monitors: Vec<&String> = claimed.iter().collect();
                    return Err(Error::Generic(format!(
                        "Monitors {:?} are claimed by other scenes, but this scene uses 'any' which conflicts",
                        monitors
                    )));
                }
            },
            MonitorsSpec::Specific(handles) => {
                for handle in handles {
                    let name = handle.name();
                    if claimed.contains(name) {
                        return Err(Error::Generic(format!(
                            "Monitor '{}' is claimed by multiple scenes",
                            name
                        )));
                    }
                    claimed.insert(name.to_string());
                }
            },
        }
    }

    Ok(())
}

pub fn load_config() -> Result<Config> {
    let xdg_dirs = xdg::BaseDirectories::new();
    let config_path = xdg_dirs.place_config_file("allwall/config.toml");

    let config_path = match config_path {
        Ok(path) => {
            if !path.exists() {
                return Ok(Config::default());
            }

            path
        },
        Err(e) => {
            warn!("Failed to determine config path: {}", e);
            return Ok(Config::default());
        },
    };

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| Error::Generic(f!("Failed to read config file at {}: {}", config_path.display(), e)))?;

    let config: Config = toml::from_str(&config_content)
        .map_err(|e| Error::Generic(f!("Failed to parse config file at {}: {}", config_path.display(), e)))?;

    Ok(config)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::engine::scene::MonitorHandle;
    use crate::transitions::TransitionType;

    fn make_scene(monitors: MonitorsSpec) -> SceneConfig {
        SceneConfig {
            path: None,
            layout: Default::default(),
            fit: Default::default(),
            monitors,
            transition: None,
        }
    }

    #[test]
    fn test_validate_monitor_overlaps_ok() {
        let scenes = vec![
            make_scene(MonitorsSpec::Specific(vec![MonitorHandle::new("DP-1".to_string())])),
            make_scene(MonitorsSpec::Specific(vec![MonitorHandle::new("HDMI-1".to_string())])),
        ];
        assert!(validate_monitor_overlaps(&scenes).is_ok());
    }

    #[test]
    fn test_validate_monitor_overlap_error() {
        let scenes = vec![
            make_scene(MonitorsSpec::Specific(vec![MonitorHandle::new("DP-1".to_string())])),
            make_scene(MonitorsSpec::Specific(vec![MonitorHandle::new("DP-1".to_string())])),
        ];
        let result = validate_monitor_overlaps(&scenes);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_monitor_any_alone_ok() {
        let scenes = vec![make_scene(MonitorsSpec::Any)];
        assert!(validate_monitor_overlaps(&scenes).is_ok());
    }

    #[test]
    fn test_validate_monitor_any_with_specific_error() {
        let scenes = vec![
            make_scene(MonitorsSpec::Specific(vec![MonitorHandle::new("DP-1".to_string())])),
            make_scene(MonitorsSpec::Any),
        ];
        let result = validate_monitor_overlaps(&scenes);
        assert!(result.is_err());
    }

    #[test]
    fn test_merged_scene_config_from_scene() {
        let global = TransitionConfig::default();
        let scene = SceneConfig {
            path: Some(PathBuf::from("/test/path")),
            layout: crate::engine::Layout::Independent,
            fit: crate::engine::Fit::Cover,
            monitors: MonitorsSpec::Specific(vec![MonitorHandle::new("DP-1".to_string())]),
            transition: None,
        };

        let merged = MergedSceneConfig::from_scene(&scene, &global);

        assert_eq!(merged.path, Some(PathBuf::from("/test/path")));
        assert_eq!(merged.layout, crate::engine::Layout::Independent);
        assert_eq!(merged.fit, crate::engine::Fit::Cover);
        assert_eq!(merged.transition.r#type, TransitionType::Fade);
    }

    #[test]
    fn test_merged_scene_config_inherits_global_transition() {
        let mut global = TransitionConfig::default();
        global.duration = 5;
        global.interval = 30;
        global.r#type = TransitionType::CircleCenter;

        let scene = SceneConfig {
            path: None,
            layout: Default::default(),
            fit: Default::default(),
            monitors: MonitorsSpec::Any,
            transition: None,
        };

        let merged = MergedSceneConfig::from_scene(&scene, &global);

        assert_eq!(merged.transition.duration, 5);
        assert_eq!(merged.transition.interval, 30);
        assert_eq!(merged.transition.r#type, TransitionType::CircleCenter);
    }

    #[test]
    fn test_merged_scene_config_scene_transition_overrides() {
        let global = TransitionConfig::default();
        let scene_transition = TransitionConfig {
            r#type: TransitionType::CircleTopLeft,
            duration: 3,
            interval: 15,
            circle: Default::default(),
        };
        let scene = SceneConfig {
            path: None,
            layout: Default::default(),
            fit: Default::default(),
            monitors: MonitorsSpec::Any,
            transition: Some(scene_transition),
        };

        let merged = MergedSceneConfig::from_scene(&scene, &global);

        assert_eq!(merged.transition.duration, 3);
        assert_eq!(merged.transition.interval, 15);
        assert_eq!(merged.transition.r#type, TransitionType::CircleTopLeft);
    }

    #[test]
    fn test_app_config_from_config() {
        let config = Config {
            general: GeneralConfig::default(),
            transition: TransitionConfig::default(),
            scenes: vec![],
            smoke: SmokeConfig::default(),
        };

        let app_config = AppConfig::from_config(config).unwrap();
        assert!(app_config.scenes.is_empty());
    }

    #[test]
    fn test_app_config_merge_cli_path() {
        let config = Config::default();
        let app_config = AppConfig::from_config(config).unwrap();
        let merged = app_config.merge_cli(Some(PathBuf::from("/new/path")), None, None, None);

        assert_eq!(merged.scenes[0].path, Some(PathBuf::from("/new/path")));
    }

    #[test]
    fn test_app_config_merge_cli_fps() {
        let config = Config::default();
        let app_config = AppConfig::from_config(config).unwrap();
        let merged = app_config.merge_cli(None, None, None, Some(60));

        assert_eq!(merged.general.fps, 60);
    }

    #[test]
    fn test_app_config_merge_cli_duration() {
        let config = Config::default();
        let app_config = AppConfig::from_config(config).unwrap();
        let merged = app_config.merge_cli(None, Some(5), None, None);

        assert_eq!(merged.transition.duration, 5);
    }

    #[test]
    fn test_app_config_merge_cli_interval() {
        let config = Config::default();
        let app_config = AppConfig::from_config(config).unwrap();
        let merged = app_config.merge_cli(None, None, Some(30), None);

        assert_eq!(merged.transition.interval, 30);
    }

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.general.fps, 30);
        assert_eq!(config.transition.duration, 1);
        assert_eq!(config.transition.interval, 10);
        assert!(config.scenes.is_empty());
    }
}
