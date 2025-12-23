pub mod branch;
pub mod status;
pub mod worktree;

pub use branch::*;
pub use status::*;
pub use worktree::*;

use std::path::Path;
use std::process::Command;

use crate::error::{DevError, Result};

/// Check if a directory is inside a git repository
pub fn is_git_repository(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the root directory of the git repository
pub fn get_repository_root(path: &Path) -> Result<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        return Err(DevError::NotGitRepository);
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(std::path::PathBuf::from(root))
}

/// Initialize a new git repository
pub fn init_repository(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevError::GitError(format!("Failed to init repository: {}", stderr)));
    }

    Ok(())
}

/// Clone a repository from a URL
pub fn clone_repository(url: &str, path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["clone", url, path.to_str().unwrap_or(".")])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevError::GitError(format!("Failed to clone repository: {}", stderr)));
    }

    Ok(())
}

/// Run a git command and return the output
pub fn run_git_command(args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevError::GitError(stderr.trim().to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
