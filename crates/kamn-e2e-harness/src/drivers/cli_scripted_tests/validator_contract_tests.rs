use super::*;

#[test]
fn unit_validate_live_s05_release_escrow_response_rejects_mismatched_escrow_id() {
    let error = validate_live_s05_release_escrow_response(
        "escrow-a",
        "escrow-b",
        "released",
        "cli live s05 release-escrow",
    )
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
fn unit_run_cli_command_capture_stdout_returns_trimmed_stdout_on_success() {
    let output = run_cli_command_capture_stdout(
        "/bin/sh",
        &["-c", "printf 'task_id=task-1 state=created'"],
        "test helper",
    )
    .expect("successful command should return stdout");
    assert_eq!(output, "task_id=task-1 state=created");
}

#[test]
fn unit_run_cli_command_capture_stdout_rejects_non_success_exit_status() {
    let error = run_cli_command_capture_stdout("/bin/sh", &["-c", "exit 7"], "test helper")
        .expect_err("non-success status should fail");
    assert!(
        error.contains("exit_status=7"),
        "error should include failing exit status: {error}",
    );
}

#[test]
fn unit_run_cli_command_expect_failure_with_agent_name_returns_stderr() {
    let output = super::run_cli_command_expect_failure_with_agent_name(
        "/bin/sh",
        &["-c", "echo replay >&2; exit 2"],
        "test helper",
        "probe",
    )
    .expect("stderr should be captured on expected failure");
    assert_eq!(output, "replay");
}

#[test]
fn unit_run_cli_command_expect_failure_with_agent_name_rejects_success_status() {
    let error = super::run_cli_command_expect_failure_with_agent_name(
        "/bin/sh",
        &["-c", "exit 0"],
        "test helper",
        "probe",
    )
    .expect_err("success status should be rejected");
    assert!(
        error.contains("unexpectedly succeeded"),
        "error should mention unexpected success: {error}",
    );
}

#[test]
fn unit_parse_text_output_field_extracts_known_keys_and_missing_is_none() {
    let output = "task_id=task-1 state=created";
    assert_eq!(parse_text_output_field(output, "task_id"), Some("task-1"));
    assert_eq!(parse_text_output_field(output, "state"), Some("created"));
    assert_eq!(parse_text_output_field(output, "missing"), None);
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
    let error = validate_s08_message_receipt_fields("message_id= status=sent", "test helper")
        .expect_err("empty message_id should fail");
    assert!(
        error.contains("empty message_id"),
        "error should mention message_id requirement: {error}",
    );
}

#[test]
fn unit_validate_s08_query_message_response_rejects_mismatched_message_id() {
    let error = validate_s08_query_message_response(
        "message_id=message-2 status=sent",
        "message-1",
        "test helper",
    )
    .expect_err("mismatched message_id should fail");
    assert!(
        error.contains("mismatched message_id"),
        "error should mention message_id mismatch: {error}",
    );
}
