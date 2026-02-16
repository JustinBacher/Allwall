use std::time::Duration;

#[cfg(feature = "generate")]
use schemars::JsonSchema;
use serde::Deserialize;

use super::{CircleOptions, TransitionType};

fn default_duration() -> u64 {
    1
}

fn default_interval() -> u64 {
    10
}

fn default_transition_type() -> TransitionType {
    TransitionType::Fade
}

/// Transition configuration for wallpaper changes
///
/// Controls how wallpapers transition between each other, including
/// the animation type, timing, and type-specific options.
///
/// # Example
///
/// ```toml
/// [transition]
/// type = "circle-center"
/// duration = 2
/// interval = 30
///
/// [transition.circle]
/// feather = 0.1
/// origin = "center"
/// ```
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema, serde_nixos::NixosType))]
pub struct TransitionConfig {
    /// Transition animation type
    ///
    /// Determines the visual effect used when switching between wallpapers.
    #[serde(default = "default_transition_type")]
    #[cfg_attr(feature = "generate", schemars(default = "default_transition_type"))]
    #[cfg_attr(feature = "generate", nixos(default = "\"fade\""))]
    pub r#type: TransitionType,

    /// Duration of the transition animation in seconds
    ///
    /// How long the transition effect takes to complete.
    /// Recommended: 0.5-2 seconds for smooth results.
    #[serde(default = "default_duration")]
    #[cfg_attr(feature = "generate", schemars(default = "default_duration"))]
    #[cfg_attr(feature = "generate", nixos(default = "1"))]
    pub duration: u64,

    /// Time between automatic wallpaper rotations in seconds
    ///
    /// How long to display each wallpaper before transitioning to the next.
    /// Set to 0 to disable automatic rotation.
    #[serde(default = "default_interval")]
    #[cfg_attr(feature = "generate", schemars(default = "default_interval"))]
    #[cfg_attr(feature = "generate", nixos(default = "10"))]
    pub interval: u64,

    /// Options specific to circle reveal transitions
    ///
    /// Only used when `type` is set to a circle variant.
    #[serde(default)]
    pub circle: CircleOptions,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            r#type: default_transition_type(),
            duration: default_duration(),
            interval: default_interval(),
            circle: CircleOptions::default(),
        }
    }
}

impl TransitionConfig {
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.duration)
    }

    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval)
    }

    pub fn merge(&self, other: Option<&Self>) -> Self {
        match other {
            Some(other) => Self {
                r#type: other.r#type,
                duration: if other.duration != default_duration() {
                    other.duration
                } else {
                    self.duration
                },
                interval: if other.interval != default_interval() {
                    other.interval
                } else {
                    self.interval
                },
                circle: self.circle.merge(&other.circle),
            },
            None => self.clone(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_config_defaults() {
        let config = TransitionConfig::default();
        assert_eq!(config.r#type, TransitionType::Fade);
        assert_eq!(config.duration, 1);
        assert_eq!(config.interval, 10);
    }

    #[test]
    fn test_transition_config_duration_method() {
        let config = TransitionConfig {
            r#type: TransitionType::Fade,
            duration: 5,
            interval: 10,
            circle: CircleOptions::default(),
        };
        assert_eq!(config.duration(), Duration::from_secs(5));
    }

    #[test]
    fn test_transition_config_interval_method() {
        let config = TransitionConfig {
            r#type: TransitionType::Fade,
            duration: 1,
            interval: 30,
            circle: CircleOptions::default(),
        };
        assert_eq!(config.interval(), Duration::from_secs(30));
    }

    #[test]
    fn test_transition_config_merge_none() {
        let config = TransitionConfig::default();
        let merged = config.merge(None);
        assert_eq!(merged.r#type, config.r#type);
        assert_eq!(merged.duration, config.duration);
        assert_eq!(merged.interval, config.interval);
    }

    #[test]
    fn test_transition_config_merge_partial() {
        let global = TransitionConfig {
            r#type: TransitionType::Fade,
            duration: 5,
            interval: 30,
            circle: CircleOptions::default(),
        };
        let scene = TransitionConfig {
            r#type: TransitionType::CircleCenter,
            duration: 1,
            interval: 10,
            circle: CircleOptions::default(),
        };

        let merged = global.merge(Some(&scene));

        assert_eq!(merged.r#type, TransitionType::CircleCenter);
        assert_eq!(merged.duration, 5);
        assert_eq!(merged.interval, 30);
    }

    #[test]
    fn test_transition_config_merge_full() {
        let global = TransitionConfig {
            r#type: TransitionType::Fade,
            duration: 1,
            interval: 10,
            circle: CircleOptions::default(),
        };
        let scene = TransitionConfig {
            r#type: TransitionType::CircleTopLeft,
            duration: 3,
            interval: 20,
            circle: CircleOptions::default(),
        };

        let merged = global.merge(Some(&scene));

        assert_eq!(merged.r#type, TransitionType::CircleTopLeft);
        assert_eq!(merged.duration, 3);
        assert_eq!(merged.interval, 20);
    }

    #[test]
    fn test_transition_config_deserialize() {
        let config: TransitionConfig = toml::from_str(
            r#"
            type = "circle-center"
            duration = 2
            interval = 30
            "#,
        )
        .unwrap();

        assert_eq!(config.r#type, TransitionType::CircleCenter);
        assert_eq!(config.duration, 2);
        assert_eq!(config.interval, 30);
    }

    #[test]
    fn test_transition_config_deserialize_defaults() {
        let config: TransitionConfig = toml::from_str("").unwrap();
        assert_eq!(config.r#type, TransitionType::Fade);
        assert_eq!(config.duration, 1);
        assert_eq!(config.interval, 10);
    }
}
