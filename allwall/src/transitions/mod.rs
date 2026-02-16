use std::time::Duration;

use clap::ValueEnum;
use derive_more::Display;
#[cfg(feature = "generate")]
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    engine::{Context, Texture},
    prelude::*,
};

pub mod circle_reveal;
pub mod config;
pub mod error;
pub mod fade;

pub use circle_reveal::{CircleOptions, CircleOrigin, CircleRevealTransition};
pub use config::TransitionConfig;
pub use fade::FadeTransition;

/// Transition animation type
///
/// Defines the visual effect used when transitioning between wallpapers.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Display, Deserialize)]
#[cfg_attr(feature = "generate", derive(JsonSchema, serde_nixos::NixosType))]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum TransitionType {
    /// Simple crossfade between wallpapers
    Fade,

    /// Circle reveal from top-left corner
    CircleTopLeft,

    /// Circle reveal from top-right corner
    CircleTopRight,

    /// Circle reveal from bottom-left corner
    CircleBottomLeft,

    /// Circle reveal from bottom-right corner
    CircleBottomRight,

    /// Circle reveal from center
    CircleCenter,

    /// Circle reveal from random position
    CircleRandom,
}

impl TransitionType {
    pub fn as_kebab_case_str(&self) -> &'static str {
        match self {
            TransitionType::Fade => "fade",
            TransitionType::CircleTopLeft => "circle-top-left",
            TransitionType::CircleTopRight => "circle-top-right",
            TransitionType::CircleBottomLeft => "circle-bottom-left",
            TransitionType::CircleBottomRight => "circle-bottom-right",
            TransitionType::CircleCenter => "circle-center",
            TransitionType::CircleRandom => "circle-random",
        }
    }
}

pub trait Transition: std::fmt::Debug {
    fn update(&mut self, dt: Duration) -> bool;
    fn progress(&self) -> f32;
    fn render(&self, ctx: &Context, current_texture: &Texture);
    fn previous_texture(&self) -> Option<&Texture>;
}

impl TryFrom<&str> for TransitionType {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value.to_lowercase().as_str() {
            "fade" => Ok(TransitionType::Fade),
            "circle-top-left" => Ok(TransitionType::CircleTopLeft),
            "circle-top-right" => Ok(TransitionType::CircleTopRight),
            "circle-bottom-left" => Ok(TransitionType::CircleBottomLeft),
            "circle-bottom-right" => Ok(TransitionType::CircleBottomRight),
            "circle-center" => Ok(TransitionType::CircleCenter),
            "circle-random" => Ok(TransitionType::CircleRandom),
            _ => Err(Error::Generic(f!("Invalid transition type: {}", value))),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_type_try_from_fade() {
        let t: TransitionType = "fade".try_into().unwrap();
        assert_eq!(t, TransitionType::Fade);
    }

    #[test]
    fn test_transition_type_try_from_circle_top_left() {
        let t: TransitionType = "circle-top-left".try_into().unwrap();
        assert_eq!(t, TransitionType::CircleTopLeft);
    }

    #[test]
    fn test_transition_type_try_from_circle_top_right() {
        let t: TransitionType = "circle-top-right".try_into().unwrap();
        assert_eq!(t, TransitionType::CircleTopRight);
    }

    #[test]
    fn test_transition_type_try_from_circle_bottom_left() {
        let t: TransitionType = "circle-bottom-left".try_into().unwrap();
        assert_eq!(t, TransitionType::CircleBottomLeft);
    }

    #[test]
    fn test_transition_type_try_from_circle_bottom_right() {
        let t: TransitionType = "circle-bottom-right".try_into().unwrap();
        assert_eq!(t, TransitionType::CircleBottomRight);
    }

    #[test]
    fn test_transition_type_try_from_circle_center() {
        let t: TransitionType = "circle-center".try_into().unwrap();
        assert_eq!(t, TransitionType::CircleCenter);
    }

    #[test]
    fn test_transition_type_try_from_circle_random() {
        let t: TransitionType = "circle-random".try_into().unwrap();
        assert_eq!(t, TransitionType::CircleRandom);
    }

    #[test]
    fn test_transition_type_try_from_invalid() {
        let result: std::result::Result<TransitionType, _> = "invalid-type".try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_transition_type_try_from_case_insensitive() {
        let t: TransitionType = "FADE".try_into().unwrap();
        assert_eq!(t, TransitionType::Fade);

        let t: TransitionType = "CIRCLE-CENTER".try_into().unwrap();
        assert_eq!(t, TransitionType::CircleCenter);
    }

    #[test]
    fn test_transition_type_kebab_roundtrip_fade() {
        let t = TransitionType::Fade;
        let s = t.as_kebab_case_str();
        let t2: TransitionType = s.try_into().unwrap();
        assert_eq!(t, t2);
    }

    #[test]
    fn test_transition_type_kebab_roundtrip_all_circle() {
        for t in [
            TransitionType::CircleTopLeft,
            TransitionType::CircleTopRight,
            TransitionType::CircleBottomLeft,
            TransitionType::CircleBottomRight,
            TransitionType::CircleCenter,
            TransitionType::CircleRandom,
        ] {
            let s = t.as_kebab_case_str();
            let t2: TransitionType = s.try_into().unwrap();
            assert_eq!(t, t2);
        }
    }

    #[derive(serde::Deserialize)]
    struct TransitionTypeConfig {
        #[serde(rename = "type")]
        transition_type: TransitionType,
    }

    #[test]
    fn test_transition_type_deserialize() {
        let config: TransitionTypeConfig = toml::from_str(r#"type = "circle-center""#).unwrap();
        assert_eq!(config.transition_type, TransitionType::CircleCenter);
    }
}
