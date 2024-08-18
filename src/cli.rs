use std::path::Path;

use clap::Parser;
use tracing::{error, info, trace, Level};
use uuid::Uuid;

use crate::config::Config;
use crate::git::clone_repo;
use crate::utils;

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

    #[arg(short, long)]
    from_branch: Option<String>,

    #[arg(long)]
    to_branch: Option<String>,

    #[arg(long)]
    of_commit: Option<String>,

    #[arg(long, help = "Fetch latest changes before comparison")]
    fetch: bool,

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

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    setup_logging(args.tracing_level);

    let config = match load_config(Path::new(&args.config)) {
        Ok(config) => config,
        Err(e) => return Err(e),
    };

    match (args.from_branch, args.to_branch) {
        (None, None) => {
            info!("No branches specified");
            match args.of_commit {
                Some(commit_id) => analyze_commit(&commit_id),
                None => {
                    info!("No commit specified. Attempting to analyze local changes");
                    try_analyze_local_changes();
                }
            }
        }
        (None, Some(_)) => {}
        (Some(_), None) => {
            error!("Incorrect CLI arguments. Specifying `from_branch` requires `to_branch` to be specified");
            return Err(Box::from("Incorrect arguments"));
        }
        (Some(from_branch), Some(to_branch)) => {
            info!(
                "Attempting to compare branch {} with branch {}",
                from_branch, to_branch
            );
            match &config.repository.url {
                Some(url) => {
                    info!("Attempting to clone repository from url: {}", url);
                    let clone_into_path = &config.options.clone_into.unwrap_or_else(|| {
                        let path = Path::new(&format!("repository{}", Uuid::new_v4())).into();
                        trace!("set fallback clone_into path to {:?}", path);
                        path
                    });
                    match utils::prepare_directory(&clone_into_path) {
                        Ok(_) => {
                            trace!("Starting to clone repository");
                            let _cloned_repo = match clone_repo(
                                &config.repository,
                                &clone_into_path,
                                Some(&from_branch),
                            ) {
                                Ok(repo) => repo,
                                Err(e) => {
                                    error!("Failed to clone repository");
                                    return Err(e.into());
                                }
                            };
                            info!("Repository cloned successfuly");
                        }
                        Err(e) => {
                            error!("Failed to prepare directory for cloning");
                            return Err(e.into());
                        }
                    };
                }
                None => {
                    info!(
                        "Attempting to analyze changes between branch {} and {} locally",
                        from_branch, to_branch
                    );
                    try_analyze_local_changes();
                }
            }
        }
    };

    Ok(())
}

fn analyze_commit(_commit_id: &str) {}

fn try_analyze_local_changes() {}

fn setup_logging(tracing_level: u8) {
    let tracing_level = match tracing_level {
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
}

fn load_config(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    trace!("Starting loading config from {:?}", path);
    match Config::load_from_file(path) {
        Ok(config) => {
            info!("Config loaded successfully");
            Ok(config)
        }
        Err(e) => {
            error!("Failed to read configuration from {:?}", path);
            return Err(e.into());
        }
    }
}
