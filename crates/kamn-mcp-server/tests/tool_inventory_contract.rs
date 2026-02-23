use kamn_mcp_server::tools::{build_tool_registry, MCP_TOOL_NAMES};

#[test]
fn spec_c03_mcp_tool_registry_contains_required_21_tools() {
    let required = [
        "register",
        "send_message",
        "create_channel",
        "list_messages",
        "query_message",
        "query_task",
        "query_agent_profile",
        "register_content",
        "expire_content",
        "tombstone_content",
        "query_content",
        "submit_bridge_message",
        "forward_bridge_message",
        "query_bridge_message",
        "create_task",
        "accept_task",
        "complete_task",
        "fund_escrow",
        "release_escrow",
        "verify_proof",
        "health",
    ];

    let registry = build_tool_registry();
    assert_eq!(registry.len(), required.len());
    assert_eq!(MCP_TOOL_NAMES.len(), required.len());

    for tool_name in required {
        assert!(
            registry.iter().any(|tool| tool.name == tool_name),
            "missing MCP tool: {tool_name}"
        );
    }
}

#[test]
fn spec_c04_mcp_tool_registry_has_deterministic_schema_descriptors() {
    let registry = build_tool_registry();
    for tool in registry {
        assert!(!tool.description.trim().is_empty());
        assert_eq!(tool.input_schema, "kamn.mcp.input.v1");
        assert_eq!(tool.output_schema, "kamn.mcp.output.v1");
    }
}

#[test]
fn spec_c05_mcp_query_tool_descriptors_match_contract_descriptions() {
    let registry = build_tool_registry();

    let query_task = registry
        .iter()
        .find(|tool| tool.name == "query_task")
        .expect("query_task descriptor should exist");
    assert_eq!(
        query_task.description, "Query one task",
        "query_task descriptor should keep canonical contract description",
    );

    let query_agent_profile = registry
        .iter()
        .find(|tool| tool.name == "query_agent_profile")
        .expect("query_agent_profile descriptor should exist");
    assert_eq!(
        query_agent_profile.description, "Query one agent profile",
        "query_agent_profile descriptor should keep canonical contract description",
    );
}

#[test]
fn spec_c08_mcp_content_tool_descriptors_match_contract_descriptions() {
    let registry = build_tool_registry();

    let register_content = registry
        .iter()
        .find(|tool| tool.name == "register_content")
        .expect("register_content descriptor should exist");
    assert_eq!(
        register_content.description, "Register one content record",
        "register_content descriptor should keep canonical contract description",
    );

    let expire_content = registry
        .iter()
        .find(|tool| tool.name == "expire_content")
        .expect("expire_content descriptor should exist");
    assert_eq!(
        expire_content.description, "Expire one content record",
        "expire_content descriptor should keep canonical contract description",
    );

    let tombstone_content = registry
        .iter()
        .find(|tool| tool.name == "tombstone_content")
        .expect("tombstone_content descriptor should exist");
    assert_eq!(
        tombstone_content.description, "Tombstone one content record",
        "tombstone_content descriptor should keep canonical contract description",
    );

    let query_content = registry
        .iter()
        .find(|tool| tool.name == "query_content")
        .expect("query_content descriptor should exist");
    assert_eq!(
        query_content.description, "Query one content record",
        "query_content descriptor should keep canonical contract description",
    );
}

#[test]
fn spec_c09_mcp_bridge_tool_descriptors_match_contract_descriptions() {
    let registry = build_tool_registry();

    let submit_bridge_message = registry
        .iter()
        .find(|tool| tool.name == "submit_bridge_message")
        .expect("submit_bridge_message descriptor should exist");
    assert_eq!(
        submit_bridge_message.description, "Submit one bridge message",
        "submit_bridge_message descriptor should keep canonical contract description",
    );

    let forward_bridge_message = registry
        .iter()
        .find(|tool| tool.name == "forward_bridge_message")
        .expect("forward_bridge_message descriptor should exist");
    assert_eq!(
        forward_bridge_message.description, "Forward one bridge message",
        "forward_bridge_message descriptor should keep canonical contract description",
    );

    let query_bridge_message = registry
        .iter()
        .find(|tool| tool.name == "query_bridge_message")
        .expect("query_bridge_message descriptor should exist");
    assert_eq!(
        query_bridge_message.description, "Query one bridge message",
        "query_bridge_message descriptor should keep canonical contract description",
    );
}
