use colored::Colorize;

use crate::editor;
use crate::error::{DevError, Result};
use crate::git::{
    branch_exists_locally, branch_exists_on_remote, create_worktree, is_git_repository,
};
use crate::package_manager;
use crate::ui::output::{info, success};
use crate::ui::prompts::{prompt_remote_branch_action, Prompts, RemoteBranchAction};

pub fn run(branch: &str) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !is_git_repository(&current_dir) {
        return Err(DevError::NotGitRepository);
    }

    let branch = branch.trim();
    if branch.is_empty() {
        return Err(DevError::Other("Branch name is required".to_string()));
    }

    // Check if branch already exists locally
    if branch_exists_locally(branch) {
        return Err(DevError::BranchExistsLocally(branch.to_string()));
    }

    let mut create_new_branch = true;

    // Check if branch exists on remote
    if branch_exists_on_remote(branch) {
        let prompts = Prompts::new();
        match prompt_remote_branch_action(&prompts, branch)? {
            RemoteBranchAction::Checkout => {
                create_new_branch = false;
            }
            RemoteBranchAction::CreateDivergent => {
                create_new_branch = true;
            }
            RemoteBranchAction::Cancel => {
                println!("{}", "Operation cancelled".yellow());
                return Ok(());
            }
        }
    }

    info(&format!("Creating worktree for branch '{}'...", branch));

    // Create the worktree
    let worktree_path = create_worktree(branch, create_new_branch)?;

    success(&format!("Worktree created at: {}", worktree_path.display()));

    // Install dependencies if needed
    info("Checking for dependencies...");
    if package_manager::install_dependencies(&worktree_path)? {
        success("Dependencies installed");
    }

    // Open in editor
    info("Opening in editor...");
    let config = crate::config::GlobalConfig::load()?;
    editor::open(&worktree_path, &config)?;

    println!();
    success(&format!("Ready to work on '{}'!", branch));
    println!("   {}: {}", "Path".dimmed(), worktree_path.display());

    Ok(())
}
