use crate::error::Result;
use crate::shell::completion::generate_zsh_completion;

pub fn run() -> Result<()> {
    // For now, just generate zsh completion with our custom script
    // that includes dynamic project/branch completion
    generate_zsh_completion();
    Ok(())
}
