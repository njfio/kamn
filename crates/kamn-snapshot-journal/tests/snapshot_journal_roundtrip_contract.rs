use std::fs;
use std::path::PathBuf;

const REQUIRED_TEST_MARKERS: [&str; 4] = [
    "fn integration_append_and_parse_round_trip_whitespace_payload()",
    "fn integration_append_and_parse_round_trip_unicode_payload()",
    "fn integration_append_and_parse_round_trip_multiline_json_payload()",
    "fn integration_checked_parse_and_decode_restore_payload_exactly()",
];

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_snapshot_journal_roundtrip_target_exists_with_required_markers() {
    let source =
        repo_file("crates/kamn-snapshot-journal/tests/snapshot_journal_roundtrip_integration.rs");
    for marker in REQUIRED_TEST_MARKERS {
        assert!(
            source.contains(marker),
            "snapshot-journal roundtrip target should contain marker: {marker}"
        );
    }
}
