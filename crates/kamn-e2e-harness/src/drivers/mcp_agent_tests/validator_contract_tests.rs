use super::*;

#[test]
fn unit_validate_live_s05_release_escrow_response_rejects_mismatched_escrow_id() {
    let error = validate_live_s05_release_escrow_response(
        "escrow-a",
        "escrow-b",
        "released",
        "mcp live s05 release_escrow",
    )
    .expect_err("mismatched escrow ids should fail");
    assert!(error.contains("mismatched escrow_id"));
}

#[test]
fn unit_validate_s12_content_id_match_rejects_mismatch() {
    assert_error_contains(
        validate_s12_content_id_match("content-a", "content-b", "test step"),
        "mismatched content_id",
    );
}

#[test]
fn unit_validate_s12_content_field_coherence_rejects_drift() {
    assert_error_contains(
        validate_s12_content_field_coherence("tombstoned", "expired", "lifecycle_state", "test step"),
        "lifecycle_state drift",
    );
}

#[test]
fn unit_validate_s13_bridge_id_match_rejects_mismatch() {
    assert_error_contains(
        validate_s13_bridge_id_match("bridge-a", "bridge-b", "test step"),
        "mismatched bridge_id",
    );
}

#[test]
fn unit_validate_s13_bridge_field_coherence_rejects_drift() {
    assert_error_contains(
        validate_s13_bridge_field_coherence("forwarded", "stale", "bridge_status", "test step"),
        "bridge_status drift",
    );
}

#[test]
fn unit_validate_s07_replay_reason_marker_accepts_expected_marker() {
    validate_s07_replay_reason_marker(
        "operation failed: service_api_auth_replay_nonce_detected",
        "test helper",
    )
    .expect("expected marker should be accepted");
}

#[test]
fn unit_validate_s07_replay_reason_marker_rejects_missing_marker() {
    assert_error_contains(
        validate_s07_replay_reason_marker("operation failed", "test helper"),
        "missing replay reason marker",
    );
}

#[test]
fn unit_validate_s08_mcp_message_receipt_fields_rejects_empty_message_id() {
    assert_error_contains(
        validate_s08_mcp_message_receipt_fields(
            r#"{"result":{"message_id":"","status":"sent"}}"#,
            "test helper",
        ),
        "empty message_id",
    );
}

#[test]
fn unit_validate_s08_mcp_query_message_response_rejects_mismatched_message_id() {
    assert_error_contains(
        validate_s08_mcp_query_message_response(
            r#"{"result":{"message_id":"message-2","status":"sent"}}"#,
            "message-1",
            "test helper",
        ),
        "mismatched message_id",
    );
}

fn assert_error_contains<T: std::fmt::Debug>(result: Result<T, String>, expected: &str) {
    let error = result.expect_err("validator should fail");
    assert!(error.contains(expected), "error should mention {expected}: {error}");
}
