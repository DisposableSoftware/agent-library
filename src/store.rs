use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::pattern::{parse_pattern, serialize_pattern, slugify, Pattern};

/// Maximum slug collisions before giving up.
const MAX_SLUG_COLLISIONS: u32 = 10_000;

pub struct PatternStore {
    pub base_dir: PathBuf,
}

impl PatternStore {
    pub fn new(base_dir: PathBuf) -> Self {
        PatternStore { base_dir }
    }

    pub fn patterns_dir(&self) -> PathBuf {
        self.base_dir.join("patterns")
    }

    /// Load all patterns from the patterns directory (recursive).
    pub fn load_all(&self) -> Result<Vec<Pattern>, AppError> {
        let dir = self.patterns_dir();
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut patterns = Vec::new();
        visit_dir(&dir, &mut patterns)?;
        patterns.sort_by(|a, b| {
            a.task_type
                .to_lowercase()
                .cmp(&b.task_type.to_lowercase())
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(patterns)
    }

    /// Write a new pattern to disk, choosing a slug-based filename.
    /// Automatically regenerates ROOT.md afterwards.
    pub fn save_pattern(&self, pattern: &Pattern) -> Result<PathBuf, AppError> {
        validate_field("name", &pattern.name)?;
        validate_field("task_type", &pattern.task_type)?;

        let task_dir = self.patterns_dir().join(slugify_dir(&pattern.task_type));
        fs::create_dir_all(&task_dir)?;

        let path = claim_unique_slug(&task_dir, &pattern.name)?;

        let fill = || -> Result<(), AppError> {
            let mut p = pattern.clone();
            p.file_path = path.clone();
            write_atomic(&path, &serialize_pattern(&p)?)
        };
        if let Err(e) = fill() {
            // Clean up the empty placeholder so the directory isn't littered.
            let _ = fs::remove_file(&path);
            return Err(e);
        }

        let all = self.load_all()?;
        self.regenerate_root(&all)?;

        Ok(path)
    }

    /// Overwrite an existing pattern file in-place (e.g. after toggle).
    pub fn update_pattern(&self, pattern: &Pattern) -> Result<(), AppError> {
        write_atomic(&pattern.file_path, &serialize_pattern(pattern)?)?;
        let all = self.load_all()?;
        self.regenerate_root(&all)?;
        Ok(())
    }

    /// Returns the ROOT.md contents, creating it if missing.
    pub fn load_root(&self) -> Result<String, AppError> {
        let root_path = self.base_dir.join("ROOT.md");
        if !root_path.exists() {
            let patterns = self.load_all()?;
            self.regenerate_root(&patterns)?;
        }
        Ok(fs::read_to_string(root_path)?)
    }

    /// Regenerate ROOT.md from the current pattern list (sorted, deterministic).
    pub fn regenerate_root(&self, patterns: &[Pattern]) -> Result<(), AppError> {
        fs::create_dir_all(&self.base_dir)?;
        let root_path = self.base_dir.join("ROOT.md");
        let content = build_root_doc(patterns, &self.base_dir);
        write_atomic(&root_path, &content)?;
        Ok(())
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn visit_dir(dir: &Path, out: &mut Vec<Pattern>) -> Result<(), AppError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_dir(&path, out)?;
        } else if path.is_file() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "ROOT.md" || !name.ends_with(".md") {
                continue;
            }
            let raw = fs::read_to_string(&path)?;
            match parse_pattern(&raw, path) {
                Ok(p) => out.push(p),
                Err(e) => eprintln!("warning: skipping malformed pattern: {e}"),
            }
        }
    }
    Ok(())
}

/// Write a file atomically via a .tmp sibling then rename.
fn write_atomic(path: &Path, content: &str) -> Result<(), AppError> {
    let tmp = path.with_extension("md.tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Reject input that would let a remote caller escape the patterns directory
/// or write to hidden / control-character paths.
fn validate_field(label: &str, value: &str) -> Result<(), AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid(format!("{label} must not be empty")));
    }
    if value.contains('/')
        || value.contains('\\')
        || value.contains('\0')
        || value.chars().any(|c| c.is_control())
        || value == ".."
        || value == "."
    {
        return Err(AppError::Invalid(format!(
            "{label} contains invalid characters"
        )));
    }
    Ok(())
}

/// Atomically claim a unique `<slug>.md` filename inside `dir` by creating an
/// empty file via O_CREAT|O_EXCL. Returns the claimed path; the caller is
/// responsible for filling it with content (e.g. via `write_atomic`, which
/// renames over the placeholder).
fn claim_unique_slug(dir: &Path, name: &str) -> Result<PathBuf, AppError> {
    let base = slugify(name);
    if base.is_empty() {
        return Err(AppError::Invalid(
            "name slug is empty after sanitisation".into(),
        ));
    }
    for n in 0..MAX_SLUG_COLLISIONS {
        let slug = if n == 0 {
            base.clone()
        } else {
            format!("{base}-{}", n + 1)
        };
        let path = dir.join(format!("{slug}.md"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(_) => return Ok(path),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(AppError::Io(e)),
        }
    }
    Err(AppError::Invalid(format!(
        "exhausted slug collisions for '{name}'"
    )))
}

/// Convert a task type string to a directory name: lowercase, non-alphanumeric → hyphens.
/// Callers must have validated the input via `validate_field` first.
fn slugify_dir(s: &str) -> String {
    let lowered: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let mut out = String::with_capacity(lowered.len());
    let mut prev_hyphen = false;
    for c in lowered.chars() {
        if c == '-' {
            if !prev_hyphen && !out.is_empty() {
                out.push('-');
            }
            prev_hyphen = true;
        } else {
            out.push(c);
            prev_hyphen = false;
        }
    }
    out.trim_end_matches('-').to_owned()
}

fn build_root_doc(patterns: &[Pattern], base_dir: &Path) -> String {
    let mut lines = vec![
        "# Agent Pattern Library".to_owned(),
        String::new(),
        "This document provides an index of all available patterns.".to_owned(),
        "Use the MCP tools to retrieve patterns:".to_owned(),
        String::new(),
        "- `lookup_pattern(task_type)` — retrieve all enabled patterns for a task type".to_owned(),
        "- `lookup_specific(name)` — retrieve a single pattern by name".to_owned(),
        "- `save_pattern(name, task_type, content)` — save a new pattern".to_owned(),
        "- `browse_patterns()` — return this document".to_owned(),
        String::new(),
        "Disabled patterns are listed here but will **not** be returned by lookup tools.".to_owned(),
        String::new(),
        "## Patterns".to_owned(),
        String::new(),
        "| Name | Task Type | Enabled | File |".to_owned(),
        "|------|-----------|---------|------|".to_owned(),
    ];

    for p in patterns {
        let enabled = if p.enabled { "✓" } else { "✗" };
        let rel_path = p
            .file_path
            .strip_prefix(base_dir)
            .unwrap_or(&p.file_path)
            .display()
            .to_string();
        lines.push(format!(
            "| {} | {} | {} | {} |",
            p.name, p.task_type, enabled, rel_path
        ));
    }

    lines.push(String::new());
    lines.push("## Task Types Present".to_owned());
    lines.push(String::new());

    let mut types: Vec<String> = patterns
        .iter()
        .map(|p| p.task_type.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    types.sort();
    lines.push(types.join(", "));
    lines.push(String::new());

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::Pattern;
    use std::path::PathBuf;

    fn tempdir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join(format!("agent-library-store-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_pattern(name: &str, task_type: &str, enabled: bool) -> Pattern {
        Pattern {
            name: name.to_owned(),
            enabled,
            date_added: "2026-05-12".to_owned(),
            task_type: task_type.to_owned(),
            file_path: PathBuf::new(),
            content: format!("# {name}\n\nContent.\n"),
        }
    }

    #[test]
    fn load_all_empty_dir() {
        let base = tempdir();
        let store = PatternStore::new(base);
        let patterns = store.load_all().unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn load_all_skips_root_doc() {
        let base = tempdir();
        fs::create_dir_all(base.join("patterns")).unwrap();
        fs::write(base.join("patterns").join("ROOT.md"), "not a pattern").unwrap();
        let store = PatternStore::new(base);
        let patterns = store.load_all().unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn load_all_skips_non_markdown() {
        let base = tempdir();
        fs::create_dir_all(base.join("patterns")).unwrap();
        fs::write(base.join("patterns").join("notes.txt"), "ignored").unwrap();
        let store = PatternStore::new(base);
        let patterns = store.load_all().unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn save_and_reload() {
        let base = tempdir();
        let store = PatternStore::new(base);
        let p = make_pattern("REST API Design", "Architecture", true);
        store.save_pattern(&p).unwrap();

        let all = store.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "REST API Design");
        assert_eq!(all[0].task_type, "Architecture");
        assert!(all[0].enabled);
    }

    #[test]
    fn regenerate_root_contains_all_patterns() {
        let base = tempdir();
        let store = PatternStore::new(base.clone());
        store.save_pattern(&make_pattern("Git Commit Style", "Style Guide", true)).unwrap();
        store.save_pattern(&make_pattern("REST API Design", "Architecture", true)).unwrap();
        store.save_pattern(&make_pattern("Code Documentation", "Documentation", true)).unwrap();

        let root = fs::read_to_string(base.join("ROOT.md")).unwrap();
        assert!(root.contains("Git Commit Style"));
        assert!(root.contains("REST API Design"));
        assert!(root.contains("Code Documentation"));
    }

    #[test]
    fn regenerate_root_sorted() {
        let base = tempdir();
        let store = PatternStore::new(base.clone());
        store.save_pattern(&make_pattern("Zebra Pattern", "Style Guide", true)).unwrap();
        store.save_pattern(&make_pattern("Alpha Pattern", "Architecture", true)).unwrap();

        let root = fs::read_to_string(base.join("ROOT.md")).unwrap();
        let arch_pos = root.find("Alpha Pattern").unwrap();
        let style_pos = root.find("Zebra Pattern").unwrap();
        // Architecture comes before Style Guide alphabetically
        assert!(arch_pos < style_pos);
    }

    #[test]
    fn disabled_pattern_in_root() {
        let base = tempdir();
        let store = PatternStore::new(base.clone());
        store.save_pattern(&make_pattern("Disabled One", "Style Guide", false)).unwrap();

        let root = fs::read_to_string(base.join("ROOT.md")).unwrap();
        assert!(root.contains("Disabled One"));
        assert!(root.contains('✗'));
    }

    #[test]
    fn save_pattern_rejects_path_traversal() {
        let base = tempdir();
        let store = PatternStore::new(base.clone());
        let mut p = make_pattern("Evil", "../../etc", true);
        let err = store.save_pattern(&p).unwrap_err();
        assert!(matches!(err, AppError::Invalid(_)));
        // Slash and backslash and control chars also rejected.
        p.task_type = "foo/bar".to_owned();
        assert!(matches!(store.save_pattern(&p).unwrap_err(), AppError::Invalid(_)));
        p.task_type = "foo\\bar".to_owned();
        assert!(matches!(store.save_pattern(&p).unwrap_err(), AppError::Invalid(_)));
        p.task_type = "ok".to_owned();
        p.name = "".to_owned();
        assert!(matches!(store.save_pattern(&p).unwrap_err(), AppError::Invalid(_)));
        // Confirm no escape file landed.
        let escaped = base.parent().unwrap().join("etc");
        assert!(!escaped.exists());
    }

    #[test]
    fn save_pattern_unique_slug_under_collision() {
        let base = tempdir();
        let store = PatternStore::new(base);
        store.save_pattern(&make_pattern("My Pattern", "Architecture", true)).unwrap();
        store.save_pattern(&make_pattern("My Pattern", "Architecture", true)).unwrap();
        let all = store.load_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn toggle_enabled_persists() {
        let base = tempdir();
        let store = PatternStore::new(base);
        store.save_pattern(&make_pattern("Foo", "Architecture", true)).unwrap();

        let mut all = store.load_all().unwrap();
        assert_eq!(all.len(), 1);
        all[0].enabled = false;
        store.update_pattern(&all[0]).unwrap();

        let reloaded = store.load_all().unwrap();
        assert!(!reloaded[0].enabled);
    }
}
