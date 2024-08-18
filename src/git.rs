use git2::{Cred, RemoteCallbacks, Repository};
use std::path::Path;

use crate::config::RepositoryConfig;

// TODO: Optimize cloning to use `--bare` flag,
// consider shallow copy if commit history turns out not to be crucial,
// and possibly setup cache strategy
pub fn clone_repo(
    options: &RepositoryConfig,
    clone_into: &Path,
) -> Result<Repository, git2::Error> {
    let token = match &options.access_token {
        Some(token) => token,
        None => {
            return Err(git2::Error::from_str("No access token provided"));
        }
    };

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(&token, "")
    });

    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);
    builder.branch(&options.branch);

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    fetch_options.depth(1);

    builder.fetch_options(fetch_options);

    builder.clone(&options.url.to_string(), clone_into)
}