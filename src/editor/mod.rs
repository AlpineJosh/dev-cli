mod vscode;
mod zed;

use std::path::Path;
use std::process::Command;

use crate::config::GlobalConfig;
use crate::error::{DevError, Result};

/// Trait for editor implementations
pub trait Editor: Send + Sync {
    /// Name of the editor for display
    fn name(&self) -> &str;

    /// Command to invoke the editor
    fn command(&self) -> &str;

    /// Check if editor is installed
    fn is_installed(&self) -> bool {
        Command::new("which")
            .arg(self.command())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Open a path in the editor
    fn open(&self, path: &Path) -> Result<()> {
        if !self.is_installed() {
            return Err(DevError::EditorNotFound(self.name().to_string()));
        }

        Command::new(self.command())
            .arg(path)
            .spawn()
            .map_err(|e| DevError::Other(format!("Failed to open editor: {}", e)))?;

        Ok(())
    }
}

/// Get an editor by name
pub fn get_editor(name: &str) -> Box<dyn Editor> {
    match name.to_lowercase().as_str() {
        "zed" => Box::new(zed::ZedEditor),
        "code" | "vscode" => Box::new(vscode::VSCodeEditor),
        _ => Box::new(GenericEditor(name.to_string())),
    }
}

/// Open a path with the configured editor
pub fn open(path: &Path, config: &GlobalConfig) -> Result<()> {
    let editor = get_editor(&config.editor);
    editor.open(path)
}

/// Generic editor that just runs the command
struct GenericEditor(String);

impl Editor for GenericEditor {
    fn name(&self) -> &str {
        &self.0
    }

    fn command(&self) -> &str {
        &self.0
    }
}
