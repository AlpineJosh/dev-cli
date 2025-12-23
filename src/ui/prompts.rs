use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::error::{DevError, Result};

/// Interactive prompts using dialoguer
pub struct Prompts {
    theme: ColorfulTheme,
}

impl Default for Prompts {
    fn default() -> Self {
        Self::new()
    }
}

impl Prompts {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
        }
    }

    /// Ask for confirmation
    pub fn confirm(&self, message: &str, default: bool) -> Result<bool> {
        Confirm::with_theme(&self.theme)
            .with_prompt(message)
            .default(default)
            .interact()
            .map_err(|_| DevError::UserCancelled)
    }

    /// Ask for text input
    pub fn input(&self, message: &str, default: Option<&str>) -> Result<String> {
        let mut input = Input::<String>::with_theme(&self.theme).with_prompt(message);

        if let Some(def) = default {
            input = input.default(def.to_string());
        }

        input.interact_text().map_err(|_| DevError::UserCancelled)
    }

    /// Ask to select from a list
    pub fn select<T: ToString>(&self, message: &str, items: &[T]) -> Result<usize> {
        Select::with_theme(&self.theme)
            .with_prompt(message)
            .items(items)
            .default(0)
            .interact()
            .map_err(|_| DevError::UserCancelled)
    }
}

/// What to do when a remote branch already exists
#[derive(Debug, Clone, Copy)]
pub enum RemoteBranchAction {
    Checkout,
    CreateDivergent,
    Cancel,
}

impl std::fmt::Display for RemoteBranchAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteBranchAction::Checkout => write!(f, "Checkout existing remote branch"),
            RemoteBranchAction::CreateDivergent => {
                write!(f, "Create new local branch (will diverge)")
            }
            RemoteBranchAction::Cancel => write!(f, "Cancel"),
        }
    }
}

/// Ask what to do when a branch exists on remote
pub fn prompt_remote_branch_action(prompts: &Prompts, branch: &str) -> Result<RemoteBranchAction> {
    let items = [
        RemoteBranchAction::Checkout,
        RemoteBranchAction::CreateDivergent,
        RemoteBranchAction::Cancel,
    ];

    let selection = prompts.select(
        &format!("Branch '{}' exists on remote. What would you like to do?", branch),
        &items,
    )?;

    Ok(items[selection])
}

/// What type of project initialization
#[derive(Debug, Clone)]
pub enum InitType {
    NewRepo,
    ExistingDir(std::path::PathBuf),
    CloneUrl(String),
}

impl std::fmt::Display for InitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitType::NewRepo => write!(f, "Create new repository"),
            InitType::ExistingDir(_) => write!(f, "Initialize in existing directory"),
            InitType::CloneUrl(_) => write!(f, "Clone from URL"),
        }
    }
}
