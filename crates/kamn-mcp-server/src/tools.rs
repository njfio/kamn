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
pub const MCP_TOOL_NAMES: [&str; 12] = [
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
