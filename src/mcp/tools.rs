use serde_json::{json, Value};

use crate::pattern::Pattern;
use crate::store::PatternStore;

/// Dispatch a `tools/call` request to the matching tool handler.
/// Returns the MCP `result` value to embed in the RpcResponse.
pub fn handle_tool_call(name: &str, params: Option<&Value>, store: &PatternStore) -> Value {
    match name {
        "lookup_pattern" => tool_lookup_pattern(params, store),
        "lookup_specific" => tool_lookup_specific(params, store),
        "save_pattern" => tool_save_pattern(params, store),
        "browse_patterns" => tool_browse_patterns(store),
        _ => json!({
            "content": [{ "type": "text", "text": format!("Unknown tool: {name}") }],
            "isError": true
        }),
    }
}

/// JSON Schema definitions for `tools/list`.
pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "lookup_pattern",
            "description": "Return all enabled patterns for a given task type.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_type": {
                        "type": "string",
                        "description": "The task type to filter by (e.g. Architecture, Style Guide)"
                    }
                },
                "required": ["task_type"]
            }
        },
        {
            "name": "lookup_specific",
            "description": "Return a specific pattern by name.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The pattern name to look up"
                    }
                },
                "required": ["name"]
            }
        },
        {
            "name": "save_pattern",
            "description": "Save a new pattern to the library.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Pattern name" },
                    "task_type": { "type": "string", "description": "Task type category" },
                    "content": { "type": "string", "description": "Pattern body in Markdown" }
                },
                "required": ["name", "task_type", "content"]
            }
        },
        {
            "name": "browse_patterns",
            "description": "Return the root index document listing all patterns.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }
    ])
}

// ── Tool implementations ───────────────────────────────────────────────────

fn tool_lookup_pattern(params: Option<&Value>, store: &PatternStore) -> Value {
    let task_type = match params.and_then(|p| p.get("task_type")).and_then(|v| v.as_str()) {
        Some(t) => t.to_lowercase(),
        None => return error_response("missing required parameter: task_type"),
    };

    match store.load_all() {
        Err(e) => error_response(e.to_string()),
        Ok(patterns) => {
            let matches: Vec<_> = patterns
                .into_iter()
                .filter(|p| p.enabled && p.task_type.to_lowercase() == task_type)
                .collect();

            if matches.is_empty() {
                return text_response(format!(
                    "No enabled patterns found for task type '{task_type}'."
                ));
            }

            let text = matches
                .iter()
                .map(pattern_to_text)
                .collect::<Vec<_>>()
                .join("\n\n---\n\n");
            text_response(text)
        }
    }
}

fn tool_lookup_specific(params: Option<&Value>, store: &PatternStore) -> Value {
    let name = match params.and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
        Some(n) => n.to_lowercase(),
        None => return error_response("missing required parameter: name"),
    };

    match store.load_all() {
        Err(e) => error_response(e.to_string()),
        Ok(patterns) => {
            match patterns.into_iter().find(|p| p.name.to_lowercase() == name) {
                None => text_response(format!("No pattern found with name '{name}'.")),
                Some(p) => text_response(pattern_to_text(&p)),
            }
        }
    }
}

fn tool_save_pattern(params: Option<&Value>, store: &PatternStore) -> Value {
    let params = match params {
        Some(p) => p,
        None => return error_response("missing parameters"),
    };

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_owned(),
        None => return error_response("missing required parameter: name"),
    };
    let task_type = match params.get("task_type").and_then(|v| v.as_str()) {
        Some(t) => t.to_owned(),
        None => return error_response("missing required parameter: task_type"),
    };
    let content = match params.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_owned(),
        None => return error_response("missing required parameter: content"),
    };

    let today = current_date();
    let pattern = Pattern {
        name: name.clone(),
        enabled: true,
        date_added: today,
        task_type,
        file_path: Default::default(),
        content,
    };

    match store.save_pattern(&pattern) {
        Ok(path) => text_response(format!(
            "Pattern '{name}' saved to {}",
            path.display()
        )),
        Err(e) => error_response(e.to_string()),
    }
}

fn tool_browse_patterns(store: &PatternStore) -> Value {
    match store.load_root() {
        Ok(text) => text_response(text),
        Err(e) => error_response(e.to_string()),
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn text_response(text: impl Into<String>) -> Value {
    json!({
        "content": [{ "type": "text", "text": text.into() }]
    })
}

fn error_response(msg: impl Into<String>) -> Value {
    json!({
        "content": [{ "type": "text", "text": msg.into() }],
        "isError": true
    })
}

fn pattern_to_text(p: &Pattern) -> String {
    format!(
        "## {}\n**Task type:** {}  |  **Added:** {}  |  **Enabled:** {}\n\n{}",
        p.name,
        p.task_type,
        p.date_added,
        if p.enabled { "yes" } else { "no" },
        p.content
    )
}

fn current_date() -> String {
    // Format today's date as YYYY-MM-DD without a chrono dependency.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    // Days since Unix epoch → Gregorian date
    unix_days_to_date(days)
}

fn unix_days_to_date(days: u64) -> String {
    // Proleptic Gregorian calendar conversion from days since 1970-01-01.
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::Pattern;
    use std::path::PathBuf;

    fn temp_store() -> PatternStore {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join(format!("agent-library-tools-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        PatternStore::new(dir)
    }

    fn add_pattern(store: &PatternStore, name: &str, task_type: &str, enabled: bool) {
        let p = Pattern {
            name: name.to_owned(),
            enabled,
            date_added: "2026-05-12".to_owned(),
            task_type: task_type.to_owned(),
            file_path: PathBuf::new(),
            content: format!("# {name}\n\nContent.\n"),
        };
        store.save_pattern(&p).unwrap();
    }

    #[test]
    fn lookup_pattern_returns_matching() {
        let store = temp_store();
        add_pattern(&store, "REST API Design", "Architecture", true);
        add_pattern(&store, "Git Commit Style", "Style Guide", true);

        let result = tool_lookup_pattern(Some(&json!({"task_type": "architecture"})), &store);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("REST API Design"));
        assert!(!text.contains("Git Commit Style"));
    }

    #[test]
    fn lookup_pattern_case_insensitive() {
        let store = temp_store();
        add_pattern(&store, "REST API Design", "Architecture", true);

        let result = tool_lookup_pattern(Some(&json!({"task_type": "ARCHITECTURE"})), &store);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("REST API Design"));
        assert_ne!(result.get("isError"), Some(&json!(true)));
    }

    #[test]
    fn lookup_pattern_enabled_only() {
        let store = temp_store();
        add_pattern(&store, "REST API Design", "Architecture", true);
        add_pattern(&store, "Old Pattern", "Architecture", false);

        let result = tool_lookup_pattern(Some(&json!({"task_type": "architecture"})), &store);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("REST API Design"));
        assert!(!text.contains("Old Pattern"));
    }

    #[test]
    fn lookup_pattern_no_match() {
        let store = temp_store();
        let result = tool_lookup_pattern(Some(&json!({"task_type": "nonexistent"})), &store);
        // Should return a message, not an error
        assert_ne!(result.get("isError"), Some(&json!(true)));
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("No enabled patterns"));
    }

    #[test]
    fn lookup_specific_finds_by_name() {
        let store = temp_store();
        add_pattern(&store, "REST API Design", "Architecture", true);

        let result = tool_lookup_specific(Some(&json!({"name": "rest api design"})), &store);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("REST API Design"));
        assert_ne!(result.get("isError"), Some(&json!(true)));
    }

    #[test]
    fn lookup_specific_not_found() {
        let store = temp_store();
        let result = tool_lookup_specific(Some(&json!({"name": "doesnotexist"})), &store);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("No pattern found"));
    }

    #[test]
    fn save_pattern_creates_file() {
        let store = temp_store();
        let result = tool_save_pattern(
            Some(&json!({"name": "New Pattern", "task_type": "Architecture", "content": "# Hello\n"})),
            &store,
        );
        assert_ne!(result.get("isError"), Some(&json!(true)));
        let all = store.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "New Pattern");
    }

    #[test]
    fn browse_patterns_returns_root() {
        let store = temp_store();
        add_pattern(&store, "Foo", "Style Guide", true);

        let result = tool_browse_patterns(&store);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Agent Pattern Library"));
        assert!(text.contains("Foo"));
    }

    #[test]
    fn unix_days_to_date_epoch() {
        assert_eq!(unix_days_to_date(0), "1970-01-01");
    }

    #[test]
    fn unix_days_to_date_known() {
        // 2026-05-12 — days since epoch
        // 56 years (1970-2025): 56*365 + 14 leap days = 20440 + 14 = 20454 → 2026-01-01
        // Jan(31)+Feb(28)+Mar(31)+Apr(30)+May days 1-11 = 131 → 2026-05-12
        // 20454 + 131 = 20585
        assert_eq!(unix_days_to_date(20585), "2026-05-12");
    }
}
