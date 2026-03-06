use std::fs;
use std::path::PathBuf;

const REQUIRED_TEST_MARKERS: [&str; 5] = [
    "fn integration_append_multiple_records_preserves_order_and_newlines()",
    "fn integration_decode_accepts_uppercase_hex_payloads()",
    "fn integration_checked_parse_accepts_corrupted_payload_hex_but_decode_fails_closed()",
    "fn integration_checked_parse_rejects_missing_schema_version_field()",
    "fn integration_checked_parse_rejects_missing_payload_hex_field()",
];

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_edge_case_target_exists_with_required_markers() {
    let source = repo_file("crates/kamn-snapshot-journal/tests/snapshot_journal_edge_cases.rs");
    for marker in REQUIRED_TEST_MARKERS {
        assert!(
            source.contains(marker),
            "snapshot-journal edge-case target should contain marker: {marker}"
        );
    }
}
