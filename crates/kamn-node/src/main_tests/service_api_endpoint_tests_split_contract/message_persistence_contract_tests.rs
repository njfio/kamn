use super::support::*;

const MESSAGE_PERSISTENCE_SUBMODULE_MARKERS: &[&str] = &[
    "mod message_restart_contract_tests;",
    "mod message_runtime_evidence_contract_tests;",
];
const MESSAGE_RESTART_MARKERS: &[&str] = &[
    "fn integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env(",
    "fn integration_service_api_endpoint_persists_message_state_across_restart()",
    "let query_path = format!(\"/v1/messages/{}\", send_payload.message_id);",
    "let _ = fs::remove_file(state_file);",
];
const MESSAGE_RUNTIME_EVIDENCE_MARKERS: &[&str] = &[
    "fn integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11()",
    "data_layer_runtime_evidence",
    "m11_decision",
];

#[test]
fn spec_c13_service_api_endpoint_root_file_removes_moved_message_persistence_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env(",
        "fn integration_service_api_endpoint_persists_message_state_across_restart()",
        "fn integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11()",
        "let query_path = format!(\"/v1/messages/{}\", send_payload.message_id);",
        "data_layer_runtime_evidence",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved message-persistence marker: {marker}"
        );
    }
}

#[test]
fn spec_c14_service_api_endpoint_message_persistence_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(MESSAGE_PERSISTENCE_MODULE_FILE);
    let restart_source = read_repo_file(MESSAGE_RESTART_FILE);
    let runtime_evidence_source = read_repo_file(MESSAGE_RUNTIME_EVIDENCE_FILE);

    assert_contains_markers(
        module_source.as_str(),
        MESSAGE_PERSISTENCE_SUBMODULE_MARKERS,
        "message-persistence module",
    );
    assert_contains_markers(
        restart_source.as_str(),
        MESSAGE_RESTART_MARKERS,
        "message-restart contract file",
    );
    assert_contains_markers(
        runtime_evidence_source.as_str(),
        MESSAGE_RUNTIME_EVIDENCE_MARKERS,
        "message-runtime-evidence contract file",
    );
}

#[test]
fn spec_c15_service_api_endpoint_root_declares_message_persistence_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod message_persistence_contract_tests;"),
        "service_api_endpoint_tests.rs should declare message-persistence submodule"
    );
}

#[test]
fn spec_c16_service_api_endpoint_message_persistence_split_files_stay_below_budget() {
    for path in [
        MESSAGE_PERSISTENCE_MODULE_FILE,
        MESSAGE_RESTART_FILE,
        MESSAGE_RUNTIME_EVIDENCE_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
