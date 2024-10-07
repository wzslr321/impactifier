mod cli;
mod config;
mod git;
mod utils;
mod transform;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    cli::run()
}
