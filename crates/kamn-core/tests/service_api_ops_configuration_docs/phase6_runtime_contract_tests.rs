use super::*;
use super::shared_support::*;

#[test]
fn service_api_ops_configuration_contains_phase6_archival_retry_policy_markers() {
    assert!(
        DOC.contains("## Phase-6 Archival Failure-Retry Policy Contracts (Issues #5285, #5287)")
    );
    assert!(DOC.contains("archival_retry_policy_status=verified"));
    assert!(DOC.contains(
        "archival_retry_reason_taxonomy_version=kamn.runtime.data-layer-m10-archival-retry-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "archival_retry_reason_codes_csv=m10_archival_retry_scheduled,m10_archival_retry_exhausted,m10_archival_failure_permanent,m10_archival_retry_policy_invalid,m10_archival_retry_attempt_invalid"
    ));
    assert!(DOC.contains(
        "archival_retry_policy_contract=max_attempts>=1;base_backoff_seconds>=1;max_backoff_seconds>=base_backoff_seconds"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c12_transient_archival_failure_projects_deterministic_retry_window -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed -- --exact"
    ));
    assert!(DOC.contains("Regression: #5287"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_execution_tick_orchestration_markers() {
    assert!(DOC.contains("## Phase-6 Retention+Archival Execution Tick Contracts (Issue #5289)"));
    assert!(DOC.contains("phase6_execution_tick_status=verified"));
    assert!(DOC.contains(
        "phase6_execution_tick_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-execution-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_execution_tick_reason_codes_csv=m10_phase6_execution_applied,m10_phase6_execution_owner_scope_denied,m10_phase6_execution_legal_hold_active,m10_phase6_execution_input_invalid,m10_phase6_execution_projection_input_invalid,m10_phase6_execution_projection_failed"
    ));
    assert!(DOC.contains(
        "phase6_execution_tick_contract=retention_due_lookup->crypto_shred->partition_projection->archive_due"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries -- --exact"
    ));
    assert!(DOC.contains("Regression: #5289"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_execution_tick_budget_markers() {
    assert!(DOC.contains("## Phase-6 Execution Tick Budget Guardrail Contracts (Issue #5291)"));
    assert!(DOC.contains("phase6_execution_tick_budget_status=verified"));
    assert!(DOC.contains(
        "phase6_execution_tick_budget_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-execution-budget-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_execution_tick_budget_reason_codes_csv=m10_phase6_execution_budget_within_limit,m10_phase6_execution_budget_due_candidates_exceeded,m10_phase6_execution_budget_shredded_messages_exceeded,m10_phase6_execution_budget_projections_exceeded,m10_phase6_execution_budget_archive_entries_exceeded,m10_phase6_execution_budget_invalid"
    ));
    assert!(DOC.contains(
        "phase6_execution_tick_budget_contract=due_candidates<=max_due_candidates;shredded_messages<=max_shredded_messages;projection_reports<=max_projection_reports;archived_entries<=max_archived_entries"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed -- --exact"
    ));
    assert!(DOC.contains("Regression: #5291"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_scheduler_cycle_markers() {
    assert_doc_contains_all(&["## Phase-6 Scheduler Cycle Trigger and Guarded Execution Contracts (Issue #5293)", "phase6_scheduler_cycle_status=verified", "phase6_scheduler_trigger_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-trigger-reason-taxonomy.v1", "phase6_scheduler_trigger_reason_codes_csv=m10_phase6_scheduler_trigger_deferred,m10_phase6_scheduler_trigger_due_threshold,m10_phase6_scheduler_trigger_interval_elapsed", "phase6_scheduler_cycle_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-cycle-reason-taxonomy.v1", "phase6_scheduler_cycle_reason_codes_csv=m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_policy_invalid,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded,m10_phase6_execution_budget_shredded_messages_exceeded,m10_phase6_execution_budget_projections_exceeded,m10_phase6_execution_budget_archive_entries_exceeded", "phase6_scheduler_cycle_contract=trigger_decision->preflight_budget_admission->phase6_execution_tick->budget_evidence", "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred -- --exact", "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution -- --exact", "Regression: #5293"]);
}
#[test]
fn service_api_ops_configuration_contains_phase6_scheduler_runtime_checkpoint_markers() {
    assert!(
        DOC.contains("## Phase-6 Stateful Scheduler Runtime Checkpoint Contracts (Issue #5295)")
    );
    assert!(DOC.contains("phase6_scheduler_runtime_checkpoint_status=verified"));
    assert!(DOC.contains(
        "phase6_scheduler_runtime_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-scheduler-runtime-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_scheduler_runtime_reason_codes_csv=m10_phase6_scheduler_runtime_initialized,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded"
    ));
    assert!(DOC.contains(
        "phase6_scheduler_runtime_state_contract=total_cycles=executed_cycles+deferred_cycles+fail_closed_cycles;last_successful_tick_epoch_seconds_updates_on_applied_only;last_observed_now_epoch_seconds_monotonic_non_decreasing"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance -- --exact"
    ));
    assert!(DOC.contains("Regression: #5295"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_runtime_evidence_bundle_markers() {
    assert!(DOC.contains("## Phase-6 Runtime Evidence Bundle Projection Contracts (Issue #5297)"));
    assert!(DOC.contains("phase6_runtime_evidence_bundle_status=verified"));
    assert!(DOC.contains(
        "phase6_runtime_evidence_reason_taxonomy_version=kamn.runtime.data-layer-m10-phase6-runtime-evidence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_runtime_evidence_reason_codes_csv=m10_phase6_runtime_evidence_applied,m10_phase6_runtime_evidence_deferred,m10_phase6_runtime_evidence_input_invalid"
    ));
    assert!(DOC.contains(
        "phase6_runtime_evidence_bundle_contract=cycle_report+runtime_state->canonical_evidence_bundle;applied_requires_execution_and_budget_payload;deferred_requires_empty_execution_payload"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete -- --exact"
    ));
    assert!(DOC.contains("Regression: #5297"));
}

#[test]
fn service_api_ops_configuration_contains_phase6_daemon_runtime_integration_markers() {
    assert!(DOC.contains("## Phase-6 Daemon Runtime Integration Contracts (Issue #5299)"));
    assert!(DOC.contains("phase6_daemon_runtime_contract_status=verified"));
    assert!(DOC.contains(
        "phase6_daemon_runtime_reason_taxonomy_version=kamn.runtime.daemon.phase6.reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "phase6_daemon_runtime_reason_codes_csv=m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded"
    ));
    assert!(DOC.contains(
        "phase6_daemon_runtime_contract=daemon_tick_executes_m10_scheduler_runtime;report_projects_phase6_reason_and_counters;clock_regression_fails_closed"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_projects_phase6_applied_runtime_markers_in_report_output -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_daemon_phase6_runtime_projection_fail_closed_reason_is_stable_on_clock_regression -- --exact"
    ));
    assert!(DOC.contains("Regression: #5299"));
}
