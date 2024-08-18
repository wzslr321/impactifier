mod config;
mod git;
mod utils;

use config::Config;
use git::clone_repo;
use std::error::Error;
use tracing::{error, info, warn, Level};

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();

    let configuration_file = "impactifier-config.yaml";
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
