/// Deterministic tool descriptor for MCP registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDescriptor {
    /// Tool name.
    pub name: &'static str,
    /// Human-readable purpose.
    pub description: &'static str,
    /// Deterministic input schema marker.
    pub input_schema: &'static str,
    /// Deterministic output schema marker.
    pub output_schema: &'static str,
}

/// Required PRD phase-2 MCP tool names.
pub const MCP_TOOL_NAMES: [&str; 18] = [
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
    "create_task",
    "accept_task",
    "complete_task",
    "fund_escrow",
    "release_escrow",
    "verify_proof",
    "health",
];

/// Builds the deterministic phase-2 MCP tool registry.
pub fn build_tool_registry() -> Vec<ToolDescriptor> {
    MCP_TOOL_NAMES
        .iter()
        .copied()
        .map(tool_descriptor_for_name)
        .collect()
}

fn tool_descriptor_for_name(name: &'static str) -> ToolDescriptor {
    let description = match name {
        "register" => "Register one agent DID",
        "send_message" => "Send one message",
        "create_channel" => "Create one channel",
        "list_messages" => "List channel messages",
        "query_message" => "Query one message",
        "query_task" => "Query one task",
        "query_agent_profile" => "Query one agent profile",
        "register_content" => "Register one content record",
        "expire_content" => "Expire one content record",
        "tombstone_content" => "Tombstone one content record",
        "query_content" => "Query one content record",
        "create_task" => "Create one task",
        "accept_task" => "Accept one task",
        "complete_task" => "Complete one task",
        "fund_escrow" => "Fund one escrow",
        "release_escrow" => "Release one escrow",
        "verify_proof" => "Verify one proof",
        "health" => "Query health status",
        _ => "Unsupported tool",
    };

    ToolDescriptor {
        name,
        description,
        input_schema: "kamn.mcp.input.v1",
        output_schema: "kamn.mcp.output.v1",
    }
}

#[cfg(test)]
mod tests {
    use super::{build_tool_registry, MCP_TOOL_NAMES};

    #[test]
    fn unit_mcp_tool_registry_count_matches_constant_inventory() {
        let registry = build_tool_registry();
        assert_eq!(registry.len(), MCP_TOOL_NAMES.len());
        assert_eq!(registry.len(), 18);
    }
}
