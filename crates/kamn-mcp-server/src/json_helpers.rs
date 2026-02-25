//! Shared JSON helper utilities for MCP protocol and dispatch surfaces.

use serde_json::Value;

/// Escapes one string for JSON embedding and strips surrounding quotes.
pub(crate) fn escape_json(input: &str) -> String {
    match serde_json::to_string(input) {
        Ok(serialized) => {
            if serialized.starts_with('"') && serialized.ends_with('"') && serialized.len() >= 2 {
                return serialized[1..serialized.len() - 1].to_owned();
            }
            serialized
        }
        Err(_) => String::new(),
    }
}

/// Looks up one field value across root, params, and arguments objects.
pub(crate) fn json_field_value<'a>(root: &'a Value, key: &str) -> Option<&'a Value> {
    root.get(key)
        .or_else(|| root.get("params").and_then(|value| value.get(key)))
        .or_else(|| {
            root.get("params")
                .and_then(|value| value.get("arguments"))
                .and_then(|value| value.get(key))
        })
        .or_else(|| root.get("arguments").and_then(|value| value.get(key)))
}

/// Parses one payload and reads an optional string field.
pub(crate) fn json_optional_string_field(payload: &str, key: &str) -> Option<String> {
    let root = serde_json::from_str::<Value>(payload).ok()?;
    let value = json_field_value(&root, key)?;
    value.as_str().map(str::to_owned)
}

/// Parses one payload and reads one required string field.
pub(crate) fn json_required_string_field(payload: &str, key: &str) -> Result<String, String> {
    json_optional_string_field(payload, key).ok_or_else(|| format!("missing required field: {key}"))
}

/// Parses one payload and reads an optional `u64` from either number or string token.
pub(crate) fn json_optional_u64_field(payload: &str, key: &str) -> Option<u64> {
    let root = serde_json::from_str::<Value>(payload).ok()?;
    let value = json_field_value(&root, key)?;
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(raw) => raw.parse::<u64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{escape_json, json_field_value};

    #[test]
    fn unit_json_helpers_escape_json_handles_control_characters() {
        let escaped = escape_json("\"\\\n\r\t");
        assert_eq!(escaped, "\\\"\\\\\\n\\r\\t");
    }

    #[test]
    fn unit_json_helpers_json_field_value_resolves_nested_argument_keys() {
        let root = serde_json::from_str::<serde_json::Value>(
            r#"{"params":{"arguments":{"payload":"ok"}}}"#,
        )
        .expect("json");
        assert_eq!(
            json_field_value(&root, "payload").and_then(|value| value.as_str()),
            Some("ok")
        );
    }
}
