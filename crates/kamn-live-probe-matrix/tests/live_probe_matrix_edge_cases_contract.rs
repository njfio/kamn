use std::fs;
use std::path::PathBuf;

const REQUIRED_TEST_MARKERS: [&str; 5] = [
    "fn integration_empty_report_has_no_overall_status_or_modes()",
    "fn integration_all_skip_report_aggregates_to_skip()",
    "fn integration_summary_projection_preserves_all_skip_counts_and_status()",
    "fn integration_status_for_trims_lookup_and_rejects_empty_lookup()",
    "fn integration_mode_status_map_is_deterministic_for_mixed_modes()",
];

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_edge_case_target_exists_with_required_markers() {
    let source = repo_file("crates/kamn-live-probe-matrix/tests/live_probe_matrix_edge_cases.rs");
    for marker in REQUIRED_TEST_MARKERS {
        assert!(
            source.contains(marker),
            "live-probe edge-case target should contain marker: {marker}"
        );
    }
}
