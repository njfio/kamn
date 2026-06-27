use std::path::{Path, PathBuf};

const BASELINE_FILE: &str = "fixtures/ci/ignored_test_inventory_baseline.json";
const METADATA_FILE: &str = "fixtures/ci/ignored_test_inventory_metadata.json";
const EXPECTED_IGNORED_TEST_COUNT: &str = r#""ignored_test_count": 18"#;
const MOVED_CHANNEL_PERFORMANCE_TEST: &str =
    "crates/kamn-core/src/channel_models/tests/file_store_contract_tests.rs";
const LIVE_S04_CLI_TEST: &str = "crates/kamn-e2e-harness/tests/live_s04_cli_scripted_execution.rs";
const LIVE_SOLANA_ASSET_MOVEMENT_TEST: &str = "task_escrow_solana_asset_movement_live_contract_tests.rs";
const LOCAL_HEAVY_REASON: &str = r#""reason": "local-heavy-live-node""#;
const REPRESENTATIVE_CURRENT_ENTRIES: &[(&str, &str)] = &[
    (MOVED_CHANNEL_PERFORMANCE_TEST, "moved channel test"),
    (LIVE_S04_CLI_TEST, "live S04 CLI"),
    (LIVE_SOLANA_ASSET_MOVEMENT_TEST, "live Solana asset-movement"),
];

#[test]
fn ignored_test_inventory_fixtures_track_generated_current_inventory() {
    let baseline = read_repo_file(BASELINE_FILE);
    let metadata = read_repo_file(METADATA_FILE);

    assert_contains(&baseline, EXPECTED_IGNORED_TEST_COUNT, "baseline count");
    for (entry, label) in REPRESENTATIVE_CURRENT_ENTRIES {
        assert_contains(&baseline, entry, &format!("{label} baseline"));
        assert_contains(&metadata, entry, &format!("{label} metadata"));
    }
    assert_contains(
        &metadata,
        LOCAL_HEAVY_REASON,
        "local-heavy live-node metadata reason",
    );
}

#[test]
fn ignored_test_inventory_spec_records_fixture_refresh_evidence() {
    let spec = read_repo_file("specs/7027-repair-ignored-test-inventory-drift-gate.md");

    assert_contains(&spec, "ignored_test_count=18", "generated count evidence");
    assert_contains(
        &spec,
        "bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh",
        "metadata policy evidence",
    );
    assert_contains(
        &spec,
        "bash scripts/ci/test_ignored_test_inventory_parser_contract.sh",
        "parser contract evidence",
    );
}

fn assert_contains(haystack: &str, needle: &str, label: &str) {
    assert!(haystack.contains(needle), "missing {label}: {needle}");
}

fn read_repo_file(relative_path: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn repo_root() -> PathBuf {
    manifest_dir()
        .parent()
        .and_then(Path::parent)
        .expect("kamn-core manifest should live under crates/kamn-core")
        .to_path_buf()
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
