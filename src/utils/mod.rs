use std::{fs, path::Path};
use tracing::{info, trace};

pub fn prepare_directory(path: &Path) -> Result<(), anyhow::Error> {
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
