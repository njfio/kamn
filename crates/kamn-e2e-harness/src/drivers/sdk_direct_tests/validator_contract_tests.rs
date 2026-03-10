use super::*;

#[test]
fn unit_run_live_s03_group_channel_probe_rejects_query_message_id_mismatch() {
    let error = validate_live_s03_query_message_response("message-1", "message-2", "sent")
        .expect_err("mismatched query message_id should fail");
    assert!(
        error.contains("mismatched message_id"),
        "error should mention message_id mismatch: {error}",
    );
}

#[test]
fn unit_run_live_s03_group_channel_probe_rejects_list_channel_id_mismatch() {
    let error = validate_live_s03_list_messages_response("channel-1", "channel-2")
        .expect_err("mismatched listed channel_id should fail");
    assert!(
        error.contains("mismatched channel_id"),
        "error should mention channel_id mismatch: {error}",
    );
}

#[test]
fn unit_validate_live_s05_release_escrow_receipt_rejects_mismatched_escrow_id() {
    let error = validate_live_s05_release_escrow_receipt("escrow-a", "escrow-b", "released")
        .expect_err("mismatched escrow ids should fail");
    assert!(
        error.contains("mismatched escrow_id"),
        "error should describe escrow-id mismatch: {error}",
    );
}

#[test]
fn unit_validate_s12_content_id_match_rejects_mismatch() {
    let error = validate_s12_content_id_match("content-a", "content-b", "test step")
        .expect_err("mismatched content ids should fail");
    assert!(
        error.contains("mismatched content_id"),
        "error should mention content_id mismatch: {error}",
    );
}

#[test]
fn unit_validate_s12_content_field_coherence_rejects_drift() {
    let error = validate_s12_content_field_coherence(
        "tombstoned",
        "expired",
        "lifecycle_state",
        "test step",
    )
    .expect_err("field drift should fail");
    assert!(
        error.contains("lifecycle_state drift"),
        "error should mention field drift: {error}",
    );
}

#[test]
fn unit_validate_s13_bridge_id_match_rejects_mismatch() {
    let error = validate_s13_bridge_id_match("bridge-a", "bridge-b", "test step")
        .expect_err("mismatched bridge ids should fail");
    assert!(
        error.contains("mismatched bridge_id"),
        "error should mention bridge_id mismatch: {error}",
    );
}

#[test]
fn unit_validate_s13_bridge_field_coherence_rejects_drift() {
    let error =
        validate_s13_bridge_field_coherence("forwarded", "stale", "bridge_status", "test step")
            .expect_err("bridge field drift should fail");
    assert!(
        error.contains("bridge_status drift"),
        "error should mention field drift: {error}",
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
    let error = validate_s07_replay_reason_marker("operation failed", "test helper")
        .expect_err("missing marker should fail");
    assert!(
        error.contains("missing replay reason marker"),
        "error should mention replay marker contract: {error}",
    );
}

#[test]
fn unit_validate_s08_message_receipt_fields_rejects_empty_message_id() {
    let error = validate_s08_message_receipt_fields("", "sent", "test helper")
        .expect_err("empty message_id should fail");
    assert!(
        error.contains("empty message_id"),
        "error should mention message_id requirement: {error}",
    );
}

#[test]
fn unit_validate_s08_query_message_response_rejects_mismatched_message_id() {
    let error = validate_s08_query_message_response("message-1", "message-2", "sent", "test")
        .expect_err("mismatched query message_id should fail");
    assert!(
        error.contains("mismatched message_id"),
        "error should mention message_id mismatch: {error}",
    );
}

#[test]
fn unit_validate_s08_distinct_message_ids_accepts_distinct_ids() {
    validate_s08_distinct_message_ids("message-1", "message-2", "test helper")
        .expect("distinct message ids should pass");
}

#[test]

fn unit_validate_s08_distinct_message_ids_rejects_duplicate_ids() {
    let error = validate_s08_distinct_message_ids("message-1", "message-1", "test helper")
        .expect_err("duplicate message ids should fail");
    assert!(
        error.contains("duplicate message_id"),
        "error should mention duplicate message_id: {error}",
    );
}
