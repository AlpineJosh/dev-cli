use colored::Colorize;

use crate::git::worktree::WorktreeStatus;

/// Print a success message
pub fn success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

/// Print an info message
pub fn info(message: &str) {
    println!("{}", message.blue());
}

/// Print a warning message
pub fn warning(message: &str) {
    println!("{} {}", "⚠".yellow(), message.yellow());
}

/// Print an error message
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red(), message.red());
}

/// Print a gray/dim message
pub fn dim(message: &str) {
    println!("{}", message.dimmed());
}

/// Get colored status icon
pub fn status_icon(status: &WorktreeStatus) -> colored::ColoredString {
    match status {
        WorktreeStatus::Clean => "✓".green(),
        WorktreeStatus::Ahead(_) => "↑".blue(),
        WorktreeStatus::Behind(_) => "↓".yellow(),
        WorktreeStatus::Diverged { .. } => "⇅".magenta(),
        WorktreeStatus::Modified => "●".red(),
        WorktreeStatus::Unknown => "?".dimmed(),
    }
}

/// Get colored status label
pub fn status_label(status: &WorktreeStatus) -> colored::ColoredString {
    match status {
        WorktreeStatus::Clean => "clean".green(),
        WorktreeStatus::Ahead(n) => format!("ahead {}", n).blue().into(),
        WorktreeStatus::Behind(n) => format!("behind {}", n).yellow().into(),
        WorktreeStatus::Diverged { ahead, behind } => {
            format!("diverged +{} -{}", ahead, behind).magenta().into()
        }
        WorktreeStatus::Modified => "modified".red(),
        WorktreeStatus::Unknown => "unknown".dimmed(),
    }
}
