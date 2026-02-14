use std::io;

use clap::CommandFactory;
use clap_complete::{generate, Shell};

use super::AllwallCommand;
use crate::prelude::*;

/// Generate shell completions for allwall
///
/// Outputs completion scripts to stdout. Redirect to a file or
/// pipe to your shell's completion directory.
///
/// # Examples
///
/// ```bash
/// # Bash
/// allwall completions bash > /etc/bash_completion.d/allwall
/// # Zsh
/// allwall completions zsh > "${fpath[1]}/_allwall"
/// # Fish
/// allwall completions fish > ~/.config/fish/completions/allwall.fish
/// ```
#[derive(clap::Parser, Debug)]
pub struct Completions {
	/// Shell to generate completions for
	#[arg(value_enum)]
	shell: Shell,
}

impl AllwallCommand for Completions {
	async fn execute(&self) -> Result<()> {
		let mut cmd = crate::Cli::command();
		let name = "allwall";
		generate(self.shell, &mut cmd, name, &mut io::stdout());
		Ok(())
	}
}
