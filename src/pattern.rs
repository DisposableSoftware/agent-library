use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub enabled: bool,
    pub date_added: String,
    pub task_type: String,
    #[serde(skip)]
    pub file_path: PathBuf,
    #[serde(skip)]
    pub content: String,
}

/// Parse a pattern from raw Markdown file text.
///
/// Expected format:
/// ```text
/// ---
/// name: Foo
/// enabled: true
/// date_added: 2026-05-12
/// task_type: Architecture
/// ---
/// # Content here
/// ```
pub fn parse_pattern(raw: &str, file_path: PathBuf) -> Result<Pattern, AppError> {
    // Strip optional leading "---\n"
    let stripped = raw.strip_prefix("---\n").unwrap_or(raw);

    // Split on the closing "---\n"; everything before is front-matter, rest is body
    let (front, body) = stripped
        .split_once("\n---\n")
        .ok_or_else(|| AppError::Parse("missing closing front-matter delimiter".into()))?;

    let mut pattern: Pattern = serde_yaml_ng::from_str(front)
        .map_err(|e| AppError::Yaml(e.to_string()))?;

    pattern.file_path = file_path;
    pattern.content = body.to_owned();

    Ok(pattern)
}

/// Serialize a pattern back to its Markdown file representation.
pub fn serialize_pattern(pattern: &Pattern) -> Result<String, AppError> {
    let front = serde_yaml_ng::to_string(pattern).map_err(|e| AppError::Yaml(e.to_string()))?;
    Ok(format!("---\n{front}---\n{}", pattern.content))
}

/// Convert a pattern name to a filesystem-safe slug.
pub fn slugify(name: &str) -> String {
    let s: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse consecutive hyphens and strip leading/trailing ones
    let mut result = String::new();
    let mut prev_hyphen = false;
    for c in s.chars() {
        if c == '-' {
            if !prev_hyphen && !result.is_empty() {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }
    // Strip trailing hyphen
    result.trim_end_matches('-').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_raw() -> &'static str {
        "---\nname: REST API Design\nenabled: true\ndate_added: \"2026-05-12\"\ntask_type: Architecture\n---\n# Content\n\nSome body.\n"
    }

    #[test]
    fn parse_roundtrip() {
        let path = PathBuf::from("test.md");
        let pattern = parse_pattern(sample_raw(), path.clone()).unwrap();

        assert_eq!(pattern.name, "REST API Design");
        assert!(pattern.enabled);
        assert_eq!(pattern.date_added, "2026-05-12");
        assert_eq!(pattern.task_type, "Architecture");
        assert_eq!(pattern.content, "# Content\n\nSome body.\n");

        let serialized = serialize_pattern(&pattern).unwrap();
        let reparsed = parse_pattern(&serialized, path).unwrap();

        assert_eq!(reparsed.name, pattern.name);
        assert_eq!(reparsed.enabled, pattern.enabled);
        assert_eq!(reparsed.date_added, pattern.date_added);
        assert_eq!(reparsed.task_type, pattern.task_type);
        assert_eq!(reparsed.content, pattern.content);
    }

    #[test]
    fn parse_with_dashes_in_content() {
        let raw = "---\nname: Foo\nenabled: true\ndate_added: \"2026-01-01\"\ntask_type: Style\n---\n# Title\n\n---\n\nA horizontal rule above.\n";
        let pattern = parse_pattern(raw, PathBuf::from("foo.md")).unwrap();
        assert!(pattern.content.contains("---"));
        assert_eq!(pattern.name, "Foo");
    }

    #[test]
    fn parse_missing_field_fails() {
        // Missing task_type
        let raw = "---\nname: Foo\nenabled: true\ndate_added: \"2026-01-01\"\n---\nContent\n";
        let result = parse_pattern(raw, PathBuf::from("foo.md"));
        assert!(result.is_err());
    }

    #[test]
    fn parse_disabled_pattern() {
        let raw = "---\nname: Foo\nenabled: false\ndate_added: \"2026-01-01\"\ntask_type: Style\n---\nContent\n";
        let pattern = parse_pattern(raw, PathBuf::from("foo.md")).unwrap();
        assert!(!pattern.enabled);
    }

    #[test]
    fn slug_basic() {
        assert_eq!(slugify("REST API Design"), "rest-api-design");
    }

    #[test]
    fn slug_special_chars() {
        assert_eq!(slugify("C++ & Rust!"), "c-rust");
        assert_eq!(slugify("  leading spaces"), "leading-spaces");
    }

}
