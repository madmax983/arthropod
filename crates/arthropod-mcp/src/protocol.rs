//! JSON-RPC 2.0 protocol types for MCP communication.
//!
//! This module defines the core protocol types used for stdio-based
//! communication with the MCP server.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// Protocol version (must be "2.0")
    pub jsonrpc: String,

    /// Request ID (can be string, number, or null)
    pub id: Value,

    /// Method name (e.g., "scene.list_nodes")
    pub method: String,

    /// Method parameters (optional)
    #[serde(default)]
    pub params: Value,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(id: impl Into<Value>, method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 successful response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// Protocol version (must be "2.0")
    pub jsonrpc: String,

    /// Request ID (matches request)
    pub id: Value,

    /// Result data (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error data (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a successful response
    pub fn success(id: impl Into<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: impl Into<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Parse error (-32700)
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    /// Invalid request (-32600)
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: message.into(),
            data: None,
        }
    }

    /// Method not found (-32601)
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    /// Invalid params (-32602)
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
            data: None,
        }
    }

    /// Internal error (-32603)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: message.into(),
            data: None,
        }
    }

    /// Custom application error (code >= -32000)
    pub fn application_error(code: i32, message: impl Into<String>) -> Self {
        assert!(
            (-32099..=-32000).contains(&code),
            "Application error codes must be in range [-32099, -32000]"
        );
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_serialization() {
        let request = JsonRpcRequest::new(
            1,
            "scene.list_nodes",
            json!({"filter": {"visible_only": true}}),
        );

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request, deserialized);
        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.method, "scene.list_nodes");
    }

    #[test]
    fn test_success_response_serialization() {
        let response = JsonRpcResponse::success(1, json!({"nodes": []}));

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response, deserialized);
        assert!(deserialized.result.is_some());
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_error_response_serialization() {
        let response = JsonRpcResponse::error(1, JsonRpcError::method_not_found("test.method"));

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response, deserialized);
        assert!(deserialized.result.is_none());
        assert!(deserialized.error.is_some());
        assert_eq!(deserialized.error.unwrap().code, -32601);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(JsonRpcError::parse_error().code, -32700);
        assert_eq!(JsonRpcError::invalid_request("test").code, -32600);
        assert_eq!(JsonRpcError::method_not_found("test").code, -32601);
        assert_eq!(JsonRpcError::invalid_params("test").code, -32602);
        assert_eq!(JsonRpcError::internal_error("test").code, -32603);
    }

    #[test]
    fn test_request_with_null_params() {
        let request = JsonRpcRequest::new(1, "test.method", Value::Null);

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"params\":null"));
    }

    #[test]
    fn test_request_round_trip() {
        let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"scene.list_nodes","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, json!(1));
        assert_eq!(request.method, "scene.list_nodes");
    }
}
