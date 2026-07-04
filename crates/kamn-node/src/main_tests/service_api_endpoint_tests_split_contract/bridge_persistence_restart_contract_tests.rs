use super::support::*;

#[test]
fn spec_c29_service_api_endpoint_root_file_removes_moved_bridge_persistence_restart_contract() {
    let source = read_repo_file(ROOT_FILE);
    let marker = "fn integration_service_api_endpoint_persists_bridge_state_across_restart()";
    assert!(
        !source.contains(marker),
        "service_api_endpoint_tests.rs should not keep moved bridge persistence restart marker: {marker}"
    );
}

#[test]
fn spec_c30_service_api_endpoint_bridge_persistence_restart_module_exists_and_owns_moved_coverage()
{
    let module_source = read_repo_file(BRIDGE_PERSISTENCE_RESTART_MODULE_FILE);
    let restart_source = read_repo_file(BRIDGE_PERSISTENCE_RESTART_FILE);

    assert!(
        module_source.contains("mod restart_contract_tests;"),
        "bridge_persistence_restart_contract_tests.rs should declare restart submodule alias"
    );
    assert!(
        restart_source
            .contains("fn integration_service_api_endpoint_persists_bridge_state_across_restart()"),
        "bridge persistence restart contract file should include moved restart marker"
    );
}

#[test]
fn spec_c31_service_api_endpoint_root_declares_bridge_persistence_restart_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod bridge_persistence_restart_contract_tests;"),
        "service_api_endpoint_tests.rs should declare bridge-persistence-restart submodule"
    );
}

#[test]
fn spec_c32_service_api_endpoint_bridge_persistence_restart_split_files_stay_below_budget() {
    for path in [
        BRIDGE_PERSISTENCE_RESTART_MODULE_FILE,
        BRIDGE_PERSISTENCE_RESTART_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
