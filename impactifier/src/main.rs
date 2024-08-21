mod config;
mod git;
mod utils;

use clap::Parser;
use config::Config;
use git::clone_repo;
use std::error::Error;
use tracing::{error, info, warn, Level};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Impactifier is a tool for analyzing code changes and assessing their impact.",
    long_about = r#"
Impactifier is an early-stage tool designed to help developers understand the impact of their code changes before they are merged. 
It simplifies the process of identifying potential issues and dependencies by analyzing code changes within a repository.

Key features include:

- Analyzing changes to identify potential impacts on other parts of the codebase.
- Integrating with CI/CD pipelines to automate impact analysis on pull requests and commits.
- Configurable to work with different repository setups and trigger actions.
"#
)]
struct Args {
    /// Path to the config file.
    /// Currently, only .yaml files are supported.
    ///
    /// Example config file can be found at: github.com/impactifier/example
    #[arg(short, long, default_value_t = String::from("impactifier-config.yaml"))]
    config: String,

    /// Sets max tracing level. Available options:
    ///
    /// 0 = Trace
    /// 1 = Debug
    /// 2 = Info
    /// 3 = Warn
    /// 4 = Error
    #[arg(long, default_value_t = 2)]
    tracing_level: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let tracing_level = match args.tracing_level {
        0 => Level::TRACE,
        1 => Level::DEBUG,
        2 => Level::INFO,
        3 => Level::WARN,
        4 => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(tracing_level)
        .init();

    let configuration_file = args.config;
    info!("Starting loading config from {}", configuration_file);
    let config = match Config::load_from_file(&configuration_file) {
        Ok(config) => {
            info!("Config loaded successfully");
            config
        }
        Err(e) => {
            error!("Failed to read configuration from {}", &configuration_file);
            return Err(e.into());
        }
    };
    match utils::prepare_directory(&config.options.clone_into) {
        Ok(_) => {
            info!("Successfully prepared directory for cloning");
        }
        Err(e) => {
            error!("Failed to prepare directory for cloning");
            return Err(e.into());
        }
    };

    info!("Starting to clone repository");
    let cloned_repo = match clone_repo(&config.repository, &config.options.clone_into) {
        Ok(repo) => repo,
        Err(e) => {
            error!("Failed to clone repository");
            return Err(e.into());
        }
    };
    info!("Repository cloned successfuly");

    let latest_commit_message = match get_latest_commit_message(&cloned_repo) {
        Ok(message) => message,
        Err(e) => {
            error!("Failed to get latest commit message");
            return Err(e.into());
        }
    };
    info!("Latest commit message: {}", latest_commit_message);

    Ok(())
}

fn get_latest_commit_message(repo: &git2::Repository) -> Result<String, Box<dyn Error>> {
    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => {
            error!("Failed to retrieve head");
            return Err(e.into());
        }
    };

    let latest_commit = match head.peel_to_commit() {
        Ok(commit) => commit,
        Err(e) => {
            error!("Failed to peel to latest commit");
            return Err(e.into());
        }
    };

    let latest_commit_message = match latest_commit.message() {
        Some(message) => message,
        None => {
            warn!("Retrieval of latest commit message failed");
            return Err(Box::from("Commit message is not valid UTF-8"));
        }
    };

    Ok(latest_commit_message.to_string())
}
