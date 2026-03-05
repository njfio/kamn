#[path = "data_layer_m10_partition_archival/compliance_projection_cases.rs"]
mod compliance_projection_cases;
#[path = "data_layer_m10_partition_archival/execution_budget_cases.rs"]
mod execution_budget_cases;
#[path = "data_layer_m10_partition_archival/lifecycle_basics_cases.rs"]
mod lifecycle_basics_cases;
#[path = "data_layer_m10_partition_archival/orchestration_ordering_cases.rs"]
mod orchestration_ordering_cases;
#[path = "data_layer_m10_partition_archival/retry_policy_cases.rs"]
mod retry_policy_cases;
#[path = "data_layer_m10_partition_archival/runtime_evidence_cases.rs"]
mod runtime_evidence_cases;
#[path = "data_layer_m10_partition_archival/scheduler_cycle_cases.rs"]
mod scheduler_cycle_cases;
#[path = "data_layer_m10_partition_archival/scheduler_runtime_cases.rs"]
mod scheduler_runtime_cases;
#[path = "data_layer_m10_partition_archival/seam_port_cases.rs"]
mod seam_port_cases;
#[path = "data_layer_m10_partition_archival/shared.rs"]
mod shared;

use shared::*;

#[test]
fn spec_c01_partition_naming_and_future_planning_are_deterministic() {
    lifecycle_basics_cases::run_spec_c01_partition_naming_and_future_planning_are_deterministic();
}

#[test]
fn spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness() {
    lifecycle_basics_cases::run_spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness();
}

#[test]
fn spec_c03_archival_index_records_and_reattach_transition_are_deterministic() {
    lifecycle_basics_cases::run_spec_c03_archival_index_records_and_reattach_transition_are_deterministic();
}

#[test]
fn spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed() {
    lifecycle_basics_cases::run_spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed();
}

#[test]
fn spec_c05_duplicate_registration_and_partition_prefix_contract_are_enforced() {
    lifecycle_basics_cases::run_spec_c05_duplicate_registration_and_partition_prefix_contract_are_enforced();
}

#[test]
fn spec_c06_partition_shred_completeness_can_be_projected_from_m8_lifecycle_records() {
    compliance_projection_cases::run_spec_c06_partition_shred_completeness_can_be_projected_from_m8_lifecycle_records();
}

#[test]
fn spec_c07_partition_shred_projection_fails_closed_when_m8_message_lookup_is_missing() {
    compliance_projection_cases::run_spec_c07_partition_shred_projection_fails_closed_when_m8_message_lookup_is_missing();
}

#[test]
fn spec_c08_partition_projection_accepts_canonical_equivalent_owner_dids() {
    compliance_projection_cases::run_spec_c08_partition_projection_accepts_canonical_equivalent_owner_dids();
}

#[test]
fn spec_c09_partition_projection_denies_non_equivalent_owner_dids() {
    compliance_projection_cases::run_spec_c09_partition_projection_denies_non_equivalent_owner_dids();
}

#[test]
fn spec_c10_partition_projection_marks_legal_hold_as_archival_denied_reason() {
    compliance_projection_cases::run_spec_c10_partition_projection_marks_legal_hold_as_archival_denied_reason();
}

#[test]
fn spec_c11_partition_archival_remains_blocked_until_legal_hold_is_released_and_shred_completes() {
    compliance_projection_cases::run_spec_c11_partition_archival_remains_blocked_until_legal_hold_is_released_and_shred_completes();
}

#[test]
fn spec_c12_transient_archival_failure_projects_deterministic_retry_window() {
    retry_policy_cases::run_spec_c12_transient_archival_failure_projects_deterministic_retry_window();
}

#[test]
fn spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum() {
    retry_policy_cases::run_spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum();
}

#[test]
fn spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed() {
    retry_policy_cases::run_spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed();
}

#[test]
fn spec_c15_archival_retry_policy_and_attempt_validation_fail_closed() {
    retry_policy_cases::run_spec_c15_archival_retry_policy_and_attempt_validation_fail_closed();
}

#[test]
fn spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive() {
    orchestration_ordering_cases::run_spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive();
}

#[test]
fn spec_c17_phase6_orchestration_tick_orders_outputs_deterministically() {
    orchestration_ordering_cases::run_spec_c17_phase6_orchestration_tick_orders_outputs_deterministically();
}

#[test]
fn spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival() {
    execution_budget_cases::run_spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival();
}

#[test]
fn spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries() {
    execution_budget_cases::run_spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries();
}

#[test]
fn spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic() {
    execution_budget_cases::run_spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic();
}

#[test]
fn spec_c21_phase6_execution_tick_budget_projection_and_archive_limits_fail_closed() {
    execution_budget_cases::run_spec_c21_phase6_execution_tick_budget_projection_and_archive_limits_fail_closed();
}

#[test]
fn spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed() {
    execution_budget_cases::run_spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed();
}

#[test]
fn spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred() {
    scheduler_cycle_cases::run_spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred();
}

#[test]
fn spec_c24_phase6_scheduler_cycle_deferred_path_returns_no_execution_side_effects() {
    scheduler_cycle_cases::run_spec_c24_phase6_scheduler_cycle_deferred_path_returns_no_execution_side_effects();
}

#[test]
fn spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution() {
    scheduler_cycle_cases::run_spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution();
}

#[test]
fn spec_c26_phase6_scheduler_cycle_triggered_executes_within_budget_evidence() {
    scheduler_cycle_cases::run_spec_c26_phase6_scheduler_cycle_triggered_executes_within_budget_evidence();
}

#[test]
fn spec_c27_phase6_scheduler_policy_and_signal_validation_fail_closed() {
    scheduler_cycle_cases::run_spec_c27_phase6_scheduler_policy_and_signal_validation_fail_closed();
}

#[test]
fn spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint() {
    scheduler_runtime_cases::run_spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint();
}

#[test]
fn spec_c29_phase6_scheduler_runtime_deferred_cycle_preserves_success_checkpoint() {
    scheduler_runtime_cases::run_spec_c29_phase6_scheduler_runtime_deferred_cycle_preserves_success_checkpoint();
}

#[test]
fn spec_c30_phase6_scheduler_runtime_applied_cycle_updates_success_checkpoint_and_counters() {
    scheduler_runtime_cases::run_spec_c30_phase6_scheduler_runtime_applied_cycle_updates_success_checkpoint_and_counters();
}

#[test]
fn spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance(
) {
    scheduler_runtime_cases::run_spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance();
}

#[test]
fn spec_c32_phase6_scheduler_runtime_clock_regression_fails_closed_and_preserves_checkpoint() {
    scheduler_runtime_cases::run_spec_c32_phase6_scheduler_runtime_clock_regression_fails_closed_and_preserves_checkpoint();
}

#[test]
fn spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts() {
    runtime_evidence_cases::run_spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts();
}

#[test]
fn spec_c34_phase6_runtime_evidence_bundle_projects_deferred_cycle_with_empty_artifacts() {
    runtime_evidence_cases::run_spec_c34_phase6_runtime_evidence_bundle_projects_deferred_cycle_with_empty_artifacts();
}

#[test]
fn spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete() {
    runtime_evidence_cases::run_spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete();
}

#[test]
fn spec_c36_phase6_runtime_evidence_bundle_fails_closed_when_deferred_payload_contains_execution_data(
) {
    runtime_evidence_cases::run_spec_c36_phase6_runtime_evidence_bundle_fails_closed_when_deferred_payload_contains_execution_data();
}

#[test]
fn spec_c37_partition_shred_projection_with_port_is_supported_without_direct_m8_registry_argument()
{
    seam_port_cases::run_spec_c37_partition_shred_projection_with_port_is_supported_without_direct_m8_registry_argument();
}

#[test]
fn spec_c38_phase6_orchestration_with_port_supports_seam_without_direct_m8_registry_argument() {
    seam_port_cases::run_spec_c38_phase6_orchestration_with_port_supports_seam_without_direct_m8_registry_argument();
}
