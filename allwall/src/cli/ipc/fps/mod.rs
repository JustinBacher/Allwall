use clap::Parser;

use crate::{
    cli::{
        AllwallCommand,
        error::CliError,
        ipc::protocol::{Request, Response, send_request},
    },
    prelude::*,
};

#[derive(Parser, Debug)]
#[command(name = "fps")]
pub struct Fps {
    /// Target framerate
    pub value: u32,
}

impl AllwallCommand for Fps {
    async fn execute(&self) -> Result<()> {
        let response = send_request(&Request::SetFps(self.value))?;

        match response {
            Response::Ok => Ok(()),
            Response::Error(msg) => Err(CliError::Ipc(msg).into()),
        }
    }
}
