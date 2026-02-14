use crate::prelude::*;

pub trait AllwallCommand {
	async fn execute(&self) -> Result<()>;
}

mod completions;
mod run;
mod version;

pub use completions::Completions;
pub use run::{Run, SourceType};
pub use version::Version;

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
	/// Run the wallpaper daemon
	Run(Run),

	/// Show version information
	Version(Version),

	/// Generate shell completions
	Completions(Completions),
}
