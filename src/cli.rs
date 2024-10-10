use std::fs::File;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use git2::Repository;
use thiserror::Error;
use tracing::{error, info, trace, Level};
use url::Url;
use uuid::Uuid;
use serde_json::to_string_pretty;

use crate::transform::init_registry;
use crate::utils;
use crate::git;
use crate::config;
use anyhow::Result;

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

    #[arg(long, default_value_t=String::from("origin"))]
    origin: String,
}

// TODO: add more credentials variants
pub enum Credentials<'a> {
    UsernamePassword {
        username: &'a str,
        password: &'a str,
    },
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

    let repository = match cfg.repository.url {
        Some(url) => match try_retrieve_repo_from_url(
            cfg.repository.access_token,
            "wzslr321",
            &url,
            cfg.options.clone_into,
        ) {
            Ok(repo) => repo,
            Err(e) => {
                return Err(e);
            }
        },
        None => match cfg.repository.path {
            Some(path) => match try_retrieve_repo_from_path(path) {
                Ok(repo) => repo,
                Err(err) => {
                    return Err(CliError::IncorrectArgs {
                        msg: "Either repository url or path must be specified".to_string(),
                        err: Some(err.into()),
                    });
                }
            },
            None => {
                error!("Repository url and path are unspecified");
                return Err(CliError::InvalidConfigPath { err: None });
            }
        },
    };

    let credentials = utils::get_mock_credentials();
    git::fetch_remote(&repository, &args.origin, &credentials).unwrap();

    let diff = git::extract_difference(
        &repository,
        &crate::git::DiffOptions::Branches {
            from: &args.from_branch.unwrap(),
            to: &args.to_branch.unwrap_or_else(|| "main".to_string()),
        },
    )
    .unwrap();
    let serialized_diff = to_string_pretty(&diff).unwrap();

    let mut file = File::create("./diff.json").unwrap();
    file.write_all(serialized_diff.as_bytes()).unwrap();

    Ok(())
}

fn check_args_validity(args: Args) -> Result<(), CliError> {
    match (&args.from_branch, &args.to_branch) {
        (None, None) => {
            trace!("No branches specified");
            match &args.of_commit {
                Some(_) => Ok(()),
                None => {
                    error!("Neither commit nor branch specified. Nothing to analyze");
                    Err(CliError::InsufficientArgs)
                }
            }
        }
        (None, Some(_)) => Ok(()),
        (Some(_), None) => {
            error!("from_branch specified, but to_branch is missing");
            Err(CliError::IncorrectArgs {
                msg: "Specifying `from_branch` requires `to_branch` to be specified".to_string(),
                err: None,
            })
        }
        (Some(_), Some(_)) => Ok(()),
    }
}

fn try_retrieve_repo_from_path(path: Box<Path>) -> Result<Repository, CliError> {
    match git::open_repo(&path) {
        Ok(repository) => {
            info!("sucessfully retrieved repository from path");
            Ok(repository)
        }
        Err(err) => {
            error!(
                "failed to retrieve repository from path: {}",
                String::from((*path).to_string_lossy())
            );
            Err(CliError::IncorrectArgs {
                msg: "Failed to retireve repository from path".to_string(),
                err: Some(err.into()),
            })
        }
    }
}

fn try_retrieve_repo_from_url(
    access_token: Option<String>,
    username: &str,
    url: &Url,
    clone_into: Option<Box<Path>>,
) -> Result<Repository, CliError> {
    trace!("attempt to start from url-specified repository");
    let clone_into_path = &clone_into.unwrap_or_else(|| {
        let path = Path::new(&format!("repository{}", Uuid::new_v4())).into();
        trace!("set fallback clone_into path to {:?}", path);
        path
    });

    match utils::prepare_directory(&clone_into_path) {
        Ok(_) => {
            trace!("Starting to clone repository");

            let credentials = Credentials::UsernamePassword {
                username,
                password: &access_token.unwrap_or_else(|| "OnlyForTesting".to_string()), // ehttps://www.twitch.tv/directory/followingxpect("access_token must be specified, as it is the only supported authentication method for now"),
            };
            let cloned_repo = match git::clone_repo(&credentials, url, &clone_into_path) {
                Ok(repo) => repo,
                Err(e) => {
                    error!("Failed to retreive repository from url.\nError: {}", e);
                    let err = match e {
                        crate::git::GitError::NoAccess { err } => CliError::InvalidArgs {
                            err: Some(err.into()),
                        },

                        _ => CliError::Unknown {
                            err: Some(e.into()),
                        },
                    };
                    return Err(err);
                }
            };
            info!("Repository retrieved successfuly from url");
            Ok(cloned_repo)
        }
        Err(err) => {
            error!("Failed to prepare directory for cloning");
            return Err(CliError::Unknown { err: Some(err) });
        }
    }
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

fn load_config(path: &Path) -> anyhow::Result<config::Config, CliError> {
    trace!("Starting loading config from {:?}", path);
    match config::Config::load_from_file(path) {
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
    #[error("Incorrect CLI arguments.{}\nError:{:?}", msg, err)]
    IncorrectArgs {
        msg: String,
        err: Option<anyhow::Error>,
    },
    #[error("Invalid arguments. Error:{:?}", err)]
    InvalidArgs { err: Option<anyhow::Error> },
    #[error("Config can not be retrieved")]
    InvalidConfigPath { err: Option<anyhow::Error> },
    #[error("Unknown error: {:?}", err)]
    Unknown { err: Option<anyhow::Error> },
}
