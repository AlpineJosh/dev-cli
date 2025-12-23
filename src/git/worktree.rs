use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{DevError, Result};

use super::status::get_worktree_status;

/// Information about a git worktree
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub is_current: bool,
    pub is_bare: bool,
    pub is_detached: bool,
    pub status: WorktreeStatus,
}

/// Status of a worktree relative to its remote
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorktreeStatus {
    Clean,
    Ahead(usize),
    Behind(usize),
    Diverged { ahead: usize, behind: usize },
    Modified,
    Unknown,
}

impl WorktreeStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            WorktreeStatus::Clean => "✓",
            WorktreeStatus::Ahead(_) => "↑",
            WorktreeStatus::Behind(_) => "↓",
            WorktreeStatus::Diverged { .. } => "⇅",
            WorktreeStatus::Modified => "●",
            WorktreeStatus::Unknown => "?",
        }
    }

    pub fn label(&self) -> String {
        match self {
            WorktreeStatus::Clean => "clean".to_string(),
            WorktreeStatus::Ahead(n) => format!("ahead {}", n),
            WorktreeStatus::Behind(n) => format!("behind {}", n),
            WorktreeStatus::Diverged { ahead, behind } => {
                format!("diverged +{} -{}", ahead, behind)
            }
            WorktreeStatus::Modified => "modified".to_string(),
            WorktreeStatus::Unknown => "unknown".to_string(),
        }
    }
}

/// List all worktrees in the repository
pub fn list_worktrees() -> Result<Vec<WorktreeInfo>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .output()?;

    if !output.status.success() {
        return Err(DevError::NotGitRepository);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let current_dir = std::env::current_dir().ok();

    parse_worktree_list(&stdout, current_dir.as_deref())
}

/// Parse the porcelain output of `git worktree list`
fn parse_worktree_list(output: &str, current_dir: Option<&Path>) -> Result<Vec<WorktreeInfo>> {
    let mut worktrees = Vec::new();
    let mut current: Option<WorktreeInfoBuilder> = None;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if exists
            if let Some(builder) = current.take() {
                if let Some(wt) = builder.build(current_dir) {
                    worktrees.push(wt);
                }
            }
            current = Some(WorktreeInfoBuilder {
                path: Some(PathBuf::from(line.strip_prefix("worktree ").unwrap())),
                ..Default::default()
            });
        } else if let Some(ref mut builder) = current {
            if line.starts_with("HEAD ") {
                builder.commit = Some(line.strip_prefix("HEAD ").unwrap().to_string());
            } else if line.starts_with("branch ") {
                let branch = line
                    .strip_prefix("branch refs/heads/")
                    .unwrap_or(line.strip_prefix("branch ").unwrap_or(""))
                    .to_string();
                builder.branch = Some(branch);
            } else if line == "bare" {
                builder.is_bare = true;
            } else if line == "detached" {
                builder.is_detached = true;
            }
        }
    }

    // Don't forget the last worktree
    if let Some(builder) = current {
        if let Some(wt) = builder.build(current_dir) {
            worktrees.push(wt);
        }
    }

    Ok(worktrees)
}

#[derive(Default)]
struct WorktreeInfoBuilder {
    path: Option<PathBuf>,
    branch: Option<String>,
    commit: Option<String>,
    is_bare: bool,
    is_detached: bool,
}

impl WorktreeInfoBuilder {
    fn build(self, current_dir: Option<&Path>) -> Option<WorktreeInfo> {
        let path = self.path?;
        let commit = self.commit.unwrap_or_default();

        let is_current = current_dir
            .map(|cd| cd.starts_with(&path))
            .unwrap_or(false);

        // Get status for non-bare worktrees
        let status = if self.is_bare || self.is_detached {
            WorktreeStatus::Unknown
        } else if let Some(ref branch) = self.branch {
            get_worktree_status(&path, branch)
        } else {
            WorktreeStatus::Unknown
        };

        Some(WorktreeInfo {
            path,
            branch: self.branch,
            commit,
            is_current,
            is_bare: self.is_bare,
            is_detached: self.is_detached,
            status,
        })
    }
}

/// Create a new worktree for a branch
pub fn create_worktree(branch: &str, create_branch: bool) -> Result<PathBuf> {
    let repo_root = super::get_repository_root(&std::env::current_dir()?)?;

    // Worktree goes in parent directory with branch name
    let worktree_path = repo_root
        .parent()
        .ok_or_else(|| DevError::GitError("Cannot determine parent directory".to_string()))?
        .join(sanitize_branch_name(branch));

    if worktree_path.exists() {
        return Err(DevError::WorktreeDirectoryExists(
            worktree_path.display().to_string(),
        ));
    }

    let mut args = vec!["worktree", "add"];
    if create_branch {
        args.push("-b");
    }
    args.push(worktree_path.to_str().unwrap());
    args.push(branch);

    let output = Command::new("git")
        .args(&args)
        .current_dir(&repo_root)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevError::GitError(format!(
            "Failed to create worktree: {}",
            stderr.trim()
        )));
    }

    Ok(worktree_path)
}

/// Remove a worktree
pub fn remove_worktree(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "remove", path.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevError::GitError(format!(
            "Failed to remove worktree: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

/// Prune stale worktree references
pub fn prune_worktrees() -> Result<()> {
    let output = Command::new("git")
        .args(["worktree", "prune"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevError::GitError(format!(
            "Failed to prune worktrees: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

/// Find a worktree by branch name
pub fn find_worktree_by_branch(branch: &str) -> Result<Option<WorktreeInfo>> {
    let worktrees = list_worktrees()?;
    Ok(worktrees
        .into_iter()
        .find(|wt| wt.branch.as_deref() == Some(branch)))
}

/// Sanitize a branch name for use as a directory name
fn sanitize_branch_name(branch: &str) -> String {
    branch.replace('/', "-")
}
