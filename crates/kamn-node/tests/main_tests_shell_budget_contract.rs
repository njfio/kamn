use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_repo_file(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative)).unwrap_or_else(|error| {
        panic!("failed to read {relative}: {error}");
    })
}

#[test]
fn regression_main_tests_root_extracts_shared_support_surface() {
    let root = read_repo_file("src/main_tests.rs");
    let support = repo_root().join("src/main_tests/support.rs");

    assert!(support.exists(), "src/main_tests/support.rs should exist");
    assert!(
        root.contains("mod support;"),
        "main_tests.rs should declare the extracted support module"
    );
    for marker in [
        "struct EnvVarGuard",
        "struct MockHttpReply",
        "fn signer_env_lock()",
        "fn lock_signer_env_guard()",
        "fn log_env_lock()",
        "fn managed_signer_public_key_hex(",
        "fn read_http_request(",
        "fn request_body(",
        "fn project_json_value_to_string(",
        "fn find_json_field_value(",
        "fn extract_json_string_field(",
        "fn spawn_kolme_live_mock_server(",
    ] {
        assert!(
            !root.contains(marker),
            "main_tests.rs should not retain shared helper marker: {marker}"
        );
    }
}

#[test]
fn regression_main_tests_root_stays_well_under_shell_cap_after_support_extraction() {
    let root = read_repo_file("src/main_tests.rs");
    let line_count = root.lines().count();
    assert!(
        line_count <= 220,
        "main_tests.rs should drop well below the shell cap after support extraction; got {line_count}"
    );
}

#[test]
fn regression_main_tests_support_file_stays_bounded() {
    let support = read_repo_file("src/main_tests/support.rs");
    let line_count = support.lines().count();
    assert!(
        line_count <= 200,
        "main_tests/support.rs should remain within the touched file budget; got {line_count}"
    );
}
