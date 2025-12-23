use colored::Colorize;

use crate::config::{GlobalConfig, ProjectConfig};
use crate::editor;
use crate::error::{DevError, Result};
use crate::git::list_worktrees;
use crate::shell::devbox;
use crate::ui::output::{info, success};

/// Open a project by name
pub fn run(project_name: &str) -> Result<()> {
    let config = GlobalConfig::load()?;

    // Look up project in registry
    let mut project = match ProjectConfig::load(project_name) {
        Ok(p) => p,
        Err(DevError::ProjectNotFound(_)) => {
            // Project not found - show available projects
            eprintln!(
                "{}",
                format!("Project '{}' not found", project_name).red()
            );
            println!();
            println!("{}", "Registered projects:".yellow());

            let projects = ProjectConfig::list_all()?;
            if projects.is_empty() {
                println!("  {}", "(no projects registered)".dimmed());
                println!();
                println!(
                    "{}",
                    "Use 'dev init <name>' to create a new project".dimmed()
                );
            } else {
                for p in &projects {
                    println!("  {} - {}", p.name.cyan(), p.path.display().to_string().dimmed());
                }
            }

            return Err(DevError::ProjectNotFound(project_name.to_string()));
        }
        Err(e) => return Err(e),
    };

    // Update last accessed
    project.touch_accessed()?;

    // Check if project path exists
    if !project.path.exists() {
        return Err(DevError::Other(format!(
            "Project directory does not exist: {}",
            project.path.display()
        )));
    }

    info(&format!("Opening project '{}'...", project_name));

    // Find the best worktree to open (main/master/default, or project root)
    let target_path = find_main_worktree(&project)?;

    // Open in editor
    editor::open(&target_path, &config)?;

    success(&format!("Opened project '{}'", project_name));
    println!("  {}: {}", "Path".dimmed(), target_path.display());

    // Handle devbox shell
    if project.uses_devbox && config.auto_devbox {
        println!();
        if devbox::has_devbox_config(&target_path) {
            devbox::print_devbox_instructions(&target_path);
        }
    }

    Ok(())
}

/// List all registered projects
pub fn list_projects() -> Result<()> {
    let projects = ProjectConfig::list_all()?;

    if projects.is_empty() {
        println!("{}", "No projects registered".yellow());
        println!();
        println!(
            "{}",
            "Use 'dev init <name>' to create a new project".dimmed()
        );
        return Ok(());
    }

    println!("{}", "\n📁 Registered Projects:\n".bold());

    for project in &projects {
        let name = project.name.cyan();
        let path = project.path.display().to_string().dimmed();
        let devbox_indicator = if project.uses_devbox {
            " 📦".to_string()
        } else {
            String::new()
        };

        println!("  {}{}", name, devbox_indicator);
        println!("    {}", path);
    }

    println!();
    println!("{}", "Legend: 📦 = uses devbox".dimmed());

    Ok(())
}

/// Find the main worktree to open (main, master, or project root)
fn find_main_worktree(project: &ProjectConfig) -> Result<std::path::PathBuf> {
    // Try to get worktrees if this is a git repo
    if let Ok(worktrees) = list_worktrees() {
        // Prefer main, then master, then first non-bare worktree
        for preferred in &["main", "master"] {
            if let Some(wt) = worktrees.iter().find(|wt| {
                wt.branch.as_deref() == Some(*preferred)
            }) {
                if wt.path.exists() {
                    return Ok(wt.path.clone());
                }
            }
        }

        // Fall back to first non-bare worktree
        if let Some(wt) = worktrees.iter().find(|wt| !wt.is_bare) {
            if wt.path.exists() {
                return Ok(wt.path.clone());
            }
        }
    }

    // Fall back to project root
    Ok(project.path.clone())
}
