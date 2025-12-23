use std::path::PathBuf;

use colored::Colorize;

use crate::config::{GlobalConfig, ProjectConfig};
use crate::editor;
use crate::error::{DevError, Result};
use crate::git;
use crate::shell::devbox;
use crate::ui::output::{info, success};
use crate::ui::prompts::Prompts;

pub fn run(
    name: Option<String>,
    clone_url: Option<String>,
    existing_path: Option<PathBuf>,
    no_devbox: bool,
) -> Result<()> {
    let prompts = Prompts::new();
    let config = GlobalConfig::load()?;

    // Determine project name
    let project_name = match &name {
        Some(n) => n.clone(),
        None => prompts.input("Project name", None)?,
    };

    if project_name.trim().is_empty() {
        return Err(DevError::Other("Project name cannot be empty".to_string()));
    }

    // Check if project already exists in registry
    if ProjectConfig::exists(&project_name) {
        return Err(DevError::ProjectExists(project_name));
    }

    // Determine the project path and how to initialize
    let project_path = if let Some(url) = clone_url {
        // Clone from URL
        init_from_clone(&project_name, &url, &config)?
    } else if let Some(path) = existing_path {
        // Initialize in existing directory
        init_from_existing(&project_name, &path)?
    } else {
        // Create new repository
        init_new_repo(&project_name, &config)?
    };

    // Set up devbox if requested
    let uses_devbox = if no_devbox {
        false
    } else {
        let setup_devbox = prompts.confirm("Set up devbox for this project?", true)?;
        if setup_devbox {
            info("Creating devbox.json...");
            devbox::init_devbox(&project_path, &[])?;
            success("devbox.json created");
            true
        } else {
            false
        }
    };

    // Register the project
    let mut project = ProjectConfig::new(&project_name, project_path.clone());
    project.uses_devbox = uses_devbox;
    project.save()?;

    success(&format!("Project '{}' registered", project_name));

    // Open in editor
    info("Opening in editor...");
    editor::open(&project_path, &config)?;

    println!();
    success(&format!("Project '{}' is ready!", project_name));
    println!("  {}: {}", "Path".dimmed(), project_path.display());

    if uses_devbox {
        println!();
        devbox::print_devbox_instructions(&project_path);
    }

    Ok(())
}

fn init_new_repo(name: &str, config: &GlobalConfig) -> Result<PathBuf> {
    let project_path = config.dev_path.join(name);

    if project_path.exists() {
        return Err(DevError::Other(format!(
            "Directory already exists: {}",
            project_path.display()
        )));
    }

    info(&format!("Creating new repository at {}...", project_path.display()));

    // Create directory
    std::fs::create_dir_all(&project_path)?;

    // Initialize git repository
    git::init_repository(&project_path)?;

    // Create initial README
    let readme_path = project_path.join("README.md");
    std::fs::write(&readme_path, format!("# {}\n", name))?;

    // Make initial commit
    git::run_git_command(&["add", "."], Some(&project_path))?;
    git::run_git_command(&["commit", "-m", "Initial commit"], Some(&project_path))?;

    success("Repository initialized");

    Ok(project_path)
}

fn init_from_clone(name: &str, url: &str, config: &GlobalConfig) -> Result<PathBuf> {
    let project_path = config.dev_path.join(name);

    if project_path.exists() {
        return Err(DevError::Other(format!(
            "Directory already exists: {}",
            project_path.display()
        )));
    }

    info(&format!("Cloning {} to {}...", url, project_path.display()));

    git::clone_repository(url, &project_path)?;

    success("Repository cloned");

    Ok(project_path)
}

fn init_from_existing(_name: &str, path: &PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        return Err(DevError::Other(format!(
            "Directory does not exist: {}",
            path.display()
        )));
    }

    let abs_path = path.canonicalize()?;

    // Check if it's a git repository
    if !git::is_git_repository(&abs_path) {
        info("Initializing git repository...");
        git::init_repository(&abs_path)?;
        success("Repository initialized");
    }

    info(&format!("Registering existing project at {}...", abs_path.display()));

    Ok(abs_path)
}
