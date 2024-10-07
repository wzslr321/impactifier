use std::{env, path::Path};
use thiserror::Error;

use git2::{Cred, RemoteCallbacks, Repository};
use url::Url;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to authorize git request, due to authentication failure")]
    NoAccess,
    #[error(
        "Failed to clone repository from url {} to given path: {}.\nError: {}",
        url,
        path,
        err,
    )]
    CloneFailure {
        url: String,
        path: String,
        err: git2::Error,
    },
}

pub fn clone_repo(
    access_token: Option<String>,
    url: &Url,
    clone_into: &Box<Path>,
) -> Result<Repository, GitError> {
    let token = match access_token.to_owned() {
        Some(token) => token,
        None => match env::var("GITHUB_ACCESS_TOKEN_2") {
            Ok(token) => token,
            Err(_) => return Err(GitError::NoAccess),
        },
    };

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext("wzslr321", &token)
    });

    let mut builder = git2::build::RepoBuilder::new();

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    fetch_options.depth(1);

    builder.fetch_options(fetch_options);


    match builder.clone(url.as_str(), &clone_into) {
        Ok(repository) => Ok(repository),
        Err(e) => Err(GitError::CloneFailure{
            url: url.to_string(),
            path: String::from(clone_into.to_string_lossy()),
            err: e,
        }),
    }
}
