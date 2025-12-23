use colored::Colorize;

use crate::editor;
use crate::error::{DevError, Result};
use crate::git::{find_worktree_by_branch, is_git_repository, list_worktrees};
use crate::package_manager;
use crate::ui::output::{info, status_label, success};

pub fn run(branch: &str) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !is_git_repository(&current_dir) {
        return Err(DevError::NotGitRepository);
    }

    let branch = branch.trim();
    if branch.is_empty() {
        return Err(DevError::Other("Branch name is required".to_string()));
    }

    // Find the worktree for the specified branch
    let worktree = find_worktree_by_branch(branch)?;

    let worktree = match worktree {
        Some(wt) => wt,
        None => {
            eprintln!("{}", format!("No worktree found for branch '{}'", branch).red());
            println!("{}", "Available branches:".yellow());

            let worktrees = list_worktrees()?;
            for wt in &worktrees {
                if let Some(ref b) = wt.branch {
                    if !wt.is_bare && !wt.is_detached {
                        let current = if wt.is_current {
                            "* ".green()
                        } else {
                            "  ".normal()
                        };
                        println!("{}{}", current, b);
                    }
                }
            }

            println!();
            println!(
                "{}",
                format!("To create a new worktree: dev --create {}", branch).dimmed()
            );

            return Err(DevError::WorktreeNotFound(branch.to_string()));
        }
    };

    // Check if already on this branch
    if worktree.is_current {
        println!("{}", format!("Already on branch '{}'", branch).yellow());
        return Ok(());
    }

    // Check if worktree path exists
    if !worktree.path.exists() {
        return Err(DevError::WorktreePathMissing(
            worktree.path.display().to_string(),
        ));
    }

    info(&format!("Switching to branch '{}'...", branch));

    // Check if dependencies need to be installed
    if !package_manager::has_node_modules(&worktree.path) {
        info("Dependencies not found, installing...");
        package_manager::install_dependencies(&worktree.path)?;
    }

    // Open in editor
    info("Opening in editor...");
    let config = crate::config::GlobalConfig::load()?;
    editor::open(&worktree.path, &config)?;

    println!();
    success(&format!("Switched to '{}'", branch));
    println!("  {}: {}", "Path".dimmed(), worktree.path.display());
    println!(
        "  {}: {}",
        "Commit".dimmed(),
        &worktree.commit[..7.min(worktree.commit.len())]
    );

    // Show status if not clean
    if worktree.status != crate::git::WorktreeStatus::Clean {
        println!("  {}: {}", "Status".dimmed(), status_label(&worktree.status));
    }

    Ok(())
}
