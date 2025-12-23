use colored::Colorize;

use crate::error::{DevError, Result};
use crate::git::{is_git_repository, list_worktrees, prune_worktrees, remove_worktree};
use crate::ui::output::{info, success};
use crate::ui::prompts::Prompts;

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !is_git_repository(&current_dir) {
        return Err(DevError::NotGitRepository);
    }

    let worktrees = list_worktrees()?;

    // Find problematic worktrees
    let problematic: Vec<_> = worktrees
        .iter()
        .filter(|wt| !wt.path.exists() || wt.is_detached)
        .collect();

    if problematic.is_empty() {
        success("All worktrees are in good condition");
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} problematic worktree(s):\n", problematic.len()).yellow()
    );

    for wt in &problematic {
        let reason = if !wt.path.exists() {
            "directory missing".red()
        } else {
            "detached HEAD".yellow()
        };

        let branch_name = wt.branch.as_deref().unwrap_or("unknown");
        println!(
            "  {} - {} ({})",
            branch_name.cyan(),
            wt.path.display(),
            reason
        );
    }

    println!();

    let prompts = Prompts::new();
    let should_clean = prompts.confirm("Remove these problematic worktrees?", false)?;

    if !should_clean {
        println!("{}", "Cleanup cancelled".yellow());
        return Ok(());
    }

    info("Cleaning up worktrees...");

    let mut cleaned = 0;
    let mut failed = 0;

    for wt in &problematic {
        let branch_name = wt.branch.as_deref().unwrap_or("unknown");

        match remove_worktree(&wt.path) {
            Ok(()) => {
                println!(
                    "{} Removed: {} ({})",
                    "✓".green(),
                    branch_name,
                    wt.path.display()
                );
                cleaned += 1;
            }
            Err(e) => {
                println!(
                    "{} Failed to remove: {} - {}",
                    "✗".red(),
                    branch_name,
                    e
                );
                failed += 1;
            }
        }
    }

    println!();
    success(&format!("Cleanup complete: {} removed", cleaned));

    if failed > 0 {
        println!("{}", format!("✗ Failed to remove: {}", failed).red());
    }

    // Run git worktree prune
    info("Running git worktree prune...");
    match prune_worktrees() {
        Ok(()) => success("Pruned worktree references"),
        Err(e) => println!("{}", format!("Warning: git worktree prune failed: {}", e).yellow()),
    }

    Ok(())
}
