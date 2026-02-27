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

const EMPTY_INPUT_SCHEMA_JSON: &str =
    r#"{"type":"object","properties":{},"additionalProperties":false}"#;
const PAYLOAD_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["payload"],"properties":{"payload":{"type":"string"}},"additionalProperties":false}"#;
const CHANNEL_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["channel_id"],"properties":{"channel_id":{"type":"string"}},"additionalProperties":false}"#;
const MESSAGE_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["message_id"],"properties":{"message_id":{"type":"string"}},"additionalProperties":false}"#;
const TASK_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["task_id"],"properties":{"task_id":{"type":"string"}},"additionalProperties":false}"#;
const DID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["did"],"properties":{"did":{"type":"string"}},"additionalProperties":false}"#;
const CONTENT_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["content_id"],"properties":{"content_id":{"type":"string"}},"additionalProperties":false}"#;
const BRIDGE_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["bridge_id"],"properties":{"bridge_id":{"type":"string"}},"additionalProperties":false}"#;
const ESCROW_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["escrow_id"],"properties":{"escrow_id":{"type":"string"}},"additionalProperties":false}"#;
const VERIFY_PROOF_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["message_id","tx_hash","block_height","finality"],"properties":{"message_id":{"type":"string"},"tx_hash":{"type":"string"},"block_height":{"type":"string"},"finality":{"type":"string"}},"additionalProperties":false}"#;
const MCP_ENVELOPE_OUTPUT_SCHEMA_JSON: &str = r#"{"type":"object","required":["ok"],"properties":{"ok":{"type":"boolean"},"id":{"type":"string"},"tool":{"type":"string"},"result":{"type":"object"},"error":{"type":"object","required":["kind","message"],"properties":{"kind":{"type":"string"},"message":{"type":"string"}},"additionalProperties":false}},"additionalProperties":false}"#;

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

    let input_schema = match name {
        "register" => EMPTY_INPUT_SCHEMA_JSON,
        "send_message" => PAYLOAD_INPUT_SCHEMA_JSON,
        "create_channel" => PAYLOAD_INPUT_SCHEMA_JSON,
        "list_messages" => CHANNEL_ID_INPUT_SCHEMA_JSON,
        "query_message" => MESSAGE_ID_INPUT_SCHEMA_JSON,
        "query_task" => TASK_ID_INPUT_SCHEMA_JSON,
        "query_agent_profile" => DID_INPUT_SCHEMA_JSON,
        "register_content" => PAYLOAD_INPUT_SCHEMA_JSON,
        "expire_content" => CONTENT_ID_INPUT_SCHEMA_JSON,
        "tombstone_content" => CONTENT_ID_INPUT_SCHEMA_JSON,
        "query_content" => CONTENT_ID_INPUT_SCHEMA_JSON,
        "submit_bridge_message" => PAYLOAD_INPUT_SCHEMA_JSON,
        "forward_bridge_message" => BRIDGE_ID_INPUT_SCHEMA_JSON,
        "query_bridge_message" => BRIDGE_ID_INPUT_SCHEMA_JSON,
        "create_task" => PAYLOAD_INPUT_SCHEMA_JSON,
        "accept_task" => TASK_ID_INPUT_SCHEMA_JSON,
        "complete_task" => TASK_ID_INPUT_SCHEMA_JSON,
        "fund_escrow" => PAYLOAD_INPUT_SCHEMA_JSON,
        "release_escrow" => ESCROW_ID_INPUT_SCHEMA_JSON,
        "verify_proof" => VERIFY_PROOF_INPUT_SCHEMA_JSON,
        "health" => EMPTY_INPUT_SCHEMA_JSON,
        _ => EMPTY_INPUT_SCHEMA_JSON,
    };

    ToolDescriptor {
        name,
        description,
        input_schema,
        output_schema: MCP_ENVELOPE_OUTPUT_SCHEMA_JSON,
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
