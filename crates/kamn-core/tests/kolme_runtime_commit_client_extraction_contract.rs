use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/kolme_runtime_commit_client.rs";
const ROOT_CAP: usize = 180;
const MODULE_FILES: &[&str] = &[
    "tests/kolme_runtime_commit_client/request_validation_contract_tests.rs",
    "tests/kolme_runtime_commit_client/adapter_backed_client_contract_tests.rs",
    "tests/kolme_runtime_commit_client/live_provider_contract_tests.rs",
    "tests/kolme_runtime_commit_client/finality_checker_contract_tests.rs",
    "tests/kolme_runtime_commit_client/support.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod request_validation_contract_tests;",
    "mod adapter_backed_client_contract_tests;",
    "mod live_provider_contract_tests;",
    "mod finality_checker_contract_tests;",
    "mod support;",
];
const MOVED_MARKERS: &[&str] = &[
    "fn parse_fixture_cases()",
    "struct RecordingProvider",
    "fn unit_commit_request_wire_payload_is_deterministic()",
    "fn unit_adapter_normalizes_wire_payload_and_idempotency_key_before_submit()",
    "fn functional_live_provider_maps_submitted_json_response_to_provider_outcome()",
    "fn functional_finality_checker_maps_confirmed_alias_to_final_receipt()",
];

#[test]
fn kolme_runtime_commit_client_root_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    assert_root_shell_budget(&root);
    assert_required_markers(&root);
    assert_moved_markers_removed(&root);
    assert_module_files_exist_and_fit_budget();
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_root_shell_budget(root: &str) {
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_CAP,
        "expected {ROOT} <= {ROOT_CAP} lines after extraction, found {lines}"
    );
}

fn assert_required_markers(root: &str) {
    for marker in REQUIRED_MARKERS {
        assert!(
            root.contains(marker),
            "missing root module marker: {marker}"
        );
    }
}

fn assert_moved_markers_removed(root: &str) {
    for marker in MOVED_MARKERS {
        assert!(
            !root.contains(marker),
            "moved marker still present in root: {marker}"
        );
    }
}

fn assert_module_files_exist_and_fit_budget() {
    for path in MODULE_FILES {
        let full = repo_path(path);
        assert!(
            full.exists(),
            "missing extracted module: {}",
            full.display()
        );
        let lines = fs::read_to_string(&full)
            .expect("read module")
            .lines()
            .count();
        assert!(
            lines <= 200,
            "extracted module exceeds 200 lines: {}",
            full.display()
        );
    }
}
