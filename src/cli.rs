use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dev")]
#[command(about = "Git worktree and project management CLI")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,


    /// Target project or branch name
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// List all worktrees with status
    #[arg(short, long)]
    pub list: bool,

    /// Create new branch and worktree
    #[arg(short, long, value_name = "BRANCH")]
    pub create: Option<String>,

    /// Remove unused worktrees
    #[arg(long)]
    pub cleanup: bool,

    /// Generate shell completion script
    #[arg(long)]
    pub completion: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project
    Init {
        /// Project name
        name: Option<String>,

        /// Clone from remote URL
        #[arg(long)]
        clone: Option<String>,

        /// Initialize in existing directory
        #[arg(long)]
        existing: Option<PathBuf>,

        /// Skip devbox setup
        #[arg(long)]
        no_devbox: bool,
    },

    /// List registered projects
    Projects,

    /// Show/edit configuration
    Config {
        /// Set a config value (format: key=value)
        #[arg(long)]
        set: Option<String>,

        /// Get a config value
        #[arg(long)]
        get: Option<String>,
    },
}
