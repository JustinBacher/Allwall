use allwall::{
	cli::{AllwallCommand, Commands},
	Cli,
};
use clap::Parser;

#[tokio::main]
async fn main() -> allwall::prelude::Result<()> {
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
		Commands::Completions(cmd) => cmd.execute().await?,
	}

	Ok(())
}
