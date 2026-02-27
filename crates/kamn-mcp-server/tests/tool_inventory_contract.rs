use kamn_mcp_server::tools::{build_tool_registry, MCP_TOOL_NAMES};
use serde_json::Value;

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
    let expected_required: [(&str, &[&str]); 21] = [
        ("register", &[]),
        ("send_message", &["payload"]),
        ("create_channel", &["payload"]),
        ("list_messages", &["channel_id"]),
        ("query_message", &["message_id"]),
        ("query_task", &["task_id"]),
        ("query_agent_profile", &["did"]),
        ("register_content", &["payload"]),
        ("expire_content", &["content_id"]),
        ("tombstone_content", &["content_id"]),
        ("query_content", &["content_id"]),
        ("submit_bridge_message", &["payload"]),
        ("forward_bridge_message", &["bridge_id"]),
        ("query_bridge_message", &["bridge_id"]),
        ("create_task", &["payload"]),
        ("accept_task", &["task_id"]),
        ("complete_task", &["task_id"]),
        ("fund_escrow", &["payload"]),
        ("release_escrow", &["escrow_id"]),
        (
            "verify_proof",
            &["message_id", "tx_hash", "block_height", "finality"],
        ),
        ("health", &[]),
    ];

    for (tool_name, required_fields) in expected_required {
        let tool = registry
            .iter()
            .find(|tool| tool.name == tool_name)
            .expect("tool descriptor should exist");
        assert!(!tool.description.trim().is_empty());

        let input_schema: Value =
            serde_json::from_str(tool.input_schema).expect("input schema should parse");
        assert_eq!(
            input_schema.get("type"),
            Some(&Value::String("object".to_owned()))
        );
        assert_eq!(
            input_schema.get("additionalProperties"),
            Some(&Value::Bool(false))
        );

        let required = input_schema
            .get("required")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let expected = required_fields
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            required, expected,
            "required fields should match contract for tool {tool_name}",
        );

        let properties = input_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("input schema should include properties object");
        for field in required_fields {
            assert!(
                properties.contains_key(*field),
                "tool {tool_name} input schema should expose required field property {field}",
            );
        }

        if tool_name == "verify_proof" {
            let block_height = properties
                .get("block_height")
                .expect("verify_proof schema should include block_height");
            assert!(
                block_height.get("oneOf").is_some(),
                "verify_proof block_height should allow deterministic numeric parsing modes",
            );
        }

        assert_ne!(
            tool.input_schema, r#"{"type":"object","additionalProperties":true}"#,
            "tool {tool_name} should not use the legacy generic input schema",
        );

        let output_schema: Value =
            serde_json::from_str(tool.output_schema).expect("output schema should parse");
        assert_eq!(
            output_schema.get("type"),
            Some(&Value::String("object".to_owned()))
        );
        assert_eq!(
            output_schema.get("additionalProperties"),
            Some(&Value::Bool(true))
        );
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
