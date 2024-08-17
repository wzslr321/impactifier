mod config;
mod git;

use config::Config;
use std::path::Path;
use tracing::{error, info, warn};
use std::error::Error;

// TODO: Improve error handling with custom errors
fn main() -> Result<(), Box<dyn Error>> {
    let configuration_file = "impactifier-config.yaml";
    let config = match Config::load_from_file(&configuration_file) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to read configuration from {}", &configuration_file);
            return Err(e.into());
        }
    };

    tracing_subscriber::fmt().init();

    info!("Starting to clone repository");
    let cloned_repo = match git::clone_repo(
        &config.repository.path,
        Path::new("./repo/"),
        config.repository.access_token.as_deref(),
    ) {
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
