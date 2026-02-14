use log::info;

use super::AllwallCommand;
use crate::prelude::*;

#[derive(clap::Parser, Debug)]
#[command()]
pub struct Version;

impl AllwallCommand for Version {
	async fn execute(&self) -> Result<()> {
		const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
		info!("allwall version {}", VERSION.unwrap_or("unknown"));
		Ok(())
	}
}
