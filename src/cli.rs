use std::path::Path;

use clap::Parser;
use git2::Repository;
use thiserror::Error;
use tracing::{error, info, trace, Level};
use url::Url;
use uuid::Uuid;

use crate::transform::init_registry;
use crate::utils;
use crate::{config::Config, git::clone_repo};

#[derive(Parser, Debug, Clone)]
#[command(
    version,
    about = "Impactifier is a tool for analyzing code changes and assessing their impact.",
    long_about = r#"
    Can be run either locally or inside a CI/CD
    CI/CD automatically can detect commit/branches

    In case of local usage, we have a few options
    - specify url & analyze impact of specified branches/commits
    from_branch fallbacks to default after clone
    if no branch & commit specified, fails as there is nothing to compare
    - dont specify url, specify path
    creates repository struct from local path
    from_branch fallbacks to default after opening
    if branches specified & local changes detected, optionally includes those
    if no branch & commit specified, tries (not yet) to analyze local changes
    if no local changes fails as there is nothing to compare
"#
)]
struct Args {
    /// Path to the config file.
    /// Currently, only .yaml files are supported.
    ///
    /// Example config file can be found at: github.com/impactifier/example
    #[arg(short, long, default_value_t = String::from("impactifier-config.yaml"))]
    config: String,

    /// From what branch changes should be compared.
    ///
    /// Defaults to the current branch.
    #[arg(short, long)]
    from_branch: Option<String>,

    /// To what branch changes should be compared
    #[arg(long)]
    to_branch: Option<String>,

    /// Commit of which changes should be analyzed. Takes precedence over
    /// branch changes, if `from_branch` or `to_branch` is specified.
    #[arg(long)]
    of_commit: Option<String>,

    /// Fetch last changes before impact analysis
    #[arg(long)]
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

pub fn run() -> Result<(), CliError> {
    let args = Args::parse();
    setup_logging(args.tracing_level);

    match check_args_validity(args.clone()) {
        Ok(_) => {
            trace!("args validation completed successfully. Continuing execution.");
        }
        Err(err) => {
            error!("args are invalid. Exiting...");
            return Err(err);
        }
    }

    let cfg = match load_config(Path::new(&args.config)) {
        Ok(config) => config,
        Err(e) => return Err(e),
    };

    init_registry(cfg.custom_transform_scripts());

    let _repository = match &cfg.repository.url {
        Some(url) => match try_retrieve_repo_from_url(
            cfg.repository.access_token,
            "wzslr321",
            url,
            cfg.options.clone_into,
        ) {
            Ok(repo) => repo,
            Err(_) => {
                return Err(CliError::Unknown);
            }
        },
        None => match &cfg.repository.path {
            Some(path) => {
                match try_retrieve_repo_from_path(
                    (*path)
                        .to_str()
                        .expect("Path is expected to be validated during serialization"),
                ) {
                    Ok(repo) => repo,
                    Err(_) => {
                        return Err(CliError::IncorrectArgs {
                            msg: "Either repository url or path must be specified".to_string(),
                        });
                    }
                }
            }
            None => {
                error!("Repository url and path are unspecified");
                return Err(CliError::InvalidConfigPath { err: None });
            }
        },
    };

    Ok(())
}

fn check_args_validity(args: Args) -> Result<(), CliError> {
    match (&args.from_branch, &args.to_branch) {
        (None, None) => {
            info!("No branches specified");
            match &args.of_commit {
                Some(_) => Ok(()),
                None => {
                    error!("No commit specified. Nothing to analyze");
                    Err(CliError::InsufficientArgs)
                }
            }
        }
        (None, Some(_)) => Ok(()),
        (Some(_), None) => {
            error!("from_branch specified, but to_branch is missing");
            Err(CliError::IncorrectArgs {
                msg: "Specifying `from_branch` requires `to_branch` to be specified".to_string(),
            })
        }
        (Some(_), Some(_)) => Ok(()),
    }
}

fn try_retrieve_repo_from_path(path: &str) -> Result<Repository, Box<dyn std::error::Error>> {
    trace!(
        "attempt to retireve repo path specified repository.\nPath:{}",
        path
    );
    todo!();
}

fn try_retrieve_repo_from_url(
    access_token: Option<String>,
    username: &str,
    url: &Url,
    clone_into: Option<Box<Path>>,
) -> Result<Repository, Box<dyn std::error::Error>> {
    trace!("attempt to start from url-specified repository");
    let clone_into_path = &clone_into.unwrap_or_else(|| {
        let path = Path::new(&format!("repository{}", Uuid::new_v4())).into();
        trace!("set fallback clone_into path to {:?}", path);
        path
    });
    match utils::prepare_directory(&clone_into_path) {
        Ok(_) => {
            trace!("Starting to clone repository");
            let cloned_repo = match clone_repo(access_token, username, url, &clone_into_path) {
                Ok(repo) => repo,
                Err(e) => {
                    error!("Failed to clone repository. error: {}", e);
                    return Err(e.into());
                }
            };
            info!("Repository cloned successfuly");
            Ok(cloned_repo)
        }
        Err(e) => {
            error!("Failed to prepare directory for cloning");
            return Err(e.into());
        }
    }
}

fn try_analyze_commit(_commit_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

fn try_analyze_local_changes() -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}

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

fn load_config(path: &Path) -> anyhow::Result<Config, CliError> {
    trace!("Starting loading config from {:?}", path);
    match Config::load_from_file(path) {
        Ok(config) => {
            info!("Config loaded successfully");
            Ok(config)
        }
        Err(err) => {
            error!("Failed to read configuration from {:?}", path);
            return Err(CliError::InvalidConfigPath { err: Some(err) });
        }
    }
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("No branches and no commit specified. No local changes detected. Nothing to analyze.")]
    InsufficientArgs,
    #[error("Incorrect CLI arguments.{}", msg)]
    IncorrectArgs { msg: String },
    #[error("Config can not be retrieved")]
    InvalidConfigPath { err: Option<anyhow::Error> },
    #[error("Unknown error")]
    Unknown,
}
