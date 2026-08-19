use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const EMBEDDED_CONFIG: &str = include_str!("../categories.toml");
pub const EMBEDDED_STARSHIP: &str = include_str!("../config/zsh-tokyonight/starship.toml");
pub const EMBEDDED_ALIASES: &str = include_str!("../config/zsh-tokyonight/aliases.zsh");
pub const EMBEDDED_FONT: &[u8] = include_bytes!("../config/zsh-tokyonight/font.ttf");
pub const EMBEDDED_I3_CONFIG: &str = include_str!("../config/i3wm/config");
pub const EMBEDDED_POLYBAR_CONFIG: &str = include_str!("../config/i3wm/config.ini");
pub const EMBEDDED_POLYBAR_LAUNCH: &str = include_str!("../config/i3wm/launch.sh");
pub const EMBEDDED_NVIM_ONEDARKPRO: &[(&str, &str)] = &[
    (".neoconf.json", include_str!("../config/nvim-onedarkpro/.neoconf.json")),
    ("init.lua", include_str!("../config/nvim-onedarkpro/init.lua")),
    ("lazy-lock.json", include_str!("../config/nvim-onedarkpro/lazy-lock.json")),
    ("lazyvim.json", include_str!("../config/nvim-onedarkpro/lazyvim.json")),
    ("stylua.toml", include_str!("../config/nvim-onedarkpro/stylua.toml")),
    ("lua/config/autocmds.lua", include_str!("../config/nvim-onedarkpro/lua/config/autocmds.lua")),
    ("lua/config/color.lua", include_str!("../config/nvim-onedarkpro/lua/config/color.lua")),
    ("lua/config/keymaps.lua", include_str!("../config/nvim-onedarkpro/lua/config/keymaps.lua")),
    ("lua/config/lazy.lua", include_str!("../config/nvim-onedarkpro/lua/config/lazy.lua")),
    ("lua/config/options.lua", include_str!("../config/nvim-onedarkpro/lua/config/options.lua")),
    ("lua/plugins/colorscheme.lua", include_str!("../config/nvim-onedarkpro/lua/plugins/colorscheme.lua")),
    ("lua/plugins/goto-preview.lua", include_str!("../config/nvim-onedarkpro/lua/plugins/goto-preview.lua")),
    ("lua/plugins/oil.lua", include_str!("../config/nvim-onedarkpro/lua/plugins/oil.lua")),
    ("lua/plugins/stylelint.lua", include_str!("../config/nvim-onedarkpro/lua/plugins/stylelint.lua")),
    ("snippets/alias-typescript.json", include_str!("../config/nvim-onedarkpro/snippets/alias-typescript.json")),
    ("snippets/layout-component.json", include_str!("../config/nvim-onedarkpro/snippets/layout-component.json")),
    ("snippets/layout-full-astro.json", include_str!("../config/nvim-onedarkpro/snippets/layout-full-astro.json")),
    ("snippets/main-components.json", include_str!("../config/nvim-onedarkpro/snippets/main-components.json")),
    ("snippets/modal-popover.json", include_str!("../config/nvim-onedarkpro/snippets/modal-popover.json")),
    ("snippets/nav-basic.json", include_str!("../config/nvim-onedarkpro/snippets/nav-basic.json")),
    ("snippets/package.json", include_str!("../config/nvim-onedarkpro/snippets/package.json")),
    ("snippets/seo-head.json", include_str!("../config/nvim-onedarkpro/snippets/seo-head.json")),
    ("snippets/var-css-base.json", include_str!("../config/nvim-onedarkpro/snippets/var-css-base.json")),
    ("snippets/web-components-init.json", include_str!("../config/nvim-onedarkpro/snippets/web-components-init.json")),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub categories: BTreeMap<String, Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub description: String,
    pub aliases: Option<Vec<String>>,
    pub display_packages: Option<Vec<String>>,
    pub debian_packages: Option<Vec<String>>,
    pub termux_packages: Option<Vec<String>>,
    pub custom: Option<BTreeMap<String, CustomInstaller>>,
    pub copy_files: Option<Vec<CopyFileAction>>,
    pub post_install_commands: Option<Vec<PostInstallCommand>>,
    pub section_injections: Option<Vec<SectionInjection>>,
    pub final_message: Option<String>,
    pub debian_final_message: Option<String>,
    pub termux_final_message: Option<String>,
    pub includes: Option<Vec<String>>,
    pub group: Option<String>,
    pub platform: Option<String>,
}

impl Category {
    pub fn final_message_for_platform(&self, platform: &crate::platform::Platform) -> Option<&str> {
        match platform {
            crate::platform::Platform::Debian => self
                .debian_final_message
                .as_deref()
                .or(self.final_message.as_deref()),
            crate::platform::Platform::Termux => self
                .termux_final_message
                .as_deref()
                .or(self.final_message.as_deref()),
            _ => self.final_message.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFileAction {
    pub src: String,
    pub dest: String,
    pub platform: Option<String>,
    pub only_if_not_exists: Option<bool>,
    pub backup: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInstallCommand {
    pub command: String,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub prompt: Option<String>,
    pub confirm: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionInjection {
    pub file: String,
    pub section: String,
    pub line: String,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub position: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomInstaller {
    pub name: String,
    #[serde(rename = "type")]
    pub installer_type: String,
    pub url: String,
    pub extract_dir: String,
    pub bin_symlink: String,
    pub binary_name: Option<String>,
}

impl Config {
    /// Loads configuration from local ./categories.toml, ~/.config/dotss/categories.toml,
    /// or falls back to the embedded default configuration compiled into the binary.
    /// Also scans for modular configuration files in ./categories.d/*.toml and ~/.config/dotss/categories.d/*.toml.
    pub fn load() -> anyhow::Result<(Self, String)> {
        let (mut config, primary_source) = Self::load_base()?;
        let mut loaded_modular_files = Vec::new();

        // 1. Scan local ./categories.d/ if it exists
        let local_d = Path::new("categories.d");
        if local_d.is_dir() {
            Self::load_directory_into(&mut config, local_d, &mut loaded_modular_files)?;
        }

        // 2. Scan XDG ~/.config/dotss/categories.d/ if it exists
        if let Some(user_dir) = Self::get_user_config_dir() {
            let xdg_d = user_dir.join("categories.d");
            if xdg_d.is_dir() {
                Self::load_directory_into(&mut config, &xdg_d, &mut loaded_modular_files)?;
            }
        }

        let source_summary = if loaded_modular_files.is_empty() {
            primary_source
        } else {
            format!(
                "{} (+ {} modular file(s) in categories.d/)",
                primary_source,
                loaded_modular_files.len()
            )
        };

        Ok((config, source_summary))
    }

    fn load_base() -> anyhow::Result<(Self, String)> {
        let local_path = Path::new("categories.toml");
        if local_path.exists() {
            let content = fs::read_to_string(local_path)
                .map_err(|e| anyhow::anyhow!("Failed to read local categories.toml: {e}"))?;
            let config: Self = toml::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse local categories.toml: {e}"))?;
            return Ok((config, "./categories.toml".to_string()));
        }

        if let Some(home) = dirs_home_dir() {
            let xdg_path = home.join(".config/dotss/categories.toml");
            if xdg_path.exists() {
                let content = fs::read_to_string(&xdg_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read {}: {e}", xdg_path.display()))?;
                let config: Self = toml::from_str(&content)
                    .map_err(|e| anyhow::anyhow!("Failed to parse {}: {e}", xdg_path.display()))?;
                return Ok((config, xdg_path.to_string_lossy().to_string()));
            }
        }

        let config: Self = toml::from_str(EMBEDDED_CONFIG).map_err(|e| {
            anyhow::anyhow!("Failed to parse embedded default categories.toml: {e}")
        })?;
        Ok((config, "Embedded default configuration".to_string()))
    }

    /// Extracts all HTTP/HTTPS URLs referenced in category custom installers and post-install commands.
    #[cfg(test)]
    pub fn extract_all_urls(&self) -> Vec<String> {
        let mut urls = Vec::new();

        for cat in self.categories.values() {
            if let Some(custom_map) = &cat.custom {
                for custom in custom_map.values() {
                    if custom.url.starts_with("http://") || custom.url.starts_with("https://") {
                        urls.push(custom.url.clone());
                    }
                }
            }

            if let Some(post_cmds) = &cat.post_install_commands {
                for cmd in post_cmds {
                    for word in cmd.command.split_whitespace() {
                        let cleaned = word.trim_matches(|c| {
                            c == '\'' || c == '"' || c == '(' || c == ')' || c == ';'
                        });
                        if cleaned.starts_with("http://") || cleaned.starts_with("https://") {
                            let url_str = cleaned.split('|').next().unwrap_or(cleaned);
                            if !urls.contains(&url_str.to_string()) {
                                urls.push(url_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        urls.sort();
        urls.dedup();
        urls
    }

    fn load_directory_into(
        config: &mut Self,
        dir: &Path,
        loaded_files: &mut Vec<PathBuf>,
    ) -> anyhow::Result<()> {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };

        let mut paths: Vec<PathBuf> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                paths.push(path);
            }
        }

        // Sort paths alphabetically for deterministic loading order
        paths.sort();

        for path in paths {
            let content = fs::read_to_string(&path).map_err(|e| {
                anyhow::anyhow!("Failed to read modular config {}: {e}", path.display())
            })?;
            let sub_config: Self = toml::from_str(&content).map_err(|e| {
                anyhow::anyhow!("Failed to parse modular config {}: {e}", path.display())
            })?;

            for (cat_name, category) in sub_config.categories {
                config.categories.insert(cat_name, category);
            }

            loaded_files.push(path);
        }

        Ok(())
    }

    /// Returns the standard user configuration directory (~/.config/dotss).
    pub fn get_user_config_dir() -> Option<PathBuf> {
        dirs_home_dir().map(|home| home.join(".config/dotss"))
    }

    /// Resolves an input query (canonical category ID or any defined alias) to the canonical category key.
    pub fn resolve_category_key<'a>(&'a self, query: &'a str) -> Option<&'a String> {
        if self.categories.contains_key(query) {
            return self.categories.get_key_value(query).map(|(k, _)| k);
        }

        for (key, category) in &self.categories {
            if category.aliases.as_ref().is_some_and(|aliases| {
                aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(query))
            }) {
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

/// Validates if an HTTP/HTTPS URL is reachable (returns HTTP 2xx or 3xx status).
#[allow(dead_code)]
pub fn validate_url_reachable(url: &str) -> anyhow::Result<u16> {
    if url.contains("${") {
        return Ok(200);
    }

    let output = std::process::Command::new("curl")
        .arg("-sIL")
        .arg("--connect-timeout")
        .arg("10")
        .arg("--max-time")
        .arg("20")
        .arg("--retry")
        .arg("2")
        .arg("-H")
        .arg("User-Agent: dotss-cli")
        .arg("-o")
        .arg("/dev/null")
        .arg("-w")
        .arg("%{http_code}")
        .arg(url)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("curl failed to reach URL: {url}");
    }

    let code_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let status_code: u16 = code_str.parse().unwrap_or(0);

    if (200..400).contains(&status_code) {
        Ok(status_code)
    } else {
        anyhow::bail!("URL '{url}' returned non-success HTTP status code: {status_code}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_config_should_parse_and_contain_default_categories() {
        println!("\n🔍 [TEST] Embedded Default TOML Configuration Parsing");
        println!(
            "   Explanation: Verifies that the category catalog parses successfully and contains 'lazyvim-minimal'."
        );

        let config: Result<Config, _> = toml::from_str(EMBEDDED_CONFIG);
        assert!(config.is_ok(), "Embedded TOML should parse without errors");

        let cfg = config.unwrap();
        println!(
            "   ✓ Valid TOML structure. Total categories loaded: {}",
            cfg.categories.len()
        );

        assert!(
            cfg.categories.contains_key("lazyvim-minimal"),
            "Must include 'lazyvim-minimal' category"
        );
        assert!(
            cfg.categories.contains_key("nodejs-pnpm"),
            "Must include 'nodejs-pnpm' category"
        );
        println!("   ✓ Category 'lazyvim-minimal' and 'nodejs-pnpm' verified in catalog.\n");
    }

    #[test]
    fn alias_resolution_should_match_canonical_and_alias_keys() {
        let config: Config = toml::from_str(EMBEDDED_CONFIG).expect("Should parse embedded config");

        // Direct canonical name match
        assert_eq!(
            config.resolve_category_key("lazyvim-minimal"),
            Some(&"lazyvim-minimal".to_string())
        );

        // Alias match
        assert_eq!(
            config.resolve_category_key("lzv-min"),
            Some(&"lazyvim-minimal".to_string())
        );

        // Non-matching query
        assert_eq!(config.resolve_category_key("nonexistent"), None);
    }

    #[test]
    fn zsh_tokyonight_should_declare_terminal_restart_message() {
        let config: Config = toml::from_str(EMBEDDED_CONFIG).expect("Should parse embedded config");

        let cat = config
            .categories
            .get("zsh-tokyonight")
            .expect("zsh-tokyonight category should exist");
        let msg_debian = cat
            .final_message_for_platform(&crate::platform::Platform::Debian)
            .expect("zsh-tokyonight should declare a final_message on Debian");
        assert!(
            msg_debian.to_lowercase().contains("terminal"),
            "Debian final message should hint to restart the terminal"
        );

        let msg_termux = cat
            .final_message_for_platform(&crate::platform::Platform::Termux)
            .expect("zsh-tokyonight should declare a final_message on Termux");
        assert!(
            msg_termux.to_lowercase().contains("termux"),
            "Termux final message should hint to restart Termux"
        );
    }

    #[test]
    fn category_names_and_aliases_should_not_have_exact_duplicates() {
        use std::collections::HashSet;

        let config: Config = toml::from_str(EMBEDDED_CONFIG).expect("Should parse embedded config");
        let mut seen_keys = HashSet::new();

        for (key, category) in &config.categories {
            assert!(
                seen_keys.insert(key.to_lowercase()),
                "Duplicate category name found: '{key}'"
            );

            if let Some(aliases) = &category.aliases {
                for alias in aliases {
                    assert!(
                        seen_keys.insert(alias.to_lowercase()),
                        "Duplicate category alias or collision with category name found: '{alias}'"
                    );
                }
            }
        }
    }

    #[test]
    fn modular_categories_directory_should_merge_categories() {
        let temp_dir = std::env::temp_dir().join("dotss_test_categories_d");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let file1 = temp_dir.join("01-editors.toml");
        let file2 = temp_dir.join("02-tools.toml");

        fs::write(
            &file1,
            r#"[categories.custom-editor]
description = "Custom modular editor setup"
debian_packages = ["vim"]
"#,
        )
        .unwrap();

        fs::write(
            &file2,
            r#"[categories.custom-tool]
description = "Custom modular tool setup"
debian_packages = ["htop"]
"#,
        )
        .unwrap();

        let mut config = Config {
            categories: BTreeMap::new(),
        };
        let mut loaded = Vec::new();
        Config::load_directory_into(&mut config, &temp_dir, &mut loaded).unwrap();

        assert_eq!(loaded.len(), 2, "Should load both .toml files");
        assert!(config.categories.contains_key("custom-editor"));
        assert!(config.categories.contains_key("custom-tool"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_configured_urls_reachability_should_pass_for_valid_urls() {
        let config: Config = toml::from_str(EMBEDDED_CONFIG).unwrap();
        let urls = config.extract_all_urls();
        assert!(
            !urls.is_empty(),
            "Embedded configuration should contain URLs"
        );

        for url in &urls {
            let res = validate_url_reachable(url);
            assert!(
                res.is_ok(),
                "Configured URL '{}' should be reachable, got error: {:?}",
                url,
                res.err()
            );
        }
    }

    #[test]
    fn test_fake_url_reachability_should_fail() {
        let fake_url =
            "https://raw.githubusercontent.com/fusoras/nonexistent-test-repo-99999/main/invalid.sh";
        let res = validate_url_reachable(fake_url);
        assert!(
            res.is_err(),
            "Validation for fake/nonexistent URL '{}' MUST fail",
            fake_url
        );
    }
}
