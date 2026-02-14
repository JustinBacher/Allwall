use std::time::Duration;

use clap::ValueEnum;
use derive_more::Display;
#[cfg(feature = "generate")]
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{engine::Context, prelude::*, sources::Source};

pub mod circle_reveal;
pub mod config;
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
	fn render(&self, ctx: &Context, current_source: &dyn Source);
	fn previous_source(&self) -> Option<&dyn Source>;
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
