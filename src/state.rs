use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State {
    #[serde(default)]
    pub packages: BTreeMap<String, TrackedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedPackage {
    pub category: String,
    pub was_preexisting: bool,
    pub installed_by_dots: bool,
    pub installed_at: String,
}

impl State {
    /// Returns the standard state file path (~/.local/state/project-dots/state.toml).
    pub fn get_state_path() -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".local/state/project-dots/state.toml")
    }

    /// Loads the persistent state file, returning a default empty state if the file does not exist.
    pub fn load() -> Self {
        let path = Self::get_state_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = toml::from_str(&content) {
                    return state;
                }
            }
        }
        State::default()
    }

    /// Saves the persistent state file atomically by writing to a .tmp file and renaming it.
    pub fn save_atomic(&self) -> Result<(), String> {
        let path = Self::get_state_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create state directory {}: {}", parent.display(), e))?;
        }

        let tmp_path = path.with_extension("toml.tmp");
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize state TOML: {}", e))?;

        fs::write(&tmp_path, content)
            .map_err(|e| format!("Failed to write temporary state file: {}", e))?;

        fs::rename(&tmp_path, &path)
            .map_err(|e| format!("Failed to atomically rename state file: {}", e))?;

        Ok(())
    }

    /// Records or updates package tracking metadata.
    pub fn track_package(&mut self, name: &str, category: &str, was_preexisting: bool) {
        let timestamp = chrono_now_string();
        self.packages.insert(
            name.to_string(),
            TrackedPackage {
                category: category.to_string(),
                was_preexisting,
                installed_by_dots: !was_preexisting,
                installed_at: timestamp,
            },
        );
    }

    /// Removes a package from state tracking after uninstallation.
    pub fn remove_package(&mut self, name: &str) {
        self.packages.remove(name);
    }
}

/// Simple ISO timestamp string generator using system time.
fn chrono_now_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("Timestamp({})", since_the_epoch.as_secs())
}
