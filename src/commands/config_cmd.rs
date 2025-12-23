use colored::Colorize;

use crate::config::GlobalConfig;
use crate::error::{DevError, Result};

pub fn run(set: Option<String>, get: Option<String>) -> Result<()> {
    let mut config = GlobalConfig::load()?;

    if let Some(key_value) = set {
        // Set a config value
        let parts: Vec<&str> = key_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(DevError::ConfigError(
                "Invalid format. Use: --set key=value".to_string(),
            ));
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        config.set(key, value)?;
        println!("{} {} = {}", "✓".green(), key, value);
    } else if let Some(key) = get {
        // Get a config value
        match config.get(&key) {
            Some(value) => println!("{}", value),
            None => {
                return Err(DevError::ConfigError(format!("Unknown config key: {}", key)));
            }
        }
    } else {
        // Show all config
        println!("{}", "\n⚙️  Configuration:\n".bold());
        println!("  {}: {}", "editor".cyan(), config.editor);
        println!("  {}: {}", "dev_path".cyan(), config.dev_path.display());
        println!(
            "  {}: {}",
            "auto_install_deps".cyan(),
            config.auto_install_deps
        );
        println!("  {}: {}", "auto_devbox".cyan(), config.auto_devbox);
        println!(
            "  {}: {:?}",
            "shell".cyan(),
            config.shell
        );

        println!();
        println!("{}", "Config file:".dimmed());
        println!("  {}", GlobalConfig::config_path().display());
    }

    Ok(())
}
