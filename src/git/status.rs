use std::path::Path;
use std::process::Command;

use super::worktree::WorktreeStatus;

/// Get the status of a worktree (clean, ahead, behind, diverged, modified)
pub fn get_worktree_status(worktree_path: &Path, branch: &str) -> WorktreeStatus {
    // First check for uncommitted changes
    if has_uncommitted_changes(worktree_path) {
        return WorktreeStatus::Modified;
    }

    // Then check ahead/behind status
    get_ahead_behind_status(worktree_path, branch)
}

/// Check if the worktree has uncommitted changes
fn has_uncommitted_changes(worktree_path: &Path) -> bool {
    Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()
        .map(|output| {
            output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty()
        })
        .unwrap_or(false)
}

/// Get the ahead/behind status relative to the remote
fn get_ahead_behind_status(worktree_path: &Path, branch: &str) -> WorktreeStatus {
    let output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("origin/{}...HEAD", branch),
        ])
        .current_dir(worktree_path)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.trim().split('\t').collect();

            if parts.len() == 2 {
                let behind: usize = parts[0].parse().unwrap_or(0);
                let ahead: usize = parts[1].parse().unwrap_or(0);

                match (ahead, behind) {
                    (0, 0) => WorktreeStatus::Clean,
                    (a, 0) if a > 0 => WorktreeStatus::Ahead(a),
                    (0, b) if b > 0 => WorktreeStatus::Behind(b),
                    (a, b) => WorktreeStatus::Diverged { ahead: a, behind: b },
                }
            } else {
                WorktreeStatus::Clean
            }
        }
        _ => {
            // If we can't get ahead/behind (e.g., no remote tracking), assume clean
            WorktreeStatus::Clean
        }
    }
}
