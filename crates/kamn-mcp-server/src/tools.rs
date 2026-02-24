/// Deterministic tool descriptor for MCP registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDescriptor {
    /// Tool name.
    pub name: &'static str,
    /// Human-readable purpose.
    pub description: &'static str,
    /// Deterministic input schema JSON.
    pub input_schema: &'static str,
    /// Deterministic output schema JSON.
    pub output_schema: &'static str,
}

const GENERIC_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":true}"#;
const GENERIC_OUTPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":true}"#;

/// Required PRD phase-2 MCP tool names.
pub const MCP_TOOL_NAMES: [&str; 21] = [
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
        "submit_bridge_message" => "Submit one bridge message",
        "forward_bridge_message" => "Forward one bridge message",
        "query_bridge_message" => "Query one bridge message",
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
        input_schema: GENERIC_INPUT_SCHEMA_JSON,
        output_schema: GENERIC_OUTPUT_SCHEMA_JSON,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_tool_registry, MCP_TOOL_NAMES};

    #[test]
    fn unit_mcp_tool_registry_count_matches_constant_inventory() {
        let registry = build_tool_registry();
        assert_eq!(registry.len(), MCP_TOOL_NAMES.len());
        assert_eq!(registry.len(), 21);
    }
}
