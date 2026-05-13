use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub name: String,
    pub enabled: bool,
    pub date_added: String,
    pub task_type: String,
    #[serde(rename = "isSkill", default, skip_serializing_if = "is_false")]
    pub is_skill: bool,
    #[serde(skip)]
    pub file_path: PathBuf,
    #[serde(skip)]
    pub content: String,
}

fn is_false(v: &bool) -> bool {
    !*v
}

/// Parse a document from raw Markdown file text.
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
pub fn parse_document(raw: &str, file_path: PathBuf) -> Result<Document, AppError> {
    // Strip optional leading "---\n"
    let stripped = raw.strip_prefix("---\n").unwrap_or(raw);

    // Split on the closing "---\n"; everything before is front-matter, rest is body
    let (front, body) = stripped
        .split_once("\n---\n")
        .ok_or_else(|| AppError::Parse("missing closing front-matter delimiter".into()))?;

    let mut document: Document = serde_yaml_ng::from_str(front)
        .map_err(|e| AppError::Yaml(e.to_string()))?;

    document.file_path = file_path;
    document.content = body.to_owned();

    Ok(document)
}

/// Serialize a document back to its Markdown file representation.
pub fn serialize_document(document: &Document) -> Result<String, AppError> {
    let front = serde_yaml_ng::to_string(document).map_err(|e| AppError::Yaml(e.to_string()))?;
    Ok(format!("---\n{front}---\n{}", document.content))
}

/// Convert a document name to a filesystem-safe slug.
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

    fn sample_skill_raw() -> &'static str {
        "---\nname: My Skill\nenabled: true\ndate_added: \"2026-05-12\"\ntask_type: Architecture\nisSkill: true\n---\n# Skill content\n"
    }

    #[test]
    fn parse_roundtrip() {
        let path = PathBuf::from("test.md");
        let document = parse_document(sample_raw(), path.clone()).unwrap();

        assert_eq!(document.name, "REST API Design");
        assert!(document.enabled);
        assert_eq!(document.date_added, "2026-05-12");
        assert_eq!(document.task_type, "Architecture");
        assert_eq!(document.content, "# Content\n\nSome body.\n");

        let serialized = serialize_document(&document).unwrap();
        let reparsed = parse_document(&serialized, path).unwrap();

        assert_eq!(reparsed.name, document.name);
        assert_eq!(reparsed.enabled, document.enabled);
        assert_eq!(reparsed.date_added, document.date_added);
        assert_eq!(reparsed.task_type, document.task_type);
        assert_eq!(reparsed.content, document.content);
        assert_eq!(reparsed.is_skill, document.is_skill);
    }

    #[test]
    fn parse_skill_document() {
        let doc = parse_document(sample_skill_raw(), PathBuf::from("skill.md")).unwrap();
        assert!(doc.is_skill);
        // Round-trip: isSkill: true must survive serialize → parse
        let serialized = serialize_document(&doc).unwrap();
        assert!(serialized.contains("isSkill: true"));
        let reparsed = parse_document(&serialized, PathBuf::from("skill.md")).unwrap();
        assert!(reparsed.is_skill);
    }

    #[test]
    fn is_skill_defaults_false() {
        // Documents without isSkill in front-matter should default to false
        let doc = parse_document(sample_raw(), PathBuf::from("test.md")).unwrap();
        assert!(!doc.is_skill);
        // And is_skill: false must NOT be written to the serialized output
        let serialized = serialize_document(&doc).unwrap();
        assert!(!serialized.contains("isSkill"));
    }

    #[test]
    fn parse_with_dashes_in_content() {
        let raw = "---\nname: Foo\nenabled: true\ndate_added: \"2026-01-01\"\ntask_type: Style\n---\n# Title\n\n---\n\nA horizontal rule above.\n";
        let document = parse_document(raw, PathBuf::from("foo.md")).unwrap();
        assert!(document.content.contains("---"));
        assert_eq!(document.name, "Foo");
    }

    #[test]
    fn parse_missing_field_fails() {
        // Missing task_type
        let raw = "---\nname: Foo\nenabled: true\ndate_added: \"2026-01-01\"\n---\nContent\n";
        let result = parse_document(raw, PathBuf::from("foo.md"));
        assert!(result.is_err());
    }

    #[test]
    fn parse_disabled_document() {
        let raw = "---\nname: Foo\nenabled: false\ndate_added: \"2026-01-01\"\ntask_type: Style\n---\nContent\n";
        let document = parse_document(raw, PathBuf::from("foo.md")).unwrap();
        assert!(!document.enabled);
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
