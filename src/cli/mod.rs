use crate::prelude::*;

pub trait AllwallCommand {
	async fn execute(&self) -> Result<()>;
}

mod run;
mod version;

pub use run::Run;
pub use version::Version;
