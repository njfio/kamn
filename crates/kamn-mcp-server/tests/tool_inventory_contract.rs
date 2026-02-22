use kamn_mcp_server::tools::{build_tool_registry, MCP_TOOL_NAMES};

#[test]
fn spec_c03_mcp_tool_registry_contains_required_12_tools() {
    let required = [
        "register",
        "send_message",
        "create_channel",
        "list_messages",
        "query_message",
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
