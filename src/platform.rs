use std::env;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    Debian,
    Termux,
    Unsupported(String),
}

impl Platform {
    /// Detects the running platform based on environment variables and filesystem indicators.
    pub fn detect() -> Self {
        if env::var("TERMUX_VERSION").is_ok() || Path::new("/data/data/com.termux").exists() {
            return Platform::Termux;
        }

        if Path::new("/etc/debian_version").exists() {
            return Platform::Debian;
        }

        if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
            let lower = os_release.to_lowercase();
            if lower.contains("id=debian") || lower.contains("id=ubuntu") || lower.contains("id_like=debian") {
                return Platform::Debian;
            }
        }

        Platform::Unsupported("Unknown Linux/POSIX system".to_string())
    }

    /// Checks if a package is currently installed in the operating system package manager.
    pub fn is_package_installed(&self, pkg_name: &str) -> bool {
        match self {
            Platform::Debian | Platform::Termux => {
                if let Ok(output) = Command::new("dpkg-query")
                    .arg("-W")
                    .arg("-f=${db:Status-Status}")
                    .arg(pkg_name)
                    .output()
                {
                    let status = String::from_utf8_lossy(&output.stdout);
                    status.trim() == "installed"
                } else {
                    false
                }
            }
            Platform::Unsupported(_) => false,
        }
    }
}

/// Utility function to check if a binary exists in the system PATH.
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Checks if apt lock frontend is locked on Debian systems.
pub fn check_apt_lock() -> Result<(), String> {
    let lock_file = Path::new("/var/lib/dpkg/lock-frontend");
    if lock_file.exists() {
        // Attempting a non-blocking test via fuser or dpkg if available
        if let Ok(output) = Command::new("fuser").arg("/var/lib/dpkg/lock-frontend").output() {
            if output.status.success() && !output.stdout.is_empty() {
                return Err("apt package manager is locked by another process (e.g. background update).".to_string());
            }
        }
    }
    Ok(())
}
