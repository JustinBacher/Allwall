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
#[command(name = "next")]
pub struct Next;

impl AllwallCommand for Next {
    async fn execute(&self) -> Result<()> {
        let response = send_request(&Request::Next)?;

        match response {
            Response::Ok => Ok(()),
            Response::Error(msg) => Err(CliError::Ipc(msg).into()),
        }
    }
}
