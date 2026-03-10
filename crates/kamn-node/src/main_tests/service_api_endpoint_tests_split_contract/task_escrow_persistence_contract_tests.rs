use super::support::*;

#[test]
fn spec_c21_service_api_endpoint_root_file_removes_moved_task_escrow_persistence_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_persists_task_and_escrow_state_across_routes()",
        "fn integration_service_api_endpoint_persists_task_and_escrow_state_across_restart()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved task/escrow persistence marker: {marker}"
        );
    }
}

#[test]
fn spec_c22_service_api_endpoint_task_escrow_persistence_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(TASK_ESCROW_PERSISTENCE_MODULE_FILE);
    let routes_source = read_repo_file(TASK_ESCROW_ROUTES_FILE);
    let restart_source = read_repo_file(TASK_ESCROW_RESTART_FILE);

    assert!(
        module_source.contains("mod task_escrow_routes_contract_tests;"),
        "task_escrow_persistence_contract_tests.rs should declare routes submodule"
    );
    assert!(
        module_source.contains("mod task_escrow_restart_contract_tests;"),
        "task_escrow_persistence_contract_tests.rs should declare restart submodule"
    );

    assert!(
        routes_source.contains(
            "fn integration_service_api_endpoint_persists_task_and_escrow_state_across_routes()"
        ),
        "task_escrow_routes_contract_tests.rs should include moved routes marker"
    );
    assert!(
        restart_source.contains(
            "fn integration_service_api_endpoint_persists_task_and_escrow_state_across_restart()"
        ),
        "task_escrow_restart_contract_tests.rs should include moved restart marker"
    );
}

#[test]
fn spec_c23_service_api_endpoint_root_declares_task_escrow_persistence_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod task_escrow_persistence_contract_tests;"),
        "service_api_endpoint_tests.rs should declare task-escrow-persistence submodule"
    );
}

#[test]
fn spec_c24_service_api_endpoint_task_escrow_persistence_split_files_stay_below_budget() {
    for path in [
        TASK_ESCROW_PERSISTENCE_MODULE_FILE,
        TASK_ESCROW_ROUTES_FILE,
        TASK_ESCROW_RESTART_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
