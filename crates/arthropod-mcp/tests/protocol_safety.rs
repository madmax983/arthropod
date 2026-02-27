use arthropod_mcp::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::{Value, json};

#[test]
fn test_application_error_clamping() {
    // Should clamp -32100 to -32099 (min valid)
    let err_low = JsonRpcError::application_error(-32100, "Too low");
    assert_eq!(
        err_low.code, -32099,
        "Should clamp codes < -32099 to -32099"
    );

    // Should clamp -31999 to -32000 (max valid)
    let err_high = JsonRpcError::application_error(-31999, "Too high");
    assert_eq!(
        err_high.code, -32000,
        "Should clamp codes > -32000 to -32000"
    );

    // Should preserve valid codes
    let err_valid = JsonRpcError::application_error(-32050, "Valid");
    assert_eq!(err_valid.code, -32050, "Should preserve valid codes");
}

#[test]
fn test_response_validation() {
    // Both result and error missing -> Invalid
    let json_both_missing = r#"{"jsonrpc": "2.0", "id": 1}"#;
    let resp_missing: JsonRpcResponse = serde_json::from_str(json_both_missing).unwrap();
    assert!(
        resp_missing.validate().is_err(),
        "Should detect missing result AND error"
    );

    // Both result and error present -> Invalid
    let json_both_present = r#"{"jsonrpc": "2.0", "id": 1, "result": "ok", "error": {"code": -32600, "message": "err"}}"#;
    let resp_present: JsonRpcResponse = serde_json::from_str(json_both_present).unwrap();
    assert!(
        resp_present.validate().is_err(),
        "Should detect result AND error both present"
    );

    // Valid result -> OK
    let json_result = r#"{"jsonrpc": "2.0", "id": 1, "result": "ok"}"#;
    let resp_result: JsonRpcResponse = serde_json::from_str(json_result).unwrap();
    assert!(resp_result.validate().is_ok(), "Should accept valid result");

    // Valid error -> OK
    let json_error = r#"{"jsonrpc": "2.0", "id": 1, "error": {"code": -32600, "message": "err"}}"#;
    let resp_error: JsonRpcResponse = serde_json::from_str(json_error).unwrap();
    assert!(resp_error.validate().is_ok(), "Should accept valid error");
}

#[test]
fn test_request_validation() {
    // Valid params (Array) -> OK
    let req_array = JsonRpcRequest::new(1, "test", json!([1, 2, 3]));
    assert!(req_array.validate().is_ok(), "Should accept Array params");

    // Valid params (Object) -> OK
    let req_obj = JsonRpcRequest::new(1, "test", json!({"a": 1}));
    assert!(req_obj.validate().is_ok(), "Should accept Object params");

    // Valid params (Null/Omitted) -> OK (Technically spec allows omitted, serde treats as Null if default)
    // Wait, JsonRpcRequest defines params as `Value`, default is `Null`.
    // Spec: "This member MAY be omitted."
    // If omitted, it's effectively null in our struct.
    // However, if present, it must be Array or Object.
    // If deserialized from `{"method": "foo"}`, params is Null.
    // Is Null allowed as a value for "params"?
    // "Structured value" usually means Array or Object.
    // Let's assume Null is acceptable for "omitted".
    let req_null = JsonRpcRequest::new(1, "test", Value::Null);
    assert!(
        req_null.validate().is_ok(),
        "Should accept Null params (omitted)"
    );

    // Invalid params (String) -> Err
    let req_str = JsonRpcRequest::new(1, "test", json!("invalid"));
    assert!(req_str.validate().is_err(), "Should reject String params");

    // Invalid params (Number) -> Err
    let req_num = JsonRpcRequest::new(1, "test", json!(123));
    assert!(req_num.validate().is_err(), "Should reject Number params");
}
