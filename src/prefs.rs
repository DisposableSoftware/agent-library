use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Preferences {
    pub theme: Option<String>,
}

impl Preferences {
    pub fn load(base_dir: &Path) -> Self {
        let path = base_dir.join("config.toml");
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("warning: could not parse config.toml: {e}");
            Self::default()
        })
    }

    pub fn save(&self, base_dir: &Path) -> Result<(), AppError> {
        std::fs::create_dir_all(base_dir)?;
        let content = toml::to_string(self).map_err(|e| AppError::Parse(e.to_string()))?;
        let path = base_dir.join("config.toml");
        let tmp = base_dir.join("config.toml.tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join(format!("agent-library-prefs-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_returns_default() {
        let dir = tempdir();
        let prefs = Preferences::load(&dir);
        assert!(prefs.theme.is_none());
    }

    #[test]
    fn save_and_reload() {
        let dir = tempdir();
        let prefs = Preferences { theme: Some("Catppuccin Latte".into()) };
        prefs.save(&dir).unwrap();

        let loaded = Preferences::load(&dir);
        assert_eq!(loaded.theme.as_deref(), Some("Catppuccin Latte"));
    }

    #[test]
    fn config_toml_is_valid_toml() {
        let dir = tempdir();
        let prefs = Preferences { theme: Some("Catppuccin Mocha".into()) };
        prefs.save(&dir).unwrap();

        let raw = std::fs::read_to_string(dir.join("config.toml")).unwrap();
        assert!(raw.contains("Catppuccin Mocha"));
        // Must round-trip through the toml crate without error
        let _: Preferences = toml::from_str(&raw).unwrap();
    }
}
