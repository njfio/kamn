use std::fs;
use std::path::{Path, PathBuf};

const ROOT_MAX_LINES: usize = 180;
const LEAF_MAX_LINES: usize = 200;
const ROOT_FILE: &str = "crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs";

#[test]
fn kolme_runtime_commit_http_transport_root_shell_budget_is_enforced() {
    assert!(line_count(repo_path(ROOT_FILE)) <= ROOT_MAX_LINES);
}

#[test]
fn kolme_runtime_commit_http_transport_root_declares_expected_modules() {
    let root = read(ROOT_FILE);
    for marker in expected_root_markers() {
        assert!(root.contains(marker), "missing root marker: {marker}");
    }
}

#[test]
fn kolme_runtime_commit_http_transport_root_no_longer_contains_representative_moved_tests() {
    let root = read(ROOT_FILE);
    for marker in moved_test_markers() {
        assert!(!root.contains(marker), "root still contains moved marker: {marker}");
    }
}

#[test]
fn kolme_runtime_commit_http_transport_extracted_files_exist_and_stay_bounded() {
    for rel in expected_leaf_files() {
        let path = repo_path(rel);
        assert!(path.is_file(), "missing extracted file: {rel}");
        assert!(line_count(path) <= LEAF_MAX_LINES, "leaf too large: {rel}");
    }
}

fn expected_root_markers() -> &'static [&'static str] {
    &[
        "mod support;",
        "mod http_transport_contract_tests;",
        "mod typed_broadcast_contract_tests;",
        "mod tls_transport_contract_tests;",
        "mod fork_profile_contract_tests;",
        "mod live_smoke_contract_tests;",
    ]
}

fn moved_test_markers() -> &'static [&'static str] {
    &[
        "fn unit_http_transport_rejects_zero_timeout_seconds()",
        "fn integration_http_transport_submit_broadcast_request_put_and_parse_txhash()",
        "fn functional_https_transport_submit_with_trusted_ca_succeeds()",
        "fn functional_kolme_fork_submit_profile_uses_put_broadcast_and_maps_txhash_response()",
        "fn integration_kolme_fork_live_node_submit_reaches_endpoint()",
    ]
}

fn expected_leaf_files() -> &'static [&'static str] {
    &[
        "crates/kamn-core/tests/kolme_runtime_commit_http_transport/support.rs",
        "crates/kamn-core/tests/kolme_runtime_commit_http_transport/http_transport_contract_tests.rs",
        "crates/kamn-core/tests/kolme_runtime_commit_http_transport/typed_broadcast_contract_tests.rs",
        "crates/kamn-core/tests/kolme_runtime_commit_http_transport/tls_transport_contract_tests.rs",
        "crates/kamn-core/tests/kolme_runtime_commit_http_transport/fork_profile_contract_tests.rs",
        "crates/kamn-core/tests/kolme_runtime_commit_http_transport/live_smoke_contract_tests.rs",
    ]
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join(rel)
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_path(rel)).expect("contract fixture should be readable")
}

fn line_count(path: PathBuf) -> usize {
    fs::read_to_string(path)
        .expect("contract fixture should be readable")
        .lines()
        .count()
}
