use serde::{Deserialize, Serialize};
use serde_json::Value;

// JSON-RPC 2.0 error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;

#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub result: Value,
}

#[derive(Debug, Serialize)]
pub struct RpcError {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub error: RpcErrorBody,
}

#[derive(Debug, Serialize)]
pub struct RpcErrorBody {
    pub code: i32,
    pub message: String,
}

impl RpcResponse {
    pub fn new(id: Value, result: Value) -> Self {
        RpcResponse { jsonrpc: "2.0", id, result }
    }
}

impl RpcError {
    pub fn new(id: Value, code: i32, message: impl Into<String>) -> Self {
        RpcError {
            jsonrpc: "2.0",
            id,
            error: RpcErrorBody { code, message: message.into() },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_request_with_null_id() {
        let raw = r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":null}"#;
        let req: RpcRequest = serde_json::from_str(raw).unwrap();
        assert!(req.id.is_none());
        assert_eq!(req.method, "notifications/initialized");
    }

    #[test]
    fn deserialize_request_with_string_id() {
        let raw = r#"{"jsonrpc":"2.0","id":"abc","method":"tools/list","params":null}"#;
        let req: RpcRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.id, Some(json!("abc")));
    }

    #[test]
    fn deserialize_request_with_integer_id() {
        let raw = r#"{"jsonrpc":"2.0","id":42,"method":"tools/list","params":null}"#;
        let req: RpcRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.id, Some(json!(42)));
    }

    #[test]
    fn serialize_response_shape() {
        let resp = RpcResponse::new(json!(1), json!({"ok": true}));
        let s = serde_json::to_string(&resp).unwrap();
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], 1);
        assert_eq!(v["result"]["ok"], true);
    }

    #[test]
    fn serialize_error_shape() {
        let err = RpcError::new(json!(1), METHOD_NOT_FOUND, "not found");
        let s = serde_json::to_string(&err).unwrap();
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["error"]["code"], METHOD_NOT_FOUND);
        assert_eq!(v["error"]["message"], "not found");
    }
}
