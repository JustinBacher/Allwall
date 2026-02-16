#[cfg(feature = "generate")]
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum EmissionMode {
    #[default]
    Continuous,
    Burst,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema))]
pub struct SmokeConfig {
    #[serde(default)]
    pub emission_mode: EmissionMode,

    #[serde(default = "default_emission_intensity")]
    pub emission_intensity: f32,

    #[serde(default = "default_background_color")]
    pub background_color: [f32; 3],

    #[serde(default = "default_smoke_color")]
    pub smoke_color: [f32; 3],
}

impl Default for SmokeConfig {
    fn default() -> Self {
        Self {
            emission_mode: EmissionMode::default(),
            emission_intensity: default_emission_intensity(),
            background_color: default_background_color(),
            smoke_color: default_smoke_color(),
        }
    }
}

fn default_emission_intensity() -> f32 {
    1.0
}

fn default_background_color() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}

fn default_smoke_color() -> [f32; 3] {
    [0.75, 0.75, 0.75]
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_emission_mode_deserialize_continuous() {
        let config: SmokeConfig = toml::from_str(r#"emission_mode = "continuous""#).unwrap();
        assert_eq!(config.emission_mode, EmissionMode::Continuous);
    }

    #[test]
    fn test_emission_mode_deserialize_burst() {
        let config: SmokeConfig = toml::from_str(r#"emission_mode = "burst""#).unwrap();
        assert_eq!(config.emission_mode, EmissionMode::Burst);
    }

    #[test]
    fn test_emission_mode_default() {
        assert_eq!(EmissionMode::default(), EmissionMode::Continuous);
    }

    #[test]
    fn test_smoke_config_defaults() {
        let config = SmokeConfig::default();
        assert_eq!(config.emission_mode, EmissionMode::Continuous);
        assert!((config.emission_intensity - 1.0).abs() < f32::EPSILON);
        assert_eq!(config.background_color, [0.0, 0.0, 0.0]);
        assert_eq!(config.smoke_color, [0.75, 0.75, 0.75]);
    }

    #[test]
    fn test_smoke_config_custom_values() {
        let config: SmokeConfig = toml::from_str(
            r#"
            emission_mode = "burst"
            emission_intensity = 2.5
            background_color = [0.1, 0.2, 0.3]
            smoke_color = [0.5, 0.6, 0.7]
            "#,
        )
        .unwrap();
        assert_eq!(config.emission_mode, EmissionMode::Burst);
        assert!((config.emission_intensity - 2.5).abs() < f32::EPSILON);
        assert_eq!(config.background_color, [0.1, 0.2, 0.3]);
        assert_eq!(config.smoke_color, [0.5, 0.6, 0.7]);
    }
}
