use clap::Parser;

pub mod cli;
pub mod config;
pub mod decode;
pub mod engine;
pub mod error;
pub mod prelude;
pub mod sources;
pub mod transitions;
pub mod utils;

pub use config::Config;

#[derive(Parser, Debug)]
#[command(name = "allwall")]
#[command(about = "Allwall a zero-copy wayland wallpaper", long_about = None)]
pub struct Cli {
	/// Increase verbosity level
	#[arg(short, long, action = clap::ArgAction::Count, global = true)]
	pub verbose: u8,

	#[command(subcommand)]
	pub command: cli::Commands,
}
