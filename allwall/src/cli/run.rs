use std::{fs, path::PathBuf};

use clap::Parser;

use super::AllwallCommand;
use crate::{
    cli::{
        error::CliError,
        ipc::protocol::{is_daemon_running, socket_path},
    },
    config::{AppConfig, load_config},
    engine::Engine,
    prelude::*,
    sources::SourceKind,
};

#[derive(Parser, Debug)]
#[command(name = "Allwall")]
pub struct Run {
    /// Path to image/video file or directory of images/videos to use as wallpaper
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Source type: media (images/videos), smoke, or grass
    #[arg(short, long)]
    pub source: SourceKind,

    /// Duration of transitions between images (in seconds)
    #[arg(short = 'd', long)]
    pub transition_duration: Option<u64>,

    /// Interval between image rotations (in seconds)
    #[arg(short, long)]
    pub transition_interval: Option<u64>,

    /// Target framerate
    #[arg(long)]
    pub fps: Option<u32>,
}

impl AllwallCommand for Run {
    async fn execute(&self) -> Result<()> {
        if is_daemon_running() {
            return Err(CliError::DaemonRunning.into());
        }

        let socket_path = socket_path();
        if socket_path.exists() {
            info!("Removing stale socket at {}", socket_path.display());
            fs::remove_file(&socket_path)?;
        }

        let config = load_config().unwrap_or_default();

        if matches!(self.source, SourceKind::Media) && self.path.is_none() && config.scenes.is_empty() {
            return Err(CliError::MediaPathRequired.into());
        }

        Engine::run(
            AppConfig::from_config(config)?.merge_cli(
                self.path.clone(),
                self.transition_duration,
                self.transition_interval,
                self.fps,
            ),
            self.source,
        )
    }
}
