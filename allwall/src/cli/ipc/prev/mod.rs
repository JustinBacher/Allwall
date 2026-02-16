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
#[command(name = "prev")]
pub struct Prev;

impl AllwallCommand for Prev {
    async fn execute(&self) -> Result<()> {
        let response = send_request(&Request::Prev)?;

        match response {
            Response::Ok => Ok(()),
            Response::Error(msg) => Err(CliError::Ipc(msg).into()),
        }
    }
}
