//! Graphics context and texture utilities.
//!
//! This module contains unsafe code for interacting with raw Wayland handles
//! to create WGPU surfaces. The unsafe is required because WGPU's safe API
//! cannot directly consume raw Wayland surface pointers from smithay-client-toolkit.
//!
//! # Safety
//!
//! The unsafe code in `context.rs` creates surfaces from raw handles. This is safe
//! when:
//! - The Wayland display pointer is valid and comes from an active Connection
//! - The Wayland surface pointer is valid and comes from an active LayerSurface
//! - Both the Connection and LayerSurface outlive the surface creation call

#![allow(unsafe_code)]

mod context;
mod texture;

pub use context::Context;
pub use texture::Texture;
