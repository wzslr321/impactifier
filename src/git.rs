use std::path::Path;

use git2::{Cred, RemoteCallbacks, Repository};
use tracing::error;

use crate::config::RepositoryConfig;

pub fn clone_repo(
    options: &RepositoryConfig,
    clone_into: &Path,
    branch: Option<&str>,
) -> Result<Repository, Box<dyn std::error::Error>> {
    let token = match &options.access_token {
        Some(token) => token,
        None => {
            return Err(Box::from("No access token provided"));
        }
    };

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
        Cred::userpass_plaintext(&token, "")
    });

    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);
    if let Some(branch) = branch {
        builder.branch(&branch);
    }

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    fetch_options.depth(1);

    builder.fetch_options(fetch_options);

    match &options.url {
        Some(url) => match builder.clone(url.as_str(), &clone_into) {
            Ok(repository) => Ok(repository),
            Err(e) => Err(e.into()),
        },
        None => {
            error!("Failed to clone the repository. Url not specified.");
            Err(Box::from("Repository url not specified"))
        }
    }
}
