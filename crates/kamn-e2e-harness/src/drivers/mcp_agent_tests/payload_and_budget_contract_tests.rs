use super::*;

#[test]
fn unit_validate_s14_mcp_verify_proof_response_accepts_valid_payload() {
    validate_s14_mcp_verify_proof_response(VALID_PROOF_PAYLOAD, "message-1", "test helper")
        .expect("valid S-14 MCP proof payload should pass");
}

#[test]
fn unit_validate_s14_mcp_verify_proof_response_rejects_mismatched_message_id() {
    assert_error_contains(
        validate_s14_mcp_verify_proof_response(
            MISMATCHED_PROOF_PAYLOAD,
            "message-1",
            "test helper",
        ),
        "mismatched message_id",
    );
}

#[test]
fn unit_validate_s14_mcp_verify_proof_response_rejects_unverified_payload() {
    assert_error_contains(
        validate_s14_mcp_verify_proof_response(
            UNVERIFIED_PROOF_PAYLOAD,
            "message-1",
            "test helper",
        ),
        "verified=false",
    );
}

#[test]
fn unit_validate_s14_mcp_verify_proof_response_rejects_non_final_finality() {
    assert_error_contains(
        validate_s14_mcp_verify_proof_response(NON_FINAL_PROOF_PAYLOAD, "message-1", "test helper"),
        "non-final finality",
    );
}

#[test]
fn unit_validate_s14_mcp_verify_proof_response_rejects_zero_block_height() {
    assert_error_contains(
        validate_s14_mcp_verify_proof_response(
            ZERO_HEIGHT_PROOF_PAYLOAD,
            "message-1",
            "test helper",
        ),
        "block_height=0",
    );
}

#[test]
fn unit_validate_s15_latency_budget_samples_accepts_within_budget_samples() {
    validate_s15_latency_budget_samples(&[10, 20, 30], 80, 100, 25, 35, "test helper")
        .expect("within-budget samples should pass");
}

#[test]
fn unit_validate_s15_latency_budget_samples_rejects_total_budget_violation() {
    assert_error_contains(
        validate_s15_latency_budget_samples(&[10, 20, 30], 120, 100, 25, 35, "test helper"),
        "total elapsed millis exceeded budget",
    );
}

#[test]
fn unit_validate_s15_latency_budget_samples_rejects_p50_budget_violation() {
    assert_error_contains(
        validate_s15_latency_budget_samples(&[10, 50, 90], 90, 200, 20, 100, "test helper"),
        "p50 millis exceeded budget",
    );
}

#[test]
fn unit_validate_s15_latency_budget_samples_rejects_p99_budget_violation() {
    assert_error_contains(
        validate_s15_latency_budget_samples(&[10, 20, 90], 90, 200, 50, 80, "test helper"),
        "p99 millis exceeded budget",
    );
}

#[test]
fn unit_parse_s15_budget_env_u128_uses_default_when_env_missing() {
    with_env_vars(&[("KAMN_E2E_S15_TOTAL_BUDGET_MS", None)], || {
        let parsed = parse_s15_budget_env_u128(
            "KAMN_E2E_S15_TOTAL_BUDGET_MS",
            91,
            "mcp-agent live s15 test helper",
        )
        .expect("missing env key should use default");
        assert_eq!(parsed, 91);
    });
}

#[test]
fn unit_parse_s15_budget_env_u128_parses_positive_env_value() {
    with_env_vars(&[("KAMN_E2E_S15_TOTAL_BUDGET_MS", Some("143"))], || {
        let parsed = parse_s15_budget_env_u128(
            "KAMN_E2E_S15_TOTAL_BUDGET_MS",
            91,
            "mcp-agent live s15 test helper",
        )
        .expect("valid env key should parse");
        assert_eq!(parsed, 143);
    });
}

#[test]
fn unit_validate_s15_latency_budget_samples_accepts_exact_budget_boundaries() {
    validate_s15_latency_budget_samples(&[10, 20, 30], 60, 60, 20, 30, "test helper")
        .expect("equal total/p50/p99 budget boundaries should pass");
}

#[test]
fn unit_percentile_index_returns_expected_midpoint_index() {
    assert_eq!(crate::drivers::shared_helpers::percentile_index(3, 50), 1);
}

#[test]
fn unit_percentile_index_clamps_percentile_above_hundred_to_last_index() {
    assert_eq!(crate::drivers::shared_helpers::percentile_index(3, 150), 2);
}

#[test]
fn unit_json_optional_string_field_extracts_known_value_and_missing_is_none() {
    let payload =
        r#"{"jsonrpc":"2.0","id":"probe","result":{"task_id":"task-1","state":"created"}}"#;
    assert_eq!(
        json_optional_string_field(payload, "task_id"),
        Some("task-1".to_owned())
    );
    assert_eq!(
        json_optional_string_field(payload, "state"),
        Some("created".to_owned())
    );
    assert_eq!(json_optional_string_field(payload, "missing"), None);
}

#[test]
fn unit_json_optional_u64_field_extracts_known_value_and_missing_is_none() {
    let payload = r#"{"jsonrpc":"2.0","id":"probe","result":{"block_height":42,"ok":true}}"#;
    assert_eq!(json_optional_u64_field(payload, "block_height"), Some(42));
    assert_eq!(json_optional_u64_field(payload, "missing"), None);
}

#[test]
fn unit_escape_json_scalar_escapes_quotes_backslashes_and_controls() {
    let escaped = escape_json_scalar("\"\\\n\r\tx");
    assert_eq!(escaped, "\\\"\\\\\\n\\r\\tx");
}

const VALID_PROOF_PAYLOAD: &str =
    r#"{"result":{"message_id":"message-1","verified":true,"finality":"FINAL","block_height":42}}"#;
const MISMATCHED_PROOF_PAYLOAD: &str =
    r#"{"result":{"message_id":"message-2","verified":true,"finality":"FINAL","block_height":42}}"#;
const UNVERIFIED_PROOF_PAYLOAD: &str = r#"{"result":{"message_id":"message-1","verified":false,"finality":"FINAL","block_height":42}}"#;
const NON_FINAL_PROOF_PAYLOAD: &str = r#"{"result":{"message_id":"message-1","verified":true,"finality":"PENDING","block_height":42}}"#;
const ZERO_HEIGHT_PROOF_PAYLOAD: &str =
    r#"{"result":{"message_id":"message-1","verified":true,"finality":"FINAL","block_height":0}}"#;

fn assert_error_contains(result: Result<(), String>, expected: &str) {
    let error = result.expect_err("validator should fail");
    assert!(
        error.contains(expected),
        "error should mention {expected}: {error}"
    );
}
