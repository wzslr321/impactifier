use anyhow::{anyhow, Context};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use git2::{Cred, CredentialType, Repository};
use serde_json::to_string_pretty;
use thiserror::Error;
use tracing::{error, info, trace, Level};
use url::Url;

use crate::config::Config;
use crate::git;
use crate::transform::init_registry;
use crate::utils;
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
    #[arg(long)]
    from_branch: Option<String>,

    /// To what branch changes should be compared.
    ///
    /// Defaults to "main"
    #[arg(long, default_value_t=String::from("main"))]
    to_branch: String,

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

    #[arg(long, env = "GIT_SSH_KEY")]
    ssh_key_path: Option<String>,

    #[clap(long, env = "GIT_PAT", help = "HTTPS Personal Access Token")]
    https_pat: Option<String>,
}

pub fn run() -> Result<(), CliError> {
    let args = Args::parse();
    setup_logging(args.tracing_level);

    let cfg = match load_config(Path::new(&args.config)) {
        Ok(config) => config,
        Err(e) => {
            error!("initial config load failed. Exciting...");
            return Err(e);
        }
    };
    trace!("Initial config load succeeded");

    init_registry(cfg.custom_transform_scripts());
    trace!("Transform functions initialized successfully");

    let clone_into = match cfg.options.clone_into.as_deref() {
        Some(path) => path,
        None => Path::new("cloned_repository"),
    };

    let credentials = utils::get_ssh_credentials(args.ssh_key_path.unwrap());

    let repository_retrieval_result = match cfg.repository.url {
        Some(url) => try_retrieve_repo_from_url(&credentials, &url, clone_into),
        None => match &cfg.repository.path {
            Some(path) => try_retrieve_repo_from_path(path),
            None => {
                return Err(CliError::InvalidArgs {
                    err: Some(anyhow!("Either path or url must be specified")),
                });
            }
        },
    };

    let repository = match repository_retrieval_result {
        Ok(repository) => repository,
        Err(err) => return Err(CliError::Unknown { err: Some(err) }),
    };
    trace!("Successfully retrieved repository");

    if let Err(fetch_err) = git::fetch_remote(&repository, &args.origin, &credentials) {
        error!("Failed to fetch remote");
        return Err(CliError::Unknown {
            err: Some(fetch_err),
        });
    }
    trace!("Successfully fetched remote");

    // TODO: Support other DiffOptions
    //
    // Current one is temporary, just for testing purposes
    let diff = match git::extract_difference(
        &repository,
        &git::DiffOptions::Branches {
            from: &args.from_branch.unwrap(),
            to: &args.to_branch,
        },
    ) {
        Ok(diff) => diff,
        Err(err) => {
            error!("Failed to extract difference");
            return Err(CliError::Unknown { err: Some(err) });
        }
    };
    trace!("Successfuly extracted difference");

    // Temporary, for testing purposes
    let serialized_diff = to_string_pretty(&diff).unwrap();

    let mut file = File::create("./diff.json").unwrap();
    file.write_all(serialized_diff.as_bytes()).unwrap();

    Ok(())
}

fn try_retrieve_repo_from_path(path: &Path) -> Result<Repository> {
    match git::open_repo(&path) {
        Ok(repository) => {
            info!("sucessfully retrieved repository from path");
            Ok(repository)
        }
        Err(err) => {
            return Err(anyhow!(
                "Failed to retrieve repository from path: {:?}.\nError:{}",
                path,
                err,
            ));
        }
    }
}

fn try_retrieve_repo_from_url<'a, F>(
    credentials: F,
    url: &Url,
    clone_into: &Path,
) -> Result<Repository>
where
    F: Fn(&str, Option<&str>, CredentialType) -> Result<Cred, git2::Error> + 'a,
{
    trace!("attempt to start from url-specified repository");
    utils::prepare_directory(&clone_into)
        .with_context(|| "Failed to prepare directory for cloning")?;

    trace!("Starting to clone repository");
    let cloned_repo = match git::clone_repo(&credentials, url, &clone_into) {
        Ok(repo) => repo,
        Err(err) => {
            return Err(anyhow!(
                "Failed to clone repository from url.\nError: {}",
                err
            ));
        }
    };

    Ok(cloned_repo)
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

fn load_config(path: &Path) -> Result<Config, CliError> {
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
    #[error("Invalid arguments. Error:{:?}", err)]
    InvalidArgs { err: Option<anyhow::Error> },
    #[error("Config can not be retrieved")]
    InvalidConfigPath { err: Option<anyhow::Error> },
    #[error("Unknown error: {:?}", err)]
    Unknown { err: Option<anyhow::Error> },
}
