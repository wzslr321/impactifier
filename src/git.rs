use std::path::Path;
use thiserror::Error;

use git2::{Cred, RemoteCallbacks, Repository};
use std::str;
use tracing::{error, info, trace};
use url::Url;

use crate::cli::Credentials;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to authorize git request, due to authentication failure. Error:{err}")]
    NoAccess { err: git2::Error },
    #[error(
        "Failed to clone repository from url {} to given path: {}.\nError: {}",
        url,
        path,
        err
    )]
    CloneFailure {
        url: String,
        path: String,
        err: git2::Error,
    },
    #[error("Failed to open repository from path: {}. Error: {}", path, err)]
    OpenRepositoryFailure { path: String, err: git2::Error },
    // #[error("Unknown error: {}", *err)]
    // Unknown { err: Box<dyn std::error::Error> },
}

#[derive(Debug)]
pub struct Diff {
    pub deltas: Vec<FileDelta>,
}

#[derive(Debug)]
pub struct FileDelta {
    pub value: String,
}

impl FileDelta {
    fn from(value: String) -> Self {
        Self { value }
    }
}

pub enum DiffOptions<'a> {
    Branches { from: &'a str, to: &'a str },
}

pub fn extract_difference(repo: &Repository, options: &DiffOptions) -> anyhow::Result<Diff> {
    match options {
        DiffOptions::Branches { from, to } => extract_difference_branches(repo, from, to),
    }
}

pub fn fetch_remote(repo: &Repository, remote_name: &str, credentials: &Credentials) -> anyhow::Result<()> {
    // Find the remote
    let mut remote = repo.find_remote(remote_name)?;

    // Set up callbacks for authentication (if needed)
    let mut cb = RemoteCallbacks::new();
    cb.credentials(|_url, _username_from_url, _allowed_types| credentials.into());

    // Configure fetch options with the callbacks
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(cb);

    // Define the refspecs to fetch. Here, we fetch all branches.
    let refspecs = ["+refs/heads/*:refs/remotes/origin/*"];

    // Perform the fetch
    remote.fetch(&refspecs, Some(&mut fetch_options), None)?;

    Ok(())
}

pub fn extract_difference_branches(
    repo: &Repository,
    from_branch: &str,
    to_branch: &str,
) -> anyhow::Result<Diff> {
    let ref_from = repo.find_reference(&format!("refs/heads/{}", from_branch))?;
    let ref_to = repo.find_reference(&format!("refs/remotes/origin/{}", to_branch))?;

    let commit_a = ref_from.peel_to_commit()?;
    let commit_b = ref_to.peel_to_commit()?;

    let tree_a = commit_a.tree()?;
    let tree_b = commit_b.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&tree_a), Some(&tree_b), None)?;
    let mut diff_output = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        diff_output.extend_from_slice(line.content());
        true
    })?;

    let diff_str = str::from_utf8(&diff_output)
        .map_err(|e| git2::Error::from_str(&format!("UTF-8 conversion error: {}", e)))?
        .to_string();

    Ok(Diff {
        deltas: vec![FileDelta::from(diff_str)],
    })
}

pub fn open_repo(path: &Path) -> Result<Repository, GitError> {
    info!("start opening repository");

    match Repository::open(path) {
        Ok(repository) => {
            trace!("repository opened successfuly");
            Ok(repository)
        }
        Err(err) => {
            error!("failed to open repository");
            Err(GitError::OpenRepositoryFailure {
                path: String::from(path.to_string_lossy()),
                err,
            })
        }
    }
}

impl Credentials<'_> {
    fn into(&self) -> Result<Cred, git2::Error> {
        let credentials = match self {
            Credentials::UsernamePassword { username, password } => {
                Cred::userpass_plaintext(&username, &password)
            }
        };

        match credentials {
            Ok(credentials) => Ok(credentials),
            Err(err) => Err(err),
        }
    }
}

pub fn clone_repo(
    credentials: &Credentials,
    url: &Url,
    clone_into: &Box<Path>,
) -> Result<Repository, GitError> {
    info!("start cloning repository");

    let mut callbacks = RemoteCallbacks::new();
    // TODO(wiktor.zajac) [https://github.com/wzslr321/impactifier/issues/9]
    // Support different credentials for git access
    //
    // Additionally, it can probably be extracted to a separate util
    callbacks.credentials(|_url, _username_from_url, _allowed_types| credentials.into());
    trace!("Callback credentials set to userpass_plaintext");

    let mut builder = git2::build::RepoBuilder::new();

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    fetch_options.depth(1);

    builder.fetch_options(fetch_options);
    // TODO(wiktor.zajac) try to guard agains future changes, to update this trace automatically
    // by using some implemented trait with display
    trace!("FetchOptions set to depth=0");

    match builder.clone(url.as_str(), &clone_into) {
        Ok(repository) => {
            info!("repository cloned successfully");
            Ok(repository)
        }
        Err(e) => {
            error!("failed to clone repository");
            let err = match e.code() {
                git2::ErrorCode::Auth => GitError::NoAccess { err: e },
                _ => GitError::CloneFailure {
                    url: url.to_string(),
                    path: String::from(clone_into.to_string_lossy()),
                    err: e,
                },
            };
            Err(err)
        }
    }
}
