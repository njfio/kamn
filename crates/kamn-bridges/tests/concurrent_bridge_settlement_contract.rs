use std::fs;
use std::path::PathBuf;

const REQUIRED_TEST_MARKERS: [&str; 3] = [
    "fn integration_concurrent_mixed_proofs_yield_expected_decisions()",
    "fn integration_concurrent_identical_final_proof_always_settles()",
    "fn integration_concurrent_pending_and_invalid_proofs_preserve_decision_boundaries()",
];

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_concurrent_settlement_target_exists_with_required_markers() {
    let source = repo_file("crates/kamn-bridges/tests/concurrent_bridge_settlement_integration.rs");
    for marker in REQUIRED_TEST_MARKERS {
        assert!(
            source.contains(marker),
            "concurrent bridge settlement target should contain marker: {marker}"
        );
    }
}
