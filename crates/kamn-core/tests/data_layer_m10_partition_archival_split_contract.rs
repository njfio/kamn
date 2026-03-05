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
