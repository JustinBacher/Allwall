use crate::cli::{AllwallCommand, Run, Version};
use crate::prelude::*;
use clap::Parser;

mod cli;
mod decode;
mod engine;
mod error;
mod prelude;
mod sources;
mod utils;

#[derive(Parser, Debug)]
#[command(name = "allwall")]
#[command(about = "Allwall a zero-copy wayland wallpaper", long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
	Run(Run),
	Version(Version),
}

#[tokio::main]
async fn main() -> Result<()> {
	env_logger::init();

	let cli = Cli::parse();

	match cli.command {
		Commands::Run(cmd) => cmd.execute().await?,
		Commands::Version(cmd) => cmd.execute().await?,
	}

	Ok(())
}
