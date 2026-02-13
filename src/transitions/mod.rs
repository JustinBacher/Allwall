use std::time::Duration;

use clap::ValueEnum;
use derive_more::Display;

use crate::engine::Context;
use crate::prelude::*;
use crate::sources::Source;

pub mod circle_reveal;
pub mod fade;

pub use circle_reveal::{CircleOrigin, CircleRevealTransition};
pub use fade::FadeTransition;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Display)]
#[clap(rename_all = "kebab-case")]
pub enum TransitionType {
	Fade,
	CircleTopLeft,
	CircleTopRight,
	CircleBottomLeft,
	CircleBottomRight,
	CircleCenter,
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

pub trait TransitionTypeConfig: Send + Sync {}
