use serde_json::{json, Value};

use crate::document::{Document, slugify};
use crate::store::DocumentStore;

/// Dispatch a `tools/call` request to the matching tool handler.
/// Returns the MCP `result` value to embed in the RpcResponse.
pub fn handle_tool_call(name: &str, params: Option<&Value>, store: &DocumentStore) -> Value {
    crate::stats::Stats::increment(&store.base_dir, name);
    match name {
        "lookup_specific" => tool_lookup_specific(params, store),
        "save_document" => tool_save_document(params, store),
        "update_document" => tool_update_document(params, store),
        "browse_documents" => tool_browse_documents(store),
        _ => {
            if let Ok(docs) = store.load_all()
                && let Some(doc) = docs
                    .iter()
                    .find(|d| d.is_skill && d.enabled && slugify(&d.name) == name)
            {
                return text_response(doc.content.clone());
            }
            json!({
                "content": [{ "type": "text", "text": format!("Unknown tool: {name}") }],
                "isError": true
            })
        }
    }
}

/// JSON Schema definitions for `tools/list`, including any enabled skill documents.
pub fn tool_definitions(store: &DocumentStore) -> Value {
    let mut tools = vec![
        json!({
            "name": "lookup_specific",
            "description": "Return a specific document by name.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The document name to look up"
                    }
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "save_document",
            "description": "Save a new document to the library.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Document name" },
                    "task_type": { "type": "string", "description": "Task type category" },
                    "content": { "type": "string", "description": "Document body in Markdown" }
                },
                "required": ["name", "task_type", "content"]
            }
        }),
        json!({
            "name": "update_document",
            "description": "Update an existing document by name. Supply at least one of content, task_type, enabled, or isSkill.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Exact document name to update (case-insensitive)" },
                    "content": { "type": "string", "description": "New document body in Markdown" },
                    "task_type": { "type": "string", "description": "New task type category" },
                    "enabled": { "type": "boolean", "description": "Enable or disable the document" },
                    "isSkill": { "type": "boolean", "description": "Expose as a callable MCP tool" }
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "browse_documents",
            "description": "Return the root index document listing all documents.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
    ];

    if let Ok(docs) = store.load_all() {
        for doc in docs.iter().filter(|d| d.is_skill && d.enabled) {
            tools.push(json!({
                "name": slugify(&doc.name),
                "description": doc.name.clone(),
                "inputSchema": { "type": "object", "properties": {} }
            }));
        }
    }

    json!(tools)
}

// ── Tool implementations ───────────────────────────────────────────────────

fn tool_lookup_specific(params: Option<&Value>, store: &DocumentStore) -> Value {
    let name = match params.and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
        Some(n) => n.to_lowercase(),
        None => return error_response("missing required parameter: name"),
    };

    match store.load_all() {
        Err(e) => error_response(e.to_string()),
        Ok(documents) => {
            match documents
                .into_iter()
                .find(|d| d.enabled && d.name.to_lowercase() == name)
            {
                None => text_response(format!("No enabled document found with name '{name}'.")),
                Some(d) => text_response(document_to_text(&d)),
            }
        }
    }
}

fn tool_save_document(params: Option<&Value>, store: &DocumentStore) -> Value {
    let params = match params {
        Some(p) => p,
        None => return error_response("missing parameters"),
    };

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) if !n.trim().is_empty() => n.trim().to_owned(),
        Some(_) => return error_response("name must not be empty"),
        None => return error_response("missing required parameter: name"),
    };
    let task_type = match params.get("task_type").and_then(|v| v.as_str()) {
        Some(t) if !t.trim().is_empty() => t.trim().to_owned(),
        Some(_) => return error_response("task_type must not be empty"),
        None => return error_response("missing required parameter: task_type"),
    };
    let content = match params.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_owned(),
        None => return error_response("missing required parameter: content"),
    };

    let today = current_date();
    let document = Document {
        name: name.clone(),
        enabled: true,
        date_added: today,
        task_type,
        is_skill: false,
        file_path: Default::default(),
        content,
    };

    match store.save_document(&document) {
        Ok(path) => text_response(format!(
            "Document '{name}' saved to {}",
            path.display()
        )),
        Err(e) => error_response(e.to_string()),
    }
}

fn tool_update_document(params: Option<&Value>, store: &DocumentStore) -> Value {
    let params = match params {
        Some(p) => p,
        None => return error_response("missing parameters"),
    };

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) if !n.trim().is_empty() => n.trim().to_lowercase(),
        Some(_) => return error_response("name must not be empty"),
        None => return error_response("missing required parameter: name"),
    };

    let new_content = params.get("content").and_then(|v| v.as_str());
    let new_task_type = params.get("task_type").and_then(|v| v.as_str());
    let new_enabled = params.get("enabled").and_then(|v| v.as_bool());
    let new_is_skill = params.get("isSkill").and_then(|v| v.as_bool());

    if new_content.is_none() && new_task_type.is_none() && new_enabled.is_none() && new_is_skill.is_none() {
        return error_response("at least one of content, task_type, enabled, or isSkill must be provided");
    }

    let mut documents = match store.load_all() {
        Ok(d) => d,
        Err(e) => return error_response(e.to_string()),
    };

    let document = match documents.iter_mut().find(|d| d.name.to_lowercase() == name) {
        Some(d) => d,
        None => return error_response(format!("no document found with name '{name}'")),
    };

    if let Some(c) = new_content {
        document.content = c.to_owned();
    }
    if let Some(t) = new_task_type {
        if t.trim().is_empty() {
            return error_response("task_type must not be empty");
        }
        document.task_type = t.trim().to_owned();
    }
    if let Some(e) = new_enabled {
        document.enabled = e;
    }
    if let Some(s) = new_is_skill {
        document.is_skill = s;
    }

    let display_name = document.name.clone();
    match store.update_document(document) {
        Ok(()) => text_response(format!("Document '{display_name}' updated.")),
        Err(e) => error_response(e.to_string()),
    }
}

fn tool_browse_documents(store: &DocumentStore) -> Value {
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

fn document_to_text(d: &Document) -> String {
    format!(
        "## {}\n**Task type:** {}  |  **Added:** {}  |  **Enabled:** {}\n\n{}",
        d.name,
        d.task_type,
        d.date_added,
        if d.enabled { "yes" } else { "no" },
        d.content
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
    // Saturate at i64::MAX rather than wrap on a runaway clock; in practice
    // `days` is on the order of 2e4 today.
    let z = i64::try_from(days).unwrap_or(i64::MAX - 719468) + 719468;
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
    use crate::document::Document;
    use std::path::PathBuf;

    fn temp_store() -> DocumentStore {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join(format!("agent-library-tools-test-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        DocumentStore::new(dir)
    }

    fn add_document(store: &DocumentStore, name: &str, task_type: &str, enabled: bool) {
        add_skill_document(store, name, task_type, enabled, false);
    }

    fn add_skill_document(
        store: &DocumentStore,
        name: &str,
        task_type: &str,
        enabled: bool,
        is_skill: bool,
    ) {
        let d = Document {
            name: name.to_owned(),
            enabled,
            date_added: "2026-05-12".to_owned(),
            task_type: task_type.to_owned(),
            is_skill,
            file_path: PathBuf::new(),
            content: format!("# {name}\n\nSkill content for {name}.\n"),
        };
        store.save_document(&d).unwrap();
    }

    #[test]
    fn lookup_specific_finds_by_name() {
        let store = temp_store();
        add_document(&store, "REST API Design", "Architecture", true);

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
        assert!(text.contains("No enabled document found"));
    }

    #[test]
    fn lookup_specific_skips_disabled() {
        let store = temp_store();
        add_document(&store, "Old Document", "Architecture", false);
        let result = tool_lookup_specific(Some(&json!({"name": "old document"})), &store);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("No enabled document found"));
    }

    #[test]
    fn save_document_creates_file() {
        let store = temp_store();
        let result = tool_save_document(
            Some(&json!({"name": "New Document", "task_type": "Architecture", "content": "# Hello\n"})),
            &store,
        );
        assert_ne!(result.get("isError"), Some(&json!(true)));
        let all = store.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "New Document");
    }

    #[test]
    fn browse_documents_returns_root() {
        let store = temp_store();
        add_document(&store, "Foo", "Style Guide", true);

        let result = tool_browse_documents(&store);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Agent Document Library"));
        assert!(text.contains("Foo"));
    }

    #[test]
    fn update_document_changes_content() {
        let store = temp_store();
        add_document(&store, "REST API Design", "Architecture", true);

        let result = tool_update_document(
            Some(&json!({"name": "REST API Design", "content": "# Updated\n\nNew content.\n"})),
            &store,
        );
        assert_ne!(result.get("isError"), Some(&json!(true)));
        let all = store.load_all().unwrap();
        assert!(all[0].content.contains("New content."));
    }

    #[test]
    fn update_document_toggles_enabled() {
        let store = temp_store();
        add_document(&store, "REST API Design", "Architecture", true);

        tool_update_document(Some(&json!({"name": "REST API Design", "enabled": false})), &store);
        let all = store.load_all().unwrap();
        assert!(!all[0].enabled);
    }

    #[test]
    fn update_document_not_found() {
        let store = temp_store();
        let result = tool_update_document(
            Some(&json!({"name": "Nonexistent", "content": "x"})),
            &store,
        );
        assert_eq!(result.get("isError"), Some(&json!(true)));
    }

    #[test]
    fn update_document_requires_at_least_one_field() {
        let store = temp_store();
        add_document(&store, "REST API Design", "Architecture", true);
        let result = tool_update_document(Some(&json!({"name": "REST API Design"})), &store);
        assert_eq!(result.get("isError"), Some(&json!(true)));
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

    #[test]
    fn skill_tool_appears_in_definitions() {
        let store = temp_store();
        add_skill_document(&store, "REST API Design", "Architecture", true, true);

        let defs = tool_definitions(&store);
        let tools = defs.as_array().unwrap();
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        assert!(names.contains(&"rest-api-design"), "skill slug missing from tool list: {names:?}");
    }

    #[test]
    fn skill_tool_call_returns_content() {
        let store = temp_store();
        add_skill_document(&store, "REST API Design", "Architecture", true, true);

        let result = handle_tool_call("rest-api-design", None, &store);
        assert_ne!(result.get("isError"), Some(&json!(true)));
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("REST API Design"));
    }

    #[test]
    fn disabled_skill_not_exposed() {
        let store = temp_store();
        add_skill_document(&store, "Hidden Skill", "Architecture", false, true);

        let defs = tool_definitions(&store);
        let tools = defs.as_array().unwrap();
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        assert!(!names.contains(&"hidden-skill"), "disabled skill must not appear in tool list");

        let result = handle_tool_call("hidden-skill", None, &store);
        assert_eq!(result.get("isError"), Some(&json!(true)));
    }

    #[test]
    fn non_skill_doc_not_exposed() {
        let store = temp_store();
        add_document(&store, "Plain Document", "Architecture", true);

        let defs = tool_definitions(&store);
        let tools = defs.as_array().unwrap();
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        assert!(!names.contains(&"plain-document"));
    }
}
