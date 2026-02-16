use crate::prelude::*;

pub trait AllwallCommand {
    async fn execute(&self) -> Result<()>;
}

pub mod completions;
pub mod error;
pub mod ipc;
mod run;
mod version;

pub use completions::Completions;
pub use ipc::{Fps, Next, Prev};
pub use run::Run;
pub use version::Version;

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    /// Run the wallpaper daemon
    Run(Run),

    /// Show version information
    Version(Version),

    /// Generate shell completions
    Completions(Completions),

    /// Skip to next image/video
    Next(Next),

    /// Go to previous image/video
    Prev(Prev),

    /// Set the target framerate
    Fps(Fps),
}
