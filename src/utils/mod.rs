use std::{fs, path::Path};
use tracing::{info, trace};
use anyhow::Result;

use crate::cli::Credentials;

pub fn prepare_directory(path: &Path) -> Result<()> {
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

pub fn get_mock_credentials<'a>() -> &'a Credentials<'a> {
    &Credentials::UsernamePassword {
        username: "wzslr321",
        password: "TEST",
    }
}
