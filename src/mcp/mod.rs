mod tools;
mod types;

use std::io::{BufRead, BufReader, Read, Write};

use serde_json::{json, Value};

use crate::error::AppError;
use crate::store::PatternStore;
use types::{INVALID_REQUEST, METHOD_NOT_FOUND, PARSE_ERROR, RpcError, RpcRequest, RpcResponse};

/// Maximum bytes allowed in a single JSON-RPC line. A malicious or buggy
/// peer that streams bytes without a newline would otherwise grow the read
/// buffer until OOM. 8 MiB is well above any realistic pattern payload.
const MAX_LINE_BYTES: u64 = 8 * 1024 * 1024;

pub fn run_server(store: PatternStore) -> Result<(), AppError> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());

    loop {
        let mut line = String::new();
        let n = (&mut reader)
            .take(MAX_LINE_BYTES)
            .read_line(&mut line)?;
        if n == 0 {
            break; // EOF — host closed the pipe
        }
        // If we hit the cap without seeing '\n', drop the rest of the line
        // and reply with a parse error so the peer can recover.
        let over_limit = n as u64 == MAX_LINE_BYTES && !line.ends_with('\n');
        if over_limit {
            // Consume the rest of the oversized line.
            let mut sink = [0u8; 4096];
            loop {
                match reader.read(&mut sink) {
                    Ok(0) => break,
                    Ok(n) if sink[..n].contains(&b'\n') => break,
                    Ok(_) => continue,
                    Err(e) => return Err(e.into()),
                }
            }
            let err = RpcError::new(Value::Null, PARSE_ERROR, "request line exceeds maximum size");
            let resp = serde_json::to_string(&err).unwrap_or_default();
            let mut out = stdout.lock();
            writeln!(out, "{resp}")?;
            out.flush()?;
            continue;
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

    // JSON-RPC 2.0 spec: the jsonrpc field MUST be exactly "2.0".
    if req.jsonrpc != "2.0" {
        let id = req.id.clone().unwrap_or(Value::Null);
        let err = RpcError::new(id, INVALID_REQUEST, "jsonrpc field must be \"2.0\"");
        return Some(serde_json::to_string(&err).unwrap_or_default());
    }

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
