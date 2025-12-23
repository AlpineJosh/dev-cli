use colored::Colorize;

use crate::error::{DevError, Result};
use crate::git::{is_git_repository, list_worktrees};
use crate::ui::output::{status_icon, status_label};

pub fn run() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !is_git_repository(&current_dir) {
        return Err(DevError::NotGitRepository);
    }

    let worktrees = list_worktrees()?;

    if worktrees.is_empty() {
        println!("{}", "No worktrees found".yellow());
        return Ok(());
    }

    println!("{}", "\n📁 Git Worktrees:\n".bold());

    // Calculate column widths for alignment
    let max_branch_len = worktrees
        .iter()
        .map(|wt| wt.branch.as_ref().map_or(8, |b| b.len()))
        .max()
        .unwrap_or(8);

    let max_path_len = worktrees
        .iter()
        .map(|wt| wt.path.display().to_string().len())
        .max()
        .unwrap_or(20);

    for worktree in &worktrees {
        let current_marker = if worktree.is_current {
            "* ".green()
        } else {
            "  ".normal()
        };

        let branch_name = worktree
            .branch
            .as_ref()
            .map(|b| b.as_str())
            .unwrap_or(if worktree.is_detached {
                "detached"
            } else if worktree.is_bare {
                "bare"
            } else {
                "unknown"
            });

        let branch_display = if worktree.is_current {
            format!("{:width$}", branch_name, width = max_branch_len)
                .green()
                .bold()
        } else {
            format!("{:width$}", branch_name, width = max_branch_len).cyan()
        };

        let path_display = format!(
            "{:width$}",
            worktree.path.display(),
            width = max_path_len
        )
        .dimmed();

        let status_icon = status_icon(&worktree.status);
        let status_label = status_label(&worktree.status);

        let short_commit = if worktree.commit.len() >= 7 {
            &worktree.commit[..7]
        } else {
            &worktree.commit
        }
        .dimmed();

        println!(
            "{}{} {} {} {} {}",
            current_marker, branch_display, path_display, status_icon, status_label, short_commit
        );
    }

    println!();

    // Show legend
    println!("{}", "Legend:".dimmed());
    println!("{}", "  * = current worktree".dimmed());
    println!(
        "{}",
        "  ✓ = clean  ↑ = ahead  ↓ = behind  ⇅ = diverged  ● = modified".dimmed()
    );

    Ok(())
}
