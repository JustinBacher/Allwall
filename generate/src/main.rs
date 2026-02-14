use std::path::PathBuf;

use clap::{Parser, Subcommand};
use env_logger::{Builder, Env};

mod nix;
mod schema;

#[derive(Parser, Debug)]
#[command(name = "xtask")]
struct Xtask {
	#[command(subcommand)]
	command: Commands,

	/// Output directory for generated files
	#[arg(short, long, global = true, default_value = "../generated")]
	output: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
	/// Generate JSON Schema for configuration
	Schema,

	/// Generate NixOS module and flake
	Nix,

	/// Generate all artifacts
	All,
}

fn main() {
	let args = Xtask::parse();

	Builder::from_env(Env::default()).init();

	// Ensure output directory exists
	std::fs::create_dir_all(&args.output).expect("Failed to create output directory");

	match args.command {
		Commands::Schema => {
			schema::generate(&args.output);
		},
		Commands::Nix => {
			nix::generate(&args.output);
		},
		Commands::All => {
			schema::generate(&args.output);
			nix::generate(&args.output);
		},
	}
}
