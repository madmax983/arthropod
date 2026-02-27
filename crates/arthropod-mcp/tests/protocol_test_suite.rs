use arthropod_mcp::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;
use serde_json::json;

#[test]
fn test_request_serialization_round_trip() {
    let request = JsonRpcRequest::new(1, "test.method", json!({"key": "value"}));

    let serialized = serde_json::to_string(&request).expect("Failed to serialize request");
    let deserialized: JsonRpcRequest =
        serde_json::from_str(&serialized).expect("Failed to deserialize request");

    assert_eq!(request, deserialized);
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "test.method");
}

#[test]
fn test_request_with_null_id() {
    let request = JsonRpcRequest::new(Value::Null, "notify", Value::Null);
    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("\"id\":null"));
}

#[test]
fn test_response_success_round_trip() {
    let response = JsonRpcResponse::success(1, json!({"status": "ok"}));

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(response, deserialized);
    assert!(deserialized.result.is_some());
    assert!(deserialized.error.is_none());
}

#[test]
fn test_response_error_round_trip() {
    let error = JsonRpcError::method_not_found("unknown");
    let response = JsonRpcResponse::error(1, error.clone());

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(response, deserialized);
    assert!(deserialized.result.is_none());
    assert!(deserialized.error.is_some());
    assert_eq!(deserialized.error.unwrap(), error);
}

#[test]
fn test_error_with_data() {
    let mut error = JsonRpcError::invalid_params("bad input");
    error.data = Some(json!({"details": "extra info"}));

    let serialized = serde_json::to_string(&error).unwrap();
    let deserialized: JsonRpcError = serde_json::from_str(&serialized).unwrap();

    assert_eq!(error, deserialized);
    assert!(deserialized.data.is_some());
    assert_eq!(deserialized.data.unwrap(), json!({"details": "extra info"}));
}

#[test]
fn test_application_error_clamp_low() {
    // Should clamp -32100 to -32099 (min valid)
    let err = JsonRpcError::application_error(-32100, "Too low");
    assert_eq!(err.code, -32099);
}

#[test]
fn test_application_error_clamp_high() {
    // Should clamp -31999 to -32000 (max valid)
    let err = JsonRpcError::application_error(-31999, "Too high");
    assert_eq!(err.code, -32000);
}

#[test]
fn test_application_error_valid() {
    let err = JsonRpcError::application_error(-32000, "Just right");
    assert_eq!(err.code, -32000);

    let err = JsonRpcError::application_error(-32099, "Just right");
    assert_eq!(err.code, -32099);
}

#[test]
fn test_deserialize_response_with_both_result_and_error() {
    // JSON-RPC 2.0 spec says ONLY one should be present, but our struct allows both optionally.
    // This test verifies the current behavior (likely accepts it).
    let json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": "success",
        "error": {
            "code": -32603,
            "message": "Internal error"
        }
    });

    let response: JsonRpcResponse = serde_json::from_value(json).unwrap();

    // Both should be present in the struct
    assert!(response.result.is_some());
    assert!(response.error.is_some());
}

#[test]
fn test_deserialize_response_with_neither_result_nor_error() {
    // This is also invalid per spec, but structurally possible
    let json = json!({
        "jsonrpc": "2.0",
        "id": 1
    });

    let response: JsonRpcResponse = serde_json::from_value(json).unwrap();

    assert!(response.result.is_none());
    assert!(response.error.is_none());
}
