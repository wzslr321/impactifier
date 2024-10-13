use anyhow::Result;
use git2::{Cred, CredentialType};
use std::{fs, path::Path};
use tracing::{info, trace};

pub fn prepare_directory(path: &Path) -> Result<()> {
    trace!("Preparing directory for repository cloning");
    if path.exists() {
        if path.read_dir()?.next().is_some() {
            info!("Directory is not empty, removing existing files...");
            fs::remove_dir_all(path)?;
            fs::create_dir(path)?;
        }
    } else {
        info!("Directory is empty, creating...");
        fs::create_dir(path)?;
    }
    trace!("Successfully prepared directory for cloning");
    Ok(())
}

pub fn get_git_credentials(
    ssh_key_path: Option<String>,
    username: String,
    https_pat: Option<String>,
) -> Option<impl Fn(&str, Option<&str>, CredentialType) -> Result<Cred, git2::Error>> {
    if let (None, None) = (&ssh_key_path, &https_pat) {
        trace!("Neither ssh key path, nor https pat was specified");
        return None;
    }

    Some(
        move |_url: &str, _username: Option<&str>, allowed_types: CredentialType| {
            if let Some(ssh) = &ssh_key_path {
                if allowed_types.contains(CredentialType::SSH_KEY) {
                    return Cred::ssh_key(&username, None, Path::new(&ssh), None);
                } else {
                    return Err(git2::Error::from_str("Unsupported credential type for SSH"));
                }
            } else {
                if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                    return Cred::userpass_plaintext(&username, &https_pat.clone().unwrap());
                } else {
                    return Err(git2::Error::from_str(
                        "Unsupported credential type for user_pass_plaintext",
                    ));
                }
            }
        },
    )
}
