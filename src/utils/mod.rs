use std::fs;
use std::path::Path;
use tracing::info;

pub fn prepare_directory(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}
