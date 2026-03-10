use super::super::DOC;
use super::super::fairness_deletion_support::assert_contains_all;

#[test]
fn doc_contains_persistence_adapter_integrity_ci_boundary_markers() {
    assert_contains_all(
        DOC,
        &[
            "## Persistence Adapter Integrity + CI Boundary Fast Lane",
            "test_validate_persistence_adapters_live.sh",
            "persistence_gate_reason_taxonomy_version=kamn.runtime.persistence-gate-reason-taxonomy.v1",
            "persistence_gate_reason_codes_csv=content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected,persistence_evidence_tamper_detected,persistence_evidence_freshness_window_exceeded,persistence_evidence_incomplete,persistence_ci_smoke_local_heavy_boundary_violation",
            "persistence_ci_smoke_local_heavy_boundary_status=verified",
            "persistence_ci_smoke_lane_cost_profile=low",
            "persistence_local_heavy_execution_mode=opt_in",
        ],
        "persistence boundary",
    );
}

#[test]
fn doc_contains_cutover_ci_exclusion_policy_contract_markers() {
    assert_contains_all(
        DOC,
        &[
            "## Cutover Rollback CI Exclusion Policy Contract",
            "python3 scripts/cutover/check_cutover_ci_exclusion_policy.py",
            "bash scripts/cutover/test_check_cutover_ci_exclusion_policy.sh",
            "cutover_ci_exclusion_policy_reason_taxonomy_version=kamn.ci.cutover-ci-exclusion-policy-reason-taxonomy.v1",
            "cutover_ci_exclusion_policy_reason_codes_csv=cutover_contract_lane_missing_in_ci_fast_gate,cutover_rollback_deep_lane_leaked_into_ci_fast_gate,cutover_contract_test_missing_in_ci_tools,cutover_deep_lane_test_leaked_into_ci_tools,ci_strategy_cutover_exclusion_markers_missing,ci_strategy_cutover_policy_command_missing,runtime_budget_exceeded",
            "cutover_rollback_deep_lane_local_only=true",
            "cutover_rollback_deep_lane_excluded_from_ci_fast_gate=true",
            "bash scripts/cutover/run_cutover_rollback_contract_lane.sh",
            "bash scripts/cutover/run_cutover_rollback_deep_lane.sh",
        ],
        "cutover exclusion policy",
    );
}

#[test]
fn doc_contains_invariant_fuzz_concurrency_ci_smoke_boundary_contract_markers() {
    assert_contains_all(
        DOC,
        &[
            "## Invariant/Fuzz/Concurrency CI Smoke Boundary Contract",
            "bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json",
            "bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json --output-json /tmp/invariant-fuzz-concurrency-policy-report.json",
            "bash scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh",
            "bash scripts/runtime/run_concurrency_state_mutation_deep_lane.sh",
            "invariant_fuzz_concurrency_ci_smoke_max_seconds=120",
            "invariant_fuzz_concurrency_local_heavy_max_seconds=900",
            "invariant_fuzz_concurrency_ci_smoke_lane_cost_profile=low",
            "invariant_fuzz_concurrency_local_heavy_execution_mode=opt_in",
            "invariant_fuzz_concurrency_local_heavy_excluded_from_ci_fast_gate=true",
        ],
        "invariant fuzz concurrency",
    );
}

#[test]
fn doc_contains_live_transport_fault_matrix_ci_exclusion_policy_contract_markers() {
    assert_contains_all(
        DOC,
        &[
            "bash scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh",
            "live_transport_fault_matrix_policy_peer_adapter_reason_projection_timeout_code_mismatch",
            "live_transport_fault_matrix_policy_marker_missing:retry_reconnect_marker_contract_status",
            "retry_reconnect_marker_contract_status=verified",
        ],
        "live transport fault matrix",
    );
}
