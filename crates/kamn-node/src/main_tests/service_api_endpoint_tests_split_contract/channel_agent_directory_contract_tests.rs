use super::support::*;

#[test]
fn spec_c17_service_api_endpoint_root_file_removes_moved_channel_agent_directory_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_lists_channel_messages_from_message_store()",
        "fn integration_service_api_endpoint_persists_channel_creation_state_across_restart()",
        "fn integration_service_api_endpoint_persists_agent_profile_query_state_across_restart()",
        "fn integration_service_api_endpoint_registers_agent_metadata_idempotently_and_conflicts_on_mismatch(",
        "fn integration_service_api_endpoint_searches_registered_agent_metadata()",
        "fn integration_service_api_endpoint_rejects_invalid_agent_search_payload()",
        "fn integration_service_api_endpoint_rejects_legacy_agent_profile_path_dids()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved channel/agent directory marker: {marker}"
        );
    }
}

#[test]
fn spec_c18_service_api_endpoint_channel_agent_directory_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(CHANNEL_AGENT_DIRECTORY_MODULE_FILE);
    let channel_state = read_repo_file(CHANNEL_STATE_FILE);
    let agent_profile = read_repo_file(AGENT_PROFILE_FILE);
    let agent_registry = read_repo_file(AGENT_REGISTRY_FILE);

    assert!(
        module_source.contains("mod channel_state_contract_tests;"),
        "channel_agent_directory_contract_tests.rs should declare channel-state submodule"
    );
    assert!(
        module_source.contains("mod agent_profile_contract_tests;"),
        "channel_agent_directory_contract_tests.rs should declare agent-profile submodule"
    );
    assert!(
        module_source.contains("mod agent_registry_contract_tests;"),
        "channel_agent_directory_contract_tests.rs should declare agent-registry submodule"
    );

    for marker in [
        "fn integration_service_api_endpoint_lists_channel_messages_from_message_store()",
        "fn integration_service_api_endpoint_persists_channel_creation_state_across_restart()",
    ] {
        assert!(
            channel_state.contains(marker),
            "channel_state_contract_tests.rs should include moved marker: {marker}"
        );
    }

    for marker in [
        "fn integration_service_api_endpoint_persists_agent_profile_query_state_across_restart()",
        "fn integration_service_api_endpoint_rejects_legacy_agent_profile_path_dids()",
    ] {
        assert!(
            agent_profile.contains(marker),
            "agent_profile_contract_tests.rs should include moved marker: {marker}"
        );
    }

    for marker in [
        "fn integration_service_api_endpoint_registers_agent_metadata_idempotently_and_conflicts_on_mismatch(",
        "fn integration_service_api_endpoint_searches_registered_agent_metadata()",
        "fn integration_service_api_endpoint_rejects_invalid_agent_search_payload()",
    ] {
        assert!(
            agent_registry.contains(marker),
            "agent_registry_contract_tests.rs should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c19_service_api_endpoint_root_declares_channel_agent_directory_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod channel_agent_directory_contract_tests;"),
        "service_api_endpoint_tests.rs should declare channel-agent-directory submodule"
    );
}

#[test]
fn spec_c20_service_api_endpoint_channel_agent_directory_split_files_stay_below_budget() {
    for path in [
        CHANNEL_AGENT_DIRECTORY_MODULE_FILE,
        CHANNEL_STATE_FILE,
        AGENT_PROFILE_FILE,
        AGENT_REGISTRY_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
