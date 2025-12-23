use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{DevError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    /// Project name (used as identifier)
    pub name: String,

    /// Absolute path to project root
    pub path: PathBuf,

    /// Git remote URL (if cloned)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_url: Option<String>,

    /// Override editor for this project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,

    /// Override auto-install for this project
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_install_deps: Option<bool>,

    /// Whether this project uses devbox
    #[serde(default)]
    pub uses_devbox: bool,

    /// Custom environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Project creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last accessed timestamp
    pub last_accessed: DateTime<Utc>,
}

impl ProjectConfig {
    /// Create a new project config
    pub fn new(name: &str, path: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            name: name.to_string(),
            path,
            remote_url: None,
            editor: None,
            auto_install_deps: None,
            uses_devbox: false,
            env: HashMap::new(),
            created_at: now,
            last_accessed: now,
        }
    }

    /// Get the path to a project's config file
    pub fn config_path(name: &str) -> PathBuf {
        super::config_dir().join("projects").join(format!("{}.json", name))
    }

    /// Check if a project exists
    pub fn exists(name: &str) -> bool {
        Self::config_path(name).exists()
    }

    /// Load a project config by name
    pub fn load(name: &str) -> Result<Self> {
        let path = Self::config_path(name);

        if !path.exists() {
            return Err(DevError::ProjectNotFound(name.to_string()));
        }

        let contents = std::fs::read_to_string(&path)?;
        let config: ProjectConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Save the project config
    pub fn save(&self) -> Result<()> {
        super::ensure_config_dirs()?;
        let path = Self::config_path(&self.name);
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    /// Delete a project config
    pub fn delete(name: &str) -> Result<()> {
        let path = Self::config_path(name);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// List all registered projects
    pub fn list_all() -> Result<Vec<ProjectConfig>> {
        let projects_dir = super::config_dir().join("projects");

        if !projects_dir.exists() {
            return Ok(Vec::new());
        }

        let mut projects = Vec::new();

        for entry in std::fs::read_dir(&projects_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<ProjectConfig>(&contents) {
                        projects.push(config);
                    }
                }
            }
        }

        // Sort by last accessed (most recent first)
        projects.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

        Ok(projects)
    }

    /// Update the last accessed timestamp and save
    pub fn touch_accessed(&mut self) -> Result<()> {
        self.last_accessed = Utc::now();
        self.save()
    }

    /// Find a project by path (checks if the given path is within any project)
    pub fn find_by_path(path: &PathBuf) -> Result<Option<ProjectConfig>> {
        let projects = Self::list_all()?;

        for project in projects {
            if path.starts_with(&project.path) {
                return Ok(Some(project));
            }
        }

        Ok(None)
    }
}
