use serde::Serialize;
use serde::ser::Error;

struct ExploitStruct;

impl Serialize for ExploitStruct {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(S::Error::custom("Exploit failed serialization!"))
    }
}

#[test]
fn test_serde_json_unwrap_safety_exploit() {
    let result = serde_json::to_string_pretty(&ExploitStruct)
        .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None));

    assert!(
        result.is_err(),
        "Serialization of invalid data should return an error, not panic"
    );
    assert!(result.unwrap_err().message.contains("Exploit failed"));
}
