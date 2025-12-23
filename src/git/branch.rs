use std::process::Command;

use crate::error::{DevError, Result};

/// List all branches (local and optionally remote)
pub fn list_branches(include_remote: bool) -> Result<Vec<String>> {
    let flag = if include_remote { "-a" } else { "" };

    let mut args = vec!["branch", "--format=%(refname:short)"];
    if include_remote {
        args = vec!["branch", "-a", "--format=%(refname:short)"];
    }

    let output = Command::new("git").args(&args).output()?;

    if !output.status.success() {
        return Err(DevError::NotGitRepository);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|branch| !branch.is_empty() && !branch.starts_with("origin/HEAD"))
        .map(|branch| branch.replace("origin/", ""))
        .collect();

    // Remove duplicates (local and remote versions of the same branch)
    let mut unique_branches: Vec<String> = branches.clone();
    unique_branches.sort();
    unique_branches.dedup();

    Ok(unique_branches)
}

/// Check if a branch exists (locally or on remote)
pub fn branch_exists(name: &str, include_remote: bool) -> bool {
    list_branches(include_remote)
        .map(|branches| branches.contains(&name.to_string()))
        .unwrap_or(false)
}

/// Check if a branch exists locally
pub fn branch_exists_locally(name: &str) -> bool {
    let output = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", name)])
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Check if a branch exists on remote
pub fn branch_exists_on_remote(name: &str) -> bool {
    let output = Command::new("git")
        .args(["show-ref", "--verify", "--quiet", &format!("refs/remotes/origin/{}", name)])
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Get the current branch name
pub fn get_current_branch() -> Result<Option<String>> {
    let output = Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(branch))
    } else {
        // Might be in detached HEAD state
        Ok(None)
    }
}

/// Create a new branch
pub fn create_branch(name: &str, start_point: Option<&str>) -> Result<()> {
    let mut args = vec!["branch", name];
    if let Some(sp) = start_point {
        args.push(sp);
    }

    let output = Command::new("git").args(&args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevError::GitError(format!(
            "Failed to create branch: {}",
            stderr.trim()
        )));
    }

    Ok(())
}
