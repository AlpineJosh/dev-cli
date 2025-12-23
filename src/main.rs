mod cli;
mod commands;
mod config;
mod editor;
mod error;
mod git;
mod package_manager;
mod shell;
mod ui;

use clap::Parser;
use colored::Colorize;

use crate::cli::{Cli, Commands};
use crate::error::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Handle flags first
    if cli.list {
        return commands::list::run();
    }

    if let Some(branch) = cli.create {
        return commands::create::run(&branch);
    }

    if cli.cleanup {
        return commands::cleanup::run();
    }

    if cli.completion {
        return commands::completion::run();
    }

    // Handle subcommands
    if let Some(command) = cli.command {
        return match command {
            Commands::Init {
                name,
                clone,
                existing,
                no_devbox,
            } => commands::init::run(name, clone, existing, no_devbox),
            Commands::Projects => commands::project::list_projects(),
            Commands::Config { set, get } => commands::config_cmd::run(set, get),
        };
    }

    // Handle positional target argument
    if let Some(target) = cli.target {
        return handle_target(&target);
    }

    // No arguments - show help
    Cli::parse_from(["dev", "--help"]);
    Ok(())
}

fn handle_target(target: &str) -> Result<()> {
    // Detect context: are we in a project or global?
    let context = detect_context();

    match context {
        Context::Project(_) => {
            // In a project - treat target as branch name
            commands::switch::run(target)
        }
        Context::GitRepo => {
            // In a git repo but not a registered project - treat as branch
            commands::switch::run(target)
        }
        Context::Global => {
            // Global context - treat target as project name
            commands::project::run(target)
        }
    }
}

#[derive(Debug)]
enum Context {
    /// In a registered project
    Project(String),
    /// In a git repo but not a registered project
    GitRepo,
    /// Not in any project or git repo
    Global,
}

fn detect_context() -> Context {
    let current_dir = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return Context::Global,
    };

    // Check if we're in a registered project
    if let Ok(projects) = config::project::ProjectConfig::list_all() {
        for project in projects {
            if current_dir.starts_with(&project.path) {
                return Context::Project(project.name);
            }
        }
    }

    // Check if we're in a git repository
    if git::is_git_repository(&current_dir) {
        return Context::GitRepo;
    }

    Context::Global
}
