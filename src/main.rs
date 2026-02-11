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
	/// Increase verbosity level
	#[arg(short, long, action = clap::ArgAction::Count, global = true)]
	verbose: u8,

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
	let cli = Cli::parse();

	let log_level = match cli.verbose {
		0 => "warn",
		1 => "info",
		2 => "debug",
		_ => "trace",
	};
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

	match cli.command {
		Commands::Run(cmd) => cmd.execute().await?,
		Commands::Version(cmd) => cmd.execute().await?,
	}

	Ok(())
}
