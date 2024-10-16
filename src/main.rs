mod cli;
mod config;
mod git;
mod transform;
mod utils;

use anyhow::Result;
use cli::CliError;

fn main() -> Result<(), CliError> {
    cli::run()
}
