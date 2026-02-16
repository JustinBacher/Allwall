use clap::{Parser, Subcommand};

mod check;
mod generate;

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "Development automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Check,

    #[command(subcommand)]
    Generate(generate::GenerateCmd),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check => {
            check::run()?;
        },
        Commands::Generate(cmd) => {
            generate::run(cmd)?;
        },
    }

    Ok(())
}
