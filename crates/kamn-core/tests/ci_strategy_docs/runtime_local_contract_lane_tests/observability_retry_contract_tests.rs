use super::super::fairness_deletion_support::assert_contains_all;
use super::super::DOC;
use super::support::assert_runtime_local_contract_lane_markers;

#[test]
fn doc_contains_runtime_observability_endpoint_contract_lane_ci_mode_markers() {
    assert_runtime_local_contract_lane_markers(
        "## Runtime Observability Endpoint Contract Lane",
        observability_endpoint_commands(),
        "ci-local contract-lane boundary rejects `--max-seconds > 240`.",
        observability_endpoint_policy_markers(),
        "runtime observability endpoint",
    );
    assert_observability_endpoint_reason_markers();
}

#[test]
fn doc_contains_runtime_local_retry_diagnostics_contract_lane_ci_mode_markers() {
    assert_runtime_local_contract_lane_markers(
        "## Runtime Local Retry/Diagnostics Contract Lane",
        retry_diagnostics_commands(),
        "ci-local contract-lane budget remains fail-closed and rejects `--max-seconds > 240`.",
        retry_diagnostics_policy_markers(),
        "runtime local retry diagnostics",
    );
    assert_retry_diagnostics_reason_markers();
}

fn observability_endpoint_commands() -> &'static [&'static str] {
    &[
        "validate_runtime_observability_endpoint_live_contract_lane.sh",
        "check_runtime_observability_endpoint_live_policy.sh",
        "check_observability_endpoint_drift_contract.sh --output-json /tmp/observability-endpoint-drift-report.json",
        "test_validate_runtime_observability_endpoint_live_contract_lane.sh",
        "test_check_observability_endpoint_drift_contract.sh",
    ]
}

fn observability_endpoint_policy_markers() -> &'static [&'static str] {
    &[
        "endpoint_readiness_status=verified",
        "stream_parity_status=verified",
        "observability_source_marker_missing:legacy_tcp_listener_import",
        "runtime_observability_endpoint_readiness_progress_stalled",
        "runtime_observability_stream_parity_bypass_detected",
        "ci_local_observability_endpoint_budget_boundary_exceeded",
    ]
}

fn assert_observability_endpoint_reason_markers() {
    assert_contains_all(
        DOC,
        &[
            "reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1",
            "reason_codes_csv=runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded",
        ],
        "runtime observability endpoint",
    );
}

fn retry_diagnostics_commands() -> &'static [&'static str] {
    &[
        "validate_local_retry_diagnostics_live.sh --mode dry-run --output-json /tmp/runtime-local-retry-diagnostics-summary.json",
        "KAMN_LOCAL_RETRY_DIAGNOSTICS_OPT_IN=1 bash scripts/runtime/validate_local_retry_diagnostics_live.sh --mode run --output-json /tmp/runtime-local-retry-diagnostics-summary.json",
        "check_local_retry_diagnostics_live_policy.sh --report-file /tmp/runtime-local-retry-diagnostics-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/runtime-local-retry-diagnostics-policy.json",
        "validate_local_retry_diagnostics_live_contract_lane.sh --output-json /tmp/runtime-local-retry-diagnostics-contract-lane-report.json --policy-output-json /tmp/runtime-local-retry-diagnostics-policy.json",
        "test_validate_local_retry_diagnostics_live.sh",
        "test_check_local_retry_diagnostics_live_policy.sh",
        "test_validate_local_retry_diagnostics_live_contract_lane.sh",
    ]
}

fn retry_diagnostics_policy_markers() -> &'static [&'static str] {
    &[
        "retry_readiness_status=verified",
        "retry_backoff_status=verified",
        "retry_jitter_parity_status=verified",
        "retry_envelope_exhaustion_fail_closed_status=verified",
        "reconnect_attempt_bound_status=verified",
        "reconnect_backoff_bound_status=verified",
        "retry_envelope_max_attempts=3",
        "retry_envelope_max_backoff_seconds=8",
        "local retry/diagnostics run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        "local_retry_diagnostics_policy_marker_missing:correlation_diagnostics_status",
        "local_retry_readiness_progress_stalled",
        "local_retry_backoff_jitter_parity_bypass_detected",
        "local_retry_envelope_exhaustion_fail_closed_missing",
        "local_retry_reconnect_attempt_bound_drift",
        "local_retry_reconnect_backoff_bound_drift",
        "ci_local_network_budget_boundary_exceeded",
    ]
}

fn assert_retry_diagnostics_reason_markers() {
    assert_contains_all(
        DOC,
        &[
            "reason_taxonomy_version=kamn.runtime.local-retry-diagnostics-reason-taxonomy.v2",
            "reason_codes_csv=local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,local_retry_envelope_exhaustion_fail_closed_missing,local_retry_reconnect_attempt_bound_drift,local_retry_reconnect_backoff_bound_drift,ci_local_network_budget_boundary_exceeded",
        ],
        "runtime local retry diagnostics",
    );
}
