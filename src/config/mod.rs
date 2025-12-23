pub mod global;
pub mod project;

pub use global::GlobalConfig;
pub use project::ProjectConfig;

use std::path::PathBuf;

/// Get the config directory path (~/.config/dev/)
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
        .join("dev")
}

/// Ensure the config directory structure exists
pub fn ensure_config_dirs() -> std::io::Result<()> {
    let config = config_dir();
    std::fs::create_dir_all(&config)?;
    std::fs::create_dir_all(config.join("projects"))?;
    Ok(())
}
