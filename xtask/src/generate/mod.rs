mod nix;
mod schema;

use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Subcommand, Debug)]
pub enum GenerateCmd {
    Schema,

    Nix,

    All,
}

pub fn run(cmd: GenerateCmd) -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "xtask=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let output = PathBuf::from("../generated");
    std::fs::create_dir_all(&output)?;

    match cmd {
        GenerateCmd::Schema => {
            schema::generate(&output)?;
        },
        GenerateCmd::Nix => {
            nix::generate(&output)?;
        },
        GenerateCmd::All => {
            schema::generate(&output)?;
            nix::generate(&output)?;
        },
    }

    Ok(())
}
