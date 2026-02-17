use allwall::{
    Cli,
    cli::{AllwallCommand, Commands},
};
use clap::Parser;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> allwall::prelude::Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| format!("allwall={}", log_level).into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let Err(e) = gstreamer::init() {
        eprintln!("Failed to initialize GStreamer: {}", e);
        std::process::exit(1);
    }

    match cli.command {
        Commands::Run(cmd) => cmd.execute().await?,
        Commands::Version(cmd) => cmd.execute().await?,
        Commands::Completions(cmd) => cmd.execute().await?,
        Commands::Next(cmd) => cmd.execute().await?,
        Commands::Prev(cmd) => cmd.execute().await?,
        Commands::Fps(cmd) => cmd.execute().await?,
    }

    Ok(())
}
