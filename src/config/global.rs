use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{DevError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    /// Default editor command (e.g., "zed", "code")
    #[serde(default = "default_editor")]
    pub editor: String,

    /// Default development directory
    #[serde(default = "default_dev_path")]
    pub dev_path: PathBuf,

    /// Whether to auto-install dependencies on switch
    #[serde(default = "default_true")]
    pub auto_install_deps: bool,

    /// Whether to launch devbox shell automatically
    #[serde(default = "default_true")]
    pub auto_devbox: bool,

    /// Shell to use for completions
    #[serde(default)]
    pub shell: Shell,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    #[default]
    Zsh,
    Bash,
    Fish,
}

fn default_editor() -> String {
    "zed".to_string()
}

fn default_dev_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("~"))
        .join("Development")
}

fn default_true() -> bool {
    true
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            editor: default_editor(),
            dev_path: default_dev_path(),
            auto_install_deps: true,
            auto_devbox: true,
            shell: Shell::default(),
        }
    }
}

impl GlobalConfig {
    /// Get the path to the global config file
    pub fn config_path() -> PathBuf {
        super::config_dir().join("config.json")
    }

    /// Load the global config, creating defaults if it doesn't exist
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(&path)?;
        let config: GlobalConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Save the global config
    pub fn save(&self) -> Result<()> {
        super::ensure_config_dirs()?;
        let path = Self::config_path();
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    /// Get a config value by key
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "editor" => Some(self.editor.clone()),
            "dev_path" => Some(self.dev_path.display().to_string()),
            "auto_install_deps" => Some(self.auto_install_deps.to_string()),
            "auto_devbox" => Some(self.auto_devbox.to_string()),
            "shell" => Some(format!("{:?}", self.shell).to_lowercase()),
            _ => None,
        }
    }

    /// Set a config value by key
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "editor" => self.editor = value.to_string(),
            "dev_path" => self.dev_path = PathBuf::from(value),
            "auto_install_deps" => {
                self.auto_install_deps = value
                    .parse()
                    .map_err(|_| DevError::ConfigError("Invalid boolean value".to_string()))?
            }
            "auto_devbox" => {
                self.auto_devbox = value
                    .parse()
                    .map_err(|_| DevError::ConfigError("Invalid boolean value".to_string()))?
            }
            "shell" => {
                self.shell = match value.to_lowercase().as_str() {
                    "zsh" => Shell::Zsh,
                    "bash" => Shell::Bash,
                    "fish" => Shell::Fish,
                    _ => {
                        return Err(DevError::ConfigError(format!(
                            "Unknown shell: {}. Valid options: zsh, bash, fish",
                            value
                        )))
                    }
                }
            }
            _ => {
                return Err(DevError::ConfigError(format!("Unknown config key: {}", key)));
            }
        }
        self.save()?;
        Ok(())
    }
}
