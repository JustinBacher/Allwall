pub use crate::{
    cli::error::CliError,
    config::error::ConfigError,
    engine::error::{ContextError, EngineError},
    sources::{error::SourceError, media::video::error::VideoError},
    transitions::error::TransitionError,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Video(#[from] VideoError),

    #[error(transparent)]
    Engine(#[from] EngineError),

    #[error(transparent)]
    Context(#[from] ContextError),

    #[error(transparent)]
    Source(#[from] SourceError),

    #[error(transparent)]
    Transition(#[from] TransitionError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Cli(#[from] CliError),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    Calloop(#[from] calloop::Error),

    #[error(transparent)]
    Bincode(#[from] bincode::Error),

    #[error("Daemon is not running")]
    DaemonNotRunning,

    #[error("No images found: {0}")]
    NoImages(String),

    #[error("No images found in directory: {0}")]
    NotADirectory(String),

    #[error("Surface error: {0}")]
    Surface(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("{0}")]
    Static(&'static str),
}
