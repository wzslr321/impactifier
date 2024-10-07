mod cli;
mod config;
mod git;
mod utils;
mod transform;

use cli::CliError;

fn main() -> Result<(), CliError> {
    cli::run() 
}
