use std::path::Path;
use thiserror::Error;

use git2::{Cred, RemoteCallbacks, Repository};
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

pub struct FileDiff {}

pub fn extract_difference(_repo: &Repository) -> Result<Vec<FileDiff>, GitError> {
    todo!()
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
            Credentials::SshKey {
                username,
                public_key_path,
                private_key_path,
                passphrase,
            } => Cred::ssh_key(
                &username,
                Some(Path::new(public_key_path)),
                Path::new(private_key_path),
                passphrase.as_deref(),
            ),
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
