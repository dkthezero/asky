use std::path::PathBuf;

/// Resolve the global configuration root according to OS standards and user preference.
/// - **macOS**: `~/.config/asky` (overriding default Library/Application Support)
/// - **Linux**: `~/.config/asky` (standard XDG path via dirs_next)
/// - **Windows**: `AppData/Roaming/asky` (standard via dirs_next)
pub fn global_config_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        // Force ~/.config/asky on macOS instead of ~/Library/Application Support/asky
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("asky")
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Use default config_dir (Linux: ~/.config, Windows: AppData/Roaming)
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("asky")
    }
}

/// Resolve the global vaults directory: `<config_root>/vaults`.
pub fn global_vaults_dir() -> PathBuf {
    global_config_root().join("vaults")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_root() {
        let root = global_config_root();
        #[cfg(target_os = "macos")]
        assert!(root.to_string_lossy().contains(".config/asky"));
        #[cfg(all(unix, not(target_os = "macos")))]
        assert!(root.to_string_lossy().contains(".config/asky"));
        #[cfg(target_os = "windows")]
        assert!(root.to_string_lossy().contains("AppData"));
    }
}
