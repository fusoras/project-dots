use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const EMBEDDED_CONFIG: &str = include_str!("../categories.toml");
pub const EMBEDDED_STARSHIP: &str = include_str!("../config/shell-tokyonight/starship.toml");
pub const EMBEDDED_FONT: &[u8] = include_bytes!("../config/shell-tokyonight/font.ttf");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub categories: BTreeMap<String, Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub description: String,
    pub aliases: Option<Vec<String>>,
    pub debian_packages: Option<Vec<String>>,
    pub termux_packages: Option<Vec<String>>,
    pub custom: Option<BTreeMap<String, CustomInstaller>>,
    pub copy_files: Option<Vec<CopyFileAction>>,
    pub post_install_commands: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFileAction {
    pub src: String,
    pub dest: String,
    pub platform: Option<String>,
    pub only_if_not_exists: Option<bool>,
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

    /// Returns the standard user configuration directory (~/.config/project-dots).
    pub fn get_user_config_dir() -> Option<PathBuf> {
        dirs_home_dir().map(|home| home.join(".config/project-dots"))
    }

    /// Resolves an input query (canonical category ID or any defined alias) to the canonical category key.
    pub fn resolve_category_key<'a>(&'a self, query: &'a str) -> Option<&'a String> {
        if self.categories.contains_key(query) {
            return self.categories.get_key_value(query).map(|(k, _)| k);
        }

        for (key, category) in &self.categories {
            if category.aliases.as_ref().is_some_and(|aliases| aliases.iter().any(|alias| alias.eq_ignore_ascii_case(query))) {
                return Some(key);
            }
        }

        None
    }
}

/// Helper to resolve the user's home directory from environment.
pub fn dirs_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_config_parsing() {
        println!("\n🔍 [TEST] Embedded Default TOML Configuration Parsing");
        println!("   Explanation: Verifies that the category catalog parses successfully and contains 'lazyvim-minimal'.");

        let config: Result<Config, _> = toml::from_str(EMBEDDED_CONFIG);
        assert!(config.is_ok(), "Embedded TOML should parse without errors");

        let cfg = config.unwrap();
        println!("   ✓ Valid TOML structure. Total categories loaded: {}", cfg.categories.len());

        assert!(cfg.categories.contains_key("lazyvim-minimal"), "Must include 'lazyvim-minimal' category");
        println!("   ✓ Category 'lazyvim-minimal' verified in catalog.\n");
    }

    #[test]
    fn test_category_alias_resolution() {
        let config: Config = toml::from_str(EMBEDDED_CONFIG).expect("Should parse embedded config");

        // Direct canonical name match
        assert_eq!(config.resolve_category_key("lazyvim-minimal"), Some(&"lazyvim-minimal".to_string()));

        // Alias match
        assert_eq!(config.resolve_category_key("lzv-min"), Some(&"lazyvim-minimal".to_string()));

        // Non-matching query
        assert_eq!(config.resolve_category_key("nonexistent"), None);
    }
}
