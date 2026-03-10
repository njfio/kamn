use super::docs_assert_support::{assert_plan_contains_all};

const PLAN_CONTAINS_RUNTIME_TRANSPORT_RETRY_RECONNECT_FAILURE_TAXONOMY_PLAN_MARKERS: &[&str] = &[
    "## Runtime Transport Retry-Reconnect Failure Taxonomy (Issue #4508)",
    "validate_live_transport_fault_matrix_live.sh",
    "live_transport_fault_matrix_policy_reason_codes_classification_mismatch",
    "live_transport_fault_matrix_policy_peer_adapter_reason_projection_timeout_code_mismatch",
    "live_transport_fault_matrix_policy_peer_integrity_fail_closed_reason_code_mismatch",
    "cargo test -p kamn-core --test p2p_peer_integrity_drift_timeout",
    "peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1",
    "peer_adapter_multi_process_validation_local_heavy_status=required",
    "peer sender-integrity drift fixtures must fail closed with `p2p_transport_unknown_sender_peer` (`Regression: #4319`).",
    "retry-timeout classification must remain stable with `p2p_live_reconnect_retry_dial_timeout` before budget exhaustion and `p2p_live_reconnect_retry_budget_exhausted` only when retry budget is reached (`Regression: #4319`).",
];

#[test]
fn plan_contains_runtime_transport_retry_reconnect_failure_taxonomy() {
    assert_plan_contains_all(PLAN_CONTAINS_RUNTIME_TRANSPORT_RETRY_RECONNECT_FAILURE_TAXONOMY_PLAN_MARKERS, "plan_contains_runtime_transport_retry_reconnect_failure_taxonomy");
}
