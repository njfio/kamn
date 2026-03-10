use super::*;

#[test]
fn unit_validate_s14_cli_verify_proof_response_accepts_valid_payload() {
    validate_s14_cli_verify_proof_response(
        "message_id=message-1 verified=true finality=FINAL block_height=42",
        "message-1",
        "test helper",
    )
    .expect("valid S-14 proof payload should pass");
}

#[test]
fn unit_validate_s14_cli_verify_proof_response_rejects_mismatched_message_id() {
    let error = validate_s14_cli_verify_proof_response(
        "message_id=message-2 verified=true finality=FINAL block_height=42",
        "message-1",
        "test helper",
    )
    .expect_err("mismatched message_id should fail");
    assert!(
        error.contains("mismatched message_id"),
        "error should mention message_id mismatch: {error}",
    );
}

#[test]
fn unit_validate_s14_cli_verify_proof_response_rejects_unverified_payload() {
    let error = validate_s14_cli_verify_proof_response(
        "message_id=message-1 verified=false finality=FINAL block_height=42",
        "message-1",
        "test helper",
    )
    .expect_err("verified=false should fail");
    assert!(
        error.contains("verified=false"),
        "error should mention verified contract: {error}",
    );
}

#[test]
fn unit_validate_s14_cli_verify_proof_response_rejects_non_final_finality() {
    let error = validate_s14_cli_verify_proof_response(
        "message_id=message-1 verified=true finality=PENDING block_height=42",
        "message-1",
        "test helper",
    )
    .expect_err("non-final finality should fail");
    assert!(
        error.contains("non-final finality"),
        "error should mention finality contract: {error}",
    );
}

#[test]
fn unit_validate_s14_cli_verify_proof_response_rejects_zero_block_height() {
    let error = validate_s14_cli_verify_proof_response(
        "message_id=message-1 verified=true finality=FINAL block_height=0",
        "message-1",
        "test helper",
    )
    .expect_err("block_height=0 should fail");
    assert!(
        error.contains("block_height=0"),
        "error should mention block-height contract: {error}",
    );
}

#[test]
fn unit_validate_s15_latency_budget_samples_accepts_within_budget_samples() {
    validate_s15_latency_budget_samples(&[10, 20, 30], 80, 100, 25, 35, "test helper")
        .expect("within-budget samples should pass");
}

#[test]
fn unit_validate_s15_latency_budget_samples_rejects_total_budget_violation() {
    let error = validate_s15_latency_budget_samples(&[10, 20, 30], 120, 100, 25, 35, "test helper")
        .expect_err("total budget violation should fail");
    assert!(
        error.contains("total elapsed millis exceeded budget"),
        "error should mention total budget: {error}",
    );
}

#[test]
fn unit_validate_s15_latency_budget_samples_rejects_p50_budget_violation() {
    let error = validate_s15_latency_budget_samples(&[10, 50, 90], 90, 200, 20, 100, "test helper")
        .expect_err("p50 budget violation should fail");
    assert!(
        error.contains("p50 millis exceeded budget"),
        "error should mention p50 budget: {error}",
    );
}

#[test]
fn unit_validate_s15_latency_budget_samples_rejects_p99_budget_violation() {
    let error = validate_s15_latency_budget_samples(&[10, 20, 90], 90, 200, 50, 80, "test helper")
        .expect_err("p99 budget violation should fail");
    assert!(
        error.contains("p99 millis exceeded budget"),
        "error should mention p99 budget: {error}",
    );
}

#[test]
fn unit_parse_s15_budget_env_u128_uses_default_when_env_missing() {
    with_env_vars(&[("KAMN_E2E_S15_TOTAL_BUDGET_MS", None)], || {
        let parsed = parse_s15_budget_env_u128(
            "KAMN_E2E_S15_TOTAL_BUDGET_MS",
            91,
            "cli-scripted live s15 test helper",
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
            "cli-scripted live s15 test helper",
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
    assert_eq!(
        crate::drivers::shared_helpers::percentile_index(3, 50),
        1,
        "len=3 and p50 should map to middle sample index",
    );
}

#[test]
fn unit_percentile_index_clamps_percentile_above_hundred_to_last_index() {
    assert_eq!(
        crate::drivers::shared_helpers::percentile_index(3, 150),
        2,
        "percentiles above 100 should clamp to the last sample index",
    );
}
