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

pub fn get_ssh_credentials(
    ssh_key_path: String,
) -> impl Fn(&str, Option<&str>, CredentialType) -> Result<Cred, git2::Error> {
    // let key_path_owned: String = ssh_key_path
    //     .unwrap_or_else(|| {
    //         dirs_2::home_dir()
    //             .map(|p| p.join(".ssh").join("id_rsa").to_string_lossy().into_owned())
    //             .unwrap_or_else(|| String::from("~/.ssh/id_rsa"))
    //     });

    move |_url, username, allowed_types| {
        if allowed_types.contains(CredentialType::SSH_KEY) {
            let cred = Cred::ssh_key(
                username.unwrap_or_else(|| "git"),
                None,
                Path::new(&ssh_key_path),
                None,
            )?;

            Ok(cred)
        } else {
            Err(git2::Error::from_str("Unsupported credential type for SSH"))
        }
    }
}
