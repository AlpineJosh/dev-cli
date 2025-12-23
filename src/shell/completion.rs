use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::cli::Cli;
use crate::config::global::Shell as ConfigShell;
use crate::error::Result;

/// Generate shell completion script
pub fn generate_completion(shell: ConfigShell) -> Result<()> {
    let mut cmd = Cli::command();
    let shell = match shell {
        ConfigShell::Zsh => Shell::Zsh,
        ConfigShell::Bash => Shell::Bash,
        ConfigShell::Fish => Shell::Fish,
    };

    generate(shell, &mut cmd, "dev", &mut std::io::stdout());

    Ok(())
}

/// Generate zsh completion with custom dynamic completions
pub fn generate_zsh_completion() {
    let completion_script = r#"#compdef dev

_dev_completion() {
  local context state line
  typeset -A opt_args

  _arguments -C \
    '(-l --list)'{-l,--list}'[List all worktrees with status]' \
    '(-c --create)'{-c,--create}'[Create new branch and worktree]:branch name:' \
    '--cleanup[Remove unused worktrees]' \
    '--completion[Generate shell completion script]' \
    '*::arg:->args' && return

  case $state in
    args)
      # Check if we're in a project context
      if git rev-parse --git-dir &>/dev/null; then
        # We're in a git repo - complete with branch names
        local -a branches
        branches=($(git branch -a --format="%(refname:short)" 2>/dev/null | sed 's|origin/||g' | sort -u))
        _describe 'branches' branches
      else
        # We're in global context - complete with project names
        local -a projects
        local config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/dev/projects"
        if [[ -d "$config_dir" ]]; then
          projects=($(ls -1 "$config_dir" 2>/dev/null | sed 's/\.json$//'))
          _describe 'projects' projects
        fi
      fi
      ;;
  esac
}

compdef _dev_completion dev"#;

    println!("{}", completion_script);
    println!();
    println!("# To install this completion script:");
    println!("# 1. Save the output to a file: dev --completion > ~/.zsh_completions/_dev");
    println!("# 2. Add this line to your ~/.zshrc: fpath=(~/.zsh_completions $fpath)");
    println!("# 3. Add this line to your ~/.zshrc: autoload -U compinit && compinit");
    println!("# 4. Restart your shell or run: source ~/.zshrc");
}
