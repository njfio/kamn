use super::*;

#[test]
fn service_api_ops_configuration_contains_live_node_drift_marker_mismatch_policy_contracts() {
    assert!(DOC.contains("### Live-Node Drift Marker Mismatch Policy Contracts (Issue #4281)"));
    assert!(DOC.contains("failover_promotion_gate_status=verified"));
    assert!(DOC.contains("live_node_drift_parity_status=verified"));
    assert!(DOC.contains("ci_local_promotion_budget_boundary_status=verified"));
    assert!(DOC.contains(
        "failover_readiness_reason_taxonomy_version=kamn.runtime.failover-readiness-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "failover_readiness_reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("failover_sync_drift_policy_status=verified"));
    assert!(DOC.contains(
        "bash scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh check-policy"
    ));
    assert!(DOC.contains("live_node_drift_marker_parity_mismatch"));
    assert!(DOC.contains("failover_readiness_progress_stalled"));
    assert!(DOC.contains("ci_local_promotion_budget_boundary_exceeded"));
    assert!(DOC.contains("failover_sync_drift_policy_required_field_missing:<field>"));
    assert!(DOC.contains("failover_sync_drift_policy_reason_taxonomy_version_mismatch"));
    assert!(DOC.contains("failover_sync_drift_policy_reason_codes_csv_mismatch"));
    assert!(DOC.contains("Regression: #4285"));
    assert!(DOC.contains("Regression: #4286"));
}

#[test]
fn service_api_ops_configuration_contains_shutdown_checkpoint_reconciliation_failure_modes() {
    assert!(DOC.contains("Shutdown signal failure matrix"));
    assert!(DOC.contains("full_supervisor_stop_graceful_drain_timeout_contract_mismatch"));
    assert!(DOC.contains(
        "shutdown_checkpoint_reconciliation_reason_taxonomy_version=kamn.runtime.shutdown-checkpoint-reconciliation-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch"));
    assert!(DOC.contains("shutdown_checkpoint_reconciliation_not_signaled_checkpoint_mismatch"));
    assert!(DOC.contains("runtime_shutdown_invariant_violation:<reason_code>"));
    assert!(DOC.contains("Regression: #4332"));
    assert!(DOC.contains("Regression: #4333"));
}

#[test]
fn service_api_ops_configuration_contains_full_stack_harness_marker_mismatch_controls() {
    assert!(DOC.contains(
        "## Full-Stack Harness Marker Completeness and Parity Mismatch Controls (Issue #4195)"
    ));
    assert!(DOC.contains("full_io_harness_marker_completeness_status=verified"));
    assert!(DOC.contains("full_io_harness_marker_parity_status=verified"));
    assert!(DOC.contains(
        "full_io_harness_policy_reason_taxonomy_version=kamn.runtime.full-io-scenario-matrix-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "full_io_harness_policy_reason_codes_csv=full_io_scenario_matrix_policy_process_harness_mismatch,full_io_scenario_matrix_policy_api_route_matrix_mismatch,full_io_scenario_matrix_policy_auth_failure_matrix_mismatch,full_io_scenario_matrix_policy_websocket_matrix_mismatch,full_io_scenario_matrix_policy_multinode_propagation_mismatch,full_io_scenario_matrix_policy_dry_run_command_count_mismatch,full_io_scenario_matrix_policy_dry_run_command_status_mismatch"
    ));
    assert!(DOC.contains("full_io_scenario_matrix_policy_process_harness_mismatch"));
    assert!(DOC.contains("full_io_scenario_matrix_policy_dry_run_command_count_mismatch"));
    assert!(DOC.contains("full_io_scenario_matrix_policy_dry_run_command_status_mismatch"));
    assert!(DOC.contains("Regression: #4195"));
}

#[test]
fn service_api_ops_configuration_contains_upgrade_compatibility_marker_matrix_controls() {
    assert!(DOC.contains("## Upgrade Compatibility Marker Matrix Controls (Issue #4181)"));
    assert!(DOC.contains(
        "check_upgrade_compatibility_marker_matrix_policy.py --version-report-file /tmp/kolme-version-report.json --fork-policy-report-file /tmp/kolme-fork-compatibility-policy-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-upgrade-compatibility-marker-matrix-policy-report.json"
    ));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=version_report_missing,fork_policy_report_missing,version_report_schema_mismatch,version_report_reason_taxonomy_mismatch,version_report_reason_codes_csv_mismatch,version_report_rehearsal_bypass_guard_status_mismatch,version_report_rehearsal_output_normalization_status_mismatch,fork_policy_report_schema_mismatch,fork_policy_report_reason_taxonomy_mismatch,fork_policy_report_reason_codes_csv_mismatch,fork_policy_report_rehearsal_bypass_guard_status_mismatch,fork_policy_report_rehearsal_output_normalization_status_mismatch,expected_final_decision_mismatch,ci_fast_gate_failed"
    ));
    assert!(DOC.contains("version_report_schema_mismatch"));
    assert!(DOC.contains("fork_policy_report_reason_codes_csv_mismatch"));
    assert!(DOC.contains("fork_policy_report_rehearsal_bypass_guard_status_mismatch"));
    assert!(DOC.contains("Regression: #4180"));
    assert!(DOC.contains("Regression: #4181"));
}

#[test]
fn service_api_ops_configuration_contains_partition_healing_mismatch_mapping_controls() {
    assert!(DOC.contains(
        "### Block Reconciliation Partition-Healing Mismatch Mapping Contracts (Issues #4251, #4255, #4256)"
    ));
    assert!(DOC.contains("partition_healing_mismatch_reason_mapping_status=verified"));
    assert!(DOC.contains(
        "partition_healing_mismatch_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-healing-mismatch-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "partition_healing_mismatch_reason_codes_csv=block_reconciliation_partition_rejoin_policy_required_field_missing,block_reconciliation_partition_rejoin_policy_marker_mismatch,block_reconciliation_partition_rejoin_policy_transport_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_mismatch,block_reconciliation_partition_rejoin_policy_recovery_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid,block_reconciliation_partition_rejoin_policy_lane_mode_contract_mismatch,block_reconciliation_partition_rejoin_policy_ci_fast_gate_failed,block_reconciliation_partition_rejoin_policy_expected_decision_mismatch,block_reconciliation_partition_rejoin_policy_violation"
    ));
    assert!(DOC.contains("partition_healing_mismatch_reason_code=none|<reason>"));
    assert!(
        DOC.contains("block_reconciliation_partition_rejoin_policy_required_field_missing:<field>")
    );
    assert!(DOC.contains(
        "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid"
    ));
    assert!(DOC.contains("Regression: #4255"));
    assert!(DOC.contains("Regression: #4256"));
}
