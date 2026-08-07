use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const EMBEDDED_CONFIG: &str = include_str!("../categories.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub categories: BTreeMap<String, Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub description: String,
    pub debian_packages: Option<Vec<String>>,
    pub termux_packages: Option<Vec<String>>,
    pub custom: Option<BTreeMap<String, CustomInstaller>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomInstaller {
    pub name: String,
    #[serde(rename = "type")]
    pub installer_type: String,
    pub url: String,
    pub extract_dir: String,
    pub bin_symlink: String,
}

impl Config {
    /// Loads configuration from local ./categories.toml, ~/.config/project-dots/categories.toml,
    /// or falls back to the embedded default configuration compiled into the binary.
    pub fn load() -> Result<(Self, String), String> {
        let local_path = Path::new("categories.toml");
        if local_path.exists() {
            let content = fs::read_to_string(local_path)
                .map_err(|e| format!("Failed to read local categories.toml: {}", e))?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse local categories.toml: {}", e))?;
            return Ok((config, "./categories.toml".to_string()));
        }

        if let Some(home) = dirs_home_dir() {
            let xdg_path = home.join(".config/project-dots/categories.toml");
            if xdg_path.exists() {
                let content = fs::read_to_string(&xdg_path)
                    .map_err(|e| format!("Failed to read {}: {}", xdg_path.display(), e))?;
                let config: Config = toml::from_str(&content)
                    .map_err(|e| format!("Failed to parse {}: {}", xdg_path.display(), e))?;
                return Ok((config, xdg_path.to_string_lossy().to_string()));
            }
        }

        let config: Config = toml::from_str(EMBEDDED_CONFIG)
            .map_err(|e| format!("Failed to parse embedded default categories.toml: {}", e))?;
        Ok((config, "Embedded default configuration".to_string()))
    }
}

/// Helper to resolve the user's home directory from environment.
pub fn dirs_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
