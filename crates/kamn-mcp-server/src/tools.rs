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
    r#"{"type":"object","additionalProperties":false,"properties":{}}"#;
const PAYLOAD_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["payload"],"properties":{"payload":{"type":"string","minLength":1}}}"#;
const CHANNEL_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["channel_id"],"properties":{"channel_id":{"type":"string","minLength":1}}}"#;
const MESSAGE_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["message_id"],"properties":{"message_id":{"type":"string","minLength":1}}}"#;
const TASK_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["task_id"],"properties":{"task_id":{"type":"string","minLength":1}}}"#;
const DID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["did"],"properties":{"did":{"type":"string","minLength":1}}}"#;
const CONTENT_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["content_id"],"properties":{"content_id":{"type":"string","minLength":1}}}"#;
const BRIDGE_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["bridge_id"],"properties":{"bridge_id":{"type":"string","minLength":1}}}"#;
const ESCROW_ID_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["escrow_id"],"properties":{"escrow_id":{"type":"string","minLength":1}}}"#;
const VERIFY_PROOF_INPUT_SCHEMA_JSON: &str = r#"{"type":"object","additionalProperties":false,"required":["message_id","tx_hash","block_height","finality"],"properties":{"message_id":{"type":"string","minLength":1},"tx_hash":{"type":"string","minLength":1},"block_height":{"oneOf":[{"type":"integer","minimum":0},{"type":"string","pattern":"^[0-9]+$"}]},"finality":{"type":"string","minLength":1}}}"#;
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
        input_schema: tool_input_schema_for_name(name),
        output_schema: GENERIC_OUTPUT_SCHEMA_JSON,
    }
}

fn tool_input_schema_for_name(name: &'static str) -> &'static str {
    match name {
        "register" | "health" => EMPTY_INPUT_SCHEMA_JSON,
        "send_message"
        | "create_channel"
        | "register_content"
        | "submit_bridge_message"
        | "create_task"
        | "fund_escrow" => PAYLOAD_INPUT_SCHEMA_JSON,
        "list_messages" => CHANNEL_ID_INPUT_SCHEMA_JSON,
        "query_message" => MESSAGE_ID_INPUT_SCHEMA_JSON,
        "query_task" | "accept_task" | "complete_task" => TASK_ID_INPUT_SCHEMA_JSON,
        "query_agent_profile" => DID_INPUT_SCHEMA_JSON,
        "expire_content" | "tombstone_content" | "query_content" => CONTENT_ID_INPUT_SCHEMA_JSON,
        "forward_bridge_message" | "query_bridge_message" => BRIDGE_ID_INPUT_SCHEMA_JSON,
        "release_escrow" => ESCROW_ID_INPUT_SCHEMA_JSON,
        "verify_proof" => VERIFY_PROOF_INPUT_SCHEMA_JSON,
        _ => EMPTY_INPUT_SCHEMA_JSON,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_tool_registry, MCP_TOOL_NAMES};

    const LEGACY_GENERIC_INPUT_SCHEMA_JSON: &str =
        r#"{"type":"object","additionalProperties":true}"#;

    #[test]
    fn unit_mcp_tool_registry_count_matches_constant_inventory() {
        let registry = build_tool_registry();
        assert_eq!(registry.len(), MCP_TOOL_NAMES.len());
        assert_eq!(registry.len(), 21);
    }

    #[test]
    fn unit_mcp_tool_registry_input_schemas_are_not_legacy_generic_payloads() {
        let registry = build_tool_registry();
        assert!(registry
            .iter()
            .all(|tool| { tool.input_schema != LEGACY_GENERIC_INPUT_SCHEMA_JSON }));
    }
}
