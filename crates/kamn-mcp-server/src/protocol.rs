use crate::dispatch_tool_request_json;
use crate::invalid_request_response_json;
#[cfg(test)]
use crate::json_helpers::json_required_string_field;
use crate::json_helpers::{escape_json, json_field_value};
use crate::tools::build_tool_registry;
use crate::McpToolBackend;
use serde_json::Value;

const JSONRPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2025-06-18";
const JSONRPC_PARSE_ERROR: i32 = -32700;
const JSONRPC_INVALID_REQUEST: i32 = -32600;
const JSONRPC_METHOD_NOT_FOUND: i32 = -32601;
const JSONRPC_INVALID_PARAMS: i32 = -32602;
const JSONRPC_INTERNAL_ERROR: i32 = -32603;

/// Processes stdin payload for either framed MCP JSON-RPC or legacy line mode.
pub fn process_stdio_input<B: McpToolBackend>(
    backend: &B,
    input: &str,
) -> Result<Vec<String>, String> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    if appears_framed(input) {
        process_framed_input(backend, input)
    } else {
        Ok(process_line_mode_input(backend, input))
    }
}

fn appears_framed(input: &str) -> bool {
    input
        .lines()
        .map(str::trim_start)
        .any(|line| line.starts_with("Content-Length:"))
}

fn process_line_mode_input<B: McpToolBackend>(backend: &B, input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| match dispatch_tool_request_json(backend, line) {
            Ok(response) => response,
            Err(error) => invalid_request_response_json(error.as_str()),
        })
        .collect::<Vec<_>>()
}

fn process_framed_input<B: McpToolBackend>(
    backend: &B,
    input: &str,
) -> Result<Vec<String>, String> {
    let requests = match decode_framed_payloads(input) {
        Ok(requests) => requests,
        Err(error) => {
            let response = jsonrpc_error_with_id(
                "null",
                JSONRPC_PARSE_ERROR,
                format!("invalid MCP frame: {error}").as_str(),
            );
            return Ok(vec![frame_jsonrpc_response(response.as_str())]);
        }
    };

    let mut responses = Vec::with_capacity(requests.len());
    for request in requests {
        let response_json = process_jsonrpc_request(backend, request.as_str());
        responses.push(frame_jsonrpc_response(response_json.as_str()));
    }
    Ok(responses)
}

fn process_jsonrpc_request<B: McpToolBackend>(backend: &B, request_json: &str) -> String {
    let request = match decode_jsonrpc_request(request_json) {
        Ok(request) => request,
        Err(JsonRpcDecodeError::ParseError) => {
            return jsonrpc_error_with_id("null", JSONRPC_PARSE_ERROR, "parse error");
        }
        Err(JsonRpcDecodeError::InvalidRequest { id_token, message }) => {
            return jsonrpc_error_with_id(id_token.as_str(), JSONRPC_INVALID_REQUEST, message);
        }
    };

    match request.method.as_str() {
        "initialize" => {
            let result = format!(
                "{{\"protocolVersion\":\"{}\",\"capabilities\":{{\"tools\":{{\"listChanged\":false}}}},\"serverInfo\":{{\"name\":\"kamn-mcp-server\",\"version\":\"{}\"}}}}",
                MCP_PROTOCOL_VERSION,
                escape_json(env!("CARGO_PKG_VERSION")),
            );
            jsonrpc_success_with_id(request.id_token.as_str(), result.as_str())
        }
        "tools/list" => {
            let tools = build_tool_registry()
                .into_iter()
                .map(|tool| {
                    format!(
                        "{{\"name\":\"{}\",\"description\":\"{}\",\"inputSchema\":{},\"outputSchema\":{}}}",
                        escape_json(tool.name),
                        escape_json(tool.description),
                        tool.input_schema,
                        tool.output_schema,
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let result = format!("{{\"tools\":[{tools}]}}");
            jsonrpc_success_with_id(request.id_token.as_str(), result.as_str())
        }
        "tools/call" => process_tools_call(backend, &request),
        _ => jsonrpc_error_with_id(
            request.id_token.as_str(),
            JSONRPC_METHOD_NOT_FOUND,
            "method not found",
        ),
    }
}

fn process_tools_call<B: McpToolBackend>(backend: &B, request: &JsonRpcRequest) -> String {
    let tool_name = match json_string_field(request.root(), "name") {
        Ok(name) => name,
        Err(_) => {
            return jsonrpc_error_with_id(
                request.id_token.as_str(),
                JSONRPC_INVALID_PARAMS,
                "tools/call requires params.name",
            );
        }
    };

    let dispatch_request = build_dispatch_payload(
        request.id_token.as_str(),
        tool_name.as_str(),
        request.root(),
    );
    let dispatch_response = match dispatch_tool_request_json(backend, dispatch_request.as_str()) {
        Ok(response) => response,
        Err(error) => {
            return jsonrpc_error_with_id(
                request.id_token.as_str(),
                JSONRPC_INVALID_PARAMS,
                error.as_str(),
            );
        }
    };

    if dispatch_response.contains("\"ok\":true") {
        return jsonrpc_success_with_id(request.id_token.as_str(), dispatch_response.as_str());
    }

    if dispatch_response.contains("\"kind\":\"invalid_request\"") {
        return jsonrpc_error_with_id(
            request.id_token.as_str(),
            JSONRPC_INVALID_PARAMS,
            "invalid tool request payload",
        );
    }

    if dispatch_response.contains("\"kind\":\"unsupported_operation\"") {
        return jsonrpc_error_with_id(
            request.id_token.as_str(),
            JSONRPC_METHOD_NOT_FOUND,
            "tool not supported",
        );
    }

    jsonrpc_error_with_id(
        request.id_token.as_str(),
        JSONRPC_INTERNAL_ERROR,
        "tool backend error",
    )
}

fn build_dispatch_payload(id_token: &str, tool_name: &str, root: &Value) -> String {
    let dispatch_id = normalize_id_for_dispatch(id_token);
    let mut fields = vec![
        format!("\"id\":\"{}\"", escape_json(dispatch_id.as_str())),
        format!("\"tool\":\"{}\"", escape_json(tool_name)),
    ];

    for key in [
        "payload",
        "channel_id",
        "message_id",
        "did",
        "task_id",
        "content_id",
        "bridge_id",
        "escrow_id",
        "tx_hash",
        "finality",
    ] {
        if let Some(value) = json_optional_string_value(root, key) {
            fields.push(format!("\"{key}\":\"{}\"", escape_json(value.as_str())));
        }
    }

    if let Some(block_height) = json_optional_u64_value(root, "block_height") {
        fields.push(format!("\"block_height\":\"{block_height}\""));
    } else if let Some(block_height_raw) = json_optional_string_value(root, "block_height") {
        fields.push(format!(
            "\"block_height\":\"{}\"",
            escape_json(block_height_raw.as_str())
        ));
    }

    format!("{{{}}}", fields.join(","))
}

fn normalize_id_for_dispatch(id_token: &str) -> String {
    let trimmed = id_token.trim();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        return unescape_json_scalar(&trimmed[1..trimmed.len() - 1]);
    }
    if trimmed.eq("null") || trimmed.is_empty() {
        return "request-unknown".to_owned();
    }
    trimmed.to_owned()
}

struct JsonRpcRequest {
    id_token: String,
    method: String,
    root: Value,
}

impl JsonRpcRequest {
    fn root(&self) -> &Value {
        &self.root
    }
}

enum JsonRpcDecodeError {
    ParseError,
    InvalidRequest {
        id_token: String,
        message: &'static str,
    },
}

fn decode_jsonrpc_request(request_json: &str) -> Result<JsonRpcRequest, JsonRpcDecodeError> {
    let root =
        serde_json::from_str::<Value>(request_json).map_err(|_| JsonRpcDecodeError::ParseError)?;
    let id_token = json_id_token(&root);
    let method = root
        .as_object()
        .and_then(|object| object.get("method"))
        .and_then(Value::as_str)
        .ok_or(JsonRpcDecodeError::InvalidRequest {
            id_token: id_token.clone(),
            message: "missing required field: method",
        })?
        .to_owned();

    Ok(JsonRpcRequest {
        id_token,
        method,
        root,
    })
}

fn json_id_token(root: &Value) -> String {
    root.as_object()
        .and_then(|object| object.get("id"))
        .and_then(|id| serde_json::to_string(id).ok())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_string_field(root: &Value, key: &str) -> Result<String, String> {
    json_optional_string_value(root, key).ok_or_else(|| format!("missing required field: {key}"))
}

fn json_optional_string_value(root: &Value, key: &str) -> Option<String> {
    json_field_value(root, key)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn json_optional_u64_value(root: &Value, key: &str) -> Option<u64> {
    match json_field_value(root, key)? {
        Value::Number(number) => number.as_u64(),
        Value::String(raw) => raw.parse::<u64>().ok(),
        _ => None,
    }
}

fn unescape_json_scalar(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            match ch {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                other => result.push(other),
            }
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        result.push(ch);
    }

    if escaped {
        result.push('\\');
    }

    result
}

fn frame_jsonrpc_response(response_json: &str) -> String {
    format!(
        "Content-Length: {}\r\n\r\n{}",
        response_json.len(),
        response_json
    )
}

fn decode_framed_payloads(input: &str) -> Result<Vec<String>, String> {
    let mut payloads = Vec::new();
    let mut cursor = 0usize;
    let bytes = input.as_bytes();

    while cursor < bytes.len() {
        while cursor < bytes.len() && (bytes[cursor] == b'\r' || bytes[cursor] == b'\n') {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }

        let remaining = &input[cursor..];
        let (header_end, separator_len) = if let Some(value) = remaining.find("\r\n\r\n") {
            (value, 4usize)
        } else if let Some(value) = remaining.find("\n\n") {
            (value, 2usize)
        } else {
            return Err("missing header terminator".to_owned());
        };
        let header = &remaining[..header_end];
        let content_length = parse_content_length(header)?;

        let payload_start = cursor + header_end + separator_len;
        let payload_end = payload_start + content_length;
        let payload = input
            .get(payload_start..payload_end)
            .ok_or_else(|| "content-length exceeds available payload bytes".to_owned())?;
        payloads.push(payload.to_owned());

        cursor = payload_end;
    }

    if payloads.is_empty() {
        return Err("no framed payloads found".to_owned());
    }
    Ok(payloads)
}

fn parse_content_length(header: &str) -> Result<usize, String> {
    for raw_line in header.lines() {
        let line = raw_line.trim();
        if line.to_ascii_lowercase().starts_with("content-length:") {
            let value = line
                .split_once(':')
                .map(|(_, rhs)| rhs.trim())
                .ok_or_else(|| "invalid content-length header".to_owned())?;
            return value
                .parse::<usize>()
                .map_err(|_| "content-length must be numeric".to_owned());
        }
    }
    Err("missing content-length header".to_owned())
}

fn jsonrpc_success_with_id(id_token: &str, result_json: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"{}\",\"id\":{},\"result\":{}}}",
        JSONRPC_VERSION, id_token, result_json
    )
}

fn jsonrpc_error_with_id(id_token: &str, code: i32, message: &str) -> String {
    format!(
        "{{\"jsonrpc\":\"{}\",\"id\":{},\"error\":{{\"code\":{},\"message\":\"{}\"}}}}",
        JSONRPC_VERSION,
        id_token,
        code,
        escape_json(message),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        decode_framed_payloads, escape_json, json_optional_u64_value, json_required_string_field,
        normalize_id_for_dispatch, parse_content_length,
    };

    #[test]
    fn spec_c01_protocol_normalize_id_contract_handles_quoted_numeric_null_and_empty() {
        assert_eq!(normalize_id_for_dispatch("\"req-1\""), "req-1");
        assert_eq!(normalize_id_for_dispatch("42"), "42");
        assert_eq!(
            normalize_id_for_dispatch("\"req-1"),
            "\"req-1",
            "partially quoted id should not be unquoted",
        );
        assert_eq!(
            normalize_id_for_dispatch("req-1\""),
            "req-1\"",
            "partially quoted id should not be unquoted",
        );
        assert_eq!(
            normalize_id_for_dispatch("null"),
            "request-unknown",
            "null id token should map to fallback dispatch id",
        );
        assert_eq!(
            normalize_id_for_dispatch(""),
            "request-unknown",
            "empty id token should map to fallback dispatch id",
        );
    }

    #[test]
    fn spec_c01_protocol_normalize_id_contract_unescapes_quoted_json_id() {
        assert_eq!(
            normalize_id_for_dispatch("\"req-\\\\\\\"1\""),
            "req-\\\"1",
            "quoted id should decode escaped json quote sequences",
        );
    }

    #[test]
    fn spec_c02_protocol_parse_content_length_contract_accepts_and_rejects_expected_headers() {
        let parsed = parse_content_length("Content-Length: 17").expect("header should parse");
        assert_eq!(parsed, 17);
        assert!(
            parse_content_length("Content-Length: not-a-number").is_err(),
            "non-numeric content-length must be rejected",
        );
        assert!(
            parse_content_length("X-Other: 1").is_err(),
            "missing content-length must be rejected",
        );
    }

    #[test]
    fn spec_c02_protocol_decode_framed_payloads_contract_supports_single_and_multi_frame_streams() {
        let single = "Content-Length: 7\r\n\r\n{\"a\":1}";
        let payloads = decode_framed_payloads(single).expect("single frame should decode");
        assert_eq!(payloads, vec![r#"{"a":1}"#]);

        let multi = "Content-Length: 7\r\n\r\n{\"a\":1}\r\nContent-Length: 7\r\n\r\n{\"b\":2}";
        let payloads = decode_framed_payloads(multi).expect("multi-frame should decode");
        assert_eq!(payloads, vec![r#"{"a":1}"#, r#"{"b":2}"#]);
    }

    #[test]
    fn spec_c02_protocol_decode_framed_payloads_contract_rejects_invalid_shapes() {
        assert!(
            decode_framed_payloads("Content-Length: 8\r\n\r\n{\"a\":1}").is_err(),
            "mismatched content-length should fail",
        );
        assert!(
            decode_framed_payloads("Content-Length: 7\r\n{\"a\":1}").is_err(),
            "missing header terminator should fail",
        );
    }

    #[test]
    fn spec_c03_protocol_json_optional_u64_contract_supports_numeric_and_quoted_forms() {
        let numeric = serde_json::from_str::<serde_json::Value>(r#"{"block_height":9}"#)
            .expect("numeric payload should parse");
        assert_eq!(json_optional_u64_value(&numeric, "block_height"), Some(9),);

        let quoted = serde_json::from_str::<serde_json::Value>(r#"{"block_height":"11"}"#)
            .expect("quoted numeric payload should parse");
        assert_eq!(json_optional_u64_value(&quoted, "block_height"), Some(11),);

        let invalid = serde_json::from_str::<serde_json::Value>(r#"{"block_height":"x"}"#)
            .expect("invalid payload should parse");
        assert_eq!(json_optional_u64_value(&invalid, "block_height"), None,);

        let missing = serde_json::from_str::<serde_json::Value>(r#"{"other":1}"#)
            .expect("payload should parse");
        assert_eq!(json_optional_u64_value(&missing, "block_height"), None,);
    }

    #[test]
    fn spec_c04_protocol_escape_json_contract_covers_control_character_paths() {
        let escaped = escape_json("\"\\\n\r\t");
        assert_eq!(
            escaped, "\\\"\\\\\\n\\r\\t",
            "quote, slash, newline, carriage-return, and tab must all be escaped",
        );
    }

    #[test]
    fn spec_c05_protocol_json_field_contract_handles_escaped_quotes_and_nested_key_noise() {
        let payload = r#"{"jsonrpc":"2.0","id":"req-5","method":"tools/call","params":{"name":"health","arguments":{"payload":"quoted value: \"alpha\" and nested key \"name\":\"noise\"","finality":"safe"}}}"#;
        assert_eq!(
            json_required_string_field(payload, "name"),
            Ok("health".to_owned()),
            "tool name must come from params.name and not from nested string contents"
        );
        assert_eq!(
            json_required_string_field(payload, "payload"),
            Ok("quoted value: \"alpha\" and nested key \"name\":\"noise\"".to_owned()),
            "escaped quotes in nested payload values must round-trip without truncation"
        );
        assert_eq!(
            json_optional_u64_value(
                &serde_json::from_str::<serde_json::Value>(
                    r#"{"params":{"arguments":{"block_height":"17"}}}"#
                )
                .expect("payload should parse"),
                "block_height",
            ),
            Some(17),
            "quoted numeric arguments in nested payload must parse as u64"
        );
    }
}
