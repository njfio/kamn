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
