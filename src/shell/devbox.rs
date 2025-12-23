use std::path::Path;
use std::process::Command;

use crate::error::{DevError, Result};

/// Check if devbox is installed
pub fn is_devbox_installed() -> bool {
    Command::new("which")
        .arg("devbox")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a directory has a devbox.json
pub fn has_devbox_config(path: &Path) -> bool {
    path.join("devbox.json").exists()
}

/// Check if we're currently in a devbox shell
pub fn in_devbox_shell() -> bool {
    std::env::var("DEVBOX_SHELL_ENABLED").is_ok()
}

/// Create a basic devbox.json at the given path
pub fn init_devbox(path: &Path, packages: &[&str]) -> Result<()> {
    let devbox_json = path.join("devbox.json");

    if devbox_json.exists() {
        return Ok(());
    }

    let packages_json: Vec<String> = packages.iter().map(|p| format!("\"{}\"", p)).collect();
    let packages_str = packages_json.join(",\n    ");

    let content = format!(
        r#"{{
  "$schema": "https://raw.githubusercontent.com/jetify-com/devbox/0.13.0/.schema/devbox.schema.json",
  "packages": [
    {}
  ],
  "shell": {{
    "init_hook": [
      "echo 'Welcome to devbox!'"
    ],
    "scripts": {{
      "test": "echo \"No tests configured\""
    }}
  }}
}}"#,
        packages_str
    );

    std::fs::write(&devbox_json, content)?;
    Ok(())
}

/// Get the command to launch devbox shell
pub fn get_devbox_shell_command(path: &Path) -> String {
    format!("cd {} && devbox shell", path.display())
}

/// Print instructions for entering devbox shell
pub fn print_devbox_instructions(path: &Path) {
    use colored::Colorize;

    if in_devbox_shell() {
        println!(
            "{}",
            "Already in devbox shell. Navigate to the project:".yellow()
        );
        println!("  cd {}", path.display());
    } else {
        println!("{}", "Enter devbox shell:".blue());
        println!("  {}", get_devbox_shell_command(path));
    }
}

/// Launch devbox shell (replaces current process on Unix)
#[cfg(unix)]
pub fn exec_devbox_shell(path: &Path) -> Result<()> {
    use std::os::unix::process::CommandExt;

    if in_devbox_shell() {
        return Err(DevError::DevboxError(
            "Already in a devbox shell. Cannot nest devbox shells.".to_string(),
        ));
    }

    if !has_devbox_config(path) {
        return Err(DevError::DevboxError(format!(
            "No devbox.json found at {}",
            path.display()
        )));
    }

    let err = Command::new("devbox")
        .arg("shell")
        .current_dir(path)
        .exec();

    // exec() only returns if there was an error
    Err(DevError::DevboxError(format!(
        "Failed to launch devbox shell: {}",
        err
    )))
}

#[cfg(not(unix))]
pub fn exec_devbox_shell(path: &Path) -> Result<()> {
    // On non-Unix systems, just print instructions
    print_devbox_instructions(path);
    Ok(())
}
