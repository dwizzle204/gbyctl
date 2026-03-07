//! Binary entrypoint.

use anyhow::Result;
use clap::Parser;

use gbyctl::cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    gbyctl::cli::dispatch(cli)
}
