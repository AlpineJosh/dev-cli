use std::path::Path;
use std::process::Command;

use colored::Colorize;

use crate::error::Result;

/// Detected package manager type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
}

impl PackageManager {
    pub fn install_command(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm install",
            PackageManager::Yarn => "yarn install",
            PackageManager::Pnpm => "pnpm install",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
        }
    }
}

/// Detect the package manager used in a project
pub fn detect_package_manager(path: &Path) -> Option<PackageManager> {
    if path.join("yarn.lock").exists() {
        Some(PackageManager::Yarn)
    } else if path.join("pnpm-lock.yaml").exists() {
        Some(PackageManager::Pnpm)
    } else if path.join("package-lock.json").exists() {
        Some(PackageManager::Npm)
    } else if path.join("package.json").exists() {
        // Has package.json but no lock file - default to npm
        Some(PackageManager::Npm)
    } else {
        None
    }
}

/// Check if package.json exists
pub fn has_package_json(path: &Path) -> bool {
    path.join("package.json").exists()
}

/// Check if node_modules exists
pub fn has_node_modules(path: &Path) -> bool {
    path.join("node_modules").exists()
}

/// Install dependencies for a project
pub fn install_dependencies(path: &Path) -> Result<bool> {
    let pm = match detect_package_manager(path) {
        Some(pm) => pm,
        None => {
            // No package.json found
            return Ok(false);
        }
    };

    println!(
        "{}",
        format!("Installing dependencies using {}...", pm.name()).blue()
    );

    let (cmd, args) = match pm {
        PackageManager::Npm => ("npm", vec!["install"]),
        PackageManager::Yarn => ("yarn", vec!["install"]),
        PackageManager::Pnpm => ("pnpm", vec!["install"]),
    };

    let status = Command::new(cmd)
        .args(&args)
        .current_dir(path)
        .status()?;

    if status.success() {
        println!(
            "{}",
            format!("Dependencies installed successfully using {}", pm.name()).green()
        );
        Ok(true)
    } else {
        println!(
            "{}",
            format!("Failed to install dependencies with {}", pm.name()).red()
        );
        Ok(false)
    }
}
