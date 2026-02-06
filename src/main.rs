use crate::cli::{AllwallCommand, Run, Version};
use crate::prelude::*;
use clap::Parser;

mod cli;
mod decode;
mod error;
mod prelude;
mod renderer;
mod utils;

#[derive(Parser, Debug)]
#[command(name = "allwall")]
#[command(about = "Zero-Copy Wayland Wallpaper (Rust)", long_about = None)]
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
