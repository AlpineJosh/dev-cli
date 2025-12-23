use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevError {
    #[error("Not in a git repository")]
    NotGitRepository,

    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("Branch '{0}' already exists locally")]
    BranchExistsLocally(String),

    #[error("Branch '{0}' not found")]
    BranchNotFound(String),

    #[error("Worktree not found for branch '{0}'")]
    WorktreeNotFound(String),

    #[error("Worktree directory already exists: {0}")]
    WorktreeDirectoryExists(String),

    #[error("Worktree path does not exist: {0}")]
    WorktreePathMissing(String),

    #[error("Project '{0}' not found")]
    ProjectNotFound(String),

    #[error("Project '{0}' already exists")]
    ProjectExists(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Editor '{0}' not found. Is it installed?")]
    EditorNotFound(String),

    #[error("Devbox error: {0}")]
    DevboxError(String),

    #[error("Operation cancelled")]
    UserCancelled,

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DevError>;
