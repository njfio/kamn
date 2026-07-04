use kamn_mcp_server::process_stdio_input;

use crate::support::{frame_request, parse_framed_json, ProtocolBackend};

#[test]
fn spec_c04_mcp_method_not_found_error_contract() {
    let backend = ProtocolBackend;
    let request = frame_request(r#"{"jsonrpc":"2.0","id":"req-4","method":"unknown/method"}"#);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    let body = parse_framed_json(responses[0].as_str());
    assert!(body.contains(r#""error""#));
    assert!(body.contains(r#""code":-32601"#));
}

#[test]
fn spec_c05_mcp_invalid_params_error_contract() {
    let backend = ProtocolBackend;
    let request =
        frame_request(r#"{"jsonrpc":"2.0","id":"req-5","method":"tools/call","params":{}}"#);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    let body = parse_framed_json(responses[0].as_str());
    assert!(body.contains(r#""error""#));
    assert!(body.contains(r#""code":-32602"#));
}

#[test]
fn spec_c06_line_mode_dispatch_remains_supported_contract() {
    let backend = ProtocolBackend;
    let responses =
        process_stdio_input(&backend, r#"{"id":"req-6","tool":"health"}"#).expect("line mode");
    assert_eq!(responses.len(), 1, "line mode should return one response");
    assert!(responses[0].contains(r#""ok":true"#));
}

#[test]
fn spec_c09_mcp_malformed_json_maps_to_parse_error_contract() {
    let backend = ProtocolBackend;
    let malformed =
        r#"{"jsonrpc":"2.0","id":"req-parse","method":"tools/call","params":{"name":"health"}"#;
    let request = frame_request(malformed);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    assert_eq!(
        responses.len(),
        1,
        "malformed frame should return one response"
    );
    let body = parse_framed_json(responses[0].as_str());
    assert!(body.contains(r#""error""#));
    assert!(body.contains(r#""code":-32700"#));
    assert!(body.contains(r#""id":null"#));
}

#[test]
fn spec_c10_mcp_non_string_method_maps_to_invalid_request_contract() {
    let backend = ProtocolBackend;
    let request = frame_request(r#"{"jsonrpc":"2.0","id":"req-invalid","method":9}"#);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    assert_eq!(
        responses.len(),
        1,
        "invalid request should return one response"
    );
    let body = parse_framed_json(responses[0].as_str());
    assert!(body.contains(r#""error""#));
    assert!(body.contains(r#""code":-32600"#));
}
