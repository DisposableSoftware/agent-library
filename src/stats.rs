use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    #[serde(default)]
    pub tools: HashMap<String, u64>,
}

impl Stats {
    pub fn load(base_dir: &Path) -> Self {
        let path = base_dir.join("stats.toml");
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("warning: could not parse stats.toml: {e}");
            Self::default()
        })
    }

    pub fn save(&self, base_dir: &Path) -> Result<(), AppError> {
        std::fs::create_dir_all(base_dir)?;
        let content = toml::to_string(self).map_err(|e| AppError::Parse(e.to_string()))?;
        let path = base_dir.join("stats.toml");
        let tmp = base_dir.join("stats.toml.tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, &path)?;
        Ok(())
    }

    pub fn increment(base_dir: &Path, tool: &str) {
        let mut stats = Self::load(base_dir);
        *stats.tools.entry(tool.to_owned()).or_insert(0) += 1;
        if let Err(e) = stats.save(base_dir) {
            eprintln!("warning: could not save stats.toml: {e}");
        }
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
            .join(format!("agent-library-stats-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn missing_file_returns_default() {
        let dir = tempdir();
        let stats = Stats::load(&dir);
        assert!(stats.tools.is_empty());
    }

    #[test]
    fn increment_creates_entry() {
        let dir = tempdir();
        Stats::increment(&dir, "browse_documents");
        let stats = Stats::load(&dir);
        assert_eq!(stats.tools["browse_documents"], 1);
    }

    #[test]
    fn increment_accumulates() {
        let dir = tempdir();
        Stats::increment(&dir, "lookup_specific");
        Stats::increment(&dir, "lookup_specific");
        Stats::increment(&dir, "lookup_specific");
        let stats = Stats::load(&dir);
        assert_eq!(stats.tools["lookup_specific"], 3);
    }

    #[test]
    fn multiple_tools_tracked_independently() {
        let dir = tempdir();
        Stats::increment(&dir, "browse_documents");
        Stats::increment(&dir, "lookup_specific");
        Stats::increment(&dir, "browse_documents");
        let stats = Stats::load(&dir);
        assert_eq!(stats.tools["browse_documents"], 2);
        assert_eq!(stats.tools["lookup_specific"], 1);
    }

    #[test]
    fn save_and_reload_round_trips() {
        let dir = tempdir();
        let mut stats = Stats::default();
        stats.tools.insert("save_document".to_owned(), 7);
        stats.save(&dir).unwrap();

        let loaded = Stats::load(&dir);
        assert_eq!(loaded.tools["save_document"], 7);
    }

    #[test]
    fn stats_toml_is_valid_toml() {
        let dir = tempdir();
        Stats::increment(&dir, "browse_documents");

        let raw = std::fs::read_to_string(dir.join("stats.toml")).unwrap();
        assert!(raw.contains("browse_documents"));
        let _: Stats = toml::from_str(&raw).unwrap();
    }
}
