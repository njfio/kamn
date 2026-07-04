use super::support::*;

#[test]
fn spec_c25_service_api_endpoint_root_file_removes_moved_content_lifecycle_restart_contract() {
    let source = read_repo_file(ROOT_FILE);
    let marker =
        "fn integration_service_api_endpoint_persists_content_lifecycle_state_across_restart()";
    assert!(
        !source.contains(marker),
        "service_api_endpoint_tests.rs should not keep moved content lifecycle restart marker: {marker}"
    );
}

#[test]
fn spec_c26_service_api_endpoint_content_lifecycle_restart_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(CONTENT_LIFECYCLE_RESTART_MODULE_FILE);
    let restart_source = read_repo_file(CONTENT_LIFECYCLE_RESTART_FILE);

    assert!(
        module_source.contains("mod restart_contract_tests;"),
        "content_lifecycle_restart_contract_tests.rs should declare restart submodule alias"
    );
    assert!(
        restart_source.contains(
            "fn integration_service_api_endpoint_persists_content_lifecycle_state_across_restart()"
        ),
        "content_lifecycle restart contract file should include moved restart marker"
    );
}

#[test]
fn spec_c27_service_api_endpoint_root_declares_content_lifecycle_restart_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod content_lifecycle_restart_contract_tests;"),
        "service_api_endpoint_tests.rs should declare content-lifecycle-restart submodule"
    );
}

#[test]
fn spec_c28_service_api_endpoint_content_lifecycle_restart_split_files_stay_below_budget() {
    for path in [
        CONTENT_LIFECYCLE_RESTART_MODULE_FILE,
        CONTENT_LIFECYCLE_RESTART_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
