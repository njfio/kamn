use std::path::{Path, PathBuf};

const BASELINE_FILE: &str = "fixtures/ci/ignored_test_inventory_baseline.json";
const METADATA_FILE: &str = "fixtures/ci/ignored_test_inventory_metadata.json";
const EXPECTED_IGNORED_TEST_COUNT: &str = r#""ignored_test_count": 18"#;
const MOVED_CHANNEL_PERFORMANCE_TEST: &str =
    "crates/kamn-core/src/channel_models/tests/file_store_contract_tests.rs";
const LIVE_S04_CLI_TEST: &str = "crates/kamn-e2e-harness/tests/live_s04_cli_scripted_execution.rs";
const LIVE_SOLANA_ASSET_MOVEMENT_TEST: &str = "task_escrow_solana_asset_movement_live_contract_tests.rs";

#[test]
fn ignored_test_inventory_fixtures_track_generated_current_inventory() {
    let baseline = read_repo_file(BASELINE_FILE);
    let metadata = read_repo_file(METADATA_FILE);

    assert_contains(&baseline, EXPECTED_IGNORED_TEST_COUNT, "baseline count");
    assert_contains(&baseline, MOVED_CHANNEL_PERFORMANCE_TEST, "moved channel test");
    assert_contains(&metadata, MOVED_CHANNEL_PERFORMANCE_TEST, "moved channel metadata");
    assert_contains(&baseline, LIVE_S04_CLI_TEST, "live S04 CLI baseline");
    assert_contains(&metadata, LIVE_S04_CLI_TEST, "live S04 CLI metadata");
    assert_contains(
        &baseline,
        LIVE_SOLANA_ASSET_MOVEMENT_TEST,
        "live Solana asset-movement baseline",
    );
    assert_contains(
        &metadata,
        LIVE_SOLANA_ASSET_MOVEMENT_TEST,
        "live Solana asset-movement metadata",
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
