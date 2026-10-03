#![forbid(unsafe_code)]
//! CLI command dispatch.
//!
//! `network` reads `NUVEX_API_URL` when set. Other operational commands return
//! [`CliError::NotImplemented`]. The CLI does not talk to a cluster.

use std::ffi::OsString;

use clap::{Parser, Subcommand};
use thiserror::Error;

pub mod commands;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CliError {
    #[error("{command} is not implemented (milestone {milestone})")]
    NotImplemented {
        command: &'static str,
        milestone: u8,
    },
    #[error("NUVEX_API_URL is unset. The CLI does not invent network status.")]
    ApiUnset,
    #[error("read API request failed: {0}")]
    ApiFailed(String),
    #[error("{0}")]
    Usage(String),
}

#[derive(Parser)]
#[command(name = "nuvex", version, about = "Nuvex protocol CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Node process helpers.
    Node,
    /// Request lifecycle.
    Request,
    /// Registry administration.
    Registry,
    /// Staking operations.
    Staking,
    /// Network status from the read API.
    Network,
}

pub fn execute<I, T>(args: I) -> Result<(), CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match Cli::try_parse_from(args) {
        Ok(cli) => dispatch(cli.command),
        Err(error)
            if error.kind() == clap::error::ErrorKind::DisplayHelp
                || error.kind() == clap::error::ErrorKind::DisplayVersion =>
        {
            let _ = error.print();
            Ok(())
        }
        Err(error) => Err(CliError::Usage(error.to_string())),
    }
}

fn dispatch(command: Command) -> Result<(), CliError> {
    match command {
        Command::Node => commands::node::run(),
        Command::Request => commands::request::run(),
        Command::Registry => commands::registry::run(),
        Command::Staking => commands::staking::run(),
        Command::Network => commands::network::run(),
    }
}
