use std::fs;
use std::path::PathBuf;

const REQUIRED_TEST_MARKERS: [&str; 4] = [
    "fn integration_plain_agent_did_reports_missing_key_binding()",
    "fn integration_bound_agent_did_rejects_mismatched_public_key_with_typed_error()",
    "fn integration_parse_helpers_preserve_displayable_typed_errors()",
    "fn integration_bound_agent_did_round_trips_after_parse()",
];

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_did_boundary_regression_target_exists_with_required_markers() {
    let source = repo_file("crates/kamn-types/tests/did_boundary_regressions.rs");
    for marker in REQUIRED_TEST_MARKERS {
        assert!(
            source.contains(marker),
            "kamn-types did boundary regression target should contain marker: {marker}"
        );
    }
}
