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

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_runtime_evidence_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/runtime_evidence_cases.rs");

    for marker in RUNTIME_EVIDENCE_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root archival contract should contain runtime-evidence delegation marker: {marker}"
        );
    }

    for marker in RUNTIME_EVIDENCE_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "runtime-evidence cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c02_seam_port_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/seam_port_cases.rs");

    for marker in SEAM_PORT_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root archival contract should contain seam-port delegation marker: {marker}"
        );
    }

    for marker in SEAM_PORT_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "seam-port cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c03_scheduler_runtime_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases =
        read_repo_file("tests/data_layer_m10_partition_archival/scheduler_runtime_cases.rs");

    for marker in SCHEDULER_RUNTIME_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root archival contract should contain scheduler-runtime delegation marker: {marker}"
        );
    }

    for marker in SCHEDULER_RUNTIME_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "scheduler-runtime cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c04_scheduler_cycle_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/scheduler_cycle_cases.rs");

    for marker in SCHEDULER_CYCLE_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root archival contract should contain scheduler-cycle delegation marker: {marker}"
        );
    }

    for marker in SCHEDULER_CYCLE_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "scheduler-cycle cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c05_execution_budget_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/execution_budget_cases.rs");

    for marker in EXECUTION_BUDGET_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root archival contract should contain execution-budget delegation marker: {marker}"
        );
    }

    for marker in EXECUTION_BUDGET_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "execution-budget cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c06_orchestration_ordering_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases =
        read_repo_file("tests/data_layer_m10_partition_archival/orchestration_ordering_cases.rs");

    for marker in ORCHESTRATION_ORDERING_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root archival contract should contain orchestration-ordering delegation marker: {marker}"
        );
    }

    for marker in ORCHESTRATION_ORDERING_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "orchestration-ordering cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c07_retry_policy_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/retry_policy_cases.rs");

    for marker in RETRY_POLICY_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root archival contract should contain retry-policy delegation marker: {marker}"
        );
    }

    for marker in RETRY_POLICY_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "retry-policy cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c08_lifecycle_basics_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m10_partition_archival.rs");
    let cases = read_repo_file("tests/data_layer_m10_partition_archival/lifecycle_basics_cases.rs");

    for marker in LIFECYCLE_BASICS_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root archival contract should contain lifecycle-basics delegation marker: {marker}"
        );
    }

    for marker in LIFECYCLE_BASICS_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "lifecycle-basics cases module should define marker: {marker}"
        );
    }
}
