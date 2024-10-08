use std::{env, path::Path};
use thiserror::Error;

use git2::{Cred, RemoteCallbacks, Repository};
use tracing::{error, info, trace};
use url::Url;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to authorize git request, due to authentication failure. Error:{msg}")]
    NoAccess { msg: String },
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
}

pub fn clone_repo(
    access_token: Option<String>,
    username: &str,
    url: &Url,
    clone_into: &Box<Path>,
) -> Result<Repository, GitError> {
    info!("start cloning repository");

    trace!("try retrieve access token");
    let token = match access_token.to_owned() {
        Some(token) => token,
        None => match env::var("GITHUB_ACCESS_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                error!("failed to retrieve access token");
                return Err(GitError::NoAccess {
                    msg: "Access token unspecified".to_string(),
                });
            }
        },
    };
    trace!("Retrieved token successfully");

    let mut callbacks = RemoteCallbacks::new();
    // TODO(wiktor.zajac) [https://github.com/wzslr321/impactifier/issues/9]
    // Support different credentials for git access
    //
    // Additionally, it can probably be extracted to a separate util
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(username, &token)
    });
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
            Err(GitError::CloneFailure {
                url: url.to_string(),
                path: String::from(clone_into.to_string_lossy()),
                err: e,
            })
        }
    }
}
