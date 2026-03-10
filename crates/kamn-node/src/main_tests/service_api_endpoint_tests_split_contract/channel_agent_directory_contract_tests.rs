use super::support::*;

const CHANNEL_AGENT_DIRECTORY_SUBMODULE_MARKERS: &[&str] = &[
    "mod channel_state_contract_tests;",
    "mod agent_profile_contract_tests;",
    "mod agent_registry_contract_tests;",
];
const CHANNEL_STATE_MARKERS: &[&str] = &[
    "fn integration_service_api_endpoint_lists_channel_messages_from_message_store()",
    "fn integration_service_api_endpoint_persists_channel_creation_state_across_restart()",
];
const AGENT_PROFILE_MARKERS: &[&str] = &[
    "fn integration_service_api_endpoint_persists_agent_profile_query_state_across_restart()",
    "fn integration_service_api_endpoint_rejects_legacy_agent_profile_path_dids()",
];
const AGENT_REGISTRY_MARKERS: &[&str] = &[
    "fn integration_service_api_endpoint_registers_agent_metadata_idempotently_and_conflicts_on_mismatch(",
    "fn integration_service_api_endpoint_searches_registered_agent_metadata()",
    "fn integration_service_api_endpoint_rejects_invalid_agent_search_payload()",
];

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

    assert_contains_markers(
        module_source.as_str(),
        CHANNEL_AGENT_DIRECTORY_SUBMODULE_MARKERS,
        "channel-agent-directory module",
    );
    assert_contains_markers(channel_state.as_str(), CHANNEL_STATE_MARKERS, "channel-state contract file");
    assert_contains_markers(agent_profile.as_str(), AGENT_PROFILE_MARKERS, "agent-profile contract file");
    assert_contains_markers(agent_registry.as_str(), AGENT_REGISTRY_MARKERS, "agent-registry contract file");
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
