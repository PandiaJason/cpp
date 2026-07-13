//! JSON-RPC 2.0 wire format message structures.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 version identifier. Always "2.0".
pub const JSONRPC_VERSION: &str = "2.0";

/// Unique identifier for a JSON-RPC request or response.
///
/// Can be a string or an integer.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageId {
    /// String identifier.
    String(String),
    /// Numeric identifier.
    Number(i64),
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Number(n) => write!(f, "{}", n),
        }
    }
}

/// A JSON-RPC 2.0 request message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcRequest {
    /// Always "2.0".
    pub jsonrpc: String,
    /// Request identifier.
    pub id: MessageId,
    /// Method name.
    pub method: String,
    /// Request parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    /// Helper to create a new request.
    pub fn new(id: MessageId, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// A JSON-RPC 2.0 response message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcResponse {
    /// Always "2.0".
    pub jsonrpc: String,
    /// Corresponding request identifier.
    pub id: MessageId,
    /// Successful result value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error object, if the request failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Helper to create a successful response.
    pub fn success(id: MessageId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Helper to create an error response.
    pub fn error(id: MessageId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// A JSON-RPC 2.0 error object.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcError {
    /// Numeric error code.
    pub code: i32,
    /// Short error message.
    pub message: String,
    /// Optional structured error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn new(code: i32, message: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }
}

/// A JSON-RPC 2.0 notification message (no ID).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonRpcNotification {
    /// Always "2.0".
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Notification parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcNotification {
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
        }
    }
}

/// Unified message enum for parsing/routing incoming JSON-RPC traffic.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_request() {
        let req = JsonRpcRequest::new(
            MessageId::Number(42),
            "cpp/query",
            Some(serde_json::json!({"foo": "bar"})),
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":42"));
        assert!(json.contains("\"method\":\"cpp/query\""));
    }

    #[test]
    fn deserialize_message() {
        let raw = r#"{"jsonrpc":"2.0","method":"cpp/initialized"}"#;
        let msg: JsonRpcMessage = serde_json::from_str(raw).unwrap();
        match msg {
            JsonRpcMessage::Notification(n) => {
                assert_eq!(n.method, "cpp/initialized");
            }
            _ => panic!("Expected notification"),
        }
    }
}
