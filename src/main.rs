mod cli;
mod config;
mod git;
mod utils;
mod transform;

use cli::CliError;
use anyhow::Result;

fn main() -> Result<(), CliError> {
    cli::run() 
}
