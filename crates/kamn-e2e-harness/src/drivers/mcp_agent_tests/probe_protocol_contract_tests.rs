use super::*;

#[test]
fn spec_c01_build_framed_jsonrpc_request_includes_content_length_and_body() {
    let request = build_framed_jsonrpc_request(r#"{"jsonrpc":"2.0","id":"req-1"}"#);
    assert!(request.starts_with("Content-Length: "));
    assert!(request.contains("\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":\"req-1\"}"));
}

#[test]
fn spec_c02_parse_framed_jsonrpc_payloads_supports_multiple_frames() {
    let first = build_framed_jsonrpc_request(r#"{"jsonrpc":"2.0","id":"init"}"#);
    let second = build_framed_jsonrpc_request(r#"{"jsonrpc":"2.0","id":"health"}"#);
    let payloads = parse_framed_jsonrpc_payloads(format!("{first}{second}").as_str())
        .expect("framed payloads should parse");
    assert_eq!(payloads, vec![INIT_PAYLOAD.to_owned(), HEALTH_PAYLOAD.to_owned()]);
}

#[test]
fn spec_c03_parse_framed_jsonrpc_payloads_rejects_malformed_stream() {
    let error = parse_framed_jsonrpc_payloads("Content-Length: 9\r\n\r\n{\"id\":1}")
        .expect_err("mismatched content-length should fail");
    assert!(error.contains("content-length"));
}

#[test]
fn spec_c04_parse_framed_jsonrpc_payloads_accepts_leading_newlines() {
    let framed = format!("\n{}", build_framed_jsonrpc_request(INIT_PAYLOAD));
    let payloads = parse_framed_jsonrpc_payloads(framed.as_str())
        .expect("leading newline should be skipped");
    assert_eq!(payloads, vec![INIT_PAYLOAD.to_owned()]);
}

#[test]
fn spec_c05_parse_framed_jsonrpc_payloads_rejects_newline_only_stream() {
    let error = parse_framed_jsonrpc_payloads("\n").expect_err("newline-only stream should fail");
    assert!(error.contains("no framed payloads parsed"));
}

#[test]
fn spec_c06_validate_probe_initialize_response_rejects_missing_jsonrpc_marker() {
    let payload = r#"{"id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}"#;
    let error = validate_probe_initialize_response(payload)
        .expect_err("missing jsonrpc marker should fail");
    assert!(error.contains("missing jsonrpc marker"));
}

#[test]
fn spec_c07_validate_probe_initialize_response_rejects_missing_server_info_marker() {
    let payload = r#"{"jsonrpc":"2.0","id":"probe-init","result":{}}"#;
    let error = validate_probe_initialize_response(payload)
        .expect_err("missing serverInfo marker should fail");
    assert!(error.contains("missing serverInfo marker"));
}

#[test]
fn spec_c08_validate_probe_health_response_rejects_non_success_payload() {
    let payload = r#"{"jsonrpc":"2.0","id":"probe-health","result":{"ok":false}}"#;
    let error = validate_probe_health_response(payload)
        .expect_err("non-success payload should fail");
    assert!(error.contains("non-success health payload"));
}

#[test]
fn spec_c23_validate_probe_initialize_response_accepts_required_markers() {
    let payload = r#"{"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn-mcp-server"}}}"#;
    validate_probe_initialize_response(payload).expect("required initialize markers should pass");
}

#[test]
fn spec_c24_validate_probe_health_response_accepts_success_payload() {
    let payload = r#"{"jsonrpc":"2.0","id":"probe-health","result":{"ok":true}}"#;
    validate_probe_health_response(payload).expect("ok=true health payload should pass");
}

#[test]
fn regression_issue_6214_validate_probe_health_response_rejects_nested_ok_true_when_root_false() {
    let payload = r#"{"jsonrpc":"2.0","id":"probe-health","result":{"ok":false,"detail":{"ok":true}}}"#;
    let error = validate_probe_health_response(payload)
        .expect_err("root result.ok=false should fail even when nested fields contain ok=true");
    assert!(error.contains("non-success health payload"));
}

const INIT_PAYLOAD: &str = r#"{"jsonrpc":"2.0","id":"init"}"#;
const HEALTH_PAYLOAD: &str = r#"{"jsonrpc":"2.0","id":"health"}"#;
