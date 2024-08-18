use std::path::Path;

use clap::Parser;
use tracing::{error, info, trace, Level};
use uuid::Uuid;

use crate::config::{Config, RepositoryConfig};
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

// Can be run either locally or inside a CI/CD
// CI/CD automatically can detect commit/branches
//
// In case of local usage, we have a few options
// - specify url & analyze impact of specified branches/commits
//   from_branch fallbacks to default after clone
//   if no branch & commit specified, fails as there is nothing to compare
// - dont specify url, specify path
//   creates repository struct from local path
//   from_branch fallbacks to default after opening
//   if branches specified & local changes detected, optionally includes those
//   if no branch & commit specified, tries to analyze local changes
//   if no local changes fails as there is nothing to compare
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    setup_logging(args.tracing_level);

    let config = match load_config(Path::new(&args.config)) {
        Ok(config) => config,
        Err(e) => return Err(e),
    };

    match &config.repository.url {
        Some(url) => {
            let _ = try_analyze_from_url(
                url.as_str(),
                args.from_branch,
                args.to_branch,
                args.of_commit,
                config.options.clone_into,
                &config.repository,
            );
        }
        None => {
            match &config.repository.path {
                Some(path) => {
                    let _ = try_analyze_from_path(
                        (*path)
                            .to_str()
                            .expect("Path is expected to be validated during serialization"),
                        args.from_branch,
                        args.to_branch,
                        args.of_commit,
                        &config.repository,
                    );
                }
                None => {
                    error!("Repository url and path are unspecified");
                    return Err(Box::from("Either repository url or path must be specified"));
                }
            };
        }
    };

    Ok(())
}

fn try_analyze_from_path(
    _path: &str,
    _from_branch: Option<String>,
    to_branch: Option<String>,
    commit_id: Option<String>,
    _config: &RepositoryConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if let None = to_branch {
        return match &commit_id {
            Some(commit_id) => try_analyze_commit(commit_id),
            None => try_analyze_local_changes(),
        };
    };

    Ok(())
}

fn try_analyze_from_url(
    url: &str,
    from_branch: Option<String>,
    to_branch: Option<String>,
    commit_id: Option<String>,
    clone_into: Option<Box<Path>>,
    config: &RepositoryConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    match (from_branch, to_branch) {
        (None, None) => {
            info!("No branches specified");
            match commit_id {
                Some(commit_id) => try_analyze_commit(&commit_id),
                None => {
                    error!("No commit specified. Nothing to analyze");
                    Err(Box::from(
                        "No branches and no commit specified. Nothing to analyze.",
                    ))
                }
            }
        }
        // TODO: implement
        (None, Some(_)) => Ok(()),
        (Some(_), None) => {
            error!("Incorrect CLI arguments. Specifying `from_branch` requires `to_branch` to be specified");
            Err(Box::from("Incorrect arguments"))
        }
        (Some(from_branch), Some(to_branch)) => {
            info!(
                "Attempting to compare branch {} with branch {}",
                from_branch, to_branch
            );
            info!("Attempting to clone repository from url: {}", url);
            let clone_into_path = &clone_into.unwrap_or_else(|| {
                let path = Path::new(&format!("repository{}", Uuid::new_v4())).into();
                trace!("set fallback clone_into path to {:?}", path);
                path
            });
            match utils::prepare_directory(&clone_into_path) {
                Ok(_) => {
                    trace!("Starting to clone repository");
                    let _cloned_repo =
                        match clone_repo(&config, &clone_into_path, Some(&from_branch)) {
                            Ok(repo) => repo,
                            Err(e) => {
                                error!("Failed to clone repository");
                                return Err(e.into());
                            }
                        };
                    info!("Repository cloned successfuly");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to prepare directory for cloning");
                    return Err(e.into());
                }
            }
        }
    }
}

fn try_analyze_commit(_commit_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn try_analyze_local_changes() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
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
