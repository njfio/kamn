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
    for tool in registry {
        assert!(!tool.description.trim().is_empty());
        let input_schema: Value =
            serde_json::from_str(tool.input_schema).expect("input schema must be valid json");
        let output_schema: Value =
            serde_json::from_str(tool.output_schema).expect("output schema must be valid json");
        assert_eq!(
            input_schema
                .get("type")
                .and_then(Value::as_str)
                .expect("input type"),
            "object"
        );
        assert!(
            input_schema.get("properties").is_some(),
            "input schema should expose properties for {tool_name}",
            tool_name = tool.name
        );
        assert_eq!(
            output_schema
                .get("type")
                .and_then(Value::as_str)
                .expect("output type"),
            "object"
        );
        assert!(
            output_schema.get("properties").is_some(),
            "output schema should expose deterministic envelope properties for {tool_name}",
            tool_name = tool.name
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

#[test]
fn spec_c10_mcp_tool_registry_output_schema_declares_envelope_shape() {
    let registry = build_tool_registry();
    let health = registry
        .iter()
        .find(|tool| tool.name == "health")
        .expect("health descriptor should exist");
    let output_schema: Value =
        serde_json::from_str(health.output_schema).expect("output schema should parse");
    let required = output_schema
        .get("required")
        .and_then(Value::as_array)
        .expect("output schema required field should exist");
    assert!(
        required.iter().any(|value| value == "ok"),
        "output schema must require ok flag"
    );
    let properties = output_schema
        .get("properties")
        .and_then(Value::as_object)
        .expect("output schema properties should exist");
    assert!(properties.contains_key("result"));
    assert!(properties.contains_key("error"));
}

#[test]
fn spec_c11_mcp_tool_registry_input_schemas_expose_required_fields_for_representative_tools() {
    let registry = build_tool_registry();

    let send_message = registry
        .iter()
        .find(|tool| tool.name == "send_message")
        .expect("send_message descriptor should exist");
    let send_input: Value =
        serde_json::from_str(send_message.input_schema).expect("send_message schema should parse");
    let send_required = send_input
        .get("required")
        .and_then(Value::as_array)
        .expect("send_message required set");
    assert!(send_required.iter().any(|value| value == "payload"));

    let verify_proof = registry
        .iter()
        .find(|tool| tool.name == "verify_proof")
        .expect("verify_proof descriptor should exist");
    let verify_input: Value =
        serde_json::from_str(verify_proof.input_schema).expect("verify_proof schema should parse");
    let verify_required = verify_input
        .get("required")
        .and_then(Value::as_array)
        .expect("verify_proof required set");
    for field in ["message_id", "tx_hash", "block_height", "finality"] {
        assert!(
            verify_required.iter().any(|value| value == field),
            "verify_proof required set should include {field}"
        );
    }

    let health = registry
        .iter()
        .find(|tool| tool.name == "health")
        .expect("health descriptor should exist");
    let health_input: Value =
        serde_json::from_str(health.input_schema).expect("health schema should parse");
    let health_properties = health_input
        .get("properties")
        .and_then(Value::as_object)
        .expect("health properties object");
    assert!(
        health_properties.is_empty(),
        "health input schema should expose no required payload fields"
    );
    assert_eq!(
        health_input
            .get("additionalProperties")
            .and_then(Value::as_bool),
        Some(false),
        "health input schema should reject undeclared fields"
    );
}
