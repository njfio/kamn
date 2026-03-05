use std::fs;

const RUNTIME_EVIDENCE_ROOT_MARKERS: [&str; 6] = [
    "#[path = \"data_layer_m10_partition_archival/runtime_evidence_cases.rs\"]",
    "mod runtime_evidence_cases;",
    "runtime_evidence_cases::run_spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts();",
    "runtime_evidence_cases::run_spec_c34_phase6_runtime_evidence_bundle_projects_deferred_cycle_with_empty_artifacts();",
    "runtime_evidence_cases::run_spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete();",
    "runtime_evidence_cases::run_spec_c36_phase6_runtime_evidence_bundle_fails_closed_when_deferred_payload_contains_execution_data();",
];

const RUNTIME_EVIDENCE_CASES_MARKERS: [&str; 4] = [
    "pub(super) fn run_spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts(",
    "pub(super) fn run_spec_c34_phase6_runtime_evidence_bundle_projects_deferred_cycle_with_empty_artifacts(",
    "pub(super) fn run_spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete(",
    "pub(super) fn run_spec_c36_phase6_runtime_evidence_bundle_fails_closed_when_deferred_payload_contains_execution_data(",
];

const SEAM_PORT_ROOT_MARKERS: [&str; 4] = [
    "#[path = \"data_layer_m10_partition_archival/seam_port_cases.rs\"]",
    "mod seam_port_cases;",
    "seam_port_cases::run_spec_c37_partition_shred_projection_with_port_is_supported_without_direct_m8_registry_argument();",
    "seam_port_cases::run_spec_c38_phase6_orchestration_with_port_supports_seam_without_direct_m8_registry_argument();",
];

const SEAM_PORT_CASES_MARKERS: [&str; 2] = [
    "pub(super) fn run_spec_c37_partition_shred_projection_with_port_is_supported_without_direct_m8_registry_argument(",
    "pub(super) fn run_spec_c38_phase6_orchestration_with_port_supports_seam_without_direct_m8_registry_argument(",
];

const SCHEDULER_RUNTIME_ROOT_MARKERS: [&str; 7] = [
    "#[path = \"data_layer_m10_partition_archival/scheduler_runtime_cases.rs\"]",
    "mod scheduler_runtime_cases;",
    "scheduler_runtime_cases::run_spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint();",
    "scheduler_runtime_cases::run_spec_c29_phase6_scheduler_runtime_deferred_cycle_preserves_success_checkpoint();",
    "scheduler_runtime_cases::run_spec_c30_phase6_scheduler_runtime_applied_cycle_updates_success_checkpoint_and_counters();",
    "scheduler_runtime_cases::run_spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance();",
    "scheduler_runtime_cases::run_spec_c32_phase6_scheduler_runtime_clock_regression_fails_closed_and_preserves_checkpoint();",
];

const SCHEDULER_RUNTIME_CASES_MARKERS: [&str; 5] = [
    "pub(super) fn run_spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint(",
    "pub(super) fn run_spec_c29_phase6_scheduler_runtime_deferred_cycle_preserves_success_checkpoint(",
    "pub(super) fn run_spec_c30_phase6_scheduler_runtime_applied_cycle_updates_success_checkpoint_and_counters(",
    "pub(super) fn run_spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance(",
    "pub(super) fn run_spec_c32_phase6_scheduler_runtime_clock_regression_fails_closed_and_preserves_checkpoint(",
];

const SCHEDULER_CYCLE_ROOT_MARKERS: [&str; 7] = [
    "#[path = \"data_layer_m10_partition_archival/scheduler_cycle_cases.rs\"]",
    "mod scheduler_cycle_cases;",
    "scheduler_cycle_cases::run_spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred();",
    "scheduler_cycle_cases::run_spec_c24_phase6_scheduler_cycle_deferred_path_returns_no_execution_side_effects();",
    "scheduler_cycle_cases::run_spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution();",
    "scheduler_cycle_cases::run_spec_c26_phase6_scheduler_cycle_triggered_executes_within_budget_evidence();",
    "scheduler_cycle_cases::run_spec_c27_phase6_scheduler_policy_and_signal_validation_fail_closed();",
];

const SCHEDULER_CYCLE_CASES_MARKERS: [&str; 5] = [
    "pub(super) fn run_spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred(",
    "pub(super) fn run_spec_c24_phase6_scheduler_cycle_deferred_path_returns_no_execution_side_effects(",
    "pub(super) fn run_spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution(",
    "pub(super) fn run_spec_c26_phase6_scheduler_cycle_triggered_executes_within_budget_evidence(",
    "pub(super) fn run_spec_c27_phase6_scheduler_policy_and_signal_validation_fail_closed(",
];

const EXECUTION_BUDGET_ROOT_MARKERS: [&str; 7] = [
    "#[path = \"data_layer_m10_partition_archival/execution_budget_cases.rs\"]",
    "mod execution_budget_cases;",
    "execution_budget_cases::run_spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival();",
    "execution_budget_cases::run_spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries();",
    "execution_budget_cases::run_spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic();",
    "execution_budget_cases::run_spec_c21_phase6_execution_tick_budget_projection_and_archive_limits_fail_closed();",
    "execution_budget_cases::run_spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed();",
];

const EXECUTION_BUDGET_CASES_MARKERS: [&str; 5] = [
    "pub(super) fn run_spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival(",
    "pub(super) fn run_spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries(",
    "pub(super) fn run_spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic(",
    "pub(super) fn run_spec_c21_phase6_execution_tick_budget_projection_and_archive_limits_fail_closed(",
    "pub(super) fn run_spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed(",
];

const ORCHESTRATION_ORDERING_ROOT_MARKERS: [&str; 4] = [
    "#[path = \"data_layer_m10_partition_archival/orchestration_ordering_cases.rs\"]",
    "mod orchestration_ordering_cases;",
    "orchestration_ordering_cases::run_spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive();",
    "orchestration_ordering_cases::run_spec_c17_phase6_orchestration_tick_orders_outputs_deterministically();",
];

const ORCHESTRATION_ORDERING_CASES_MARKERS: [&str; 2] = [
    "pub(super) fn run_spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive(",
    "pub(super) fn run_spec_c17_phase6_orchestration_tick_orders_outputs_deterministically(",
];

const RETRY_POLICY_ROOT_MARKERS: [&str; 7] = [
    "#[path = \"data_layer_m10_partition_archival/retry_policy_cases.rs\"]",
    "mod retry_policy_cases;",
    "retry_policy_cases::run_spec_c12_transient_archival_failure_projects_deterministic_retry_window();",
    "retry_policy_cases::run_spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum();",
    "retry_policy_cases::run_spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed();",
    "retry_policy_cases::run_spec_c15_archival_retry_policy_and_attempt_validation_fail_closed();",
    "fn spec_c12_transient_archival_failure_projects_deterministic_retry_window()",
];

const RETRY_POLICY_CASES_MARKERS: [&str; 4] = [
    "pub(super) fn run_spec_c12_transient_archival_failure_projects_deterministic_retry_window(",
    "pub(super) fn run_spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum(",
    "pub(super) fn run_spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed(",
    "pub(super) fn run_spec_c15_archival_retry_policy_and_attempt_validation_fail_closed(",
];

const LIFECYCLE_BASICS_ROOT_MARKERS: [&str; 8] = [
    "#[path = \"data_layer_m10_partition_archival/lifecycle_basics_cases.rs\"]",
    "mod lifecycle_basics_cases;",
    "lifecycle_basics_cases::run_spec_c01_partition_naming_and_future_planning_are_deterministic();",
    "lifecycle_basics_cases::run_spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness();",
    "lifecycle_basics_cases::run_spec_c03_archival_index_records_and_reattach_transition_are_deterministic();",
    "lifecycle_basics_cases::run_spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed();",
    "lifecycle_basics_cases::run_spec_c05_duplicate_registration_and_partition_prefix_contract_are_enforced();",
    "fn spec_c01_partition_naming_and_future_planning_are_deterministic()",
];

const LIFECYCLE_BASICS_CASES_MARKERS: [&str; 5] = [
    "pub(super) fn run_spec_c01_partition_naming_and_future_planning_are_deterministic(",
    "pub(super) fn run_spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness(",
    "pub(super) fn run_spec_c03_archival_index_records_and_reattach_transition_are_deterministic(",
    "pub(super) fn run_spec_c04_invalid_month_identifiers_and_illegal_transitions_fail_closed(",
    "pub(super) fn run_spec_c05_duplicate_registration_and_partition_prefix_contract_are_enforced(",
];

const COMPLIANCE_PROJECTION_ROOT_MARKERS: [&str; 9] = [
    "#[path = \"data_layer_m10_partition_archival/compliance_projection_cases.rs\"]",
    "mod compliance_projection_cases;",
    "compliance_projection_cases::run_spec_c06_partition_shred_completeness_can_be_projected_from_m8_lifecycle_records();",
    "compliance_projection_cases::run_spec_c07_partition_shred_projection_fails_closed_when_m8_message_lookup_is_missing();",
    "compliance_projection_cases::run_spec_c08_partition_projection_accepts_canonical_equivalent_owner_dids();",
    "compliance_projection_cases::run_spec_c09_partition_projection_denies_non_equivalent_owner_dids();",
    "compliance_projection_cases::run_spec_c10_partition_projection_marks_legal_hold_as_archival_denied_reason();",
    "compliance_projection_cases::run_spec_c11_partition_archival_remains_blocked_until_legal_hold_is_released_and_shred_completes();",
    "fn spec_c06_partition_shred_completeness_can_be_projected_from_m8_lifecycle_records()",
];

const COMPLIANCE_PROJECTION_CASES_MARKERS: [&str; 6] = [
    "pub(super) fn run_spec_c06_partition_shred_completeness_can_be_projected_from_m8_lifecycle_records(",
    "pub(super) fn run_spec_c07_partition_shred_projection_fails_closed_when_m8_message_lookup_is_missing(",
    "pub(super) fn run_spec_c08_partition_projection_accepts_canonical_equivalent_owner_dids(",
    "pub(super) fn run_spec_c09_partition_projection_denies_non_equivalent_owner_dids(",
    "pub(super) fn run_spec_c10_partition_projection_marks_legal_hold_as_archival_denied_reason(",
    "pub(super) fn run_spec_c11_partition_archival_remains_blocked_until_legal_hold_is_released_and_shred_completes(",
];

const SHARED_HELPERS_ROOT_MARKERS: [&str; 3] = [
    "#[path = \"data_layer_m10_partition_archival/shared.rs\"]",
    "mod shared;",
    "use shared::*;",
];

const SHARED_HELPERS_CASES_MARKERS: [&str; 7] = [
    "pub(super) use kamn_core::{",
    "pub(super) use kamn_data_layer::{",
    "pub(super) fn partition_input(",
    "pub(super) fn m8_message_input(",
    "pub(super) fn project_request(",
    "pub(super) fn phase6_request(",
    "pub(super) fn phase6_runtime_state(",
];

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

fn normalize_contract_surface(content: &str) -> String {
    content.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn assert_contract_markers(surface: &str, markers: &[&str], message_prefix: &str) {
    let normalized_surface = normalize_contract_surface(surface);
    for marker in markers {
        let normalized_marker = normalize_contract_surface(marker);
        assert!(
            normalized_surface.contains(&normalized_marker),
            "{message_prefix}: {marker}"
        );
    }
}

#[test]
fn spec_c01_runtime_evidence_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/runtime_evidence_cases.rs");

    assert_contract_markers(
        &root,
        &RUNTIME_EVIDENCE_ROOT_MARKERS,
        "root archival contract should contain runtime-evidence delegation marker",
    );
    assert_contract_markers(
        &cases,
        &RUNTIME_EVIDENCE_CASES_MARKERS,
        "runtime-evidence cases module should define marker",
    );
}

#[test]
fn spec_c02_seam_port_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/seam_port_cases.rs");

    assert_contract_markers(
        &root,
        &SEAM_PORT_ROOT_MARKERS,
        "root archival contract should contain seam-port delegation marker",
    );
    assert_contract_markers(
        &cases,
        &SEAM_PORT_CASES_MARKERS,
        "seam-port cases module should define marker",
    );
}

#[test]
fn spec_c03_scheduler_runtime_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases =
        read_repo_file("tests/data_layer_m10_partition_archival/scheduler_runtime_cases.rs");

    assert_contract_markers(
        &root,
        &SCHEDULER_RUNTIME_ROOT_MARKERS,
        "root archival contract should contain scheduler-runtime delegation marker",
    );
    assert_contract_markers(
        &cases,
        &SCHEDULER_RUNTIME_CASES_MARKERS,
        "scheduler-runtime cases module should define marker",
    );
}

#[test]
fn spec_c04_scheduler_cycle_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/scheduler_cycle_cases.rs");

    assert_contract_markers(
        &root,
        &SCHEDULER_CYCLE_ROOT_MARKERS,
        "root archival contract should contain scheduler-cycle delegation marker",
    );
    assert_contract_markers(
        &cases,
        &SCHEDULER_CYCLE_CASES_MARKERS,
        "scheduler-cycle cases module should define marker",
    );
}

#[test]
fn spec_c05_execution_budget_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/execution_budget_cases.rs");

    assert_contract_markers(
        &root,
        &EXECUTION_BUDGET_ROOT_MARKERS,
        "root archival contract should contain execution-budget delegation marker",
    );
    assert_contract_markers(
        &cases,
        &EXECUTION_BUDGET_CASES_MARKERS,
        "execution-budget cases module should define marker",
    );
}

#[test]
fn spec_c06_orchestration_ordering_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases =
        read_repo_file("tests/data_layer_m10_partition_archival/orchestration_ordering_cases.rs");

    assert_contract_markers(
        &root,
        &ORCHESTRATION_ORDERING_ROOT_MARKERS,
        "root archival contract should contain orchestration-ordering delegation marker",
    );
    assert_contract_markers(
        &cases,
        &ORCHESTRATION_ORDERING_CASES_MARKERS,
        "orchestration-ordering cases module should define marker",
    );
}

#[test]
fn spec_c07_retry_policy_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/retry_policy_cases.rs");

    assert_contract_markers(
        &root,
        &RETRY_POLICY_ROOT_MARKERS,
        "root archival contract should contain retry-policy delegation marker",
    );
    assert_contract_markers(
        &cases,
        &RETRY_POLICY_CASES_MARKERS,
        "retry-policy cases module should define marker",
    );
}

#[test]
fn spec_c08_lifecycle_basics_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/lifecycle_basics_cases.rs");

    assert_contract_markers(
        &root,
        &LIFECYCLE_BASICS_ROOT_MARKERS,
        "root archival contract should contain lifecycle-basics delegation marker",
    );
    assert_contract_markers(
        &cases,
        &LIFECYCLE_BASICS_CASES_MARKERS,
        "lifecycle-basics cases module should define marker",
    );
}

#[test]
fn spec_c09_compliance_projection_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases =
        read_repo_file("tests/data_layer_m10_partition_archival/compliance_projection_cases.rs");

    assert_contract_markers(
        &root,
        &COMPLIANCE_PROJECTION_ROOT_MARKERS,
        "root archival contract should contain compliance-projection delegation marker",
    );
    assert_contract_markers(
        &cases,
        &COMPLIANCE_PROJECTION_CASES_MARKERS,
        "compliance-projection cases module should define marker",
    );
}

#[test]
fn spec_c10_shared_helpers_are_extracted_to_shared_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let shared = read_repo_file("tests/data_layer_m10_partition_archival/shared.rs");

    assert_contract_markers(
        &root,
        &SHARED_HELPERS_ROOT_MARKERS,
        "root archival contract should contain shared-helper wiring marker",
    );
    assert_contract_markers(
        &shared,
        &SHARED_HELPERS_CASES_MARKERS,
        "shared helper module should define marker",
    );

    for removed_root_helper in [
        "fn partition_input(",
        "fn m8_message_input(",
        "fn project_request(",
        "fn phase6_request(",
        "fn phase6_budget(",
        "fn phase6_scheduler_policy(",
        "fn phase6_runtime_state(",
    ] {
        assert!(
            !root.contains(removed_root_helper),
            "root archival contract should not keep extracted helper: {removed_root_helper}"
        );
    }
}

#[test]
fn spec_c11_root_archival_contract_file_stays_within_size_budget() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let line_count = root.lines().count();
    assert!(
        line_count <= 200,
        "data_layer_m10_partition_archival.rs should stay within 200-line budget; got {line_count}"
    );
}
