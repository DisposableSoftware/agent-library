mod tools;
mod types;

use std::io::{BufRead, BufReader, Write};

use serde_json::{json, Value};

use crate::error::AppError;
use crate::store::PatternStore;
use types::{METHOD_NOT_FOUND, PARSE_ERROR, RpcError, RpcRequest, RpcResponse};

pub fn run_server(store: PatternStore) -> Result<(), AppError> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break; // EOF — host closed the pipe
        }
        let line = line.trim_end_matches(['\n', '\r']);
        if line.is_empty() {
            continue;
        }

        let response = handle_line(line, &store);
        if let Some(resp_str) = response {
            let mut out = stdout.lock();
            writeln!(out, "{resp_str}")?;
            out.flush()?;
        }
    }

    Ok(())
}

fn handle_line(line: &str, store: &PatternStore) -> Option<String> {
    let req: RpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            let err = RpcError::new(Value::Null, PARSE_ERROR, e.to_string());
            return Some(serde_json::to_string(&err).unwrap_or_default());
        }
    };

    // Notifications have no id — do not respond
    let id = req.id.clone()?;

    let response_str = match req.method.as_str() {
        "initialize" => {
            let result = json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "agent-library",
                    "version": env!("CARGO_PKG_VERSION")
                }
            });
            serde_json::to_string(&RpcResponse::new(id, result)).unwrap_or_default()
        }
        "tools/list" => {
            let result = json!({ "tools": tools::tool_definitions() });
            serde_json::to_string(&RpcResponse::new(id, result)).unwrap_or_default()
        }
        "tools/call" => {
            let params = req.params.as_ref();
            let name = params
                .and_then(|p| p.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tool_params = params.and_then(|p| p.get("arguments"));
            let result = tools::handle_tool_call(name, tool_params, store);
            serde_json::to_string(&RpcResponse::new(id, result)).unwrap_or_default()
        }
        other => {
            let err = RpcError::new(
                id,
                METHOD_NOT_FOUND,
                format!("method not found: {other}"),
            );
            serde_json::to_string(&err).unwrap_or_default()
        }
    };

    Some(response_str)
}
